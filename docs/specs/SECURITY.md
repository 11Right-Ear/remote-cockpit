# SECURITY.md

## Security Philosophy

Remote terminal access is extremely dangerous.

Security is a feature.

Not an afterthought.

---

## Threat Model

Assume:

* Public Internet exposure
* Credential theft attempts
* Automated scanning
* Brute force attacks

---

## Authentication

Requirements:

* Account Login
* Device Registration
* Access Token
* Refresh Token

---

## Device Binding

Each phone receives:

Device ID

Unknown devices require approval.

---

## Encryption

All traffic encrypted.

No plaintext communication.

---

## Terminal Protection

Dangerous commands require confirmation.

Examples:

* rm -rf
* shutdown
* reboot

Version 1 may only warn users.

Future versions may require approval.

---

## File Access Protection

Default:

Read-only

Version 1 should avoid file editing.

---

## Audit Logs

Record:

* Login
* Terminal Commands
* File Access
* Configuration Changes

---

## AI Safety

AI may:

* Read logs
* Read source code
* Analyze repositories

AI may not:

* Execute terminal commands automatically
* Modify files automatically
* Restart services automatically

Human approval required.

---

## Recovery

Provide:

* Device revoke
* Session revoke
* Emergency logout
* Key rotation

---

## Security Goal

Compromise of the Android phone should not immediately provide unrestricted access to the workstation.
