$ErrorActionPreference = "Stop"

$DistDir = "dist\game-lens"
if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

Copy-Item "target\release\game-lens.exe" -Destination $DistDir
Copy-Item "config.toml" -Destination $DistDir
Copy-Item "Start.ps1" -Destination $DistDir

if (Test-Path "dist\game-lens.7z") { Remove-Item -Force "dist\game-lens.7z" }
& 7z.exe a -t7z -mx7 "dist\game-lens.7z" ".\$DistDir\*"
