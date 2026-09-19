# 2026-09-19 P0 GUI 端到端验收状态

## 已确认

- 服务器本机控制面健康接口通过。
- 服务器本机经 DNS-01 + Caddy:8443 访问控制面返回 HTTP 200，响应为 `ok=true`、API `v1`、服务状态 `ok`。
- Windows 发行包中的 `WhaleLink.Desktop.exe` 已存在并已启动。

## 当前阻塞

本次 Windows Computer Use 辅助运行时初始化失败，错误为系统找不到其运行时路径；无法可靠获取 Avalonia 窗口、输入邀请码或点击“兑换并安全保存”。根据自动化安全规则，不使用猜测坐标、PowerShell UI Automation 或命令行冒充 GUI 操作，因此没有消费邀请码，也没有把 GUI 兑换标记为通过。

## 下一步

恢复 Windows Computer Use 运行时后，使用新的一次性邀请码重新执行：GUI 输入 HTTPS 控制面地址与邀请码 → 兑换 → 检查 DPAPI 当前用户凭据文件 → 通过 GUI 启动守护进程 → 隔离测试节点真实联机验收。

本报告不记录邀请码、管理员令牌、网络名、Relay、网络密钥或虚拟地址。
