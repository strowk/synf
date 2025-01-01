# synf - hot reload for MCP servers

`synf` is command line tool supporting development of MCP servers by working as a stdio proxy between MCP client and server and hot-reloading the server by rebuilding/restarting and refreshing the states (such as sending list_changed notifications).

## Development

Try starting:

```bash
cargo run dev examples/typescript
```

Then send:

```json
{"method":"initialize", "params":{"protocolVersion":"2024","clientInfo":{"name": "tst","version":"1.0.0"}, "capabilities":{}}, "jsonrpc": "2.0", "id":1}

```