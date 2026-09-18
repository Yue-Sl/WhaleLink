# 本地脱敏诊断摘要（2026-09-18）

## 结果

`diagnostics.export` 不再返回占位状态。它返回 schema 版本、EasyTier 运行状态和明确的
脱敏声明：不含凭据、不含网络地址。诊断不会读取或回显 daemon 可执行路径、入网材料、Relay、
房间名、密钥或管理员令牌。

- 新增单元测试验证 `redacted=true`、`contains_credentials=false`、
  `contains_network_addresses=false`。
- Rust workspace：17 项测试通过。
- Clippy：通过。
- daemon/CLI 真实无 TUN 生命周期冒烟：通过。
