# ARCHITECTURE.md

> **Scope:** System architecture and subsystem boundaries. See [PROJECT_CONTEXT.md](../../PROJECT_CONTEXT.md) for the one-line overview and [AGENTS.md](../../AGENTS.md) for ownership.

## System Overview

Remote Cockpit consists of four major subsystems:

1. Android Client
2. Backend Gateway
3. Desktop Agent
4. AI Agent

```text
Android Client
        |
        |
     Internet
        |
        |
Backend Gateway
        |
        |
 Desktop Agent
        |
 --------------------
 |        |         |
Terminal  Files    ROS2
 |        |         |
IsaacLab Logs    Images
```

---

## Android Client

Responsibilities:

* Authentication
* Terminal display
* File browser
* Code viewer
* Dashboard rendering

The client should contain minimal business logic.

The client should never directly access local files on the PC.

---

## Backend Gateway

Responsibilities:

* Authentication
* Session management
* Device registration
* Message routing

The gateway is the public entry point.

The gateway should not execute terminal commands.

---

## Desktop Agent

Responsibilities:

* Terminal access
* File access
* Log access
* ROS2 integration
* Isaac Lab integration

The desktop agent runs on the developer workstation.

This component has the highest privilege level.

---

## AI Agent

Responsibilities:

* Repository understanding
* Log analysis
* Natural language interface

The AI Agent must remain separated from terminal execution.

AI recommendations require human approval.

---

## Communication

Protocol:

* WebSocket

Data Types:

* Terminal Stream
* File Request
* Image Request
* Dashboard Metrics
* AI Query

---

## Deployment Modes

Mode A:

Home Network + VPN

Recommended for personal use.

---

Mode B:

Cloud Relay

Recommended for multi-device access.

---

Mode C:

Direct P2P

Useful for experimentation.

Not preferred for production.
