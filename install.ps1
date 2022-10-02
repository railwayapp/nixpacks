#!/usr/bin/env pwsh
# Copyright 2018 the Deno authors. All rights reserved. MIT license.
# TODO(everyone): Keep this script simple and easily auditable.

$ErrorActionPreference = 'Stop'

if ($v) {
  $Version = "v${v}"
}
if ($args.Length -eq 1) {
  $Version = $args.Get(0)
}

$NixpacksInstall = $env:NIXPACKS_INSTALL
$BinDir = if ($NixpacksInstall) {
  "$NixpacksInstall\bin"
} else {
  "$Home\.nixpacks\bin"
}

$NixpacksZip = "$BinDir\nixpacks.zip"
$NixpacksExe = "$BinDir\nixpacks.exe"
$Target = 'x86_64-pc-windows-msvc'

# GitHub requires TLS 1.2
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$NixpacksUriObject = if (!$Version) {
    $Json = Invoke-RestMethod -Uri "https://api.github.com/repos/railwayapp/nixpacks/releases/latest"
    $LatestVersionJson = $Json | Get-Member -Name tag_name
    $LatestVersionSplit = [System.Management.Automation.LanguagePrimitives]::ConvertTo($LatestVersionJson, [string]).Split('=')
    $LatestVersion = $LatestVersionSplit[1]
  "https://github.com/railwayapp/nixpacks/releases/latest/download/nixpacks-${LatestVersion}-${Target}.zip"
} else {
  "https://github.com/railwayapp/nixpacks/releases/download/v${Version}/nixpacks-v${Version}-${Target}.zip"
}

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}
$NixpacksUri = ($NixpacksUriObject | Out-String).Trim()

curl.exe -Lo $NixpacksZip $NixpacksUri
Write-Output $NixpacksZip
tar.exe xf $NixpacksZip -C $BinDir

Remove-Item $NixpacksZip

$User = [EnvironmentVariableTarget]::User
$Path = [Environment]::GetEnvironmentVariable('Path', $User)
if (!(";$Path;".ToLower() -like "*;$BinDir;*".ToLower())) {
  [Environment]::SetEnvironmentVariable('Path', "$Path;$BinDir", $User)
  $Env:Path += ";$BinDir"
}

Write-Output "Nixpacks was installed successfully to $NixpacksExe"
Write-Output "Run 'nixpacks --help' to get started"