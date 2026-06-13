# BACKEND.md

> **Scope:** Backend Gateway (`server/`) — routing, identity, sessions. Terminal execution, file reads, and metrics originate in the Desktop Agent and are only routed here. See [AGENTS.md](../../AGENTS.md) for the agent-to-subsystem mapping.

## Philosophy

Backend should be lightweight.

Heavy work belongs to Desktop Agent.

---

## Services

### Authentication Service

Responsibilities:

* Login
* Token generation
* Device binding

---

### Session Service

Responsibilities:

* Session creation
* Session expiration
* Connection tracking

---

### Routing Service

Responsibilities:

* Forward messages
* Manage active channels

---

## Terminal Stream

Flow:

Phone
→ Gateway
→ Desktop Agent
→ PTY
→ Desktop Agent
→ Gateway
→ Phone

---

## File Access

Rules:

Allowed:

* Source Code
* Images
* Logs

Forbidden:

* System Password Files
* Private Keys
* Browser Credentials

---

## Metrics API

Provide:

* CPU Usage
* Memory Usage
* GPU Usage
* Disk Usage
* Network Status

---

## Dashboard API

Provide:

* ROS2 Nodes
* ROS2 Topics
* Isaac Lab Statistics
* Training Metrics

---

## Future Services

* Multi-PC Support
* Team Collaboration
* Shared Dashboards
* Cloud Storage
