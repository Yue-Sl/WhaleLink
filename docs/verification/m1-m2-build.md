# M1/M2 本地构建与测试证据

**执行日期：** 2026-09-17
**状态：** PASS（源码质量门禁）；PARTIAL（运行态与真实网络）

| 检查 | 结果 |
| --- | --- |
| Rust 工具链 | PASS：官方 rustup 安装 `1.85.1` minimal profile |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --workspace` | PASS：9 项测试，0 失败 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| CLI 配置示例 | PASS：`valid schema_version=1` |
| Avalonia Release 构建 | PASS：0 warnings，0 errors |

测试覆盖 Core TOML 校验、日志脱敏、状态序列化、协议包络、IPC 版本拒绝、邀请码单次
兑换、邀请码撤销、凭据 epoch 轮换以及 `/api/v1/rooms` 管理员鉴权。

未执行：daemon 的 Windows Named Pipe 服务端 ACL、DPAPI 存储、Server 持久化、真实
EasyTier 进程、Linux 容器、双节点网络、轮换后的在线重连和发行安装/回滚。它们不能从
本报告推断为已实现或已验证。
