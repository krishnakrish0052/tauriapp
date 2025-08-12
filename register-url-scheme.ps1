# Register MockMate URL Scheme
# This script manually registers the mockmate:// URL scheme with Windows

param(
    [switch]$Unregister = $false
)

$exePath = Join-Path $PSScriptRoot "src-tauri\target\release\mockmate.exe"
$registryPath = "HKCU:\SOFTWARE\Classes\mockmate"
$commandPath = "$registryPath\shell\open\command"

Write-Host "MockMate URL Scheme Registration Tool" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

if ($Unregister) {
    Write-Host "Unregistering mockmate:// URL scheme..." -ForegroundColor Yellow
    
    if (Test-Path $registryPath) {
        Remove-Item -Path $registryPath -Recurse -Force
        Write-Host "✅ URL scheme unregistered successfully" -ForegroundColor Green
    } else {
        Write-Host "ℹ️ URL scheme was not registered" -ForegroundColor Yellow
    }
} else {
    # Check if executable exists
    if (-not (Test-Path $exePath)) {
        Write-Host "❌ MockMate executable not found at: $exePath" -ForegroundColor Red
        Write-Host "Please build the app first with: npm run build" -ForegroundColor Yellow
        exit 1
    }

    Write-Host "Registering mockmate:// URL scheme..." -ForegroundColor Green
    Write-Host "Executable: $exePath" -ForegroundColor Gray

    try {
        # Create registry entries
        New-Item -Path $registryPath -Force | Out-Null
        Set-ItemProperty -Path $registryPath -Name "(Default)" -Value "URL:MockMate Protocol"
        Set-ItemProperty -Path $registryPath -Name "URL Protocol" -Value ""

        New-Item -Path "$registryPath\shell" -Force | Out-Null
        New-Item -Path "$registryPath\shell\open" -Force | Out-Null
        New-Item -Path $commandPath -Force | Out-Null
        Set-ItemProperty -Path $commandPath -Name "(Default)" -Value "`"$exePath`" `"%1`""

        Write-Host "✅ URL scheme registered successfully!" -ForegroundColor Green
        Write-Host ""
        
        # Test the registration
        Write-Host "Testing URL scheme registration..." -ForegroundColor Cyan
        $testResult = Get-ItemProperty -Path $commandPath -Name "(Default)" -ErrorAction SilentlyContinue
        if ($testResult) {
            Write-Host "✅ Registry entry confirmed: $($testResult.'(Default)')" -ForegroundColor Green
        } else {
            Write-Host "⚠️ Could not verify registry entry" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "You can now test with: Start-Process 'mockmate://session/test-id'" -ForegroundColor Cyan
        
    } catch {
        Write-Host "❌ Failed to register URL scheme: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "Usage:" -ForegroundColor Yellow
Write-Host "  Register:   .\register-url-scheme.ps1" -ForegroundColor Gray
Write-Host "  Unregister: .\register-url-scheme.ps1 -Unregister" -ForegroundColor Gray
