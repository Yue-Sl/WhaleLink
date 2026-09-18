# daemon 异常退出与受管重启（2026-09-18）

## 范围

验证 `whalelinkd` 不会在 EasyTier 子进程意外退出后持续报告错误的“运行中”状态，并能够在
本地 IPC 观察到退出时按最后一次明确启动的房间重新启动子进程。

## 实现

- `EasyTierProcess::observe_exit` 通过 `try_wait` 回收已退出子进程。
- `ManagedDaemon` 在 `health`、`status.get` 和 `diagnostics.export` 前检查子进程；若已退出且
  存在活动房间，则从当前用户保护的入网材料重新启动。
- 重启仅由 IPC 观察触发，并以一秒节流，避免坏二进制或配置导致紧密重生循环。
- 用户明确停止隧道会清除活动房间，阻止随后自动重启。

## 验证

- Rust 单元测试以短命令进程验证退出回收；workspace 当前共 20 项 Rust 测试。
- `scripts/daemon-restart-smoke.ps1` 以临时批处理模拟无 TUN、两秒后异常退出的受管进程。
  启动房间、等待退出、调用健康 IPC 后状态为 `running`，说明 daemon 已启动替代进程；随后
  `tunnel.stop` 成功停止替代进程。所有临时配置、DPAPI 目录和脚本均被清理。
- 脚本使用有界 Pipe 就绪等待，避免首次 Rust 编译期间的误判超时。

## 边界

该验证不创建网络适配器、不连接 Relay，也不能证明跨主机数据面恢复。真实 TUN 和跨主机恢复
仍是发布阻断项。
