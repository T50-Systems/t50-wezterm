[CmdletBinding()]
param(
  [Parameter(Mandatory = $true, Position = 0)]
  [ValidateSet('key', 'activate', 'populate', 'verify')]
  [string]$Mode,

  [string]$CacheDir,
  [string]$TargetDir,
  [string]$EnvironmentFile = $env:GITHUB_ENV,
  [string]$OutputFile = $env:GITHUB_OUTPUT
)

$ErrorActionPreference = 'Stop'
$targetTriple = 'x86_64-pc-windows-msvc'
$manifestName = 'cache-manifest.json'

$repo = Split-Path -Parent $PSScriptRoot
if (-not $TargetDir) {
  $TargetDir = if ($env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR
  } else {
    Join-Path $repo 'target'
  }
}
if (-not $CacheDir) {
  $CacheDir = Join-Path $TargetDir "ci-cache\openssl\$targetTriple"
}

$requiredFiles = @(
  $manifestName,
  'include\openssl\ssl.h',
  'lib\libcrypto.lib',
  'lib\libssl.lib'
)

function Write-StepOutput([string]$Name, [string]$Value) {
  if ([string]::IsNullOrWhiteSpace($OutputFile)) {
    Write-Host "$Name=$Value"
  } else {
    "$Name=$Value" | Add-Content -LiteralPath $OutputFile -Encoding utf8
  }
}

function Get-CacheContextHash {
  Push-Location $repo
  try {
    $metadataJson = & cargo metadata --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) {
      throw 'cargo metadata failed while fingerprinting OpenSSL cache inputs'
    }
  } finally {
    Pop-Location
  }

  $metadata = $metadataJson | ConvertFrom-Json
  $inputPaths = @(
    Join-Path $repo 'Cargo.lock'
    Join-Path $repo 'Cargo.toml'
    Join-Path $repo '.cargo\config.toml'
    $PSCommandPath
    $metadata.packages.manifest_path
  ) | Sort-Object -Unique

  $context = [System.Collections.Generic.List[string]]::new()
  $context.Add('schema=2')
  $context.Add("target=$targetTriple")
  $context.Add("image-os=$($env:ImageOS)")
  $context.Add("image-version=$($env:ImageVersion)")
  $context.Add("processor=$($env:PROCESSOR_ARCHITECTURE)")
  $context.Add("rustc=$((& rustc -vV) -join "`n")")

  $cl = Get-Command cl.exe -ErrorAction SilentlyContinue
  $clVersion = if ($cl) {
    (Get-Item -LiteralPath $cl.Source).VersionInfo.FileVersion
  } else {
    'not-on-path'
  }
  $context.Add("cl=$clVersion")

  foreach ($path in $inputPaths) {
    $resolved = (Resolve-Path -LiteralPath $path).Path
    $relative = [System.IO.Path]::GetRelativePath($repo, $resolved)
    $hash = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
    $context.Add("file=$relative`:$hash")
  }

  $bytes = [System.Text.Encoding]::UTF8.GetBytes($context -join "`n")
  $sha256 = [System.Security.Cryptography.SHA256]::Create()
  try {
    return ([System.BitConverter]::ToString($sha256.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant()
  } finally {
    $sha256.Dispose()
  }
}

function Test-OpenSslInstall([string]$Root) {
  foreach ($relativePath in $requiredFiles) {
    if (-not (Test-Path -LiteralPath (Join-Path $Root $relativePath) -PathType Leaf)) {
      return $false
    }
  }

  try {
    $manifest = Get-Content -LiteralPath (Join-Path $Root $manifestName) -Raw |
      ConvertFrom-Json
  } catch {
    return $false
  }

  if ($manifest.schema -ne 1 -or $manifest.target -ne $targetTriple) {
    return $false
  }
  if ($env:OPENSSL_CACHE_KEY -and $manifest.cacheKey -ne $env:OPENSSL_CACHE_KEY) {
    return $false
  }
  return $true
}

function Assert-OpenSslInstall([string]$Root) {
  if (-not (Test-OpenSslInstall $Root)) {
    throw "OpenSSL cache at '$Root' is incomplete or incompatible"
  }
}

switch ($Mode) {
  'key' {
    Write-StepOutput 'key' (Get-CacheContextHash)
  }

  'activate' {
    if (-not (Test-Path -LiteralPath $CacheDir)) {
      Write-Host 'Static OpenSSL cache miss; Cargo will build the vendored source.'
      Write-StepOutput 'active' 'false'
      return
    }

    if (-not (Test-OpenSslInstall $CacheDir)) {
      Write-Warning 'Discarding an incomplete or incompatible static OpenSSL cache.'
      Remove-Item -LiteralPath $CacheDir -Recurse -Force
      Write-StepOutput 'active' 'false'
      return
    }
    if ([string]::IsNullOrWhiteSpace($EnvironmentFile)) {
      throw 'GITHUB_ENV or -EnvironmentFile is required to activate cached OpenSSL'
    }

    $resolvedCacheDir = (Resolve-Path -LiteralPath $CacheDir).Path
    @(
      'OPENSSL_NO_VENDOR=1'
      "OPENSSL_DIR=$resolvedCacheDir"
      'OPENSSL_STATIC=1'
    ) | Add-Content -LiteralPath $EnvironmentFile -Encoding utf8
    Write-StepOutput 'active' 'true'
    Write-Host "Activated cached static OpenSSL from '$resolvedCacheDir'."
  }

  'populate' {
    if (Test-Path -LiteralPath $CacheDir) {
      Assert-OpenSslInstall $CacheDir
      Write-Host "Static OpenSSL cache already exists at '$CacheDir'."
      return
    }

    $buildRoot = Join-Path $TargetDir 'release\build'
    $candidates = @(
      Get-ChildItem -LiteralPath $buildRoot -Directory -Filter 'openssl-sys-*' |
        ForEach-Object { Join-Path $_.FullName 'out\openssl-build\install' } |
        Where-Object {
          (Test-Path -LiteralPath (Join-Path $_ 'include\openssl\ssl.h') -PathType Leaf) -and
          (Test-Path -LiteralPath (Join-Path $_ 'lib\libcrypto.lib') -PathType Leaf) -and
          (Test-Path -LiteralPath (Join-Path $_ 'lib\libssl.lib') -PathType Leaf)
        }
    )
    if ($candidates.Count -eq 0) {
      throw "No vendored OpenSSL installation found under '$buildRoot'"
    }

    $source = $candidates |
      ForEach-Object { Get-Item -LiteralPath $_ } |
      Sort-Object LastWriteTimeUtc -Descending |
      Select-Object -First 1 -ExpandProperty FullName

    $parent = Split-Path -Parent $CacheDir
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    $temporary = "$CacheDir.tmp-$PID"
    Remove-Item -LiteralPath $temporary -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $temporary | Out-Null

    try {
      Copy-Item -Path (Join-Path $source '*') -Destination $temporary -Recurse -Force
      $cacheKey = if ($env:OPENSSL_CACHE_KEY) {
        $env:OPENSSL_CACHE_KEY
      } else {
        Get-CacheContextHash
      }
      [ordered]@{
        schema = 1
        target = $targetTriple
        cacheKey = $cacheKey
        rustc = (& rustc -vV) -join "`n"
        imageOS = $env:ImageOS
        imageVersion = $env:ImageVersion
        createdAt = (Get-Date).ToUniversalTime().ToString('o')
      } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $temporary $manifestName) -Encoding utf8
      Assert-OpenSslInstall $temporary
      Move-Item -LiteralPath $temporary -Destination $CacheDir
    } finally {
      Remove-Item -LiteralPath $temporary -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "Populated static OpenSSL cache from '$source'."
  }

  'verify' {
    Assert-OpenSslInstall $CacheDir
    Write-Host "Verified static OpenSSL cache at '$CacheDir'."
  }
}
