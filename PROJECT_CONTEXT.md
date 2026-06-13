# PROJECT_CONTEXT.md

## Project Name

Remote Robotics Developer Cockpit

---

# Project Background

The project owner is a robotics engineer.

Current technical interests include:

* ROS2
* Isaac Lab
* Reinforcement Learning
* MPC
* OCS2
* Robot Simulation
* C++
* AI Agent Systems
* Vibe Coding Workflows

The project is intended to improve developer productivity and remote access to development environments.

The goal is not to create another remote desktop solution.

The goal is to create a specialized developer cockpit optimized for robotics engineers.

---

# Core Problem

The developer frequently runs:

* Isaac Lab training
* ROS2 systems
* Long-running builds
* Reinforcement learning experiments
* Robot simulations

The developer wants visibility and control while away from the workstation.

Examples:

* Outside
* Commuting
* Eating
* Traveling

The developer should be able to inspect and interact with the development workstation from an Android phone.

---

# Product Vision

The phone becomes a portable operations center.

The workstation remains the primary development machine.

The phone becomes:

* monitor
* observer
* controller
* assistant

for development workflows.

---

# Non Goals

Do NOT build:

* Full IDE
* VSCode replacement
* Remote desktop clone
* Desktop operating system replacement

Avoid feature creep.

---

# Target User

Primary User:

Robotics Engineer

Examples:

* ROS2 Developer
* RL Engineer
* Simulation Engineer
* MPC Engineer
* Autonomous Systems Engineer

Future users may include:

* AI Engineers
* Infrastructure Engineers
* Embedded Engineers

---

# Primary Use Cases

## Use Case 1

Remote Terminal Access

User opens phone.

User views workstation terminal output.

User sends commands.

Expected functionality:

* realtime output
* realtime command execution
* command history

---

## Use Case 2

File Inspection

User browses project files.

User opens:

* cpp
* hpp
* py
* yaml
* json
* md

files.

Version 1 should be read-only.

---

## Use Case 3

Image Preview

User views:

* screenshots
* generated figures
* robot images
* training images

without remote desktop software.

---

## Use Case 4

Log Analysis

User views:

* build logs
* ROS2 logs
* training logs

directly from phone.

---

## Use Case 5

Git Awareness

User checks:

* modified files
* current branch
* recent commits

while away from workstation.

---

## Use Case 6

Robotics Monitoring

User views:

* ROS2 node list
* topic list
* robot status
* system status

from mobile device.

---

## Use Case 7

Isaac Lab Monitoring

User views:

* reward
* loss
* episode
* FPS
* experiment metadata

without opening workstation.

---

# Long-Term Vision

The system evolves into an AI-powered operations platform.

The user communicates naturally:

Example:

"Why did training stop?"

Example:

"Find MPC weights."

Example:

"Summarize today's work."

The system analyzes:

* source code
* logs
* terminal history
* git history

and returns answers.

---

# Technical Architecture

Four major components:

1. Android Client
2. Backend Gateway
3. Desktop Agent
4. AI Agent

Architecture:

Android Client
↕

Backend Gateway
↕

Desktop Agent
↕

Workstation Resources

Resources include:

* Terminal
* Files
* Images
* Logs
* ROS2
* Isaac Lab

---

# System Philosophy

Thin Client

The Android application should remain lightweight.

Heavy logic belongs to:

* Desktop Agent
* Backend Services
* AI Agent

---

# Most Important Component

Desktop Agent

Reason:

The Desktop Agent provides:

* terminal access
* file access
* log access
* ROS2 integration
* Isaac Lab integration

The mobile application is primarily a visualization layer.

The Desktop Agent is the actual product.

Prioritize Desktop Agent architecture before mobile UI development.

---

# Security Requirements

Assume Internet exposure.

Security is mandatory.

Requirements:

* Authentication
* Device Binding
* Encryption
* Session Management
* Audit Logs

Version 1 should default to read-only access wherever possible.

AI systems must never execute commands automatically.

Human approval is required.

---

# Preferred Technology Stack

Mobile:

Flutter

Backend:

Rust or C++

Realtime Communication:

WebSocket

Media Streaming:

WebRTC

Database:

SQLite

Deployment:

Docker

AI:

MCP
Local LLM
Cloud LLM

---

# Development Strategy

Phase 0

Research

Deliverables:

* Requirements
* Architecture
* Hardware Validation

---

Phase 1

Remote Terminal MVP

Deliverables:

* Terminal Streaming
* Command Input
* Authentication

Success Criteria:

User can remotely operate terminal.

---

Phase 2

Developer Workspace

Deliverables:

* File Browser
* Code Viewer
* Image Viewer
* Log Viewer

Success Criteria:

User can inspect project status remotely.

---

Phase 3

Robotics Dashboard

Deliverables:

* ROS2 Monitoring
* Node Viewer
* Topic Viewer
* Robot State

Success Criteria:

User can monitor robot systems.

---

Phase 4

Isaac Lab Dashboard

Deliverables:

* Reward Curves
* Training Metrics
* Experiment Monitoring

Success Criteria:

User can monitor RL training remotely.

---

Phase 5

AI Operations Assistant

Deliverables:

* Repository Search
* Log Analysis
* Daily Summaries
* Context-Aware Assistance

Success Criteria:

User can query project state using natural language.

---

# Design Principles

Rule 1

The phone is not the workstation.

---

Rule 2

Readability is more important than feature count.

---

Rule 3

Remote monitoring is more important than remote editing.

---

Rule 4

Every feature must solve a real robotics workflow problem.

---

Rule 5

Avoid desktop paradigms.

Design specifically for mobile operation.

---

# Instructions For Claude Code

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
