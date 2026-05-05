# ─────────────────────────────────────────────
# E2E Test Orchestrator
# ─────────────────────────────────────────────
# 1. Ensures PostgreSQL is running
# 2. Seeds test data
# 3. Starts the backend API server
# 4. Starts the frontend dev server
# 5. Runs Playwright e2e tests
# 6. Cleans up servers

param(
  [switch]$SkipSeed,
  [switch]$SkipBackend,
  [switch]$KeepServers,
  [string]$TestFilter = ""
)

$ErrorActionPreference = "Continue"
$RepoRoot = "E:\realmforge"
$LogDir = "$RepoRoot\.realmforge\logs"
$null = New-Item -ItemType Directory -Force -Path $LogDir

Write-Host "╔═══════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║    RealmForge E2E Test Orchestrator      ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════╝" -ForegroundColor Cyan

# Function to clean up background jobs
function Cleanup-Servers {
  Write-Host "`nCleaning up servers..." -ForegroundColor Yellow
  Get-Job -Name "backend-*" -ErrorAction SilentlyContinue | Stop-Job | Remove-Job
  Get-Job -Name "frontend-*" -ErrorAction SilentlyContinue | Stop-Job | Remove-Job
}

Register-EngineEvent -SourceIdentifier (Get-Random) -Action { Cleanup-Servers } | Out-Null

try {
  # Step 1: Seed test data
  if (-not $SkipSeed) {
    Write-Host "`n[1/4] Seeding test data..." -ForegroundColor Cyan
    & "$RepoRoot\scripts\seed-test-data.ps1"
    if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne $null) {
      Write-Host "⚠ Seed had issues (non-zero exit: $LASTEXITCODE)" -ForegroundColor Yellow
    }
  } else {
    Write-Host "`n[1/4] Skipping seed (--SkipSeed)" -ForegroundColor DarkGray
  }

  # Step 2: Start backend API
  if (-not $SkipBackend) {
    Write-Host "[2/4] Starting backend API server..." -ForegroundColor Cyan
    
    # Check if backend is already running
    $backendRunning = $false
    try {
      $resp = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
      if ($resp.StatusCode -eq 200) {
        Write-Host "  Backend already running at http://localhost:8080" -ForegroundColor Green
        $backendRunning = $true
      }
    } catch {}

    if (-not $backendRunning) {
      Write-Host "  Starting backend (cargo run --bin control-api)..." -ForegroundColor Gray
      $backendJob = Start-Job -Name "backend-api" -ScriptBlock {
        param($dir, $logDir)
        Set-Location $dir
        $env:RUST_LOG = "info"
        cargo run --bin control-api 2>&1 | Out-File "$logDir\backend.log"
      } -ArgumentList $RepoRoot, $LogDir
      
      # Wait for backend to be ready
      Write-Host "  Waiting for backend to start..." -ForegroundColor DarkGray
      $maxWait = 60
      $waited = 0
      $backendReady = $false
      while ($waited -lt $maxWait) {
        Start-Sleep -Seconds 2
        $waited += 2
        try {
          $resp = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
          if ($resp.StatusCode -eq 200) {
            $backendReady = $true
            break
          }
        } catch {}
        Write-Host "  .. waited ${waited}s for backend" -ForegroundColor DarkGray
      }
      
      if (-not $backendReady) {
        Write-Host "✗ Backend failed to start within $maxWait seconds" -ForegroundColor Red
        Write-Host "  Check logs: $LogDir\backend.log" -ForegroundColor Yellow
        Cleanup-Servers
        exit 1
      }
      Write-Host "  Backend is ready!" -ForegroundColor Green
    }
  } else {
    Write-Host "[2/4] Skipping backend start (--SkipBackend)" -ForegroundColor DarkGray
  }

  # Step 3: Start frontend dev server
  Write-Host "[3/4] Starting frontend dev server..." -ForegroundColor Cyan
  Set-Location "$RepoRoot\frontend"
  $frontendJob = Start-Job -Name "frontend-dev" -ScriptBlock {
    param($dir, $logDir)
    Set-Location $dir
    npm run dev 2>&1 | Out-File "$logDir\frontend.log"
  } -ArgumentList "$RepoRoot\frontend", $LogDir

  # Wait for frontend
  Write-Host "  Waiting for frontend dev server..." -ForegroundColor DarkGray
  $maxWait = 30
  $waited = 0
  $frontendReady = $false
  while ($waited -lt $maxWait) {
    Start-Sleep -Seconds 2
    $waited += 2
    try {
      $resp = Invoke-WebRequest -Uri "http://localhost:5173" -TimeoutSec 2 -ErrorAction SilentlyContinue
      if ($resp.StatusCode -eq 200) {
        $frontendReady = $true
        break
      }
    } catch {}
    Write-Host "  .. waited ${waited}s for frontend" -ForegroundColor DarkGray
  }

  if (-not $frontendReady) {
    Write-Host "✗ Frontend failed to start within $maxWait seconds" -ForegroundColor Red
    Write-Host "  Check logs: $LogDir\frontend.log" -ForegroundColor Yellow
    Cleanup-Servers
    exit 1
  }
  Write-Host "  Frontend is ready at http://localhost:5173!" -ForegroundColor Green

  # Step 4: Run Playwright tests
  Write-Host "[4/4] Running Playwright e2e tests..." -ForegroundColor Cyan
  Set-Location "$RepoRoot\frontend"
  
  $testArgs = "test"
  if ($TestFilter) {
    $testArgs += " --grep ""$TestFilter"""
  }
  
  Write-Host "  Command: npx playwright $testArgs" -ForegroundColor DarkGray
  npx playwright $testArgs 2>&1
  $testExitCode = $LASTEXITCODE

  if ($testExitCode -eq 0) {
    Write-Host "`n✓ All e2e tests passed!" -ForegroundColor Green
  } else {
    Write-Host "`n✗ Some e2e tests failed (exit code: $testExitCode)" -ForegroundColor Red
    Write-Host "  Check Playwright report in .realmforge/playwright-report/" -ForegroundColor Yellow
  }

  exit $testExitCode
}
finally {
  if (-not $KeepServers) {
    Cleanup-Servers
  } else {
    Write-Host "`nServers kept running (--KeepServers). Clean up manually:" -ForegroundColor Yellow
    Write-Host "  Get-Job | Stop-Job | Remove-Job" -ForegroundColor DarkGray
  }
}
