# PowerShell script to sign MockMate executables
$signtool = "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe"
$certThumbprint = "5436D3EC98DBD6D237F46484FABBF52414054C15"
$timestampUrl = "http://timestamp.sectigo.com"

# Find the built executables
$exePath = "src-tauri\target\release\mockmate.exe"
$msiPath = "src-tauri\target\release\bundle\msi\MockMate_0.1.0_x64_en-US.msi"
$nsisPath = "src-tauri\target\release\bundle\nsis\MockMate_0.1.0_x64-setup.exe"

Write-Host "Signing MockMate executables with self-signed certificate..."

# Sign the main executable
if (Test-Path $exePath) {
    Write-Host "Signing main executable: $exePath"
    & $signtool sign /sha1 $certThumbprint /t $timestampUrl /fd sha256 $exePath
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Successfully signed main executable" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to sign main executable" -ForegroundColor Red
    }
} else {
    Write-Host "⚠️ Main executable not found: $exePath" -ForegroundColor Yellow
}

# Sign the MSI installer
if (Test-Path $msiPath) {
    Write-Host "Signing MSI installer: $msiPath"
    & $signtool sign /sha1 $certThumbprint /t $timestampUrl /fd sha256 $msiPath
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Successfully signed MSI installer" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to sign MSI installer" -ForegroundColor Red
    }
} else {
    Write-Host "⚠️ MSI installer not found: $msiPath" -ForegroundColor Yellow
}

# Sign the NSIS installer
if (Test-Path $nsisPath) {
    Write-Host "Signing NSIS installer: $nsisPath"
    & $signtool sign /sha1 $certThumbprint /t $timestampUrl /fd sha256 $nsisPath
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Successfully signed NSIS installer" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to sign NSIS installer" -ForegroundColor Red
    }
} else {
    Write-Host "⚠️ NSIS installer not found: $nsisPath" -ForegroundColor Yellow
}

Write-Host "`nSigning process completed!"
Write-Host "Note: Self-signed certificates will show security warnings to users."
Write-Host "For production apps, consider getting a certificate from a trusted CA."
