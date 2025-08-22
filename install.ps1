#Requires -Version 5.1

<#
.SYNOPSIS
    nu_plugin_secret Installation Script for Windows

.DESCRIPTION
    This script automatically downloads and installs the latest release
    of nu_plugin_secret for Windows.

.PARAMETER InstallDir
    Directory to install the binary (default: $env:USERPROFILE\.cargo\bin)

.PARAMETER SkipRegister
    Skip automatic plugin registration with Nushell

.PARAMETER Help
    Show help information

.EXAMPLE
    .\install.ps1
    Install to default location and register with Nushell

.EXAMPLE
    .\install.ps1 -InstallDir "C:\Tools"
    Install to C:\Tools

.EXAMPLE
    .\install.ps1 -SkipRegister
    Install but don't register with Nushell
#>

param(
    [string]$InstallDir = "$env:USERPROFILE\.cargo\bin",
    [switch]$SkipRegister,
    [switch]$Help
)

# GitHub repository configuration
$REPO = "nushell-works/nu_plugin_secret"
$BINARY_NAME = "nu_plugin_secret"

# Function to write colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Show help if requested
if ($Help) {
    Get-Help $PSCommandPath -Detailed
    exit 0
}

# Detect platform architecture
function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64-pc-windows-msvc" }
        "ARM64" { return "aarch64-pc-windows-msvc" }
        default {
            Write-Error "Unsupported architecture: $arch"
            exit 1
        }
    }
}

# Get the latest release version
function Get-LatestVersion {
    Write-Status "Fetching latest release information..."
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
        return $response.tag_name
    }
    catch {
        Write-Error "Failed to get latest version: $_"
        exit 1
    }
}

# Download and install the binary
function Install-Binary {
    param(
        [string]$Version,
        [string]$Platform,
        [string]$TargetDir
    )
    
    $filename = "$BINARY_NAME-$Version-$Platform.zip"
    $downloadUrl = "https://github.com/$REPO/releases/download/$Version/$filename"
    
    Write-Status "Downloading $filename..."
    
    # Create temporary directory
    $tempDir = Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    
    try {
        # Download file
        $zipPath = Join-Path $tempDir $filename
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
        
        Write-Status "Extracting binary..."
        
        # Extract zip
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force
        
        # Create install directory if it doesn't exist
        if (!(Test-Path $TargetDir)) {
            New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
        }
        
        # Copy binary
        $binaryName = "$BINARY_NAME.exe"
        $sourcePath = Join-Path $tempDir $binaryName
        $targetPath = Join-Path $TargetDir $binaryName
        
        Copy-Item $sourcePath $targetPath -Force
        
        Write-Success "Binary installed to $targetPath"
        return $targetPath
    }
    finally {
        # Cleanup
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Register plugin with Nushell
function Register-Plugin {
    param([string]$BinaryPath)
    
    # Check if nushell is available
    if (!(Get-Command nu -ErrorAction SilentlyContinue)) {
        Write-Warning "Nushell (nu) not found in PATH. Please install Nushell first."
        Write-Status "You can install Nushell from: https://nushell.sh/book/installation.html"
        return $false
    }
    
    Write-Status "Registering plugin with Nushell..."
    
    # Initialize nushell config directory
    $configDir = Join-Path $env:APPDATA "nushell"
    if (!(Test-Path $configDir)) {
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    }
    
    try {
        # Register the plugin
        $registerResult = & nu -c "plugin add '$BinaryPath'" 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Plugin registered successfully!"
            
            # Activate the plugin
            Write-Status "Activating plugin..."
            $useResult = & nu -c "plugin use secret" 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Plugin activated successfully!"
            }
            else {
                Write-Warning "Failed to activate plugin. You may need to run 'plugin use secret' manually."
            }
            
            # Test basic functionality
            Write-Status "Testing basic functionality..."
            $testResult = & nu -c 'echo "test" | secret wrap-string' 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Plugin is working correctly!"
            }
            else {
                Write-Warning "Plugin test failed. Please check your installation."
            }
            
            return $true
        }
        else {
            Write-Error "Failed to register plugin with Nushell."
            Write-Status "Try running manually: nu -c 'plugin add $BinaryPath'"
            return $false
        }
    }
    catch {
        Write-Error "Error during plugin registration: $_"
        return $false
    }
}

# Add directory to PATH if not already there
function Add-ToPath {
    param([string]$Directory)
    
    $currentPath = $env:PATH
    if ($currentPath -notlike "*$Directory*") {
        Write-Warning "$Directory is not in your PATH."
        Write-Status "Consider adding it to your PATH environment variable."
        
        # Offer to add to user PATH
        $response = Read-Host "Would you like to add $Directory to your user PATH? (y/N)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            try {
                $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
                if ($userPath) {
                    $newPath = "$userPath;$Directory"
                } else {
                    $newPath = $Directory
                }
                [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
                Write-Success "Added $Directory to user PATH. Please restart your terminal."
            }
            catch {
                Write-Error "Failed to update PATH: $_"
            }
        }
    }
}

# Main installation function
function Main {
    Write-Host "🔐 nu_plugin_secret Installation Script" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host ""
    
    # Detect platform
    Write-Status "Detecting platform..."
    $platform = Get-Platform
    Write-Success "Detected platform: $platform"
    
    # Get latest version
    $version = Get-LatestVersion
    if (!$version) {
        Write-Error "Failed to get latest version information"
        exit 1
    }
    Write-Success "Latest version: $version"
    
    # Download and install
    $binaryPath = Install-Binary -Version $version -Platform $platform -TargetDir $InstallDir
    
    # Add to PATH if needed
    Add-ToPath -Directory $InstallDir
    
    # Register with Nushell unless skipped
    if (!$SkipRegister) {
        $registerSuccess = Register-Plugin -BinaryPath $binaryPath
    }
    else {
        Write-Status "Skipping plugin registration (-SkipRegister specified)"
        Write-Status "To register manually, run: nu -c 'plugin add $binaryPath'"
    }
    
    Write-Host ""
    Write-Success "Installation completed successfully! 🎉"
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "1. Restart your terminal or reload your environment"
    Write-Host "2. Run 'nu' to start Nushell"
    Write-Host "3. Try: echo `"secret`" | secret wrap-string"
    Write-Host ""
    Write-Host "For more information, visit: https://github.com/$REPO" -ForegroundColor Blue
}

# Run main function
try {
    Main
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}