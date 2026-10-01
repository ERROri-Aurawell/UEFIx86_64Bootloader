cargo build

if ($LASTEXITCODE -ne 0) {
    Write-Error "A compilação falhou : $LASTEXITCODE!"
    exit $LASTEXITCODE
}

$src = ".\target\x86_64-unknown-uefi\debug\BOOTX64.efi"
$dest = "..\..\saborDiscoFAT\EFI\BOOT"

Copy-Item -Path $src -Destination $dest -Force
Write-Host "Binário copiado"

..\..\iniciarSaporra.ps1