$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
& (Join-Path $PSScriptRoot "build.ps1")

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

$ImageWsl = Convert-ToWslPath $Image
wsl --exec bash -lc "timeout 5 qemu-system-x86_64 -drive file='$ImageWsl',format=raw,if=floppy -boot a -display none -monitor none -serial stdio -no-reboot; code=`$?; if [ `$code -eq 124 ]; then echo QEMU_STILL_RUNNING_AFTER_5S; else echo QEMU_EXITED_`$code; fi"
