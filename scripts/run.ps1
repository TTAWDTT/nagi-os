param(
    [ValidateSet("sdl", "gtk", "curses", "serial", "none")]
    [string]$Display = "curses"
)

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
$DisplayMode = $Display
if ($DisplayMode -eq "none") {
    $DisplayMode = "serial"
}

$QemuArgs = "-drive file='$ImageWsl',format=raw,if=floppy,snapshot=on -boot a"
if ($DisplayMode -eq "serial") {
    $QemuArgs = "$QemuArgs -display none -serial stdio -monitor none -no-reboot"
} else {
    $QemuArgs = "$QemuArgs -display $DisplayMode"
}

wsl --exec bash -lc "qemu-system-x86_64 $QemuArgs"
