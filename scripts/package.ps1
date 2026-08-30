param([string]$Version = "0.3.0")
$ErrorActionPreference = "Stop"
$cargoPath = (Get-Command cargo -ErrorAction Stop).Source
& $cargoPath build --release --locked --target wasm32-wasip2
New-Item -ItemType Directory -Force -Path dist | Out-Null
Copy-Item -LiteralPath target\wasm32-wasip2\release\calabaza_factions.wasm -Destination dist\CalabazaFactions.wasm -Force
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath dist\CalabazaFactions.wasm).Hash.ToLowerInvariant()
Set-Content -LiteralPath dist\SHA256SUMS.txt -Value "$hash  CalabazaFactions.wasm" -Encoding ascii
Write-Output "CalabazaFactions v$Version packaged"
Get-Item dist\CalabazaFactions.wasm | Select-Object Name, Length
