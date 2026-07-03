# T50 local Windows regression guards.
# Run from the repository root on Windows.

$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

$strawberryPerl = 'C:\Strawberry\perl\bin'
$strawberryC = 'C:\Strawberry\c\bin'
if (Test-Path $strawberryPerl) {
  $env:PATH = "$strawberryPerl;$strawberryC;$env:PATH"
}

$commands = @(
  @('cargo', @('test', '-p', 'wezterm-gui', 'pane_label', '--', '--nocapture')),
  @('cargo', @('test', '-p', 'wezterm-gui', 'bar_geometry', '--', '--nocapture')),
  @('cargo', @('test', '-p', 'wezterm-gui', 'tab_bar_constructor', '--', '--nocapture')),
  @('cargo', @('test', '-p', 'wezterm-gui', 'ui_item_geometry', '--', '--nocapture')),
  @('cargo', @('test', '-p', 'wezterm-gui', 'ui_item_hit_test', '--', '--nocapture')),
  @('cargo', @('test', '-p', 'wezterm-gui', 'secondary_bar_state', '--', '--nocapture')),
  @('cargo', @('test', '-p', 'wezterm-gui'))
)

foreach ($command in $commands) {
  $exe = $command[0]
  $args = $command[1]
  Write-Host "> $exe $($args -join ' ')" -ForegroundColor Cyan
  & $exe @args
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }
}
