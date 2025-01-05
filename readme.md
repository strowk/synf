# synf - hot reload for MCP servers

`synf` dan help you developing MCP server by hot reloading it on file changes.

`synf dev` command proxies stdio transport between MCP client and server and hot-reloads the server by rebuilding/restarting and refreshing the states (such as sending list_changed notifications).

## Installation

### With bash script

In bash shell run:

```bash
curl -s https://raw.githubusercontent.com/strowk/synf/main/install.sh | bash
```

Tested on Linux bash and Windows Git Bash. Should work for MacOS too.

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
