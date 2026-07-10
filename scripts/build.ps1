$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Build = Join-Path $Root "build"
$Target = "x86_64-unknown-none"
$KernelElf = Join-Path $Root "target\$Target\release\nagi-kernel"
$KernelBin = Join-Path $Build "kernel.bin"
$Stage1Bin = Join-Path $Build "stage1.bin"
$Stage2Bin = Join-Path $Build "stage2.bin"
$Image = Join-Path $Build "nagi-os.img"

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

New-Item -ItemType Directory -Force -Path $Build | Out-Null

Push-Location $Root
try {
    cargo build --release --target $Target

    $KernelElfPath = (Resolve-Path $KernelElf).Path
    $KernelElfWsl = Convert-ToWslPath $KernelElfPath
    $KernelBinWsl = Convert-ToWslPath $KernelBin
    $Stage1AsmWsl = Convert-ToWslPath (Join-Path $Root "boot\stage1.asm")
    $Stage2AsmWsl = Convert-ToWslPath (Join-Path $Root "boot\stage2.asm")
    $Stage1BinWsl = Convert-ToWslPath $Stage1Bin
    $Stage2BinWsl = Convert-ToWslPath $Stage2Bin
    $ImageWsl = Convert-ToWslPath $Image

    wsl --exec bash -lc "objcopy -O binary '$KernelElfWsl' '$KernelBinWsl'"
    $KernelSize = (Get-Item $KernelBin).Length
    $KernelSectors = [int][Math]::Ceiling($KernelSize / 512.0)
    $PaddedSize = $KernelSectors * 512

    wsl --exec bash -lc "truncate -s $PaddedSize '$KernelBinWsl'"
    wsl --exec bash -lc "nasm -f bin '$Stage1AsmWsl' -o '$Stage1BinWsl'"
    wsl --exec bash -lc "nasm -f bin -D__KERNEL_SECTORS__=$KernelSectors '$Stage2AsmWsl' -o '$Stage2BinWsl'"
    wsl --exec bash -lc "dd if=/dev/zero of='$ImageWsl' bs=512 count=2880 status=none"
    wsl --exec bash -lc "dd if='$Stage1BinWsl' of='$ImageWsl' conv=notrunc status=none"
    wsl --exec bash -lc "dd if='$Stage2BinWsl' of='$ImageWsl' bs=512 seek=1 conv=notrunc status=none"
    wsl --exec bash -lc "dd if='$KernelBinWsl' of='$ImageWsl' bs=512 seek=9 conv=notrunc status=none"

    Write-Host "Built $Image"
    Write-Host "Kernel size: $KernelSize bytes ($KernelSectors sectors)"
}
finally {
    Pop-Location
}
