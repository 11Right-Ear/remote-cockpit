# PROJECT_CONTEXT.md

The one-page project charter.

Detailed specifications live in docs/specs/.

---

## Documentation Map

| Document | Purpose |
|----------|---------|
| AGENTS.md | HARNESS agent roles, collaboration flow, phase activation |
| docs/specs/ARCHITECTURE.md | System architecture, subsystem boundaries |
| docs/specs/BACKEND.md | Backend Gateway service design |
| docs/specs/FRONTEND.md | Android Client design, screens |
| docs/specs/SECURITY.md | Security model, threat model |
| docs/specs/PLAN.md | Development phases, deliverables, success criteria |
| docs/specs/PRODUCT_SENSE.md | Product scope, problems, success metrics |

---

## Project Name

Remote Robotics Developer Cockpit

---

## Project Background

The project owner is a robotics engineer.

Technical interests:

* ROS2
* Isaac Lab
* Reinforcement Learning
* MPC
* OCS2
* Robot Simulation
* C++
* AI Agent Systems

The goal is a specialized developer cockpit for robotics engineers.

This is not another remote desktop solution.

---

## Core Problem

The developer frequently runs long tasks:

* Isaac Lab training
* ROS2 systems
* Long builds
* Reinforcement learning experiments
* Robot simulations

The developer wants visibility and control while away from the workstation: outside, commuting, eating, traveling.

The developer should inspect and interact with the workstation from an Android phone.

---

## Product Vision

The phone becomes a portable operations center.

The workstation remains the primary development machine.

The phone becomes monitor, observer, controller, assistant for development workflows.

---

## Non Goals

Do NOT build:

* Full IDE
* VSCode replacement
* Remote desktop clone
* Desktop OS replacement

Avoid feature creep.

---

## Target User

Primary user: Robotics Engineer.

Examples: ROS2 Developer, RL Engineer, Simulation Engineer, MPC Engineer, Autonomous Systems Engineer.

---

## Primary Use Cases

High-level list. Detailed screen design in docs/specs/FRONTEND.md.

1. Remote Terminal Access
2. File Inspection (read-only in V1)
3. Image Preview
4. Log Analysis
5. Git Awareness
6. Robotics Monitoring (ROS2)
7. Isaac Lab Monitoring

---

## Long-Term Vision

The system evolves into an AI-powered operations platform.

The user asks natural questions:

* "Why did training stop?"
* "Find MPC weights."
* "Summarize today's work."

The system analyzes source code, logs, terminal history, git history and returns answers.

---

## Technical Architecture

Four components:

1. Android Client
2. Backend Gateway
3. Desktop Agent
4. AI Agent

```text
Android Client  <->  Backend Gateway  <->  Desktop Agent  <->  Workstation Resources
```

Resources: Terminal, Files, Images, Logs, ROS2, Isaac Lab.

Detailed boundaries in docs/specs/ARCHITECTURE.md.

---

## System Philosophy

Thin Client.

The Android application should remain lightweight.

Heavy logic belongs to the Desktop Agent, Backend Services, and AI Agent.

---

## Most Important Component

Desktop Agent.

The Desktop Agent provides terminal, file, log, ROS2, and Isaac Lab access.

The mobile application is primarily a visualization layer.

Prioritize Desktop Agent architecture before mobile UI.

---

## Security Requirements

Assume Internet exposure. Security is mandatory.

Requirements:

* Authentication
* Device Binding
* Encryption
* Session Management
* Audit Logs

V1 defaults to read-only wherever possible.

AI systems must never execute commands automatically. Human approval is required.

Detailed model in docs/specs/SECURITY.md.

---

## Preferred Technology Stack

| Layer | Choice |
|-------|--------|
| Mobile | Flutter |
| Backend | Rust or C++ |
| Realtime | WebSocket |
| Media Streaming | WebRTC |
| Database | SQLite |
| Deployment | Docker |
| AI | MCP, Local LLM, Cloud LLM |

---

## Development Phases

* Phase 0 - Research
* Phase 1 - Remote Terminal MVP
* Phase 2 - Developer Workspace
* Phase 3 - Robotics Dashboard
* Phase 4 - Isaac Lab Dashboard
* Phase 5 - AI Operations Assistant

Detailed deliverables and success criteria in docs/specs/PLAN.md.

---

## Design Principles

Rule 1: The phone is not the workstation.

Rule 2: Readability is more important than feature count.

Rule 3: Remote monitoring is more important than remote editing.

Rule 4: Every feature must solve a real robotics workflow problem.

Rule 5: Avoid desktop paradigms. Design for mobile operation.

---

## Instructions For Claude Code

Before implementing anything:

1. Review architecture.
2. Challenge unnecessary complexity.
3. Prefer maintainability over cleverness.
4. Keep Version 1 extremely small.
5. Focus on Desktop Agent first.
6. Focus on security before convenience.
7. Do not implement AI features until terminal and file browsing are stable.
8. Continuously update architecture documentation.
9. Reject features that turn the product into a remote desktop clone.
10. Optimize for robotics engineers and long-running experiments.

The final product should feel like:

"A mission control center for robotics development."

Not:

"A tiny computer screen on a phone."
