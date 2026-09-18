# daemon 本地 Named Pipe 冒烟测试

**日期：** 2026-09-17
**状态：** PASS

执行 `scripts/daemon-pipe-smoke.ps1`：启动 `whalelinkd serve`，通过 .NET `NamedPipeClientStream`
连接 `WhaleLink.v1`，发送 IPC v1 `health` 请求并验证返回 `stopped` 状态。测试结束后停止 daemon。

未启动 EasyTier，未修改网络配置；未覆盖当前用户 SID ACL 或 GUI 界面自动化。
