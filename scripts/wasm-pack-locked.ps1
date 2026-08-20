[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$WasmPackArguments
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-Application([string]$Name) {
    return (Get-Command $Name -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
}

if ([string]::IsNullOrWhiteSpace($env:CARGO)) {
    $realCargo = Resolve-Application 'cargo'
} elseif ([System.IO.Path]::IsPathRooted($env:CARGO) -or $env:CARGO -match '[\\/]') {
    $realCargo = $env:CARGO
} else {
    $realCargo = Resolve-Application $env:CARGO
}
$wasmPack = Resolve-Application 'wasm-pack'
$rustc = Resolve-Application 'rustc'
$shimDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("rhwp-wasm-pack-$([Guid]::NewGuid().ToString('N'))")
$shimSource = Join-Path $shimDirectory 'cargo-proxy.rs'
$shimCargo = Join-Path $shimDirectory 'cargo.exe'
$shimWasmPack = Join-Path $shimDirectory (Split-Path -Leaf $wasmPack)
$previousPath = $env:PATH
$hadRealCargo = Test-Path Env:RHWP_WASM_PACK_REAL_CARGO
$previousRealCargo = $env:RHWP_WASM_PACK_REAL_CARGO
$exitCode = 1

New-Item -ItemType Directory -Path $shimDirectory -Force | Out-Null
@'
use std::env;
use std::process::{exit, Command};

fn main() {
    let real_cargo = env::var_os("RHWP_WASM_PACK_REAL_CARGO")
        .expect("RHWP_WASM_PACK_REAL_CARGO must name the real cargo executable");
    let mut arguments: Vec<_> = env::args_os().skip(1).collect();
    let is_metadata = arguments
        .first()
        .map(|argument| argument == "metadata")
        .unwrap_or(false);
    let has_locked = arguments.iter().any(|argument| argument == "--locked");
    if is_metadata && !has_locked {
        arguments.push("--locked".into());
    }

    let status = Command::new(real_cargo)
        .args(arguments)
        .status()
        .expect("failed to start the real cargo executable");
    exit(status.code().unwrap_or(1));
}
'@ | Set-Content -LiteralPath $shimSource -Encoding Ascii
& $rustc $shimSource '--edition=2021' '-O' '-o' $shimCargo
if ($LASTEXITCODE -ne 0) {
    throw "failed to build the temporary cargo.exe proxy: $LASTEXITCODE"
}
Copy-Item -LiteralPath $wasmPack -Destination $shimWasmPack

try {
    $arguments = @('build') + $WasmPackArguments
    if ($arguments -notcontains '--locked') {
        $arguments += '--locked'
    }
    $env:PATH = "$shimDirectory;$previousPath"
    $env:RHWP_WASM_PACK_REAL_CARGO = $realCargo
    # Windows searches the wasm-pack executable directory before PATH. Run a
    # temporary sibling copy so cargo_metadata resolves this proxy cargo.exe.
    & $shimWasmPack @arguments
    $exitCode = $LASTEXITCODE
} finally {
    $env:PATH = $previousPath
    if ($hadRealCargo) {
        $env:RHWP_WASM_PACK_REAL_CARGO = $previousRealCargo
    } else {
        Remove-Item Env:RHWP_WASM_PACK_REAL_CARGO -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $shimDirectory -Recurse -Force -ErrorAction SilentlyContinue
}

exit $exitCode
