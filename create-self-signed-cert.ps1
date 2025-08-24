# Create Self-Signed Certificate for MockMate
# Run this as Administrator

# Create certificate
$cert = New-SelfSignedCertificate -Subject "CN=MockMate App" -Type CodeSigningCert -KeyUsage DigitalSignature -FriendlyName "MockMate Code Signing" -CertStoreLocation "Cert:\CurrentUser\My" -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}false")

# Get thumbprint
$thumbprint = $cert.Thumbprint
Write-Host "Certificate created with thumbprint: $thumbprint"

# Export certificate (optional)
$certPath = ".\mockmate-cert.pfx"
$password = ConvertTo-SecureString -String "MockMate2024!" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath $certPath -Password $password

Write-Host "Certificate exported to: $certPath"
Write-Host "Password: MockMate2024!"
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Copy this thumbprint to your tauri.conf.json:"
Write-Host "   `"certificateThumbprint`": `"$thumbprint`""
Write-Host ""
Write-Host "2. Set environment variable:"
Write-Host "   `$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=`"MockMate2024!`""
Write-Host ""
Write-Host "3. Build your app:"
Write-Host "   npm run tauri build"
