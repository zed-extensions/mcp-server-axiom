# Axiom MCP Server

> [!WARNING]
> **This extension is deprecated and no longer maintained.**
>
> The [Axiom MCP server](https://github.com/axiomhq/mcp-server-axiom) this extension wraps has been deprecated and archived by Axiom. Please use the official Axiom remote MCP server at [https://mcp.axiom.co](https://mcp.axiom.co/) instead.

A Zed extension for the [Axiom MCP server](https://github.com/axiomhq/mcp-server-axiom).

## Configuration

To configure the Axiom MCP server, create a [`config`.txt file](https://github.com/axiomhq/mcp-server-axiom/blob/master/README.md#config-file-example-configtxt) somewhere on your system.

At a minimum it should contain your Axiom API token:

```
token xaat-your-token
```

Then in your Zed `settings.json`, add the path to the `config.txt` file to your settings as the `config_path`:

```json
"context_servers": {
  "mcp-server-axiom": {
    "settings": {
      "config_path": "/path/to/config.txt"
    }
  }
}
```
