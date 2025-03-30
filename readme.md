# synf - hot reload for MCP servers

`synf` can help you developing MCP server by hot reloading it on file changes.

It proxies stdio transport between MCP client and server and hot-reloads the server by rebuilding/restarting and refreshing the states (such as sending list_changed notifications).

## Usage

Firstly you would need to initialize synf file using command `synf init` - run this in the folder with your project. It would automatically detect used language and ask confirmation.

Once `synf init` have created `synf.toml` file, command `synf dev` can do following:

- build and run your MCP server
- wait for the first initialization request from client and cache it
- watch for changes you make to files configurable within `synf.toml`
- whenever you change watched files - rebuild and restart your server
- repeat initialization request that was sent by client the first time
- notify MCP client to repeat request for tools, prompts and resources
- drop initialization response from server after restart, to avoid repeating it
- if configured: cache resource subscriptions and resend them after restart

You would need to configure the command `synf dev` to be run by client that you want to integrate with your server. Command takes path to folder with your project as first argument.

Here is example for Claude Desktop:

```json
{
    "mcpServers": {
        "synf_test": {
            "command": "synf",
            "args": [
                "dev",
                "C:/work/synf/examples/typescript"
            ]
        }
    }
}
```

Note: Claude Desktop appears to have a bug at the moment, where it ignores list_changed notification that synf is sending and list of tools and their descriptions would not be hot-reloaded when using that client. I expect that they would fix it eventually.

### Configuration for Windows

If you are using Windows, you might need to configure `synf.toml` to use powershell for running some commands, depending on how programming language is installed for you.

For example Node.js developers would have problems with `npm` since it might be provided as a cmd script rather than an executable. You can configure `synf.toml` to use powershell for running npm:

```toml
[build]
command = "powershell"
args = ["npm run build"]
```

## Installation

### Windows with [Scoop](https://github.com/ScoopInstaller/Scoop)

```bash
scoop install https://raw.githubusercontent.com/strowk/synf/main/scoop/synf.json
```

, or if you already have it installed with scoop:

```bash
scoop update synf
```

### With bash script

In bash shell run:

```bash
curl -s https://raw.githubusercontent.com/strowk/synf/main/install.sh | bash
```

Should work in Linux bash, Windows Git Bash and MacOS.
For Windows users: you might need to start Git Bash from Administrator.

#### Disabling sudo

By default the script would try to install synf to `/usr/local/bin` and would require sudo rights for that,
but you can disable this behavior by setting `NO_SUDO` environment variable:

```bash
curl -s https://raw.githubusercontent.com/strowk/synf/main/install.sh | NO_SUDO=1 bash
```

Sudo is disabled by default for Windows Git Bash.

### Manually

Head to [latest release](https://github.com/strowk/synf/releases/latest), download archive for your OS/arch, unpack it and put binary somewhere in your PATH.

### From sources

If your system/architecture is not supported by the script above,
you can install Rust and install synf from sources:

```bash
git clone https://github.com/strowk/synf
cargo install --path ./synf
```

## Under the hood

## Initialization

When client connects to server, it sends initialization request that would look f.e like this:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "ExampleClient",
      "version": "1.0.0"
    }
  }
}
```

`synf` would cache this request and repeat it after server restart, while also dropping the response from server, since the client already received it.

On first run client also would send initialized notification looking like this:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

When server restarted, `synf` would send same notification to server after initialization response is dropped.

### List Changed

After server restarts, it might have new tools, prompts or resources. 

`synf` would notify client to repeat requests by sending such notifications:

```json
{"method":"notifications/tools/list_changed","jsonrpc":"2.0"}
{"method":"notifications/prompts/list_changed","jsonrpc":"2.0"}
{"method":"notifications/resources/list_changed","jsonrpc":"2.0"}
```

Note that some clients might be bugged at the moment and would ignore these notifications, so you might need to manually restart them until the client is following specification correctly.

### Subscriptions

The protocol supports optional subscriptions to resource changes. 
Clients can subscribe to specific resources and receive notifications when they change:

Subscribe Request:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/subscribe",
  "params": {
    "uri": "file:///project/src/main.rs"
  }
}
```

Update Notification:


```json
{
  "jsonrpc": "2.0",
  "method": "notifications/resources/updated",
  "params": {
    "uri": "file:///project/src/main.rs"
  }
}
```

`synf` can cache the resources subscribed to and would resend the subscriptions to server after restart.