-- Release template for the LuaRocks channel (lumen-oss/luarocks-tag-release,
-- driven by .github/workflows/luarocks-publish.yml). The placeholders in
-- this file are filled in by the action from the pushed tag and the
-- workflow inputs; this is a template, not a local development rockspec.
--
-- One thing here is not the action's generated default: source.dir. The
-- client lives at clients/nvim in the epher monorepo (ADR-0066: clients
-- are version-locked to epher, not separate repositories), and the
-- action fetches the tag's GitHub archive and builds from the archive
-- root, where no lua/ or runtime directories exist. Pointing source.dir
-- at the subdirectory makes the standard Neovim plugin layout visible to
-- the action's defaults: lua/ is installed by the builtin convention and
-- build.copy_directories copies the runtime directories (the default
-- expands to Neovim's runtimepath names; only directories that exist are
-- copied).
--
-- The layout this expects (lua/, ftdetect/, ftplugin/, syntax/) is kept
-- in sync with clients/vim by clients/nvim/sync-runtime.py. The rock name
-- comes from the workflow (epher); the version comes from the pushed tag
-- (v0.5.57 becomes rock 0.5.57-1).

local git_ref = '$git_ref'
local modrev = '$modrev'
local specrev = '$specrev'

local repo_url = '$repo_url'

rockspec_format = '3.0'
package = '$package'
version = modrev .. '-' .. specrev

description = {
  summary = '$summary',
  detailed = $detailed_description,
  labels = $labels,
  homepage = '$homepage',
  $license
}

dependencies = $dependencies

test_dependencies = $test_dependencies

source = {
  url = repo_url .. '/archive/' .. git_ref .. '.zip',
  dir = '$repo_name-' .. '$archive_dir_suffix' .. '/clients/nvim',
}

build = {
  type = 'builtin',
  copy_directories = $copy_directories,
}
