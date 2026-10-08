# Releasing Enjin Blockchain REST API

This is an Enjin-maintained GPL-3.0-or-later fork of Parity Technologies'
Polkadot REST API. Keep LICENSE, original copyright/SPDX notices, NOTICE and
third-party notices in every distribution. Record material modifications and
their date in NOTICE. Historical CHANGELOG entries describe upstream releases.

## Prepare a fork release

1. Choose an Enjin release version; the current 0.3.2 is inherited from upstream.
2. Update the workspace version in Cargo.toml, local dependency versions in the
   server/integration-test manifests, and the OpenAPI version annotation. Update
   the Dockerfile's fallback VERSION. Refresh Cargo.lock and regenerate the spec.
3. Update the changelog with an explicitly identified Enjin release entry.
4. Run the checks from README.md, regenerate and build documentation, and test
   all four networks locally with `scripts/smoke-ebc.py`. Complete the
   funded-account and signed-transaction acceptance checks before release.
5. Confirm Docker Hub publishing is configured, then tag the reviewed commit
   `vX.Y.Z` (or a semver prerelease). Existing organization tag rules apply.

```sh
npm ci --prefix docs
npm run update-spec --prefix docs
npm run build --prefix docs
cargo build --locked --release -p polkadot-rest-api
```

## Artifacts and publishing

On default-branch pushes and `v*` tags, the container workflow automatically publishes
`docker.io/enjin/blockchain-rest-api` for linux/amd64 and linux/arm64. Stable
numeric version tags also update `latest`; prereleases do not. Default-branch
builds use dev-SHA and timestamp tags. Record the image digest for deployment.
No Parity hosting or Kargo pipeline is used.

Containers include the executable, license/notices and a source archive under
`/usr/share/doc/blockchain-rest-api/source.tar.gz`. The archive includes Rust,
configuration, documentation sources, lockfiles, build script and Dockerfile.
The docs bundle separately includes `source.tar.gz`, `license.txt`,
`attribution.txt` and generated third-party JS license notices.

CI uploads the Linux binary, LICENSE, NOTICE and a source archive from the same
Git commit. It does not automatically create a GitHub Release or publish crates.
For a release, attach the binary and its matching source archive, license and
notices together. Build any additional OS/architecture binaries separately and
label them accurately. Both Rust packages have `publish = false` until an Enjin
crates.io publishing policy and namespace are established.

## Corresponding source

Provide recipients the corresponding source for the exact binaries, containers
or JavaScript bundles you distribute, including modifications, dependency
lockfiles and scripts needed to build them. A link to a private repository is
not sufficient for recipients without access. The included source archives
provide a practical delivery path; verify them before distributing artifacts.
Never substitute a newer default-branch checkout for the release source.

Preserve upstream references used for attribution, dependency identification,
historical releases and regression fixtures. Use Enjin naming for product
identity, commands, images and current support links. The rename does not alter
the GPL-3.0-or-later terms or third-party dependency licenses.
