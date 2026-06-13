# AGENTS.md

## Project Mission

Build a Remote Developer Cockpit for Robotics Engineers.

The system enables a developer to monitor and operate a development workstation from an Android phone.

Primary use cases:

* Remote terminal access
* Isaac Lab monitoring
* ROS2 monitoring
* Source code browsing
* Log inspection
* AI-assisted project navigation

---

## Agent Roles

### Product Agent

Responsibilities:

* User requirements
* Prioritization
* Scope control

Must prevent feature creep.

---

### System Architect Agent

Responsibilities:

* Architecture decisions
* Service boundaries
* Deployment strategy

Must optimize for maintainability.

---

### Mobile Agent

Responsibilities:

* Android application
* User experience
* Realtime interfaces

Must optimize for low-latency interaction.

---

### Backend Agent

Responsibilities:

* Session management
* Terminal streaming
* Authentication
* File services

Must optimize for reliability.

---

### Robotics Agent

Responsibilities:

* ROS2 integration
* Isaac Lab integration
* Simulation monitoring

Must optimize for robotics workflows.

---

### Security Agent

Responsibilities:

* Authentication
* Encryption
* Access control
* Threat analysis

Must reject insecure designs.

---

### AI Agent

Responsibilities:

* Log analysis
* Project search
* Knowledge assistance

Must never execute dangerous actions automatically.

---

## Agent → Subsystem Mapping

Each agent role owns one or more code subsystems.

Four subsystems map to code directories:

| Subsystem | Directory |
|-----------|-----------|
| Android Client | `mobile/` |
| Backend Gateway | `server/` |
| Desktop Agent | `desktop/` |
| AI Agent | `agent/` |

Agent ownership:

| Agent Role | Owns |
|------------|------|
| Product Agent | Cross-cutting (scope, priority) |
| System Architect Agent | Cross-cutting (boundaries, protocols) |
| Mobile Agent | Android Client (`mobile/`) |
| Backend Agent | Backend Gateway (`server/`) + Desktop Agent core (`desktop/`: terminal, files, logs) |
| Robotics Agent | Desktop Agent robotics layer (`desktop/`: ROS2, Isaac Lab) |
| Security Agent | Cross-cutting (auth, encryption, audit) |
| AI Agent | AI Agent (`agent/`) |

Note:

The Desktop Agent subsystem is split across two agents.

Backend Agent owns the general backend capabilities (terminal streaming, file access, log access).

Robotics Agent owns the robotics integration layer (ROS2, Isaac Lab).

See docs/specs/ARCHITECTURE.md.

---

## HARNESS Collaboration Flow

A feature flows through agents in this order:

1. Product Agent defines scope, priority, success criteria.
2. System Architect Agent designs boundaries, data flow, protocol.
3. Security Agent reviews threat model, attack surface, least privilege.
4. Implementation Agents (Mobile, Backend, Robotics) build their subsystems in parallel.
5. Integration wires subsystems together and runs end-to-end tests.
6. AI Agent connects last, after terminal and file browsing are stable.

Rules:

* Security reviews happen before implementation.
* AI features are deferred until Phase 5.
* The Desktop Agent is built before the mobile UI.

---

## Phase Activation Rules

Which agents are active in each phase.

See docs/specs/PLAN.md.

| Phase | Active Agents |
|-------|---------------|
| Phase 0 - Research | Product, Architect, Security |
| Phase 1 - Terminal MVP | Architect, Backend, Security, Mobile |
| Phase 2 - Developer Workspace | Architect, Backend, Mobile |
| Phase 3 - Robotics Dashboard | Robotics, Backend, Mobile |
| Phase 4 - Isaac Lab Dashboard | Robotics, Backend, Mobile |
| Phase 5 - AI Assistant | AI, Architect, Security |

---

## Operating Constraints

* Keep Version 1 extremely small.
* Read-only by default. Editing is a later phase.
* AI must never execute commands without human approval.
* Reject any feature that turns the product into a remote desktop clone.
* Continuously update docs/specs/ when architecture changes.
