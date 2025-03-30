# Contribution to synf

Please create Github issues to contribute either for submitting bugs, or preparing to make pull requests.

## Development

Try starting:

```bash
cargo run dev examples/typescript
```

Then send:

```json
{"method":"initialize", "params":{"protocolVersion":"2024","clientInfo":{"name": "tst","version":"1.0.0"}, "capabilities":{}}, "jsonrpc": "2.0", "id":1}

```

Try sending resource subscriptions:
```json
{"jsonrpc":"2.0","id":2,"method":"resources/subscribe","params":{"uri":"file:///project/src/main.ts"}}
```

Try running this:

```bash
cargo run init examples/typescript
```

Using with inspector:

```bash
npx @modelcontextprotocol/inspector cargo run dev examples/typescript
```

## Release

To release a new version make sure that CHANGELOG.md is up to date with unreleased changes.

Then to release new version run one of the following commands:

```bash
cargo release patch --execute
# or
cargo release minor --execute
```
