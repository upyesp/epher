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
--
-- Running scripts (ADR-0069, decision 3: the text-first editors run
-- the same `epher/run` request the VS Code results pane uses, and
-- show the text results in a scratch buffer; graphs are written as
-- SVG files and opened with the system viewer). `:EpherRun` does all
-- of that for the current buffer; setup({ pane = "float" }) puts the
-- results in a floating window instead of a vertical split.

local M = {}

M.ns = vim.api.nvim_create_namespace("epher-results")

--- The results buffer of the last run, so a rerun replaces it
--- instead of stacking windows.
local results_buf = nil
--- The buffer of the last run. Focus lands in the results pane after
--- a run, and rerunning from there should just work: it re-runs this
--- one, not whatever the pane happens to be.
local last_run = nil

--- <CR> in the results buffer: a row jumps to its statement, a
--- graph row reopens its SVG with the system viewer.
local function follow()
  local row = vim.api.nvim_win_get_cursor(0)[1]
  local ok_graphs, graphs = pcall(vim.api.nvim_buf_get_var, 0, "epher_graphs")
  if ok_graphs and graphs[row] ~= nil then
    vim.ui.open(graphs[row])
    return
  end
  local ok_src, src_lines = pcall(vim.api.nvim_buf_get_var, 0, "epher_src_lines")
  if not (ok_src and src_lines[row] ~= nil) then
    return
  end
  local ok_buf, src_bufnr = pcall(vim.api.nvim_buf_get_var, 0, "epher_src_bufnr")
  if not (ok_buf and vim.api.nvim_buf_is_valid(src_bufnr)) then
    return
  end
  local win = vim.fn.win_findbuf(src_bufnr)[1]
  if win == nil then
    return
  end
  vim.api.nvim_set_current_win(win)
  local last = vim.api.nvim_buf_line_count(src_bufnr)
  vim.api.nvim_win_set_cursor(win, { math.min(src_lines[row], last), 0 })
end

--- Renders one report into a fresh scratch buffer and opens it
--- beside the script (or floating, with pane = "float").
local function show_report(src_bufnr, report, pane)
  -- One fresh buffer per run: the old one is deleted, so its window
  -- (if any) falls back to another buffer and two runs never stack
  -- two result panes side by side.
  if results_buf ~= nil and vim.api.nvim_buf_is_valid(results_buf) then
    vim.api.nvim_buf_delete(results_buf, { force = true })
  end
  results_buf = vim.api.nvim_create_buf(false, true)

  -- The rows are the script's output, not a re-reading of the
  -- script: only statements that produced something, errors marked,
  -- every row carrying its source line for the <CR> jump.
  local rows = {}
  -- results line (1-based) -> source line.
  local src_lines = {}
  -- results line (1-based) -> svg path.
  local graphs = {}
  -- results lines (1-based) carrying an error, marked once the text exists.
  local error_rows = {}
  for _, statement in ipairs(report.statements or {}) do
    if statement.display ~= nil then
      -- The wire carries 0-based lines; humans and nvim cursor rows
      -- count from 1, so both the label and the jump target add one.
      table.insert(rows, string.format("L%-5d %s", statement.line + 1, statement.display))
      src_lines[#rows] = statement.line + 1
      if statement.error then
        error_rows[#error_rows + 1] = #rows
      end
    end
  end
  if report.svgs ~= nil and #report.svgs > 0 then
    local dir = vim.fs.normalize(vim.fn.stdpath("cache") .. "/epher/runs")
    vim.fn.mkdir(dir, "p")
    table.insert(rows, "")
    table.insert(rows, string.format("Graphs (%d, each opened with the system viewer):", #report.svgs))
    for i, svg in ipairs(report.svgs) do
      local path = dir .. string.format("/run-%d-%d.svg", os.time(), i)
      local file = io.open(path, "w")
      if file ~= nil then
        file:write(svg)
        file:close()
        table.insert(rows, "  " .. path)
        graphs[#rows] = path
      end
    end
  end

  if #rows == 0 then
    rows = { "No output." }
  end
  vim.api.nvim_buf_set_lines(results_buf, 0, -1, false, rows)
  local name = vim.fs.basename(vim.api.nvim_buf_get_name(src_bufnr))
  vim.api.nvim_buf_set_name(results_buf, "epher results: " .. (name ~= "" and name or "untitled"))
  vim.bo[results_buf].buftype = "nofile"
  vim.bo[results_buf].bufhidden = "wipe"
  vim.bo[results_buf].swapfile = false
  vim.bo[results_buf].modifiable = false
  vim.api.nvim_buf_set_var(results_buf, "epher_src_bufnr", src_bufnr)
  vim.api.nvim_buf_set_var(results_buf, "epher_src_lines", src_lines)
  vim.api.nvim_buf_set_var(results_buf, "epher_graphs", graphs)
  -- Error rows, after the text exists so the marks land exactly.
  for _, row in ipairs(error_rows) do
    vim.api.nvim_buf_set_extmark(results_buf, M.ns, row - 1, 0, {
      end_line = row - 1,
      end_col = 0,
      hl_group = "ErrorMsg",
      hl_eol = true,
    })
  end
  vim.keymap.set("n", "<CR>", follow, { buffer = results_buf, nowait = true, silent = true })

  -- The window: a vertical split by default (the pane beside the
  -- script, like every other editor's results pane), or a right
  -- anchored float.
  if pane == "float" then
    vim.api.nvim_open_win(results_buf, false, {
      relative = "editor",
      anchor = "NE",
      row = 1,
      col = vim.o.columns - 1,
      width = math.max(40, math.floor(vim.o.columns * 0.45)),
      height = math.max(8, math.floor(vim.o.lines * 0.5)),
      border = "rounded",
      focusable = true,
    })
  else
    vim.cmd("vsplit")
    vim.api.nvim_win_set_buf(vim.api.nvim_get_current_win(), results_buf)
  end
end

--- Runs the current epher buffer: one `epher/run` request to the
--- attached language server, results in the pane.
function M.run()
  local bufnr = vim.api.nvim_get_current_buf()
  if vim.bo[bufnr].filetype ~= "epher" then
    -- Running from the results pane (where a run leaves the cursor)
    -- re-runs the script the pane belongs to.
    if last_run ~= nil and vim.api.nvim_buf_is_valid(last_run)
      and vim.bo[last_run].filetype == "epher" then
      bufnr = last_run
    else
      vim.notify("epher: the current buffer is not an epher script", vim.log.levels.WARN)
      return
    end
  end
  last_run = bufnr
  local clients = vim.lsp.get_clients({ bufnr = bufnr, name = "epher" })
  local client = clients[1]
  if client == nil then
    vim.notify("epher: the language server is not attached; see :checkhealth vim.lsp", vim.log.levels.WARN)
    return
  end

  -- The server knows documents by didOpen. Attached buffers were
  -- opened at attach time; anything else (a buffer the client never
  -- saw) is opened here first, so the run sees what the editor sees.
  local uri = vim.uri_from_bufnr(bufnr)
  if not client.attached_buffers[bufnr] then
    local text = table.concat(vim.api.nvim_buf_get_lines(bufnr, 0, -1, false), "\n")
    client:notify("textDocument/didOpen", {
      textDocument = { uri = uri, languageId = "epher", version = 0, text = text },
    })
  end

  -- client:request answers (handled, request_id): false means no
  -- transport took the request, which for us means a stopped server.
  local pane = M.pane or "split"
  local handled = client:request("epher/run", { textDocument = { uri = uri } }, function(err, result)
    -- The callback runs on an LSP thread; every editor call waits
    -- for the scheduler.
    vim.schedule(function()
      if err ~= nil then
        vim.notify("epher run failed: " .. tostring(err.message or err), vim.log.levels.ERROR)
        return
      end
      show_report(bufnr, result or {}, pane)
    end)
  end)
  if not handled then
    vim.notify("epher: the language server stopped before the run started", vim.log.levels.ERROR)
  end
end

--- Set up the epher language server. Call once, from anywhere.
--- opts.cmd: the command that starts epher-lsp (default {"epher-lsp"}).
--- opts.pane: "split" (default) or "float" for the results window.
function M.setup(opts)
  opts = opts or {}
  M.pane = opts.pane or "split"

  vim.lsp.config("epher", {
    cmd = opts.cmd or { "epher-lsp" },
    filetypes = { "epher" },
    -- A calculator file stands alone: its own directory is the root.
    root_dir = function(bufnr, on_dir)
      on_dir(vim.fs.dirname(vim.api.nvim_buf_get_name(bufnr)))
    end,
  })

  vim.lsp.enable("epher")

  vim.api.nvim_create_user_command("EpherRun", function()
    M.run()
  end, { desc = "Run the current epher script and show the results pane" })
end

return M
