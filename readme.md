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

## Installation

### Windows with [Scoop](https://github.com/ScoopInstaller/Scoop)

```bash
scoop install https://raw.githubusercontent.com/strowk/synf/main/scoop-synf.json
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
