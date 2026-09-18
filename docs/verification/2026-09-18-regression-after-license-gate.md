# 许可证门禁后的回归（2026-09-18）

## 已执行

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/server-smoke.ps1`
- `scripts/daemon-pipe-smoke.ps1`
- `dotnet build apps/WhaleLink.Desktop/WhaleLink.Desktop.csproj --configuration Release --no-restore`

## 结果

所有命令通过：Rust workspace 共 17 个单元测试通过、Clippy 无警告；Server 冒烟验证了健康检查、
鉴权、邀请码持久化、兑换和成员状态；daemon 冒烟使用真实 EasyTier 执行了不创建 TUN 的
DPAPI 导入与 Pipe 控制的启动/停止生命周期；Avalonia Release 构建为 0 warnings、0 errors。

## 执行环境说明

首次在受限服务身份下执行时，Windows DPAPI 无法访问交互用户的密码库，因而 DPAPI 单测和
daemon 冒烟失败，错误为“找不到文件”。随后在交互用户的受控本地执行环境复跑，全部通过。
本结果不将受限服务身份的 DPAPI 失败归类为产品失败；发行与用户运行态要求当前用户凭据库可用。
