# T50 secondary bar

This fork adds an optional secondary status bar intended for Windows daily use.

## Enable

```lua
config.enable_secondary_bar = true
```

When disabled, calls to `window:set_secondary_bar(...)` are still accepted and stored, but the secondary bar is not rendered.

## Update from Lua

Use `set_secondary_bar` from `update-status`:

```lua
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

config.enable_secondary_bar = true

wezterm.on('update-status', function(window, pane)
  window:set_secondary_bar({
    left = '',
    center = 'workspace: ' .. window:active_workspace(),
    right = wezterm.strftime('%H:%M'),
  })
end)

return config
```

## Contract

`set_secondary_bar` expects a Lua table with these optional string fields:

- `left`
- `center`
- `right`

Missing fields default to the empty string, so partial updates are valid:

```lua
window:set_secondary_bar({ center = 'ready' })
```

Non-table values, or fields that cannot be converted to strings, are rejected by the Lua API.
