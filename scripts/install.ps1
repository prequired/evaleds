# EvalEds Installation Script for Windows (PowerShell)
# 
# This script installs EvalEds on Windows systems.
# Run with: powershell -ExecutionPolicy Bypass -File install.ps1

param(
    [switch]$BuildFromSource,
    [string]$InstallDir = "",
    [switch]$Help
)

# Colors for output (if supported)
function Write-Status { 
    Write-Host "==> $args" -ForegroundColor Blue 
}

function Write-Success { 
    Write-Host "✅ $args" -ForegroundColor Green 
}

function Write-Warning { 
    Write-Host "⚠️ $args" -ForegroundColor Yellow 
}

function Write-Error { 
    Write-Host "❌ $args" -ForegroundColor Red 
}

function Write-Info { 
    Write-Host "💡 $args" -ForegroundColor Cyan 
}

# Configuration
$RepoUrl = "https://github.com/prequired/evaleds"
$BinaryName = "evaleds.exe"
$DefaultInstallDir = "$env:USERPROFILE\.local\bin"
$ConfigDir = "$env:APPDATA\evaleds"

if ($Help) {
    Write-Host "EvalEds Installation Script for Windows"
    Write-Host ""
    Write-Host "Usage: powershell -ExecutionPolicy Bypass -File install.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -BuildFromSource    Force build from source instead of downloading binary"
    Write-Host "  -InstallDir DIR     Custom installation directory"
    Write-Host "  -Help               Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install.ps1                    # Download and install binary"
    Write-Host "  .\install.ps1 -BuildFromSource   # Build from source"
    Write-Host "  .\install.ps1 -InstallDir C:\tools\bin"
    exit 0
}

# Set install directory
if ($InstallDir -eq "") {
    $InstallDir = $DefaultInstallDir
}

Write-Host ""
Write-Host "🎯 EvalEds Installation Script for Windows" -ForegroundColor Cyan
Write-Host "   AI evaluation platform with PromptEds integration" -ForegroundColor Cyan
Write-Host ""

Write-Status "Installing to: $InstallDir"
Write-Status "Configuration: $ConfigDir"

# Create directories
Write-Status "Creating directories..."
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

if (!(Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    
    # Create default configuration
    $defaultConfig = @"
# EvalEds Configuration File
# See https://github.com/prequired/evaleds for documentation

[defaults]
temperature = 0.7
max_tokens = 1000
timeout_seconds = 120
max_concurrent = 5
retry_attempts = 3

[analysis]
enable_similarity_analysis = true
enable_content_analysis = true
enable_quality_assessment = true
similarity_threshold = 0.7
max_keywords = 10

# Provider configurations will be loaded from environment variables
# Set OPENAI_API_KEY, ANTHROPIC_API_KEY, GOOGLE_API_KEY as needed
"@
    
    $defaultConfig | Out-File -FilePath "$ConfigDir\config.toml" -Encoding UTF8
    Write-Success "Created default configuration at $ConfigDir\config.toml"
}

# Function to get latest release version
function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/prequired/evaleds/releases/latest"
        return $response.tag_name
    }
    catch {
        return $null
    }
}

# Function to download and install binary
function Install-Binary {
    $version = Get-LatestVersion
    
    if ($null -eq $version) {
        Write-Warning "Could not determine latest version, building from source..."
        Build-FromSource
        return
    }
    
    Write-Status "Installing EvalEds $version for Windows"
    
    $downloadUrl = "$RepoUrl/releases/download/$version/evaleds-$version-windows-x86_64.zip"
    $tempFile = "$env:TEMP\evaleds.zip"
    $tempDir = "$env:TEMP\evaleds-install"
    
    # Clean temp directory
    if (Test-Path $tempDir) {
        Remove-Item $tempDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    
    try {
        Write-Status "Downloading from $downloadUrl"
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -ErrorAction Stop
        
        # Extract and install
        Write-Status "Extracting and installing binary..."
        Expand-Archive -Path $tempFile -DestinationPath $tempDir -Force
        
        $binaryPath = Get-ChildItem -Path $tempDir -Name $BinaryName -Recurse | Select-Object -First 1
        if ($binaryPath) {
            $sourcePath = Join-Path $tempDir $binaryPath
            $destPath = Join-Path $InstallDir $BinaryName
            Copy-Item $sourcePath $destPath -Force
            Write-Success "Binary installed to $destPath"
        } else {
            Write-Error "Binary not found in archive"
            Build-FromSource
            return
        }
    }
    catch {
        Write-Warning "Binary download failed: $_"
        Write-Warning "Building from source..."
        Build-FromSource
        return
    }
    finally {
        # Cleanup
        if (Test-Path $tempFile) { Remove-Item $tempFile -Force }
        if (Test-Path $tempDir) { Remove-Item $tempDir -Recurse -Force }
    }
}

# Function to build from source
function Build-FromSource {
    Write-Status "Building EvalEds from source..."
    
    # Check for Rust/Cargo
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Cargo (Rust) is required to build from source"
        Write-Info "Install Rust from: https://rustup.rs/"
        exit 1
    }
    
    # Check for Git
    if (!(Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Error "Git is required to clone the repository"
        exit 1
    }
    
    $tempDir = "$env:TEMP\evaleds-build"
    
    # Clean temp directory
    if (Test-Path $tempDir) {
        Remove-Item $tempDir -Recurse -Force
    }
    
    try {
        # Clone repository
        Write-Status "Cloning repository..."
        git clone "$RepoUrl.git" $tempDir
        
        Set-Location $tempDir
        
        # Build release binary
        Write-Status "Building release binary... (this may take a few minutes)"
        cargo build --release
        
        # Install binary
        $builtBinary = "target\release\$($BinaryName)"
        if (Test-Path $builtBinary) {
            $destPath = Join-Path $InstallDir $BinaryName
            Copy-Item $builtBinary $destPath -Force
            Write-Success "Binary built and installed to $destPath"
        } else {
            Write-Error "Build failed - binary not found"
            exit 1
        }
    }
    catch {
        Write-Error "Build failed: $_"
        exit 1
    }
    finally {
        # Cleanup
        Set-Location $env:USERPROFILE
        if (Test-Path $tempDir) {
            Remove-Item $tempDir -Recurse -Force
        }
    }
}

# Function to update PATH
function Update-Path {
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    
    if ($currentPath -notlike "*$InstallDir*") {
        Write-Warning "$InstallDir is not in your PATH"
        
        try {
            $newPath = "$InstallDir;$currentPath"
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            Write-Success "Added $InstallDir to your PATH"
            Write-Info "Restart your terminal or run: refreshenv"
        }
        catch {
            Write-Warning "Could not automatically update PATH"
            Write-Info "Please manually add $InstallDir to your PATH environment variable"
        }
    }
}

# Function to verify installation
function Test-Installation {
    $binaryPath = Join-Path $InstallDir $BinaryName
    
    if (Test-Path $binaryPath) {
        Write-Success "EvalEds installed successfully!"
        
        # Test the binary
        try {
            $version = & $binaryPath --version 2>$null
            Write-Success "Version: $version"
        }
        catch {
            Write-Warning "Could not verify version (PATH may not be updated yet)"
        }
        
        Write-Info "Configuration directory: $ConfigDir"
        Write-Info "Binary location: $binaryPath"
        
        Write-Host ""
        Write-Info "Get started with:"
        Write-Host "  evaleds create my-first-evaluation --interactive"
        Write-Host "  evaleds --help"
        Write-Host ""
        Write-Info "Documentation: $RepoUrl#readme"
    } else {
        Write-Error "Installation verification failed"
        exit 1
    }
}

# Main installation
try {
    if ($BuildFromSource) {
        Build-FromSource
    } else {
        Install-Binary
    }
    
    Update-Path
    Test-Installation
    
    Write-Success "🚀 EvalEds installation complete!"
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}