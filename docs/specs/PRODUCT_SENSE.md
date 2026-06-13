# PRODUCT_SENSE.md

> **Scope:** User problems, success metrics, and scope control. Vision, target user, and non-goals are defined once in [PROJECT_CONTEXT.md](../../PROJECT_CONTEXT.md) and not repeated here.

This document looks at the product from a product-management angle: which problems we solve, how success is measured, and where the scope boundary sits.

---

## User Problems

### Problem 1: Training runs for hours

Isaac Lab and reinforcement-learning jobs run unattended for hours.

The engineer wants visibility into reward, loss, and status while away from the workstation.

---

### Problem 2: Builds fail remotely

C++ and ROS2 builds fail while the engineer is away.

The engineer wants fast diagnostics: read the build log and the error without opening a laptop.

---

### Problem 3: Source code is hard to read on a phone

Codebases are large. Standard remote desktop is unreadable on a phone screen.

The engineer wants a lightweight, mobile-native code browser (read-only in V1).

---

### Problem 4: Logs are scattered

Build logs, ROS2 logs, and training logs live in different places.

The engineer wants centralized, mobile-friendly log access.

---

## Success Metrics

V1 (Terminal + Workspace):

* Terminal is usable for remote command execution.
* Project files are browsable.
* Images and figures are viewable.

V2 (Robotics dashboards):

* ROS2 nodes and topics are visible.
* Isaac Lab training metrics are visible.

V3 (AI assistant):

* Natural-language project search works.
* Logs are analyzable by AI.
* Daily work can be summarized by AI.

Phase deliverables and release versions live in [PLAN.md](PLAN.md).

---

## Scope Boundaries

The phone is a monitoring and intervention device, not a workstation replacement.

Rejected (see PROJECT_CONTEXT.md Non Goals):

* Full IDE on the phone.
* Desktop replacement.
* Complex code editing.

Deferred:

* File editing (V1 is read-only).
* AI command execution (always requires human approval).
