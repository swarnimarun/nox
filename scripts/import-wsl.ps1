param(
    [Parameter(Mandatory=$true)][string]$Image,
    [Parameter(Mandatory=$true)][string]$InstallLocation,
    [string]$Name = 'Nox'
)
$ErrorActionPreference = 'Stop'
$ResolvedImage = (Resolve-Path -LiteralPath $Image).Path
if (Test-Path -LiteralPath $InstallLocation) {
    throw 'Use a new installation directory; existing directories are not overwritten.'
}
# WSL refuses an existing distribution name. Never unregister automatically.
& wsl.exe --import $Name $InstallLocation $ResolvedImage --version 2
if ($LASTEXITCODE -ne 0) { throw "WSL import failed ($LASTEXITCODE)" }
Write-Host "Start with: wsl -d $Name"
