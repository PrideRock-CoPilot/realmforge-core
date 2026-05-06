@echo off
REM ─────────────────────────────────────────────
REM RealmForge — Start Dev Environment
REM Launches PostgreSQL, Backend API, and Frontend
REM ─────────────────────────────────────────────

echo ╔═══════════════════════════════════════════╗
echo ║    RealmForge Development Environment     ║
echo ╚═══════════════════════════════════════════╝
echo.

REM ── Step 1: Start PostgreSQL via Docker ──
echo [1/3] Starting PostgreSQL...
docker compose up -d 2>nul
if %errorlevel% neq 0 (
    echo   ! Docker not running or compose failed. Make sure Docker Desktop is running.
    echo   ! Starting backend anyway — will fail if no database.
) else (
    echo   ✓ PostgreSQL ready at localhost:5432
)

REM ── Step 2: Start Backend API ──
echo [2/3] Starting backend API server...
start "RealmForge Backend" cmd /c "cd /d E:\realmforge && cargo run --bin control-api"
echo   ✓ Backend starting on http://localhost:8080

REM ── Step 3: Start Frontend Dev Server ──
echo [3/3] Starting frontend dev server...
start "RealmForge Frontend" cmd /c "cd /d E:\realmforge\frontend && npm run dev"
echo   ✓ Frontend starting on http://localhost:5173

echo.
echo ── Environment Starting ──
echo   PostgreSQL : localhost:5432
echo   Backend    : http://localhost:8080
echo   Frontend   : http://localhost:5173
echo.
echo Close the terminal windows to stop the servers.
echo.
pause
