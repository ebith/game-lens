$ErrorActionPreference = "Stop"

$DistDir = "dist\game-lens"
$Archive = "dist\game-lens.7z"
$Tag = "v" + (Get-Date -Format "yyyyMMdd")

cargo build --release

if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

Copy-Item "target\release\game-lens.exe" -Destination $DistDir
Copy-Item "config.toml" -Destination $DistDir
Copy-Item "Start.ps1" -Destination $DistDir

if (Test-Path $Archive) { Remove-Item -Force $Archive }
& 7z.exe a -t7z -mx7 $Archive ".\$DistDir\*"

gh release create $Tag $Archive --generate-notes
