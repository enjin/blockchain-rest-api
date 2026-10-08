#!/usr/bin/env python3
# Copyright (C) 2026 Enjin
# SPDX-License-Identifier: GPL-3.0-or-later
"""Read-only Enjin Blockchain smoke checks, using only Python's standard library."""
import argparse
import json
import time
import urllib.error
import urllib.parse
import urllib.request

NETWORKS = {
    "enjin": ("ENJ", 2135),
    "canary": ("cENJ", 69),
    "matrix-enjin": ("ENJ", 1110),
    "matrix": ("cENJ", 9030),
}
RELAYS = {"matrix-enjin": "enjin", "matrix": "canary"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_spec(body, spec):
    symbol, prefix = NETWORKS[spec]
    require(body["specName"] == spec, f"Expected {spec}, received {body['specName']}")
    properties = body["properties"]
    require(properties["tokenSymbol"] == [symbol], "Unexpected token symbol")
    require([int(n) for n in properties["tokenDecimals"]] == [18], "Unexpected decimals")
    require(int(properties["ss58Format"]) == prefix, "Unexpected SS58 prefix")


def check_balance(body, spec):
    require(body["tokenSymbol"] == NETWORKS[spec][0], "Unexpected balance symbol")
    for field in ("free", "reserved", "nonce", "frozen", "transferable"):
        require(int(body[field]) >= 0, f"Invalid balance field: {field}")
    require(int(body["at"]["height"]) > 0, "Missing balance block context")


def run(args):
    base = args.base_url.rstrip("/")
    require(not args.relay_spec or RELAYS.get(args.spec) == args.relay_spec,
            "The selected relay does not match the Matrixchain")

    def get(path):
        with urllib.request.urlopen(base + path, timeout=60) as response:
            return json.load(response)

    deadline = time.monotonic() + args.wait_seconds
    while True:
        try:
            require(get("/v1/health")["status"] == "ok", "API is not healthy")
            break
        except (OSError, ValueError):
            if time.monotonic() >= deadline:
                raise
            time.sleep(2)

    check_spec(get("/v1/runtime/spec"), args.spec)
    head = get("/v1/blocks/head")
    require(int(head["number"]) > 0 and len(head["hash"]) == 66, "Invalid head block")
    require(isinstance(head["extrinsics"], list), "Missing decoded extrinsics")
    require(head["extrinsics"], "Expected at least an inherent extrinsic in the head block")
    block = urllib.parse.quote(head["hash"], safe="")
    extrinsic = get(f"/v1/blocks/{block}/extrinsics/0")
    require(bool(extrinsic), "Missing extrinsic response")
    account = urllib.parse.quote(args.account, safe="")
    check_balance(get(f"/v1/accounts/{account}/balance-info?at={block}"), args.spec)
    get("/v1/transaction/material")
    spec = get("/api-docs/openapi.json")
    require(spec["info"]["title"] == "Enjin Blockchain REST API", "Unexpected API identity")
    require(spec["servers"][0]["url"] == "http://localhost:8080", "OpenAPI server must not duplicate /v1")
    for path in ("/v1/ahm/info", "/v1/coretime/info", f"/v1/accounts/{account}/asset-balances"):
        try:
            get(path)
        except urllib.error.HTTPError as error:
            require(error.code == 404, f"Unexpected status for removed route {path}: {error.code}")
        else:
            raise ValueError(f"Unsupported route is still exposed: {path}")
    if args.relay_spec:
        check_spec(get("/v1/rc/runtime/spec"), args.relay_spec)
        check_balance(get(f"/v1/rc/accounts/{account}/balance-info"), args.relay_spec)
    print(f"PASS {args.spec}: identity, health, block {head['number']}, extrinsic, balance, transaction material, OpenAPI and removed routes")
    if args.relay_spec:
        print(f"PASS matching relay proxy: {args.relay_spec}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", default="http://127.0.0.1:8080")
    parser.add_argument("--spec", choices=NETWORKS, required=True)
    parser.add_argument("--relay-spec", choices=("enjin", "canary"))
    parser.add_argument("--account", default="0x" + "00" * 32)
    parser.add_argument("--wait-seconds", type=int, default=0)
    run(parser.parse_args())
