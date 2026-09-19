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
| Variable | `WHALELINK_TEST_IPV4` | 可选；本节点在测试网络使用的固定 IPv4 CIDR。为空时不写入配置。 |
| Secret | `WHALELINK_TEST_SECRET` | 受隔离测试网络密钥；仅注入该工作流运行环境。 |

工作流不会在 fork PR 上运行，以免将自托管 Runner 或测试密钥暴露给不受信任代码。脚本不回显
输入、命令行参数或 EasyTier 原始日志；失败时仅返回不含网络值的阶段性错误。

隔离测试网段、节点地址或密钥轮换时，管理员只更新相应的 Actions Variables/Secret；不得修改
源码、示例配置或本报告来保存实际值。`WHALELINK_TEST_IPV4` 与 `WHALELINK_TEST_PEER` 必须
属于同一隔离覆盖网段，且两端地址不得冲突。

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

## 第二次远程执行（2026-09-19）

- 提交：`5c121dfa5365c7d5b92ce09b394746f3983b11a3`；
- Run：`35407188078`，URL：`https://github.com/Yue-Sl/WhaleLink/actions/runs/35407188078`；
- 该次提交将仓库 Variable 契约更正为部署者指定的 `WHALELINK_TEST_NETNAME`；
- 前置检查：PASS；
- `Build Server image`：PASS；
- `Verify real EasyTier TUN peer path`：FAIL（退出码 1）。

这证明服务器本地 Rust 镜像缓存已使镜像构建门禁通过，但不证明真实覆盖网络可达。公开 Checks
API 仅返回 TUN 脚本的退出码，无法安全判定受保护配置、EasyTier、TUN 路由或远端对等端中的
具体根因。必须从仓库管理员可访问的原始步骤中提取非敏感错误末段并修复，再以一次完整 PASS
取代本结果。

## 第三次远程执行（2026-09-19）

脚本生成的配置已按 EasyTier v2 嵌套 TOML 结构改为 `[network_identity]`、`[[peer]]` 与
`[flags]`；在启动前执行静默 `--check-config`。`WHALELINK_TEST_IPV4` 是可选变量，非空时写入
`[flags].ipv4`。真实值不会写入 Git 或本报告。第三次 Run 必须先通过该配置校验，再报告 TUN
路由与对端 ICMP 的实际结果。

- 提交：`39ca2c10aa206b1852d3d07737aad1bc568fdbe3`；
- Run：`35409779704`，URL：`https://github.com/Yue-Sl/WhaleLink/actions/runs/35409779704`；
- 前置检查：PASS；
- `Build Server image`：PASS；
- `Verify real EasyTier TUN peer path`：FAIL（退出码 1）。

嵌套 TOML 修正后仍未通过真实 TUN 门禁。公开注释没有提供更细的非敏感失败阶段，因此不得将
结果归因于特定配置字段或远端网络。下一轮需要由具备仓库管理员权限的操作者提供已打码的
步骤末段，以进行针对性诊断。

## 入库前静态验证

- 已运行 `git diff --check`，通过；
- 已扫描即将追踪的源代码，确认真实测试 Relay、网络名、密钥和对端地址均未出现；
- 当前 Windows 开发主机没有 Bash/Linux 运行时，无法在本机解析或执行该 Bash 脚本。语法和
  真实 TUN 连通性以受保护自托管 Runner 的首次运行作为权威证据。
