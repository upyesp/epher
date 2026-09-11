-- Neovim glue for the shared epher language server (ADR-0066 ships
-- Neovim as a ready-made config, not a plugin). Neovim 0.11+ only:
-- this uses the native vim.lsp.config/vim.lsp.enable pair.
--
-- Usage:
--   vim.opt.rtp:append("/path/to/epher/clients/nvim")
--   vim.opt.rtp:append("/path/to/epher/clients/vim")  -- syntax + ft
--   require("epher").setup({ cmd = { "/path/to/epher-lsp" } })
--
-- The cmd default is `epher-lsp` on your PATH; the README has the
-- download lines per platform.

local M = {}

--- Set up the epher language server. Call once, from anywhere.
--- opts.cmd: the command that starts epher-lsp (default {"epher-lsp"}).
function M.setup(opts)
  opts = opts or {}

  vim.lsp.config("epher", {
    cmd = opts.cmd or { "epher-lsp" },
    filetypes = { "epher" },
    -- A calculator file stands alone: its own directory is the root.
    root_dir = function(bufnr, on_dir)
      on_dir(vim.fs.dirname(vim.api.nvim_buf_get_name(bufnr)))
    end,
  })

  vim.lsp.enable("epher")
end

return M
