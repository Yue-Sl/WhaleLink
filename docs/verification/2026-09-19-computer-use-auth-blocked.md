# 2026-09-19 Computer Use 初始化复核

## 结果

- `cua_node` 运行时目录、Windows x64 runtime、`node_repl` 和 `codex-computer-use.exe` 均存在。
- 重新初始化后，运行时不再报 kernel assets 路径错误；但 CUA 服务返回 `unsupported Codex auth method: apikey`，应用枚举为空。
- 桌面端 `WhaleLink.Desktop.exe` 进程仍在运行，但无法取得可靠窗口句柄或 accessibility 状态。
- 部署者重启 Codex 后再次执行一次只读 `getState()` 复测，结果未变：应用与浏览器枚举仍为空，并返回相同的认证方式不受支持错误。该项复测没有输入邀请码、访问控制面或改变 EasyTier/Windows 服务状态。
- 后续非敏感本机诊断确认 `codex login status` 为 API key 会话，且本地认证文件仅含 API key 凭据；未发现 `OPENAI_API_KEY` 环境变量覆盖。已启动官方 `codex login --device-auth` 的账户设备授权流程；该流程正在等待账户持有人确认，未读取、复制或记录一次性授权码。

## 影响

GUI 一次性邀请码输入、DPAPI 保存、GUI 启动守护进程和后续真实联机尚未执行。没有用直接 API 调用冒充 GUI 结果，也没有用终端 UI 自动化绕过 Computer Use。

## 下一步

完成已启动的设备授权后，重启 Codex 桌面端以载入账户会话；再重新运行 Computer Use 初始化。恢复后使用新的短期邀请码继续验收。

本报告不记录邀请码、管理员令牌、网络名、Relay、网络密钥或虚拟地址。
