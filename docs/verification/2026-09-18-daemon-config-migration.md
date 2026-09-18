# daemon 配置迁移、原子状态与结构化日志（2026-09-18）

## 实现

- `DaemonConfig::parse` 接受同一嵌套 `[easytier]` 结构的 schema v0，并在内存中无损迁移为
  v1；未知 schema 明确拒绝。
- `write_state_atomically` 的回归测试连续写入同一路径两次，验证 Windows 上最终状态为第二次写入，
  防止覆盖既有状态时退化。
- `whalelinkd` 在验证配置、启动 EasyTier、停止 EasyTier 和开始本地 IPC 服务时向 stderr 写入
  有界 JSON 事件。事件只含组件、事件名与 schema，不含路径、Relay、令牌、邀请码或入网材料。

## 验证

- `cargo test -p whalelink-core -p whalelinkd`：9 项测试通过，其中包含 v0→v1 迁移和重复原子写。
- `cargo clippy -p whalelink-core -p whalelinkd --all-targets -- -D warnings`：通过。
- 使用临时、非敏感 v0 TOML 运行 `whalelinkd validate-config`：输出 schema v1 有效和
  `configuration.validated` JSON 事件；断言该输出不包含可执行文件路径或目录名。临时文件在测试后删除。

## 边界

迁移只覆盖已定义的 v0→v1 无损结构，未知 schema 必须由发布方提供明确迁移，而不是猜测字段语义。
结构化事件是本地运维信号，不是诊断包，也不应添加任何配置内容。
