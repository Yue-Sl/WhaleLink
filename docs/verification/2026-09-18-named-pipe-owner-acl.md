# Windows Named Pipe 当前用户 ACL（2026-09-18）

## 范围

加强 `\\.\pipe\WhaleLink.v1` 的本地 IPC 边界，禁止同机其他用户连接 daemon。

## 实现

- Pipe 创建使用 Windows SDDL `D:P(A;;GA;;;OW)`：受保护 DACL，仅向对象所有者（即启动
  daemon 的当前用户令牌）授予完全访问。
- Pipe 仍设置 `PIPE_REJECT_REMOTE_CLIENTS`，作为第二层传输边界。
- Pipe 创建路径使用 `first_pipe_instance`，避免 daemon 静默连接到同名的先占实例。
- SDDL 的安全描述符仅在 `CreateNamedPipe` 调用期间存活，随后用 `LocalFree` 释放。

## 验证

- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- 同一用户 daemon/CLI 冒烟：通过，覆盖 Pipe 导入、CLI 状态、CLI 启动、运行状态和 CLI 停止。

## 限制

本机没有第二个独立 Windows 用户会话，未进行跨用户拒绝的运行时试验；DACL 的访问规则由 Windows
内核在连接前执行，且 SDDL 作为实现依据已经代码审阅。该限制不应被表述为已完成的跨用户验收。
