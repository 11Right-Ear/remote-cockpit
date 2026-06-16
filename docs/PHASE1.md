# Phase 1 — Remote Terminal MVP（验收报告）

> Phase 1 完成于 2026-06-16。本文件是 Phase 1 的正式验收记录：交付物核对、安全约束核对、真机端到端验证、已知遗留，以及 Phase 2 展望。规格定义见 [PLAN.md](specs/PLAN.md) 与 [PROJECT_CONTEXT.md](../PROJECT_CONTEXT.md)。

## 1. 范围

Phase 1 = **Remote Terminal MVP**。目标（success criteria）：用户能从 Android 手机远程操作工作站终端。

不在 Phase 1 范围（留给后续 Phase）：文件浏览 / 代码 / 图片 / 日志查看（Phase 2）、ROS2 / Isaac Lab 监控（Phase 3 / 4）、AI 助手（Phase 5）。

## 2. 交付物核对（PLAN deliverable）

| Deliverable | 状态 | 实现位置 | 验证 |
|---|---|---|---|
| Terminal streaming | ✅ | `rc-desktop` portable-pty 双向 + `protocol` 二进制帧（tag `0x01`）+ `CockpitClient` wss + xterm.dart 渲染 | 真机 E2E |
| Command execution | ✅ | desktop `agent_loop` PTY 写入 + 危险命令检测（行尾累积，warn-only） | `echo hello` 执行 |
| Authentication | ✅ | `rc-gateway` JWT HS256（共享密钥 `dev-secret`）+ wss TLS | `auth_ok` 审计 |

**Success criteria：「User can operate terminal remotely」→ ✅ 真机端到端验证（见 §4）。**

## 3. 安全约束核对（PROJECT_CONTEXT §Security / SECURITY.md）

| 约束 | 状态 | 说明 |
|---|---|---|
| Authentication | ✅ | JWT HS256；claims `{device_id, role, exp}` |
| Device Binding | ✅ | 按 `device_id` 路由——phone 的 `device_id` 填目标 desktop 的 id（`dev-ws`），gateway 据此找到 desktop 注册 |
| Encryption | ✅ | wss TLS（dev：rcgen 自签名；Phase 2 换真实 CA + 证书 pinning） |
| Session Management | ✅ | 两个 session_id 不混用：连接级 `auth_ok.session_id` ≠ PTY 级 `session_opened.session_id` |
| Audit Logs | ✅ | SQLite `audit_log`：`desktop_online` / `auth_ok` / `session_open` / `danger_warn`，E2E 已验证落库 |
| Terminal Protection | ✅ | 危险命令 warn-only（`rm -rf` 等模式触发红色 DangerWarn）；Phase 1 不阻断，仅警告 |
| AI Safety | ✅ | Phase 1 无 AI 功能；「AI 不得自动执行命令」约束天然满足 |
| Default read-only | ✅ | Phase 1 仅终端；文件访问（Phase 2）将默认只读 |

## 4. 端到端验证（真机）

- **设备**：vivo V2241A / PD2241（Android 15, API 35），USB 连接。
- **链路**：phone app → gateway（`rc-gateway`，`0.0.0.0:8443`，自签名 TLS）→ desktop agent（`dev-ws`，cmd.exe PTY）。
- **验证项**：
  - 连接 + 认证 + session 建立 ✅（审计：`auth_ok` ×2、`session_open`、`desktop_online`）
  - `echo hello` → 正确执行并回显 ✅
  - `rm -rf /tmp/test` → 红色 DangerWarn + `danger_warn` 审计落库（pattern `rm\s+(-[a-zA-Z]*r[a-zA-Z]*f|--recursive\b.*--force\b)`）✅
  - 审计完整性 ✅

## 5. 已知遗留 / 技术债（移交后续）

| 项 | 严重度 | 处理 |
|---|---|---|
| 自签名 TLS | 中（dev 可接受） | Phase 2：真实 CA + 证书 pinning（见 SECURITY.md） |
| Android 底部输入栏增量噪声 | 低 | 快速退格/重打时命令字符串累积噪声；不影响 danger 检测。后续优化 diff 逻辑 |
| rustls `Illegal SNI extension` 警告 | 低 | 手机用 IP 作 SNI，rustls 仅 warn 并忽略，无害；用域名可消除 |
| 28 处 `unwrap`/`expect`（`protocol/` 为主） | 低 | 多在确定性编解码路径且有单测覆盖；后续评估热路径 |
| app 层显式命令历史 | 低 | cmd 自带历史；app 层 history UI 可作 Phase 1.x 增强 |

## 6. 文档一致性

`docs/specs/FRONTEND.md` 与 `docs/specs/BACKEND.md` 是**前瞻性产品规格**（描述跨 Phase 1-5 的完整产品愿景）。Phase 1 实现了其中的 Terminal 子集（FRONTEND 的 Terminal Screen + BACKEND 的 Authentication / Session / Routing / Terminal Stream）。文档**无需修正**——它们正确描述目标，实现按 Phase 逐步推进。

## 7. Phase 2 展望（Developer Workspace）

PLAN Phase 2 deliverable：file browser、code viewer、image viewer、log viewer。Success：「User can inspect project status remotely」。

建议拆分（首切片优先，复用 Phase 1 的 wss + JWT + 审计 + 默认只读骨架，在 desktop agent 增加能力）：

1. **切片 1 — 文件浏览**：只读目录树 + 文件列表（复用现有协议骨架，新增文件列举消息）。
2. **切片 2 — 代码 / 文本查看器**：只读，可选语法高亮。
3. **切片 3 — 图片预览**。
4. **切片 4 — 日志查看器**：`tail -f` 式流式。

硬约束沿用：默认只读、全程审计、AI 不得自动操作。详细规格进入 Phase 2 时再定。

---

**Phase 1 完成。下一步：Phase 2 规划。**
