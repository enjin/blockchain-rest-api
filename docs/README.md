# Enjin Blockchain REST API documentation

From the repository root, with Rust 1.94.0 and Node 22:

```sh
npm ci --prefix docs
npm run update-spec --prefix docs
npm run build --prefix docs
cargo build --locked --release -p polkadot-rest-api
```

`openapi.json` is generated from Rust annotations without RPC access. CI checks
it for drift. The binary embeds `dist`; rebuild Rust after updating the docs.
The bundle includes the project license, modification notice and third-party
JS notices. Historical guides retained in source are not all included in the
Enjin Blockchain documentation UI. Documentation hosting is managed separately; this
repository builds the embedded documentation but does not publish a website.
