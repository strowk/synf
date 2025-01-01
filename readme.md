# synf - hot reload for MCP servers

`synf` dan help you developing MCP server by hot reloading it on file changes.

`synf dev` command proxies stdio transport between MCP client and server and hot-reloads the server by rebuilding/restarting and refreshing the states (such as sending list_changed notifications).

## Development

Try starting:

```bash
cargo run dev examples/typescript
```

Then send:

```json
{"method":"initialize", "params":{"protocolVersion":"2024","clientInfo":{"name": "tst","version":"1.0.0"}, "capabilities":{}}, "jsonrpc": "2.0", "id":1}

```