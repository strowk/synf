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

Try running this:

```bash
cargo run init examples/typescript
```

Using with inspector:

```bash
npx @modelcontextprotocol/inspector cargo run dev examples/typescript
```

