# ADR 0003：GUI 与 daemon 采用版本化本地 IPC

- **状态：** Accepted
- **日期：** 2026-09-17

## 决策

Windows GUI 与 CLI 通过当前用户范围 Named Pipe 与 Rust daemon 通信，采用 JSON-RPC
风格 v1 包络和稳定错误码。禁止 GUI 直接管理 EasyTier 子进程。

## 后果

Core 可无界面运行且进程故障隔离更好；需要维护 IPC 兼容性测试和访问控制。
