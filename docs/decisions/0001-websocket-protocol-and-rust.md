# ADR 0001: Backend 语言用 Rust，通信用 WebSocket

- **状态**：Accepted
- **日期**：2026-06-13
- **阶段**：Phase 0 收尾 / Phase 1

## 背景

Phase 1（Remote Terminal MVP）需要为 Backend Gateway（`server/`）和 Desktop Agent（`desktop/`）选定实现语言，并确定 `Phone ↔ Gateway ↔ Desktop Agent ↔ PTY` 的通信协议。PROJECT_CONTEXT 把 Backend 列为 "Rust or C++"（未定项），通信列为 WebSocket（已定）。

Hardware validation 显示工作站当前未装 Rust / C++ 工具链。

## 决策

1. **Backend 语言 = Rust**（Gateway 与 Desktop Agent 均用 Rust）。
2. **通信协议 = WebSocket over TLS（`wss://`）**，强制加密（满足 SECURITY.md "无明文"）。
3. **帧规则**：
   - 文本帧 = JSON 信封（控制面：认证、会话、危险告警、心跳）。
   - 二进制帧 = 原始 PTY 字节，前置 1 字节流标签（`0x01` = terminal stdout），数据面零拷贝。
4. **DB = SQLite**，审计日志用 `rusqlite`（bundled），写入经 `spawn_blocking`。

## 理由

**选 Rust 而非 C++**：
- 内存安全直接契合 "assume internet exposure / 安全优先"（SECURITY.md、PROJECT_CONTEXT）。
- tokio + axum 异步生态成熟，WebSocket 服务器 + 并发连接开发快，契合 "V1 极小"。
- Phase 3/4 的 ROS2/Isaac Lab（C++ 生态）可经 FFI 或子进程集成，不阻塞 Phase 1。

**二进制帧承载 PTY 输出**：把编译器级别的大输出包进 base64-in-JSON 浪费约 33% 带宽且每块都要 JSON 解析；原始二进制是远程终端的标准模式（ttyd / xterm.js 后端同此）。1 字节流标签允许未来加文件/图像流而不改线路。

## 后果

- 需安装 rustup 工具链。
- 共享 `protocol` crate 锁定双方契约，server 与 desktop 均依赖。
- V1 token 用共享密钥签的 JWT（dev-only 妥协；Phase 2 改每设备密钥，线路格式不破坏式升级）。
- Flutter（切片 2）的 Dart WebSocket 原生支持二进制帧，无阻塞。

## 关键库

| 用途 | 库 |
|------|----|
| 异步运行时 | tokio |
| HTTP/WS 服务器 | axum |
| WS 客户端 | tokio-tungstenite |
| PTY | portable-pty（Win11 ConPTY / Unix forkpty） |
| 序列化 | serde / serde_json |
| Token | jsonwebtoken |
| DB | rusqlite (bundled) |
| 并发 map | dashmap |
| TLS / 自签名 | axum-server + rustls + rcgen |
| 日志 | tracing |
