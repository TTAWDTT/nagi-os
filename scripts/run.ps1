$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Image = Join-Path $Root "build\nagi-os.img"

function Convert-ToWslPath {
    param([string]$Path)
    $FullPath = [System.IO.Path]::GetFullPath($Path)
    if ($FullPath -match '^([A-Za-z]):\\(.*)$') {
        $Drive = $Matches[1].ToLowerInvariant()
        $Rest = $Matches[2] -replace '\\', '/'
        return "/mnt/$Drive/$Rest"
    }
    throw "Cannot convert path to WSL path: $Path"
}

if (-not (Test-Path $Image)) {
    & (Join-Path $PSScriptRoot "build.ps1")
}

$ImageWsl = Convert-ToWslPath $Image
wsl --exec bash -lc "qemu-system-x86_64 -drive file='$ImageWsl',format=raw,if=floppy -boot a"
