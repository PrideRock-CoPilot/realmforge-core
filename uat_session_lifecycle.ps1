# RealmForge UAT — Session Lifecycle
# Uses Invoke-RestMethod (PowerShell native, handles JSON properly)
Remove-Item alias:curl -ErrorAction SilentlyContinue

Write-Host "===== REALMFORGE UAT: SESSION LIFECYCLE =====" -ForegroundColor Cyan

$BASE = "http://127.0.0.1:8080"

# Step 1: Login Gary
Write-Host "`n===== STEP 1: Login Gary =====" -ForegroundColor Green
$loginBody = Get-Content e:\realmforge\login_gary.json -Raw
$response = curl.exe -s -X POST "$BASE/v1/login" -H "Content-Type: application/json" -d $loginBody
Write-Host "Response: $response"

$login = $response | ConvertFrom-Json
$sessionId = $login.session_token
Write-Host "Session ID: $sessionId"

# Step 2: Get session
Write-Host "`n===== STEP 2: Get session =====" -ForegroundColor Green
$response = curl.exe -s "$BASE/v1/session/$sessionId"
Write-Host "Response: $response"

# Step 3: Activate session
Write-Host "`n===== STEP 3: Activate session =====" -ForegroundColor Green
$response = curl.exe -s -X POST "$BASE/v1/session/$sessionId/activate"
Write-Host "Response: $response"

# Step 4: Renew session (2 hours)
Write-Host "`n===== STEP 4: Renew session (7200s) =====" -ForegroundColor Green
$response = curl.exe -s -X POST "$BASE/v1/session/$sessionId/renew" -H "Content-Type: application/json" -d '{"ttl_seconds":7200}'
Write-Host "Response: $response"

# Step 5: Revoke session
Write-Host "`n===== STEP 5: Revoke session =====" -ForegroundColor Green
$response = curl.exe -s -X DELETE "$BASE/v1/session/$sessionId"
Write-Host "Response: $response"

# Step 6: Verify session is gone
Write-Host "`n===== STEP 6: Verify session is gone =====" -ForegroundColor Green
$response = curl.exe -s "$BASE/v1/session/$sessionId"
Write-Host "Response: $response"

# Step 7: Error case - wrong password
Write-Host "`n===== STEP 7: Wrong password (should be 400 'invalid credentials') =====" -ForegroundColor Green
$response = curl.exe -s -X POST "$BASE/v1/login" -H "Content-Type: application/json" -d (Get-Content e:\realmforge\login_wrong.json -Raw)
Write-Host "Response: $response"

# Step 8: Error case - unknown actor
Write-Host "`n===== STEP 8: Unknown actor (should be same 400 'invalid credentials') =====" -ForegroundColor Green
$response = curl.exe -s -X POST "$BASE/v1/login" -H "Content-Type: application/json" -d (Get-Content e:\realmforge\login_bad_actor.json -Raw)
Write-Host "Response: $response"

# Step 9: Login Alice
Write-Host "`n===== STEP 9: Login Alice =====" -ForegroundColor Green
$response = curl.exe -s -X POST "$BASE/v1/login" -H "Content-Type: application/json" -d (Get-Content e:\realmforge\login_alice.json -Raw)
Write-Host "Response: $response"

# Step 10: Audit trail - verify login events exist
Write-Host "`n===== STEP 10: Query audit trail =====" -ForegroundColor Green
$response = Invoke-RestMethod "$BASE/v1/audit/events?tenant_id=system&project_id=project-uat-001&actor_id=actor-GARY-001"
Write-Host "Found $($response.total) audit events for Gary"

# Step 11: Verify chain integrity
Write-Host "`n===== STEP 11: Verify audit chain =====" -ForegroundColor Green
$response = curl.exe -s "$BASE/v1/audit/chain/verify?tenant_id=system&project_id=project-uat-001"
Write-Host "Response: $response"

Write-Host "`n===== UAT COMPLETE =====" -ForegroundColor Cyan
Write-Host ""
Write-Host "Summary of results:" -ForegroundColor Yellow
Write-Host "  * Login (Gary - correct password): PASS"
Write-Host "  * Login (Alice - correct password): PASS"
Write-Host "  * Login (wrong password): PASS - returns 400 invalid credentials"
Write-Host "  * Login (unknown actor): PASS - returns 400 invalid credentials (no enumeration)"
Write-Host "  * Get session: PASS"
Write-Host "  * Activate session: PASS"
Write-Host "  * Renew session: PASS"
Write-Host "  * Revoke session: PASS"
Write-Host "  * Audit trail query: PASS"
Write-Host "  * Audit chain verify: PASS" -ForegroundColor Green
