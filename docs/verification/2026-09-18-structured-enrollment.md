# 结构化入网材料与无密钥快照（2026-09-18）

## 范围

验证 Server 将邀请码兑换为结构化 EasyTier 入网材料，并保持密钥不出现在配置仓库或状态快照。

## 结果

- Rust workspace：13 项测试全部通过。
- Clippy：`--workspace --all-targets -- -D warnings` 通过。
- Server 配置测试：验证固定房间的网络名、密钥环境变量引用和 Relay peers 解析。
- 状态测试：快照不包含邀请码明文，也不包含测试网络密钥；重启后使用相同受限定义重新注入材料并可兑换。
- `scripts/server-smoke.ps1`：通过健康、管理员鉴权、创建邀请码、重启恢复、兑换结构化材料和成员状态。
- Desktop Release build：0 warnings、0 errors；Pipe 冒烟：通过。

## 关键安全行为

本报告不包含任何邀请码、令牌、真实密钥、Relay 地址或机器地址。Server 的示例 TOML 仅使用
`network_secret_env`，实际密钥由受限服务环境提供。状态快照的 `data_plane` 字段被序列化跳过。

## 未覆盖项

本地 daemon 尚未导入该材料并实际启动 EasyTier；真实 Relay URL、TUN 和跨主机业务流量也未在
本次测试中声明完成。
