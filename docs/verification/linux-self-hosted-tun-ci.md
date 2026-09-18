# 自托管 Linux 真实 TUN CI 门禁

**状态：** FAIL（首次远程执行在镜像构建阶段停止）
**范围：** GitHub Actions Runner 标签为 `self-hosted`、`linux`、`whalelink-linux` 的受保护主机。

`/.github/workflows/linux-verify.yml` 在受保护的 `main` 推送与手动触发时执行以下门禁：

1. 验证 `/dev/net/tun`、免交互 `sudo`、Docker、systemd 工具和锁定的 EasyTier 版本；
2. 由 `deploy/Dockerfile.server` 构建并检查 `whalelink-server` 镜像；
3. 调用 `scripts/easytier-linux-tun-smoke.sh`，以临时 `0600` 配置启动 EasyTier；
4. 仅在路由设备确认为 TUN 且对保护的远程测试对端 ICMP 成功后通过；进程与临时配置会在退出时删除。

## Actions 配置契约

在仓库 **Settings → Secrets and variables → Actions** 中设置下列值。真实值不得写入 Git、
Actions 日志、诊断包或本报告：

| 类型 | 名称 | 用途 |
| --- | --- | --- |
| Variable | `WHALELINK_EASYTIER_CORE_PATH` | 可选；EasyTier 不在 `PATH` 时的绝对可执行路径。 |
| Variable | `WHALELINK_TEST_RELAY` | 受隔离测试环境使用的 TCP Relay URL。 |
| Variable | `WHALELINK_TEST_NETNAME` | 受隔离测试网络名。 |
| Variable | `WHALELINK_TEST_PEER` | 覆盖网络中的远程 IPv4 测试对端。 |
| Secret | `WHALELINK_TEST_SECRET` | 受隔离测试网络密钥；仅注入该工作流运行环境。 |

工作流不会在 fork PR 上运行，以免将自托管 Runner 或测试密钥暴露给不受信任代码。脚本不回显
输入、命令行参数或 EasyTier 原始日志；失败时仅返回不含网络值的阶段性错误。

## 通过证据

首次成功运行后，在此追加 Actions run URL、提交 SHA、UTC 时间和各门禁的 PASS 摘要。不要记录
Runner 主机名、Relay、网络名、密钥、对端地址、MAC、路由表或原始日志。

## 首次远程执行（2026-09-18）

- 提交：`52365101da6f6df4b6acd7f8c76857f4add52ce3`；
- Run：`35357585207`，URL：`https://github.com/Yue-Sl/WhaleLink/actions/runs/35357585207`；
- `Verify protected runner prerequisites`：PASS，证明指定标签 Runner、`/dev/net/tun`、免交互
  `sudo`、Docker、systemd 工具和 EasyTier 可发现性均通过前置检查；
- `Build Server image`：FAIL（退出码 1）；
- `Verify real EasyTier TUN peer path`：SKIPPED，未因构建失败而错误地宣称通过。

公开 Checks API 仅返回退出码；Actions 原始日志下载要求仓库管理员权限。因此该报告不推断构建
根因，也不复制可能含环境信息的原始日志。修复需要该步骤的非敏感错误末段或已认证终端的同等
构建输出；随后必须重跑工作流并以实际 TUN PASS 取代本状态。

## 入库前静态验证

- 已运行 `git diff --check`，通过；
- 已扫描即将追踪的源代码，确认真实测试 Relay、网络名、密钥和对端地址均未出现；
- 当前 Windows 开发主机没有 Bash/Linux 运行时，无法在本机解析或执行该 Bash 脚本。语法和
  真实 TUN 连通性以受保护自托管 Runner 的首次运行作为权威证据。
