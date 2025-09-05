# Code Signing Setup for MockMate

## Current Setup (Self-Signed Certificate)

✅ Your MockMate app is now signed with a self-signed certificate!

### Files created:
- `mockmate-cert.pfx` - The certificate file (password: `mockmate2024`)
- `sign-app.ps1` - PowerShell script to sign executables
- Certificate installed in Windows certificate store (thumbprint: `5436D3EC98DBD6D237F46484FABBF52414054C15`)

### What this means:
- ✅ Your app is digitally signed
- ⚠️ Windows will show security warnings because it's self-signed
- ⚠️ Windows Defender SmartScreen may block the app initially

## How to use:

### Automatic signing during build:
Your `tauri.conf.json` is configured to automatically sign during build:
```bash
cargo tauri build
```

### Manual signing:
Run the PowerShell script after building:
```powershell
.\sign-app.ps1
```

## For Production Use

For a production app, you should get a certificate from a trusted Certificate Authority (CA):

### Option 1: Free SSL Certificate (for open source projects)
Some CAs offer free certificates for open source projects.

### Option 2: Commercial Code Signing Certificate (~$200-400/year)
- **DigiCert** - Industry standard, trusted by all major platforms
- **Sectigo** - Cost-effective option
- **GlobalSign** - Good reputation
- **Entrust** - Enterprise focused

### Option 3: EV Code Signing Certificate (~$300-600/year)
- Enhanced Validation certificates
- Highest trust level
- No SmartScreen warnings from day one
- Requires hardware token (USB key)

### Steps for commercial certificate:
1. Purchase certificate from a CA
2. Complete identity verification process
3. Receive certificate file (.p12 or .pfx)
4. Update `tauri.conf.json` with new certificate thumbprint
5. Store certificate password securely (environment variable)

## Environment Variables for Production

For production builds, set these environment variables instead of hardcoding:
```bash
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=your_cert_password
TAURI_SIGNING_PRIVATE_KEY=path_to_your_cert.pfx
```

## Troubleshooting

### Windows SmartScreen Warning
- This is normal for self-signed certificates
- Users need to click "More info" → "Run anyway"
- Will disappear once you get a trusted certificate

### Certificate not found error
- Make sure certificate is in Windows certificate store
- Verify thumbprint matches in `tauri.conf.json`
- Check that SignTool is installed

### Signature verification
Check if your app is signed:
```powershell
Get-AuthenticodeSignature "path\to\your\app.exe"
```

## Security Notes

- 🔐 Keep your certificate password secure
- 🔒 Store certificates in secure locations
- ⏰ Monitor certificate expiration dates
- 🔄 Plan certificate renewal process

## Next Steps

1. Test your signed app on clean Windows machines
2. Monitor user feedback about security warnings
3. Consider upgrading to commercial certificate for production
4. Set up automated signing in CI/CD pipeline
