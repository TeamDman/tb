$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$svgPath = Join-Path $scriptDir 'main.svg'
$pngPath = Join-Path $scriptDir 'main.png'

if (-not (Get-Command magick -ErrorAction SilentlyContinue)) {
    throw 'ImageMagick (magick) is required but was not found on PATH.'
}

if (-not (Test-Path $svgPath)) {
    throw "Input SVG not found: $svgPath"
}

magick $svgPath -background none -alpha on -resize 256x256 "PNG32:$pngPath"
magick $pngPath -alpha on -fuzz 2% -transparent white "PNG32:$pngPath"

Write-Host "Rendered $pngPath from $svgPath"
