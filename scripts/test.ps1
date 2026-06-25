param(
    [Parameter(Mandatory=$true)]
    [string]$RomfsDir
)

$env:PITA_ROMFS_DIR = $RomfsDir
Write-Host "PITA_ROMFS_DIR = $env:PITA_ROMFS_DIR"
cargo test --release @args
