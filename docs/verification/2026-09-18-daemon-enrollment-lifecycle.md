# daemon 加密入网材料与 EasyTier 生命周期（2026-09-18）

## 范围

验证 Windows 本地守护进程通过 Named Pipe 接收结构化入网材料、以当前用户 DPAPI 保存，并
使用真实 EasyTier v2.6.4 进行无 TUN 的启动与停止。所有房间、网络名、密钥和 Relay 值均为
每次运行生成的临时测试数据，未记录在本报告。

## 结果

- `cargo test --workspace`：16 项测试通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- DPAPI 单元测试：加密输出不包含已知明文，解密回环通过。
- Pipe 冒烟：`enrollment.import`、`tunnel.start`、运行状态查询与 `tunnel.stop` 均通过。
- EasyTier 以 `--no-tun` 运行；测试不创建虚拟网卡、不修改路由或防火墙。
- 测试凭据目录仅在子进程的临时 `LOCALAPPDATA` 内创建，测试结束时删除。

## 排障留痕

第一次尝试因会话服务身份对真实用户 `LocalAppData` 的权限边界而失败，未启动 EasyTier；测试
随后改为临时用户数据目录。第二次尝试收到 `VALIDATION_FAILED`，原因是 PowerShell
`ConvertTo-Json` 默认深度截断嵌套 enrollment；增加 `-Depth 5` 后通过。

## 未覆盖项

未执行 TUN、真实 Relay、跨主机业务通信或驱动/防火墙变更。发行版不内置任何用户服务器地址或
网络密钥；这些由部署后的控制面配置和邀请码兑换提供。
