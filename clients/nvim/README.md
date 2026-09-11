# epher for Neovim

The native-LSP glue for the shared `epher-lsp` server (ADR-0066
ships Neovim as a ready-made config, not a plugin). Requires
Neovim 0.11 or newer (that is where `vim.lsp.config` and
`vim.lsp.enable` landed). The filetype detection and syntax files
come from the shared runtime in `clients/vim/`.

## Install

Point your runtimepath at both directories, then set up the server
once:

```lua
-- anywhere in your config, after lazy-loading is fine
vim.opt.rtp:append("/path/to/epher/clients/vim")
vim.opt.rtp:append("/path/to/epher/clients/nvim")

require("epher").setup({
  cmd = { "/path/to/epher-lsp" },  -- or just {"epher-lsp"} if on PATH
})
```

With [lazy.nvim](https://github.com/folke/lazy.nvim), a local plugin
entry does the rtp part for you:

```lua
{
  dir = "~/code/epher/clients/nvim",
  config = function()
    vim.opt.rtp:append(vim.fn.expand("~/code/epher/clients/vim"))
    require("epher").setup({ cmd = { vim.fn.expand("~/bin/epher-lsp") } })
  end,
}
```

## Getting the binary

Download the asset for your platform from the releases page
(`https://github.com/upyesp/epher/releases`), uncompress it, and
mark it executable:

```sh
# linux x86_64 (arm64 and macos-aarch64 analogous)
curl -LO https://github.com/upyesp/epher/releases/latest/download/epher-lsp-linux-x86_64.gz
gunzip epher-lsp-linux-x86_64.gz && mv epher-lsp-linux-x86_64 ~/.local/bin/epher-lsp
chmod +x ~/.local/bin/epher-lsp

# windows (powershell): epher-lsp-windows-x86_64.zip -> epher-lsp.exe
```

First download needs the network once; the server runs entirely
locally after that.

## What you get

- live diagnostics from spans (parse and evaluation errors);
- inline answers on the statement that produced them (inlay hints);
- hover signatures for catalog names, current values for your own
  constants;
- completion with the shared snippets;
- vim `<C-]>`-style definition jumps for names defined in the file.

Neovim's syntax engine does not consume LSP semantic tokens today,
so highlighting comes from the shared regex syntax in `clients/vim/`
(the conservative approximation of the parser's unit rule).

## Verify by hand

```vim
:checkhealth vim.lsp
:e test.epher
:lua = vim.lsp.get_clients({ bufnr = 0 })
```
