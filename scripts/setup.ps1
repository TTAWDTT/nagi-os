$ErrorActionPreference = "Stop"

$Target = "x86_64-unknown-none"

function Add-CargoPath {
    $CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if ((Test-Path -LiteralPath $CargoBin) -and ($env:Path -notlike "*$CargoBin*")) {
        $env:Path = "$CargoBin;$env:Path"
    }
}

function Ensure-Rustup {
    Add-CargoPath
    if (Get-Command rustup -ErrorAction SilentlyContinue) {
        return
    }

    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw "rustup is missing and winget is unavailable. Install Rust from https://rustup.rs/ and rerun this script."
    }

    Write-Host "Installing rustup with winget..."
    winget install --id Rustlang.Rustup -e --source winget --accept-package-agreements --accept-source-agreements
    Add-CargoPath

    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        throw "rustup was installed, but it is not on PATH yet. Open a new PowerShell window and rerun this script."
    }
}

function Ensure-Wsl {
    if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
        throw "WSL is missing. Run 'wsl --install -d Ubuntu', reboot if requested, then rerun this script."
    }

    wsl --exec bash -lc "echo WSL_READY" | Out-Null
}

function Ensure-WslPackages {
    Write-Host "Installing WSL build tools: nasm qemu-system-x86 binutils..."
    wsl --exec bash -lc "set -e; if ! command -v apt-get >/dev/null 2>&1; then echo 'This setup script expects an apt-based WSL distro such as Ubuntu.' >&2; exit 1; fi; sudo apt-get update; sudo apt-get install -y nasm qemu-system-x86 binutils"

    wsl --exec bash -lc "set -e; command -v nasm >/dev/null; command -v qemu-system-x86_64 >/dev/null; command -v objcopy >/dev/null"
}

Ensure-Rustup
Write-Host "Installing Rust target: $Target..."
rustup target add $Target

Ensure-Wsl
Ensure-WslPackages

Write-Host ""
Write-Host "Nagi OS toolchain is ready."
Write-Host "Verify with:"
Write-Host "  .\scripts\build.ps1"
Write-Host "  .\scripts\smoke.ps1"
Write-Host "  .\scripts\run.ps1"
