# Copyright (C) 2026 Enjin
# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("smoke", Path(__file__).with_name("smoke-ebc.py"))
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class SmokeValidationTests(unittest.TestCase):
    def test_wrong_network_and_denomination_fail(self):
        body = {"specName": "enjin", "properties": {
            "tokenSymbol": ["ENJ"], "tokenDecimals": ["18"], "ss58Format": "2135"}}
        smoke.check_spec(body, "enjin")
        with self.assertRaises(ValueError):
            smoke.check_spec(body, "canary")
        body["properties"]["tokenDecimals"] = ["12"]
        with self.assertRaises(ValueError):
            smoke.check_spec(body, "enjin")

    def test_balance_error_or_invalid_amount_fails(self):
        with self.assertRaises(KeyError):
            smoke.check_balance({"error": "RPC failure"}, "enjin")
        body = dict(tokenSymbol="ENJ", free="1000000000000000000", reserved="0",
                    frozen="0", transferable="900000000000000000", nonce="2", at={"height": "1"})
        smoke.check_balance(body, "enjin")
        body["free"] = "-1"
        with self.assertRaises(ValueError):
            smoke.check_balance(body, "enjin")
