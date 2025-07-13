# EvalEds Uninstallation Script for Windows (PowerShell)
# 
# This script removes EvalEds from Windows systems.
# Run with: powershell -ExecutionPolicy Bypass -File uninstall.ps1

param(
    [switch]$RemoveConfig,
    [switch]$RemoveData,
    [switch]$RemoveAll,
    [switch]$Force,
    [switch]$DryRun,
    [switch]$Help
)

# Colors for output
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

function Write-DryRun {
    Write-Host "[DRY RUN] Would $args" -ForegroundColor Yellow
}

# Configuration
$BinaryName = "evaleds.exe"
$PossibleInstallDirs = @(
    "$env:USERPROFILE\.local\bin",
    "$env:USERPROFILE\bin",
    "$env:ProgramFiles\evaleds",
    "$env:LOCALAPPDATA\Programs\evaleds"
)
$ConfigDirs = @(
    "$env:APPDATA\evaleds",
    "$env:USERPROFILE\.config\evaleds",
    "$env:USERPROFILE\.evaleds"
)
$DataDirs = @(
    "$env:LOCALAPPDATA\evaleds",
    "$env:USERPROFILE\.local\share\evaleds",
    "$env:USERPROFILE\.evaleds"
)

if ($Help) {
    Write-Host "EvalEds Uninstallation Script for Windows"
    Write-Host ""
    Write-Host "Usage: powershell -ExecutionPolicy Bypass -File uninstall.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -RemoveConfig       Remove configuration files without prompting"
    Write-Host "  -RemoveData         Remove data files (evaluations, databases) without prompting"
    Write-Host "  -RemoveAll          Remove everything without prompting"
    Write-Host "  -Force              Skip all confirmation prompts (use with caution)"
    Write-Host "  -DryRun             Show what would be removed without actually removing"
    Write-Host "  -Help               Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\uninstall.ps1              # Interactive uninstall"
    Write-Host "  .\uninstall.ps1 -DryRun      # See what would be removed"
    Write-Host "  .\uninstall.ps1 -RemoveAll   # Remove everything including data"
    exit 0
}

# Apply -RemoveAll flag
if ($RemoveAll) {
    $RemoveConfig = $true
    $RemoveData = $true
}

# Function to ask for confirmation
function Confirm-Action {
    param([string]$Message, [bool]$DefaultYes = $false)
    
    if ($Force) {
        return $true
    }
    
    $prompt = if ($DefaultYes) { "$Message [Y/n]" } else { "$Message [y/N]" }
    
    do {
        $response = Read-Host "❓ $prompt"
        if ([string]::IsNullOrEmpty($response)) {
            $response = if ($DefaultYes) { "y" } else { "n" }
        }
        
        switch ($response.ToLower()) {
            { $_ -in @("y", "yes") } { return $true }
            { $_ -in @("n", "no") } { return $false }
            default { Write-Host "Please answer y or n." }
        }
    } while ($true)
}

# Function to find EvalEds binaries
function Find-Binaries {
    $foundPaths = @()
    
    foreach ($dir in $PossibleInstallDirs) {
        $binaryPath = Join-Path $dir $BinaryName
        if (Test-Path $binaryPath) {
            $foundPaths += $binaryPath
        }
    }
    
    # Also check PATH
    $pathBinary = Get-Command evaleds -ErrorAction SilentlyContinue
    if ($pathBinary -and $pathBinary.Source -notin $foundPaths) {
        $foundPaths += $pathBinary.Source
    }
    
    return $foundPaths
}

# Function to remove binaries
function Remove-Binaries {
    $binaries = Find-Binaries
    
    if ($binaries.Count -eq 0) {
        Write-Info "No EvalEds binaries found"
        return
    }
    
    Write-Status "Found EvalEds binaries:"
    foreach ($binary in $binaries) {
        Write-Host "  $binary"
    }
    
    if (Confirm-Action "Remove EvalEds binary files?" $true) {
        foreach ($binary in $binaries) {
            if ($DryRun) {
                Write-DryRun "remove $binary"
            } else {
                try {
                    Remove-Item $binary -Force
                    Write-Success "Removed $binary"
                }
                catch {
                    Write-Warning "Could not remove $binary`: $_"
                }
            }
        }
    } else {
        Write-Info "Skipping binary removal"
    }
}

# Function to remove configuration
function Remove-Configuration {
    $foundConfigs = @()
    
    foreach ($dir in $ConfigDirs) {
        if (Test-Path $dir) {
            $foundConfigs += $dir
        }
    }
    
    if ($foundConfigs.Count -eq 0) {
        Write-Info "No configuration directories found"
        return
    }
    
    if ($RemoveConfig -or (Confirm-Action "Remove configuration files and directories?" $false)) {
        foreach ($configDir in $foundConfigs) {
            if ($DryRun) {
                Write-DryRun "remove configuration directory $configDir"
            } else {
                try {
                    Remove-Item $configDir -Recurse -Force
                    Write-Success "Removed configuration directory $configDir"
                }
                catch {
                    Write-Warning "Could not remove $configDir`: $_"
                }
            }
        }
    } else {
        Write-Info "Preserving configuration files"
        Write-Host "Configuration files preserved in:"
        foreach ($configDir in $foundConfigs) {
            Write-Host "  $configDir"
        }
    }
}

# Function to remove data files
function Remove-Data {
    $foundData = @()
    
    foreach ($dir in $DataDirs) {
        if (Test-Path $dir) {
            $foundData += $dir
        }
    }
    
    # Also check for database files in config directories
    foreach ($dir in $ConfigDirs) {
        if (Test-Path $dir) {
            $dbFiles = Get-ChildItem -Path $dir -Filter "*.db*" -ErrorAction SilentlyContinue
            if ($dbFiles) {
                $foundData += "$dir\*.db*"
            }
        }
    }
    
    if ($foundData.Count -eq 0) {
        Write-Info "No data directories found"
        return
    }
    
    if ($RemoveData -or (Confirm-Action "Remove data files (evaluations, databases, exports)?" $false)) {
        foreach ($dataPath in $foundData) {
            if ($DryRun) {
                Write-DryRun "remove data files at $dataPath"
            } else {
                try {
                    if ($dataPath -like "*\*.db*") {
                        # Remove database files specifically
                        $dirPath = Split-Path $dataPath
                        Get-ChildItem -Path $dirPath -Filter "*.db*" | Remove-Item -Force
                        Write-Success "Removed database files from $dirPath"
                    } else {
                        # Remove entire directory
                        Remove-Item $dataPath -Recurse -Force
                        Write-Success "Removed data directory $dataPath"
                    }
                }
                catch {
                    Write-Warning "Could not remove $dataPath`: $_"
                }
            }
        }
    } else {
        Write-Info "Preserving data files"
        Write-Host "Data files preserved in:"
        foreach ($dataPath in $foundData) {
            Write-Host "  $dataPath"
        }
    }
}

# Function to remove from PATH
function Remove-FromPath {
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $systemPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    
    $pathsToRemove = @()
    
    foreach ($dir in $PossibleInstallDirs) {
        if ($userPath -like "*$dir*" -or $systemPath -like "*$dir*") {
            $pathsToRemove += $dir
        }
    }
    
    if ($pathsToRemove.Count -eq 0) {
        Write-Info "No PATH entries found to remove"
        return
    }
    
    if (Confirm-Action "Remove EvalEds-related PATH entries?" $false) {
        foreach ($pathToRemove in $pathsToRemove) {
            if ($DryRun) {
                Write-DryRun "remove $pathToRemove from PATH"
            } else {
                try {
                    $newUserPath = $userPath -replace [regex]::Escape("$pathToRemove;"), ""
                    $newUserPath = $newUserPath -replace [regex]::Escape(";$pathToRemove"), ""
                    
                    if ($newUserPath -ne $userPath) {
                        [Environment]::SetEnvironmentVariable("PATH", $newUserPath, "User")
                        Write-Success "Removed $pathToRemove from user PATH"
                    }
                }
                catch {
                    Write-Warning "Could not update PATH: $_"
                    Write-Info "Please manually remove $pathToRemove from your PATH environment variable"
                }
            }
        }
    }
}

# Function to check for running processes
function Stop-RunningProcesses {
    $processes = Get-Process -Name "evaleds" -ErrorAction SilentlyContinue
    
    if ($processes) {
        Write-Warning "EvalEds processes are currently running"
        if (Confirm-Action "Stop running EvalEds processes?" $true) {
            if ($DryRun) {
                Write-DryRun "stop EvalEds processes"
            } else {
                try {
                    $processes | Stop-Process -Force
                    Write-Success "Stopped running EvalEds processes"
                }
                catch {
                    Write-Warning "Could not stop all processes: $_"
                }
            }
        } else {
            Write-Warning "Some processes may still be running after uninstall"
        }
    }
}

# Main uninstallation
Write-Host ""
Write-Host "🗑️ EvalEds Uninstallation Script for Windows" -ForegroundColor Cyan
Write-Host ""

if ($DryRun) {
    Write-Warning "DRY RUN MODE - Nothing will actually be removed"
    Write-Host ""
}

# Check for running processes
Stop-RunningProcesses

Write-Status "Scanning system for EvalEds components..."

# Count what we found
$binaries = Find-Binaries
$configCount = ($ConfigDirs | Where-Object { Test-Path $_ }).Count
$dataCount = ($DataDirs | Where-Object { Test-Path $_ }).Count

Write-Host ""
Write-Info "Found:"
Write-Host "  📦 Binaries: $($binaries.Count)"
Write-Host "  ⚙️ Configuration directories: $configCount"
Write-Host "  💾 Data directories: $dataCount"
Write-Host ""

if ($binaries.Count -eq 0 -and $configCount -eq 0 -and $dataCount -eq 0) {
    Write-Success "EvalEds does not appear to be installed"
    exit 0
}

if (-not $DryRun) {
    if (-not (Confirm-Action "Proceed with uninstallation?" $true)) {
        Write-Info "Uninstallation cancelled"
        exit 0
    }
    Write-Host ""
}

# Perform uninstallation steps
Remove-Binaries
Remove-Configuration
Remove-Data
Remove-FromPath

Write-Host ""
if ($DryRun) {
    Write-Info "Dry run complete. Run without -DryRun to actually uninstall"
} else {
    Write-Success "🎯 EvalEds uninstallation complete!"
    Write-Info "Thank you for using EvalEds!"
    Write-Host ""
    Write-Info "If you encountered any issues, please report them at:"
    Write-Info "https://github.com/prequired/evaleds/issues"
}