# 2026-09-19 Computer Use 运行时检查

## 检查结果

| 项目 | 结果 |
| --- | --- |
| `%LOCALAPPDATA%\\OpenAI\\Codex\\runtimes\\cua_node` | 存在 |
| CUA runtime manifest | 存在；平台 Windows x64，runtime archive version `0.0.16/20260915001755-492f19756c31` |
| `node.exe` / `node_repl.exe` | 存在 |
| `@oai/cua` 的 `codex-computer-use.exe` | 存在，1,550,128 bytes |
| `@oai/sky` 的 `codex-computer-use.exe` | 存在，1,550,128 bytes |
| CUA helper 初始化 | 失败：工具返回“failed to write kernel assets: 系统找不到指定的路径” |

## 结论

运行时文件并非缺失，也没有证据表明版本目录为空；故障发生在 Computer Use 工具初始化/写入其 kernel assets 阶段。已重置并重试一次，错误保持不变。未修改运行时目录。

## 下一步

请在 Codex 桌面端执行一次“重装/更新运行时”，然后完全退出并重启 Codex。重启后重新打开本任务，我再继续 GUI 邀请码→DPAPI→守护进程→真实联机验收。

本检查未接触邀请码、管理员令牌或入网材料。
