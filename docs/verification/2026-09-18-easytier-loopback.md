# EasyTier 双节点回环验证

**日期：** 2026-09-18
**状态：** PASS

在经过三重 SHA-256 对账的官方 EasyTier v2.6.4 Windows x86_64 资产上执行：

- `easytier-core --check-config` 使用 network name、secret、DHCP、无监听器和禁用 UPnP 参数解析成功；
- 启动两个独立的 `easytier-core`：节点 A 监听 `tcp:19000`，节点 B 连接 `tcp://127.0.0.1:19000`；
- 两个节点均使用 `--no-tun`、`--disable-upnp`、回环 RPC 端口和随机测试网络名/密钥；
- `easytier-cli --rpc-portal 127.0.0.1:19101 --output json peer` 成功返回 peer RPC 数据；
- 两进程均在测试结束时停止。

该测试证明官方二进制、真实握手和 peer 发现可工作；它不替代创建 TUN 的真实业务连通、跨机器 NAT、SSH 或防火墙验收。
