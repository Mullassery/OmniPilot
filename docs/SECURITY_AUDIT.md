# Security Vulnerability Audit - OmniPilot

**Audit Date:** 2026-10-27
**Repository:** /Users/georgimullassery/Desktop/Projects/OmniPilot
**Total Findings:** 99

## Summary
- **Critical:** 57
- **High:** 6
- **Medium:** 36

## Findings

### 1. [Critical] Command Injection
- **File:** apps/admin_console/services/device_stream_service.py:47
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 2. [Critical] Command Injection
- **File:** apps/admin_console/services/device_stream_service.py:76
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 3. [Critical] Command Injection
- **File:** apps/admin_console/services/task_queue_service.py:855
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 4. [Critical] Command Injection
- **File:** apps/showcase_ui/src/app/utils/image-overlay.util.ts:129
- **Description:** exec()
- **Evidence (Line):** `while ((m = pairRegex.exec(trimmed)) !== null) {`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 5. [Critical] Command Injection
- **File:** artemis/agents/video_analyzer/chunk_conversation.py:471
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 6. [Critical] Command Injection
- **File:** artemis/agents/video_analyzer/universal_engine.py:320
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 7. [Critical] Hardcoded Secret
- **File:** artemis/config/constants.py:64
- **Description:** API Key Assignment detected
- **Evidence (Line):** `ENV_ANTHROPIC_API_KEY = "ANTHROPIC_API_KEY"`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 8. [Critical] Hardcoded Secret
- **File:** artemis/config/constants.py:65
- **Description:** API Key Assignment detected
- **Evidence (Line):** `ENV_OPEN_ROUTER_API_KEY = "OPEN_ROUTER_API_KEY"`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 9. [Critical] Hardcoded Secret
- **File:** artemis/config/constants.py:35
- **Description:** Token Assignment detected
- **Evidence (Line):** `ENV_ARTEMIS_TENANT_TOKEN = "ARTEMIS_TENANT_TOKEN"`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 10. [Critical] Command Injection
- **File:** artemis/controllers/unified_controller.py:67
- **Description:** exec()
- **Evidence (Line):** `return await asyncio.create_subprocess_exec(*command, **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 11. [Critical] Command Injection
- **File:** artemis/core/diagnostics/emulator_manager.py:311
- **Description:** exec()
- **Evidence (Line):** `p = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 12. [Critical] Command Injection
- **File:** artemis/core/diagnostics/emulator_manager.py:376
- **Description:** exec()
- **Evidence (Line):** `p = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 13. [Critical] Command Injection
- **File:** artemis/core/diagnostics/probes/adb_probe.py:121
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 14. [Critical] Command Injection
- **File:** artemis/core/diagnostics/probes/adb_probe.py:205
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 15. [Critical] Command Injection
- **File:** artemis/core/diagnostics/probes/adb_probe.py:291
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 16. [Critical] Command Injection
- **File:** artemis/core/diagnostics/probes/adb_probe.py:322
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 17. [Critical] Command Injection
- **File:** artemis/core/diagnostics/probes/adb_probe.py:481
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 18. [Critical] Command Injection
- **File:** artemis/drivers/android/adb_driver.py:453
- **Description:** exec()
- **Evidence (Line):** `self._scrcpy_process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 19. [Critical] Command Injection
- **File:** artemis/drivers/android/adb_driver.py:479
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 20. [Critical] Command Injection
- **File:** artemis/mcp/action_executor.py:525
- **Description:** eval()
- **Evidence (Line):** `sequence = ast.literal_eval(sequence_str)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 21. [Critical] Command Injection
- **File:** artemis/resources/showcase_ui/main-BGAFACUZ.js:4
- **Description:** exec()
- **Evidence (Line):** ``);return o>=0?e.slice(0,o):e}function fc(n,t){return n?t?`${n} ${t}`:n:t||""}var zO=he({__forward_r`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 22. [Critical] Command Injection
- **File:** artemis/resources/showcase_ui/main-BGAFACUZ.js:5
- **Description:** exec()
- **Evidence (Line):** ``).forEach(e=>{let o=e.indexOf(":");if(o>0){let i=e.slice(0,o),r=e.slice(o+1).trim();this.addHeaderE`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 23. [Critical] Command Injection
- **File:** artemis/resources/showcase_ui/polyfills-5CFQRCPP.js:1
- **Description:** exec()
- **Evidence (Line):** `var ce=globalThis;function te(t){return(ce.__Zone_symbol_prefix||"__zone_symbol__")+t}function ht(){`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 24. [Critical] Command Injection
- **File:** artemis/resources/showcase_ui/polyfills-5CFQRCPP.js:1
- **Description:** exec()
- **Evidence (Line):** `var ce=globalThis;function te(t){return(ce.__Zone_symbol_prefix||"__zone_symbol__")+t}function ht(){`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 25. [Critical] Command Injection
- **File:** artemis/runtime/device_pool.py:136
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 26. [Critical] Command Injection
- **File:** artemis/runtime/device_pool.py:252
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 27. [Critical] Command Injection
- **File:** artemis/runtime/supervisor.py:55
- **Description:** exec()
- **Evidence (Line):** `return await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 28. [Critical] Command Injection
- **File:** artemis/tools/command_tool.py:738
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 29. [Critical] Command Injection
- **File:** artemis/tools/command_tool.py:1104
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 30. [Critical] Command Injection
- **File:** artemis/utils/coordinates.py:480
- **Description:** eval()
- **Evidence (Line):** `seq = ast.literal_eval(seq)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 31. [Critical] Command Injection
- **File:** artemis/utils/task_tree.py:360
- **Description:** eval()
- **Evidence (Line):** `parsed = ast.literal_eval(cleaned)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 32. [Critical] Command Injection
- **File:** artemis/utils/video.py:158
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 33. [Critical] Command Injection
- **File:** artemis/utils/video.py:219
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 34. [Critical] Command Injection
- **File:** artemis/utils/video.py:253
- **Description:** exec()
- **Evidence (Line):** `fallback_proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 35. [Critical] Command Injection
- **File:** artemis/utils/video.py:294
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 36. [Critical] Command Injection
- **File:** artemis/utils/video.py:334
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 37. [Critical] Command Injection
- **File:** artemis/utils/video.py:542
- **Description:** exec()
- **Evidence (Line):** `process = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 38. [Critical] Command Injection
- **File:** artemis/utils/visualization.py:617
- **Description:** eval()
- **Evidence (Line):** `sequence = ast.literal_eval(sequence_str)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 39. [Critical] Command Injection
- **File:** mcp_server/notifiers/script.py:79
- **Description:** subprocess with shell=True
- **Evidence (Line):** `subprocess.run(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 40. [Critical] Command Injection
- **File:** tests/e2e/play_youtube.py:26
- **Description:** subprocess with shell=True
- **Evidence (Line):** `result = subprocess.run(cmd, shell=True, capture_output=True, text=True)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 41. [Critical] Hardcoded Secret
- **File:** tests/integration/test_artemis_login.py:77
- **Description:** Password Assignment detected
- **Evidence (Line):** `if (b"Password:" in data or b"password:" in data) and not password_sent:`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 42. [Critical] Hardcoded Secret
- **File:** tests/integration/test_artemis_login.py:77
- **Description:** Password Assignment detected
- **Evidence (Line):** `if (b"Password:" in data or b"password:" in data) and not password_sent:`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 43. [Critical] Hardcoded Secret
- **File:** tests/unit/admin_console/test_task_queue_service.py:405
- **Description:** Token Assignment detected
- **Evidence (Line):** `token="external-owner-token",`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 44. [Critical] Hardcoded Secret
- **File:** tests/unit/admin_console/test_task_queue_service.py:542
- **Description:** Token Assignment detected
- **Evidence (Line):** `token="external-owner-token",`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 45. [Critical] Command Injection
- **File:** tests/unit/admin_console/test_task_queue_service.py:219
- **Description:** exec()
- **Evidence (Line):** `async def fake_subprocess_exec(*args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 46. [Critical] Command Injection
- **File:** tests/unit/admin_console/test_task_queue_service.py:265
- **Description:** exec()
- **Evidence (Line):** `async def fake_subprocess_exec(*args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 47. [Critical] Command Injection
- **File:** tests/unit/admin_console/test_task_queue_service.py:573
- **Description:** exec()
- **Evidence (Line):** `async def fake_subprocess_exec(*args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 48. [Critical] Command Injection
- **File:** tests/unit/admin_console/test_task_queue_service.py:627
- **Description:** exec()
- **Evidence (Line):** `async def fake_subprocess_exec(*args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 49. [Critical] Command Injection
- **File:** tests/unit/admin_console/test_task_queue_service.py:773
- **Description:** exec()
- **Evidence (Line):** `async def fake_subprocess_exec(*args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 50. [Critical] Command Injection
- **File:** tests/unit/admin_console/test_task_queue_service.py:1016
- **Description:** exec()
- **Evidence (Line):** `async def fake_subprocess_exec(*args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 51. [Critical] Hardcoded Secret
- **File:** tests/unit/core/test_adb_keys.py:99
- **Description:** Private Key (PKCS8) detected
- **Evidence (Line):** `"-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC...\n-----END PRIVA`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 52. [Critical] Hardcoded Secret
- **File:** tests/unit/core/test_adb_keys.py:118
- **Description:** Private Key (PKCS8) detected
- **Evidence (Line):** `"-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC...\n-----END PRIVA`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 53. [Critical] Command Injection
- **File:** third_party/mobile_use/utils/shell_utils.py:25
- **Description:** subprocess with shell=True
- **Evidence (Line):** `result = subprocess.run(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 54. [Critical] Command Injection
- **File:** third_party/mobile_use/utils/video.py:199
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 55. [Critical] Command Injection
- **File:** third_party/mobile_use/utils/video.py:304
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 56. [Critical] Command Injection
- **File:** third_party/mobile_use/utils/video.py:354
- **Description:** exec()
- **Evidence (Line):** `probe = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 57. [Critical] Command Injection
- **File:** third_party/mobile_use/utils/video.py:380
- **Description:** exec()
- **Evidence (Line):** `proc = await asyncio.create_subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 58. [High] .env File with Secrets
- **File:** .env.example:1
- **Description:** .env.example contains potential secrets
- **Evidence (Line):** `# ==============================================================================`
- **Exploit Scenario:** Secrets in committed .env files exposed in git history
- **Fix Recommendation:** Add .env to .gitignore, remove from git history, rotate secrets
- **Confidence:** High

### 59. [High] SQL Injection
- **File:** artemis/agents/video_analyzer/blackboard.py:160
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `conn.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 60. [High] SQL Injection
- **File:** artemis/data_engine/storage.py:1102
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `conn.execute(f"DELETE FROM {table}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 61. [High] SQL Injection
- **File:** artemis/data_engine/storage.py:1175
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `conn.executemany(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 62. [High] SQL Injection
- **File:** artemis/data_engine/storage.py:1191
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `conn.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 63. [High] SQL Injection
- **File:** artemis/resources/showcase_ui/main-BGAFACUZ.js:4
- **Description:** SQL string concatenation
- **Evidence (Line):** ``);return o>=0?e.slice(0,o):e}function fc(n,t){return n?t?`${n} ${t}`:n:t||""}var zO=he({__forward_r`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 64. [Medium] Weak Cryptography
- **File:** artemis/utils/plan_grammar.py:77
- **Description:** MD5 usage
- **Evidence (Line):** `return hashlib.md5(text.strip().encode("utf-8")).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 65. [Medium] Weak Cryptography
- **File:** artemis/utils/plan_grammar.py:77
- **Description:** Weak hash algorithm
- **Evidence (Line):** `return hashlib.md5(text.strip().encode("utf-8")).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 66. [Medium] Weak Cryptography
- **File:** tests/unit/agents/test_history_analyzer.py:372
- **Description:** MD5 usage
- **Evidence (Line):** `expected_hash = hashlib.md5(b"Active Subgoal").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 67. [Medium] Weak Cryptography
- **File:** tests/unit/agents/test_history_analyzer.py:372
- **Description:** Weak hash algorithm
- **Evidence (Line):** `expected_hash = hashlib.md5(b"Active Subgoal").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 68. [Medium] Weak Cryptography
- **File:** tests/unit/agents/test_operator.py:506
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_old_hash = hashlib.md5(b"subgoal_old").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 69. [Medium] Weak Cryptography
- **File:** tests/unit/agents/test_operator.py:507
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_current_hash = hashlib.md5(b"subgoal_current").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 70. [Medium] Weak Cryptography
- **File:** tests/unit/agents/test_operator.py:506
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_old_hash = hashlib.md5(b"subgoal_old").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 71. [Medium] Weak Cryptography
- **File:** tests/unit/agents/test_operator.py:507
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_current_hash = hashlib.md5(b"subgoal_current").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 72. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:59
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_hash_1 = hashlib.md5(b"Open Settings app").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 73. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:60
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_hash_2 = hashlib.md5(b"Navigate to System settings").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 74. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:134
- **Description:** MD5 usage
- **Evidence (Line):** `h_settings = hashlib.md5(b"Open Settings app").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 75. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:135
- **Description:** MD5 usage
- **Evidence (Line):** `h_system = hashlib.md5(b"Navigate to System settings").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 76. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:149
- **Description:** MD5 usage
- **Evidence (Line):** `assert sub == hashlib.md5(b"Scroll down to find System settings").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 77. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:175
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_hash_1 = hashlib.md5(b"Subgoal 1").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 78. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:176
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_hash_2 = hashlib.md5(b"Subgoal 2").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 79. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:177
- **Description:** MD5 usage
- **Evidence (Line):** `subgoal_hash_3 = hashlib.md5(b"Subgoal 3").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 80. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:562
- **Description:** MD5 usage
- **Evidence (Line):** `hash_completed_1 = hashlib.md5(b"Completed Milestone 1").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 81. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:563
- **Description:** MD5 usage
- **Evidence (Line):** `hash_completed_2 = hashlib.md5(b"Completed Milestone 2").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 82. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:564
- **Description:** MD5 usage
- **Evidence (Line):** `hash_active_c = hashlib.md5(b"Active Milestone C").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 83. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:566
- **Description:** MD5 usage
- **Evidence (Line):** `hash_failed_old = hashlib.md5(b"Failed Attempt Old").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 84. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:567
- **Description:** MD5 usage
- **Evidence (Line):** `hash_failed_a = hashlib.md5(b"Failed Attempt A").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 85. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:568
- **Description:** MD5 usage
- **Evidence (Line):** `hash_failed_b = hashlib.md5(b"Failed Attempt B").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 86. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:59
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_hash_1 = hashlib.md5(b"Open Settings app").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 87. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:60
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_hash_2 = hashlib.md5(b"Navigate to System settings").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 88. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:134
- **Description:** Weak hash algorithm
- **Evidence (Line):** `h_settings = hashlib.md5(b"Open Settings app").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 89. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:135
- **Description:** Weak hash algorithm
- **Evidence (Line):** `h_system = hashlib.md5(b"Navigate to System settings").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 90. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:149
- **Description:** Weak hash algorithm
- **Evidence (Line):** `assert sub == hashlib.md5(b"Scroll down to find System settings").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 91. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:175
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_hash_1 = hashlib.md5(b"Subgoal 1").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 92. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:176
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_hash_2 = hashlib.md5(b"Subgoal 2").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 93. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:177
- **Description:** Weak hash algorithm
- **Evidence (Line):** `subgoal_hash_3 = hashlib.md5(b"Subgoal 3").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 94. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:562
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hash_completed_1 = hashlib.md5(b"Completed Milestone 1").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 95. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:563
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hash_completed_2 = hashlib.md5(b"Completed Milestone 2").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 96. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:564
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hash_active_c = hashlib.md5(b"Active Milestone C").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 97. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:566
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hash_failed_old = hashlib.md5(b"Failed Attempt Old").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 98. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:567
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hash_failed_a = hashlib.md5(b"Failed Attempt A").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 99. [Medium] Weak Cryptography
- **File:** tests/unit/utils/test_task_tree.py:568
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hash_failed_b = hashlib.md5(b"Failed Attempt B").hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High
