@echo off
REM Session Lifecycle UAT
REM =========================

echo ===== STEP 1: Login Gary (issue session) =====
curl -s -X POST http://127.0.0.1:8080/v1/login -H "Content-Type: application/json" -d @e:\realmforge\login_request.json
echo.
echo.

echo ===== STEP 2: Get session (valid token from above) =====
curl -s http://127.0.0.1:8080/v1/session/e3662865-c044-4da9-95b9-af011eca7da4
echo.
echo.

echo ===== STEP 3: Activate session =====
curl -s -X POST http://127.0.0.1:8080/v1/session/e3662865-c044-4da9-95b9-af011eca7da4/activate
echo.
echo.

echo ===== STEP 4: Renew session =====
curl -s -X POST http://127.0.0.1:8080/v1/session/e3662865-c044-4da9-95b9-af011eca7da4/renew -H "Content-Type: application/json" -d "{\"ttl_seconds\":7200}"
echo.
echo.

echo ===== STEP 5: Revoke session =====
curl -s -X DELETE http://127.0.0.1:8080/v1/session/e3662865-c044-4da9-95b9-af011eca7da4
echo.
echo.

echo ===== STEP 6: Verify session is gone =====
curl -s http://127.0.0.1:8080/v1/session/e3662865-c044-4da9-95b9-af011eca7da4
echo.
echo.
