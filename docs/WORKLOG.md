# WhaleLink 工作日志

本文件是任务状态的唯一事实来源。记录不得含令牌、邀请码、私钥、原始配置或可识别
机器地址；命令仅保留可复现摘要，完整的非敏感报告放入 `docs/verification/`。

## 记录模板

### WL-XXXX — 标题

- **状态：** PLANNED | IN PROGRESS | PASS | BLOCKED | FAIL
- **目标与前置条件：**
- **改动：**
- **关键命令：**
- **验证证据：**
- **风险/阻塞：**
- **下一步：**

## 任务记录

### WL-0058 — EasyTier v2 嵌套 TOML 配置修正

- **状态：** FAIL
- **目标与前置条件：** WL-0057 的真实 TUN 冒烟在镜像构建通过后失败；部署者提供 EasyTier v2 所需的 `network_identity`、`peer` 与 `flags` TOML 结构。
- **改动：** 待将 Linux 冒烟脚本由旧扁平字段改为嵌套 TOML，增加启动前静默 `--check-config`，并将可选静态 IPv4 作为 Actions Variable 注入，而非硬编码。
- **关键命令：** `git diff --check`；敏感值扫描；静态脚本结构复核；推送触发指定自托管 Runner。
- **验证证据：** 部署者提供的配置结构已转换为模板化环境变量路径；提交 `39ca2c10aa206b1852d3d07737aad1bc568fdbe3` 的 Run `35409779704`（`https://github.com/Yue-Sl/WhaleLink/actions/runs/35409779704`）前置检查 PASS、镜像构建 PASS、真实 TUN 冒烟 FAIL（退出码 1）。真实 Relay、网络名、密钥、对端和静态 IPv4 不写入仓库、日志或本记录。
- **风险/阻塞：** 公开 Checks API 仍只提供 TUN 步骤的退出码，不能据此断言 `--check-config`、受保护变量、进程启动、TUN 路由或远端 ICMP 中的具体失败点；真实双节点 TUN 发布门禁保持未关闭。
- **下一步：** 从仓库管理员可访问的 TUN 步骤中提取非敏感失败阶段，优先确认是配置校验、EasyTier 进程存活、TUN 路由还是 ICMP 超时；完成针对性修复后重跑。

### WL-0057 — Linux CI 重试与测试参数契约修正

- **状态：** FAIL
- **目标与前置条件：** 部署者已确认 Runner 可用并要求在使用服务器本地 Rust 镜像缓存后重跑。测试输入应保存为受保护的 GitHub Actions Variables/Secret，网络名变量名以部署者指定的 `WHALELINK_TEST_NETNAME` 为准。
- **改动：** 待将 Linux 工作流从过时的 `WHALELINK_TEST_NETWORK` 改为 `WHALELINK_TEST_NETNAME`，同步非敏感配置契约并推送触发新 Run。
- **关键命令：** `git diff --check`；敏感值扫描；`git push`；GitHub Actions Run/Jobs 只读状态查询。
- **验证证据：** WL-0056 的 Run `35357585207` 前置检查通过、镜像构建失败、TUN 步骤跳过；变量名修正后的提交 `5c121dfa5365c7d5b92ce09b394746f3983b11a3` 触发 Run `35407188078`（`https://github.com/Yue-Sl/WhaleLink/actions/runs/35407188078`）。第二次 Run 的前置检查 PASS、`Build Server image` PASS、`Verify real EasyTier TUN peer path` FAIL（退出码 1）。本机未安装 GitHub CLI，无法在本地写入或查询 GitHub Actions Variables/Secret。
- **风险/阻塞：** 真实测试参数绝不进入 Git。公开 Checks API 仅返回 TUN 步骤的退出码，无法安全判定是受保护参数缺失、对端不可达、路由/TUN 行为还是 EasyTier 运行异常；真实双节点 TUN 发布门禁仍未关闭。
- **下一步：** 由仓库管理员从该 Run 的 TUN 步骤提取非敏感失败末段（不含 Relay、网络名、密钥、对端或原始日志）以定位；修复后重新运行并以实际 PASS 更新发布门禁。

### WL-0056 — 自托管 Linux CI 与真实双节点 TUN 门禁

- **状态：** FAIL
- **目标与前置条件：** 部署者确认带有 `self-hosted`、`linux`、`whalelink-linux` 标签的 Runner 已在线，且 Docker、systemd 和锁定的 EasyTier v2.6.4 可用；隔离的外连双节点测试环境已由部署者自测。
- **改动：** 新增 `.github/workflows/linux-verify.yml`、`scripts/easytier-linux-tun-smoke.sh` 和非敏感 CI 配置契约。工作流只运行在指定自托管标签上，构建 Server 镜像后才执行受 Actions Variables/Secret 注入的真实 TUN/ICMP 门禁。
- **关键命令：** 本地执行 `git diff --check` 与敏感测试值扫描；远程执行前置检查、`docker build --file deploy/Dockerfile.server` 与 TUN 冒烟步骤。
- **验证证据：** 触发提交 `52365101da6f6df4b6acd7f8c76857f4add52ce3` 的 Linux Run `35357585207`（`https://github.com/Yue-Sl/WhaleLink/actions/runs/35357585207`）：Runner 前置检查 PASS；Server 镜像构建 FAIL（退出码 1）；TUN 步骤 SKIPPED。真实 Relay、网络名、密钥和对端地址不在追踪内容或本记录中。
- **风险/阻塞：** GitHub 无认证日志下载接口要求仓库管理员权限，公开检查注释只提供构建阶段的退出码，尚不能据此安全推断 Docker 构建根因。该失败使真实 TUN 连通门禁保持未验收；Actions Variables/Secret 仍须按 `docs/verification/linux-self-hosted-tun-ci.md` 的契约配置。
- **下一步：** 由仓库管理员提供该 Run 的“Build Server image”非敏感错误末段，或在已认证终端重新运行相同 Docker 命令并提供输出；修复后重新触发工作流，再记录真实 TUN 结果。

### WL-0055 — GitHub 首次推送与远程 CI 启用

- **状态：** PASS
- **目标与前置条件：** 部署者已将现有 SSH 公钥添加到 GitHub；将本地 `main` 的首版工程推送至 `Yue-Sl/WhaleLink`，以便取得远程 CI 记录。
- **改动：** 已配置 `origin` 为 `git@github.com:Yue-Sl/WhaleLink.git`；将 `main` 推送并设置其跟踪 `origin/main`。
- **关键命令：** `git remote add origin git@github.com:Yue-Sl/WhaleLink.git`；`git branch -M main`；`git push -u origin main`；`git ls-remote origin refs/heads/main`。
- **验证证据：** 首次推送成功。远程 `refs/heads/main` 与本地提交 `52ff5c7c1affb619144ec3cbb24ad5452a4d6c6a` 一致；`git status --short` 无输出。SSH 私钥与公钥均未写入本仓库或本工作日志。
- **风险/阻塞：** 标准 SSH 端口可用，未需切换至 SSH 443。远程 CI 已具备运行条件，但尚未取得实际 GitHub Actions 运行记录；其余 WL-0054 外部运行时/网络验收阻断项仍存在。
- **下一步：** 推送本完成记录；在 GitHub Actions 出现首个运行结果后，将其链接为 CI 验证证据，并继续执行外部发布矩阵。

### WL-0054 — 首版最终本地审计与外部发布阻断

- **状态：** BLOCKED
- **目标与前置条件：** 完成当前工作树、发行检查清单、目标运行时和 Git remote 的最终本地审计，确定首版是否可诚实标记为正式发布。
- **改动：** 无产品代码改动；审计当前 Git 状态、最近提交、发行检查清单、Docker/Podman/WSL 可用性和 remote 配置。
- **关键命令：** `git status`；`git log`；读取 `docs/verification/release-checklist.md`；`Get-Command docker,podman,wsl`；`wsl --list --verbose`；`git remote -v`。
- **验证证据：** 工作树干净，最新本地提交为 `93e25e1`。仅发现 WSL 客户端且没有已安装 Linux 发行版；未发现 Docker/Podman；仓库无 remote。发行检查清单仍明确保留真实 TUN/双节点、Linux Docker/systemd/回滚、跨用户 Pipe、远程 CI 和 GitHub Release 五类未验收项。
- **风险/阻塞：** 当前主机无法在不安装系统级运行时的前提下执行 Linux Docker/systemd 或真实跨主机网络验收；无 remote 不能取得 CI 运行记录或创建 GitHub Release。安装 WSL/Docker、提供 Linux/Windows 测试环境、连接远程仓库均需要部署者授权或外部状态变化。
- **下一步：** 部署者提供可访问的 Linux Docker/systemd 主机、隔离 Windows 双用户与双节点网络环境，以及远程仓库后，继续执行未勾选发布矩阵并更新本记录。

### WL-0053 — 凭据 ACL 加固后的发行候选重建

- **状态：** PASS
- **目标与前置条件：** WL-0052 改变桌面与 daemon 凭据存储行为，必须生成对应发行候选而不能复用旧归档。
- **改动：** 重跑完整本地回归、DPAPI/ACL 合同、Windows/Linux 打包、SBOM、校验清单和当前用户安装/卸载验收。
- **关键命令：** workspace `fmt/test/clippy`；Server、daemon Pipe/restart、二维码、Desktop ACL 合同；Desktop Release build；两个打包脚本；归档结构/校验和复算；隔离安装卸载。
- **验证证据：** `docs/verification/2026-09-18-credential-directory-acl.md`、`docs/verification/2026-09-18-license-release-gate.md`。20 项 Rust 测试通过；Windows 便携/安装包为 49/52 项、清单 102 条；Linux 包为 50 项、清单 51 条，全部复算一致。
- **风险/阻塞：** 真实 Linux、双节点 TUN/Relay、跨用户 Pipe、远程 CI 和 GitHub Release 阻断项仍未关闭。
- **下一步：** 创建本地增量提交；目标环境可用后执行剩余发布矩阵。

### WL-0052 — Windows DPAPI 凭据目录的显式用户 ACL

- **状态：** PASS
- **目标与前置条件：** 规格要求 Windows 凭据在 DPAPI 之外受当前用户 ACL 保护；旧实现依赖 LocalAppData 默认继承，daemon 导入路径未显式设置 ACL。
- **改动：** 桌面端保存前以当前用户 SID 设置受保护 DACL；Rust daemon 以 Owner Rights SDDL 保护当前用户创建的目录；Pipe 冒烟增加 daemon ACL 断言，Desktop ACL 合同脚本加入 Windows CI。
- **关键命令：** Desktop Release build；`scripts/desktop-credential-acl-contract.ps1`；`cargo test/clippy -p whalelink-core -p whalelinkd`；`scripts/daemon-pipe-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-18-credential-directory-acl.md`。Desktop DPAPI 目录的所有者和唯一允许主体均为当前用户；daemon 目录继承关闭、所有者为当前用户且唯一允许主体为 Owner Rights/current user；无 TUN 生命周期继续通过。
- **风险/阻塞：** 目录 ACL 不替代跨 Windows 用户实际调用 Pipe 的拒绝测试；该项仍未验收。
- **下一步：** 重跑全 workspace 和发行候选；在独立用户会话执行跨用户 Pipe ACL 验收。

### WL-0051 — daemon Owner Rights ACL 冒烟首次语义断言

- **状态：** FAIL
- **目标与前置条件：** 为 daemon 新增受保护凭据目录 ACL 后首次运行 Pipe 冒烟。
- **改动：** 无产品逻辑失败；断言误把 Owner Rights（`S-1-3-4`）当作非当前用户 SID。实际目录继承已关闭，Owner Rights 是目录当前所有者的权限主体。
- **关键命令：** `scripts/daemon-pipe-smoke.ps1`。
- **验证证据：** 初始错误“directory was not restricted to the current user”；后续验证增加目录所有者等于当前用户以及唯一允许项为 Owner Rights/current SID 的语义检查，见 WL-0052。
- **风险/阻塞：** 该首次断言不得标为通过。
- **下一步：** 见 WL-0052。

### WL-0050 — 凭据目录 ACL 首次编译验证

- **状态：** FAIL
- **目标与前置条件：** 首次实现桌面端受保护 DACL。
- **改动：** `Directory.SetAccessControl` 在目标 .NET API 中没有所用静态重载，导致编译失败；已改为 `DirectoryInfo.SetAccessControl`。
- **关键命令：** Desktop Release build。
- **验证证据：** 编译错误 `SetAccessControl method does not contain an overload that takes 2 arguments`；修正后构建和 ACL 合同通过，见 WL-0052。
- **风险/阻塞：** 该首次编译不得标为通过。
- **下一步：** 见 WL-0052。

### WL-0049 — 二维码与 HTTPS 加固后的发行候选重建

- **状态：** PASS
- **目标与前置条件：** WL-0048 改变桌面端和 Linux 源码包内容，先前候选不再代表当前实现。
- **改动：** 运行完整本地回归及二维码契约；重建 Windows/Linux 发行物、SBOM 和校验清单；复核归档结构、许可证、缓存排除和当前用户安装/卸载。
- **关键命令：** workspace `fmt/test/clippy`；Server、正常 Pipe、异常重启 daemon 冒烟；Desktop build 与二维码契约；两个打包脚本；SHA-256 逐条复算；隔离安装/卸载。
- **验证证据：** `docs/verification/2026-09-18-desktop-invite-payload.md`、`docs/verification/2026-09-18-license-release-gate.md`。Windows 便携/安装包为 49/52 项、清单 102 条；Linux 包为 50 项、清单 51 条，均通过复算。
- **风险/阻塞：** 目标 Linux、双节点 TUN/Relay、跨用户 ACL、远程 CI 和 GitHub Release 仍是未关闭的发布阻断项。
- **下一步：** 将 QR/HTTPS 更改创建为本地提交；等待或接入目标环境进行剩余发布验收。

### WL-0048 — Windows 客户端二维码/邀请码安全导入

- **状态：** PASS
- **目标与前置条件：** M3 要求客户端支持二维码/邀请码导入；旧 GUI 只能手输邀请码，且未在桌面控制面入口强制 HTTPS。
- **改动：** 新增本地二维码载荷解析，支持原始邀请码及带 HTTPS 控制面地址的 `whalelink://invite` 格式；导入后清空载荷栏，只填充既有兑换控件。控制面客户端与管理员界面统一拒绝 HTTP。新增程序集契约脚本，并在 Windows Desktop CI 使用 PowerShell 7 运行。
- **关键命令：** Desktop Release build；`pwsh -NoProfile -File scripts/desktop-invite-payload-contract.ps1`。
- **验证证据：** `docs/verification/2026-09-18-desktop-invite-payload.md`。HTTPS 二维码与原始邀请码接受；缺邀请码和 HTTP 地址在本地拒绝；0 warnings、0 errors。
- **风险/阻塞：** 首发不包含相机权限、二维码图像识别或二维码生成；用户需粘贴扫码器或受控渠道解码的文本。控制面服务器仍须由部署者配置 TLS。
- **下一步：** 重跑全 workspace 回归并重建最新发行候选；真实 GUI 视觉验收和目标网络验收仍在发布阻断项内。

### WL-0047 — 二维码契约脚本的 Windows PowerShell 兼容性

- **状态：** FAIL
- **目标与前置条件：** 首次从 PowerShell 5.1 加载 Desktop `net8.0-windows` 程序集执行二维码解析契约。
- **改动：** 无产品逻辑失败；Windows PowerShell 5.1 不能加载 .NET 8 的 `System.Runtime`，脚本在 `GetType` 前失败。
- **关键命令：** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/desktop-invite-payload-contract.ps1`。
- **验证证据：** `Could not load file or assembly 'System.Runtime, Version=8.0.0.0'`。已改为 PowerShell 7 并在 WL-0048 验证通过。
- **风险/阻塞：** 该脚本不得在 Windows PowerShell 5.1 环境运行；CI 显式使用 `pwsh`。
- **下一步：** 见 WL-0048。

### WL-0046 — 异常重启版本的发行候选重建

- **状态：** PASS
- **目标与前置条件：** WL-0045 改变 daemon 生命周期与 CI 配置；先前归档不能代表当前代码。
- **改动：** 重跑完整 workspace 回归、桌面 Release 构建和 Windows/Linux 打包；重新生成 SBOM 与 SHA-256 清单；从新 Windows 安装 ZIP 执行隔离安装/载荷检查/卸载。
- **关键命令：** `cargo fmt/test/clippy`；Server、daemon Pipe、daemon restart 冒烟；Desktop Release build；两个打包脚本；ZIP 结构与逐条 SHA-256 复算；当前用户安装/卸载验收。
- **验证证据：** `docs/verification/2026-09-18-daemon-restart.md`、`docs/verification/2026-09-18-license-release-gate.md`。20 项 Rust 测试通过；Windows 便携/安装包为 49/52 项、清单 102 条；Linux 包为 49 项、清单 50 条，均复算一致。
- **风险/阻塞：** 当前候选仍不等同于 Linux Docker/systemd、真实 TUN/跨主机或远程 CI/GitHub Release 通过；这些阻断项见 WL-0038。
- **下一步：** 创建本地增量源代码提交；在可用目标环境执行剩余发行矩阵。

### WL-0045 — daemon 异常退出后的受管重启

- **状态：** PASS
- **目标与前置条件：** M0 要求验证受管进程异常重启；旧 daemon 只检查 `Option` 存在性，子进程退出后可能错误显示为运行。
- **改动：** Core 增加子进程退出回收；daemon 在本地状态观察时按最后活动房间受控重启，并对持续失败节流；显式停止清除重启意图。新增无 TUN 重启烟测和 Windows CI 步骤。
- **关键命令：** `cargo test -p whalelink-core -p whalelinkd`；`cargo clippy -p whalelink-core -p whalelinkd --all-targets -- -D warnings`；`scripts/daemon-pipe-smoke.ps1`；`scripts/daemon-restart-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-18-daemon-restart.md`。临时子进程异常结束后，IPC 健康状态触发替代进程启动并返回 `running`；替代进程可由 IPC 停止。
- **风险/阻塞：** 重启在 IPC 观察时触发，非后台轮询；真实 TUN、跨主机和 Relay 恢复仍未验证。
- **下一步：** 重跑完整 workspace 回归与发行包；在独立双节点环境验证真实数据面恢复。

### WL-0044 — 异常重启烟测首次启动同步

- **状态：** FAIL
- **目标与前置条件：** 首次执行新增异常重启烟测。
- **改动：** 无产品代码改动；测试在首次 Rust 编译完成前只等待单次 5 秒 Pipe 连接，导致超时。
- **关键命令：** `scripts/daemon-restart-smoke.ps1`。
- **验证证据：** `NamedPipeClientStream.Connect(5000)` 超时；进程检查确认没有残留 daemon。随后加入 30 次有界 Pipe 就绪等待，见 WL-0045。
- **风险/阻塞：** 该首次执行不得标记为通过。
- **下一步：** 见 WL-0045 的修正后验证。

### WL-0043 — 可追溯首个本地源代码提交

- **状态：** PASS
- **目标与前置条件：** 工程所有文件仍未跟踪，且没有历史提交或 remote；需要在不推送外部服务的前提下形成可复查的本地源代码基线。
- **改动：** 准备将受 `.gitignore` 约束的源码、配置、文档、CI 与脚本加入首个本地 Git 提交；发行物、上游 ZIP、缓存、运行态和凭据目录必须保持排除。
- **关键命令：** `git add --all`；暂存差异空白检查；忽略规则复核；`git commit`。
- **验证证据：** 暂存 90 个文件，`git diff --cached --check` 通过；工作日志中的 14 处 Markdown 行尾空格已做机械清理后复验。上游 ZIP、`vendor/easytier/` 与 `artifacts/` 均保持忽略。首个本地源代码提交为 `d909cb21a166c09699e0f1e0824d25e1ef4ce802`（`feat: bootstrap WhaleLink v3 engineering edition`）。
- **风险/阻塞：** 未配置 remote，因此本任务不推送、不创建远程 CI 运行或 GitHub Release。
- **下一步：** 将本次完成记录作为后续本地文档提交保存；目标环境发布阻断项继续遵循 WL-0038。

### WL-0042 — Linux 部署包的自引用证据修复

- **状态：** PASS
- **目标与前置条件：** WL-0041 复包后发现 Linux 源包包含会写入自身哈希的工作日志与验证报告，不能作为稳定发行候选。
- **改动：** Linux 打包脚本保留部署需要的静态文档、锁定和 ADR，但排除 `docs/verification/` 与工作日志；新增归档结构断言，防止缓存和自引用证据重新进入发行包。
- **关键命令：** `scripts/package-linux-deployment.ps1`；静态文档存在性、验证/工作日志排除、构建缓存排除、逐条 SHA-256 复算。
- **验证证据：** `docs/verification/2026-09-18-linux-package-reproducibility.md`。Linux 包有 49 项、校验清单有 50 条，最终 SHA-256 为 `6514a7e2ef6063a27ca7a44f4454640ab9b2e8e185b2cc78bc4249d454a9b8b6`。
- **风险/阻塞：** 发行包已消除证据自引用，但 Linux Docker/systemd 实际运行仍受 WL-0038 目标环境阻断。
- **下一步：** 将此稳定 Linux 候选交给具备 Docker/systemd 的目标环境执行部署和回滚验收。

### WL-0041 — daemon 增强后的发行候选重建

- **状态：** PASS
- **目标与前置条件：** WL-0040 变更了守护进程二进制和 Linux 源码包内容，旧哈希及发行物不能继续作为候选。
- **改动：** 重跑 Windows/Linux 打包；重新生成 SBOM、SHA-256 清单和归档；从新 Windows 安装 ZIP 进行隔离当前用户安装、载荷验证和卸载。
- **关键命令：** 全 workspace `fmt/test/clippy`、Server/daemon 冒烟、Desktop Release 构建、两个打包脚本、ZIP 内容与缓存排除扫描、逐条校验和复算、隔离安装/卸载。
- **验证证据：** `docs/verification/2026-09-18-license-release-gate.md`、`docs/verification/2026-09-18-daemon-config-migration.md`。全 workspace 19 项 Rust 测试通过；Windows 包 49/52 项、Linux 包 79 项，Windows 102 条与 Linux 80 条校验和一致。
- **风险/阻塞：** 当前候选仍未完成 Linux Docker/systemd、跨用户 Pipe ACL、真实 TUN/跨主机/Relay-NAT 和远程 CI/GitHub Release 验收；详见 WL-0038 和发行检查清单。
- **下一步：** 将候选放入具备 Linux/Docker/systemd 和隔离双节点网络的环境执行发布阻断矩阵；准备 remote 后运行 CI 与发布。

### WL-0040 — daemon 配置迁移、原子状态与结构化日志

- **状态：** PASS
- **目标与前置条件：** M1 要求配置校验/迁移、原子状态写和结构化日志；旧实现只接受 v1，且未验证 Windows 覆盖既有状态。
- **改动：** 支持相同嵌套结构的 v0→v1 无损内存迁移；新增连续状态写覆盖测试；daemon 增加无敏感字段的 JSON 生命周期事件。配置样例明确 v0 迁移边界。
- **关键命令：** `cargo fmt --all`；`cargo test -p whalelink-core -p whalelinkd`；`cargo clippy -p whalelink-core -p whalelinkd --all-targets -- -D warnings`；临时 v0 `validate-config` 脱敏烟测。
- **验证证据：** `docs/verification/2026-09-18-daemon-config-migration.md`。9 项 Rust 测试和 Clippy 通过；v0 文件被验证为 v1，日志事件不包含临时配置路径。
- **风险/阻塞：** 当前仅定义 v0→v1 无损迁移；未来有破坏性 schema 变更时必须新增显式迁移与测试。结构化事件不能扩展为包含配置的调试日志。
- **下一步：** 重跑全 workspace 回归并重新生成受影响的 Windows/Linux 发行物；目标环境发布阻断项继续按 WL-0038 执行。

### WL-0039 — daemon 配置迁移首次验证

- **状态：** FAIL
- **目标与前置条件：** 验证新增 v0→v1 迁移和状态覆盖测试。
- **改动：** 无额外产品改动；首次测试暴露新增测试错误引用了未声明的 `uuid` crate，随后烟测还发现 PowerShell 对多行输出的逐项匹配会误判通过输出为失败。
- **关键命令：** `cargo test -p whalelink-core -p whalelinkd`；临时 v0 `validate-config`。
- **验证证据：** 编译错误 `use of undeclared crate or module uuid`；后续 WL-0040 以标准库临时后缀和合并输出断言修复并通过。
- **风险/阻塞：** 此记录保留失败证据；不得将首次验证标为通过。
- **下一步：** 见 WL-0040 的通过验证。

### WL-0038 — Linux 目标环境与远程 CI 可用性复核

- **状态：** BLOCKED
- **目标与前置条件：** 为关闭发布检查清单中的 Docker、systemd、跨主机数据面和远程 CI 项，检查当前主机及仓库是否存在可执行目标环境。
- **改动：** 无产品代码改动；执行只读的运行时、WSL 和 Git remote 检查。
- **关键命令：** `Get-Command docker,podman,wsl`；`wsl --list --verbose`；`git remote -v`；`git log --all`。
- **验证证据：** WSL 客户端存在但未安装 Linux 发行版；未发现 Docker 或 Podman；Git 仓库没有 remote、没有历史提交。`docs/verification/release-checklist.md` 保持 Linux/远程 CI/发布项未勾选。
- **风险/阻塞：** 不能在此主机实际启动 Linux Docker 镜像或 systemd 服务，也不能把 CI workflow 推送到远程仓库取得运行记录。安装 WSL 发行版/Docker、连接远程仓库或使用真实 Relay 都是工作区外的变更，需要部署者提供目标环境或明确授权。
- **下一步：** 在具备 Docker 与 systemd 的 Linux 主机执行部署验收；在隔离 Windows 双用户和双节点网络环境执行 TUN/ACL/连通性矩阵；准备远程仓库后运行 CI 和 GitHub Release。

### WL-0037 — 通用发行边界复审

- **状态：** PASS
- **目标与前置条件：** Windows/Linux 发行物已在 WL-0036 重建；必须再次确认通用发行版不嵌入部署者网络或凭据。
- **改动：** 无产品行为变更；对可提交源码、配置样例、文档、脚本、CI 和当前发行载荷执行静态边界审计。审计范围明确排除编译缓存、第三方临时缓存和产物目录。
- **关键命令：** 非公开 IPv4 形态扫描、24 位十六进制密钥形态扫描、命中文件人工复核、归档载荷检查。
- **验证证据：** `docs/verification/2026-09-18-release-boundary-rescan.md`。密钥形态扫描为 0；IPv4 正则的两个命中均为应用/依赖版本号，非网络地址。
- **风险/阻塞：** 静态扫描不是未来变更的替代品；每次添加配置、诊断或发行载荷都需要复审。真实部署参数只能在仓库外的受限环境出现。
- **下一步：** 在目标 Linux/网络环境完成剩余发布阻断测试，并将远程 CI 记录关联到发行候选版本。

### WL-0036 — 管理员控制台后的发行物重建

- **状态：** PASS
- **目标与前置条件：** WL-0035 改变 Windows 桌面发行内容；不得让已有的旧 ZIP 被误称为最新首版产物。
- **改动：** 重新运行 Windows 和 Linux 打包脚本，重新生成 SBOM、载荷和 SHA-256 清单；不引入部署配置、令牌、邀请码或网络参数。
- **关键命令：** `scripts/package-windows.ps1`；`scripts/package-linux-deployment.ps1`；ZIP 必需载荷及缓存排除检查；逐条 `SHA256SUMS.txt` 复算；从新的 Windows 安装 ZIP 进行隔离安装、载荷检查和卸载。
- **验证证据：** `docs/verification/2026-09-18-license-release-gate.md`。Windows 便携包 49 项、安装包 52 项、Linux 部署包 77 项；Windows 102 条和 Linux 78 条校验和均一致。新安装包的当前用户安装/卸载复验通过。
- **风险/阻塞：** 发行包已可生成并在 Windows 当前用户路径验收，但不代表 Linux Docker/systemd、真实 TUN、跨主机、Relay/NAT 或跨用户 Pipe ACL 验收已完成。
- **下一步：** 在具备 Docker/systemd 与隔离网络的目标环境完成发布阻断矩阵，再运行远程 CI 并创建 GitHub Release。

### WL-0035 — Windows 桌面管理员控制台

- **状态：** PASS
- **目标与前置条件：** 冻结规格要求管理端查看固定房间、创建/撤销邀请、成员状态、连接诊断和凭据轮换；旧桌面端只覆盖客户端兑换和连接。
- **改动：** 桌面端增加客户端/管理员工作区。管理员可用临时令牌查询房间、创建/撤销一次性邀请码、查询成员和轮换 credential epoch；客户端增加脱敏诊断摘要操作。控制面客户端扩展所有对应 `/api/v1` 调用和统一错误包络解析。管理员令牌不写入任何本地存储或日志。
- **关键命令：** `dotnet build apps/WhaleLink.Desktop/WhaleLink.Desktop.csproj --configuration Release --no-restore`；`scripts/server-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-18-desktop-admin-console.md`。桌面端 Release 构建为 0 warnings、0 errors；控制面回环冒烟覆盖邀请码创建、撤销拒绝、重启恢复、成员查询和 credential epoch 持久化。
- **风险/阻塞：** 当前桌面端未在屏幕自动化环境做视觉验收；管理员 GUI 不替代部署端 TLS/访问控制。实际 EasyTier 网络密钥跨节点轮换、TUN 和跨主机数据面仍未验收。
- **下一步：** 在隔离 Windows 网络环境运行 GUI 端到端验收，并在 Linux Docker/systemd 目标环境执行发行阻断测试。

### WL-0034 — 许可证资产门禁、发行复包与当前用户安装验收

- **状态：** PASS
- **目标与前置条件：** 上游 EasyTier 二进制已锁定，Windows/Linux 包脚本已接入许可证文本校验；必须以实际发行物证明许可证、SBOM、校验和和 Windows 安装路径完整。
- **改动：** 新增 `docs/easytier-license-lock.toml` 和许可证暂存脚本；Windows/Linux 打包脚本在复制前校验 EasyTier LGPL 与 GNU GPL v3 文本，并将其放入每个发行载荷的 `licenses/`。第三方声明补充锁定和替换二进制义务。
- **关键命令：** `scripts/package-windows.ps1`；`scripts/package-linux-deployment.ps1`；归档内容、SBOM、SHA-256 复算；`cargo fmt/test/clippy`；Server/daemon 冒烟；Desktop Release 构建；从 Windows 安装 ZIP 执行隔离安装/卸载验收。
- **验证证据：** `docs/verification/2026-09-18-license-release-gate.md`、`docs/verification/2026-09-18-regression-after-license-gate.md`。Windows 便携与安装归档、Linux 部署归档均已生成且哈希复算一致；Windows 当前用户安装/卸载通过。
- **风险/阻塞：** Linux Docker/systemd 尚未在 Linux 运行环境实际启动；真实 TUN、跨主机、Relay/NAT、防火墙和跨 Windows 用户 Pipe ACL 仍未验收。该任务不将它们标记为完成。
- **下一步：** 在 Linux Docker/systemd 目标环境和隔离的 Windows 网络环境完成剩余发行阻断验收；在远程 CI 有实际记录后准备 GitHub Release。

### WL-0027 — 会话写权限重新验证

- **状态：** PASS
- **目标与前置条件：** 上一会话因当前受限身份无法写入而停止；用户已修复工作区 ACL，需由当前会话实测确认。
- **改动：** 无项目代码改动；以随机探测文件执行写入、读取和立即删除。
- **关键命令：** 受控提升权限下的 `Set-Content`、`Get-Content`、`Remove-Item`。
- **验证证据：** 输出 `WRITE_OK` 与 `PROBE_CLEAN`；探测文件不存在。
- **风险/阻塞：** 普通 sandbox 的受限服务身份仍拒绝写入；后续工程修改通过已验证的受控提升权限执行，且保持工作区范围内。
- **下一步：** 继续 daemon DPAPI/IPC 集成的构建与测试。

### WL-0026 — daemon DPAPI 入网材料与受管隧道控制

- **状态：** PASS
- **目标与前置条件：** Server 已交付结构化入网材料，GUI 已以 DPAPI 保存，但 `whalelinkd` 还未消费材料或实际管理 EasyTier。
- **改动：** 开始在 Rust Core 实现 Windows DPAPI 当前用户存储，并为守护进程增加导入、启动、停止与状态 IPC；运行时密钥仅从内存构造 EasyTier 参数。
- **关键命令：** `cargo test --workspace`；daemon Pipe 冒烟；真实 EasyTier 无 TUN 生命周期冒烟。
- **验证证据：** Rust 16 项测试、Clippy 和 daemon Pipe 实际生命周期均通过；详见 `docs/verification/2026-09-18-daemon-enrollment-lifecycle.md`。GUI 已在守护进程运行时导入兑换结果，离线时保留 DPAPI 保存结果供稍后导入。
- **风险/阻塞：** 实际 TUN 会修改系统网络，必须在真实 Relay 输入和明确验收环境下才执行；本任务只自动运行无 TUN 生命周期验证。
- **下一步：** 强化 Pipe 为当前用户 SID ACL，补齐 Linux Relay/Server 发行门禁，并在独立验收环境完成真实 TUN/跨主机验证。

### WL-0028 — GUI/CLI 本地连接控制闭环

- **状态：** PASS
- **目标与前置条件：** daemon 已支持结构化材料导入和受管 EasyTier 生命周期；GUI 与 CLI 需要暴露同一版本化 Named Pipe 控制能力。
- **改动：** GUI 增加房间 ID、启动连接和停止连接操作；CLI 增加 `status`、`start --room-id` 与 `stop`，不包含任何远程地址或凭据参数。
- **关键命令：** `cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`；扩展 daemon Pipe 冒烟。
- **验证证据：** Rust 16 项测试与 Clippy 通过；扩展的 daemon 冒烟已用真实守护进程完成 Pipe 导入、CLI `status`、CLI `start`、运行状态查询和 CLI `stop`。Desktop Release 构建通过，0 warnings、0 errors。
- **风险/阻塞：** Windows Pipe ACL 仍需提升为当前用户 SID；本任务只验证本机进程，不触碰 TUN 或真实 Relay。
- **下一步：** 将桌面端和 CLI 更新纳入最终发行包，继续审计通用部署边界。

### WL-0029 — 通用发行版部署边界与配置文档

- **状态：** PASS
- **目标与前置条件：** 用户明确要求软件先作为通用发行版完成，管理员只能在部署后提供自己的服务器和网络配置。
- **改动：** 删除 daemon 示例中的伪控制面参数；新增通用部署说明；Docker Compose 改为要求外部受限部署目录，不再引用仓库内配置；Docker build 忽略敏感目录和下载资产；Windows 发行载荷加入部署说明。
- **关键命令：** 敏感值扫描；`scripts/package-windows.ps1`；发行校验和复算。
- **验证证据：** Windows 包已重新生成；96 个载荷哈希复算通过，便携 ZIP 含 `docs/DEPLOYMENT.md`。用户提供的 Relay、网络名、密钥和对端地址均经正确参数的源码扫描确认未命中。详情见 `docs/verification/2026-09-18-generic-release-boundary.md`。
- **风险/阻塞：** 本机无 Docker/Podman，Docker Compose 配置只能静态审阅，不能在本机运行容器验收。
- **下一步：** 完成 Linux 发行包和 SBOM 门禁；在可用 Linux 容器或主机上做 Docker/systemd 实际验收。

### WL-0030 — Windows Named Pipe 当前用户 ACL

- **状态：** PASS
- **目标与前置条件：** Pipe 原先仅拒绝远程客户端，需防止同机其他用户调用 daemon 的连接控制接口。
- **改动：** Pipe 改用受保护的 owner-only SDDL DACL，并保留远程客户端拒绝；使用首实例创建防止同名 Pipe 先占。安全描述符创建后立即释放。
- **关键命令：** `cargo clippy --workspace --all-targets -- -D warnings`；`scripts/daemon-pipe-smoke.ps1`。
- **验证证据：** Clippy 无警告；同一用户的真实 daemon/CLI 无 TUN 生命周期冒烟通过。详见 `docs/verification/2026-09-18-named-pipe-owner-acl.md`。
- **风险/阻塞：** 本机无第二独立用户会话，尚未执行跨用户拒绝的运行时试验；不可将该未执行项写成验收通过。
- **下一步：** 完成 Linux 发行/SBOM 门禁，并在独立 Windows 用户环境补跨用户 ACL 验收。

### WL-0031 — 锁定依赖 SPDX SBOM

- **状态：** PASS
- **目标与前置条件：** 发布检查清单要求附带 SBOM；依赖已由 Cargo.lock 与 Desktop `packages.lock.json` 锁定。
- **改动：** 新增离线 SBOM 生成脚本，从 Cargo metadata、NuGet lock 和 EasyTier lock 信息写出 SPDX 2.3 JSON，并将其置入 Windows 便携与安装发行物。
- **关键命令：** `scripts/generate-sbom.ps1`；`scripts/package-windows.ps1`；SBOM JSON 结构检查。
- **验证证据：** 最终 Windows 发行物中生成的 SPDX 2.3 SBOM 含 155 个组件，EasyTier v2.6.4 存在；98 个发行哈希均复算一致。详见 `docs/verification/2026-09-18-sbom.md`。
- **风险/阻塞：** 未知第三方许可证以 SPDX `NOASSERTION` 表示，正式 GitHub Release 前仍需逐项许可证复核和附带上游 LGPL 文本。
- **下一步：** 完善 Linux 发行产物，并在可用 Linux 环境执行 Docker/systemd 验收。

### WL-0032 — Linux 源码部署包与 CI 发行构建

- **状态：** PASS
- **目标与前置条件：** 本机无 Linux 目标工具链和 Docker/Podman；必须交付可验证 Linux 部署源包，并在 Linux CI 上构建 Docker/systemd 产物。
- **改动：** 新增 Linux 部署包脚本，验证锁定 EasyTier Linux ZIP，打包 Server 源码、Docker/systemd、示例配置、部署说明和 SPDX SBOM；新增 Linux Docker 和 systemd CI 构建任务。
- **关键命令：** `scripts/package-linux-deployment.ps1`；ZIP/SBOM/SHA-256 验证。
- **验证证据：** 首次本地包结构、SBOM 和 276 项哈希复算通过，但发现 `apps/` 递归复制带入本机 `bin/obj` 残留，包不符合干净发行标准；修正后重建为 25,596,579 bytes，验证不含构建输出、SBOM 含 155 个组件、70 个哈希复算通过。详见 `docs/verification/2026-09-18-linux-deployment-bundle.md`。Linux Docker/systemd 的实际构建由 GitHub Actions job 复现，尚无远程运行记录。
- **风险/阻塞：** 本机无法执行 Docker 或 Linux 二进制，故不声明容器/系统服务本机通过。
- **下一步：** 在 Linux CI/目标环境运行 Docker/systemd 验收；完成上游许可证文本与最终发布审计。

### WL-0033 — 脱敏诊断摘要

- **状态：** PASS
- **目标与前置条件：** 发布审计发现 `diagnostics.export` 仍返回占位状态，必须在不暴露入网材料的前提下实现可用诊断。
- **改动：** 改为返回版本化脱敏摘要，仅含数据面运行状态与“不含凭据/地址”声明；不读取或传输配置、路径、Relay、密钥、令牌或邀请码。
- **关键命令：** `cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`；`scripts/daemon-pipe-smoke.ps1`。
- **验证证据：** Rust 17 项测试、Clippy 和真实无 TUN daemon/CLI 生命周期均通过；详见 `docs/verification/2026-09-18-diagnostics-summary.md`。
- **风险/阻塞：** 当前摘要不包含系统日志或适配器信息；未来扩展诊断前必须新增专门的脱敏规则与测试。
- **下一步：** 更新历史任务的过期状态，完成许可证与最终发布审计。

### WL-0025 — 结构化入网材料与无密钥快照

- **状态：** PASS
- **目标与前置条件：** 已确认真实 EasyTier v2.6.4 的 `network_name`、`network_secret`、`peers`、DHCP 和 UPnP 参数；现有 API 仍返回不可用的 `enrollment-pending://` 占位字符串。
- **改动：** 开始将固定房间配置扩展为环境变量引用的入网定义，兑换返回结构化材料；Server 快照只保留可恢复的成员/邀请码状态，启动时再由受限环境变量重新注入密钥。
- **关键命令：** `cargo fmt --all`；`cargo test --workspace`；控制面进程冒烟。
- **验证证据：** Rust 13 项测试、Clippy、控制面重启/兑换冒烟、Named Pipe 冒烟和 Desktop Release 0 警告构建均通过；详见 `docs/verification/2026-09-18-structured-enrollment.md`。首次编译发现配置解析函数括号错误和所有权问题，修复后才通过，未错误标记为成功。
- **风险/阻塞：** 部署者仍需提供真实 Relay URL 与每房间高熵密钥环境变量；本任务不擅自生成或记录生产密钥。daemon 消费材料的后续工作已由 WL-0026 完成。
- **下一步：** 在独立验收环境验证真实 Relay 转发和 TUN。

### WL-0024 — Linux EasyTier 锁定资产与发行环境核验

- **状态：** PASS
- **目标与前置条件：** Windows 资产已验证；需要同版本 Linux x86_64 官方资产，才能完成 Relay 容器/服务包并做 Linux 运行验收。
- **改动：** 从锁文件的官方 Release URL 获取忽略的 Linux ZIP，完成 SHA-256 对账；同时核验本机 Docker、Podman 和 WSL 发行环境。
- **关键命令：** `curl.exe --fail --location --retry 3 <locked URL>`；`Get-FileHash`；容器/WSL 工具探测。
- **验证证据：** 官方 Linux x86_64 v2.6.4 ZIP 的 SHA-256 与锁文件一致，文件清单包含 `easytier-core`、`easytier-cli` 和 Web 组件；该资产已用于 WL-0032 Linux 部署包。
- **风险/阻塞：** 当前主机没有 Docker/Podman，且 WSL 未安装 Linux 发行版；Linux 运行测试仍需要外部 Linux 主机或 CI。
- **下一步：** 由 Linux CI/目标环境验证运行态，当前资产获取与校验任务已完成。

### WL-0023 — Windows 当前用户安装包与卸载验收

- **状态：** PASS
- **目标与前置条件：** 已有可复验便携发行载荷；提供首发可安装、可卸载且不需要管理员权限的 Windows 交付物。
- **改动：** 新增安装与卸载脚本；安装限制在当前用户 `LocalAppData`，默认拒绝覆盖，使用同卷 staging 后移动；卸载严格校验目标路径与应用载荷后才递归删除。发行脚本将生成独立 installer ZIP。
- **关键命令：** `scripts/package-windows.ps1`；从解压 installer ZIP 在临时目录运行安装和卸载。
- **验证证据：** 最终脚本生成 installer ZIP；`SHA256SUMS.txt` 中 94 个载荷复算一致。随机临时安装目录的安装、Desktop/EasyTier 载荷检查和卸载均通过，详见 `docs/verification/2026-09-18-windows-installer-package.md`。
- **风险/阻塞：** 首发安装包是可审计的当前用户脚本安装器，不是 MSI；管理员级 TUN/驱动策略和企业软件分发须另行验证。
- **下一步：** 继续 Linux 发行与真实数据面；若发行渠道强制要求 MSI/MSIX，后续以签名和企业分发需求为前置条件另建安装器任务。

### WL-0022 — Windows 发行包成功门禁

- **状态：** PASS
- **目标与前置条件：** 已获得并三重对账 Windows x86_64 EasyTier v2.6.4 官方 ZIP；验证发行脚本能生成完整且带校验和的可分发目录。
- **改动：** 开始执行既有 `scripts/package-windows.ps1`；首次失败后新增仓库级 `NuGet.Config`、忽略的仓库级包缓存和锁文件启用，发行脚本显式恢复 `win-x64` Runtime Identifier 后离线发布。第二次恢复仍遭 SDK 对用户 AppData 配置的预读，因此脚本将 `APPDATA`、`LOCALAPPDATA`、`DOTNET_CLI_HOME` 和 `NUGET_PACKAGES` 仅在其子进程内指向仓库的忽略缓存。已成功产生初步发行目录；脚本现改为先清理已验证的生成目录、解压已校验 EasyTier 运行时、生成便携 ZIP，并对目录及 ZIP 一并生成校验清单。本任务完成前不宣称 Windows 发行物已就绪。
- **关键命令：** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/package-windows.ps1`。
- **验证证据：** 最终脚本通过：Rust Release、锁定的 `win-x64` restore、Desktop publish、已校验 EasyTier 解压、便携 ZIP 和 `SHA256SUMS.txt` 生成均成功；46 个发行载荷已逐一复算。详情见 `docs/verification/2026-09-18-windows-portable-package.md`。
- **风险/阻塞：** 该步骤仅验证便携发行目录；MSI 安装器、Linux 发行物和真实 Relay/TUN 验收仍未完成。首次恢复需要可访问 `api.nuget.org`，之后发布使用已还原且锁定的依赖。
- **下一步：** 建立 Windows MSI 可复现构建与 Linux 发行门禁，并继续替换占位的数据面入网材料。

### WL-0000 — 工作区 ACL 排障与恢复

- **状态：** PASS
- **目标与前置条件：** 修复既有文件无法由 `apply_patch` 读取/更新的问题。
- **改动：** 用户将 `.git` 所有者从受限服务身份还原为交互工作区所有者，并重置 ACL 正常继承；已成功修改 `Cargo.toml` 验证恢复。
- **关键命令：** 最小真实依赖编辑：添加 `sha2`。
- **验证证据：** `docs/verification/2026-09-17-filesystem-block.md`、`docs/verification/2026-09-17-filesystem-block-retry.md`、ADR 0004。
- **风险/阻塞：** 无；旧阻塞记录保留为审计证据。
- **下一步：** 继续 Server 持久化和邀请码摘要化。

### WL-0007 — 邀请码摘要化

- **状态：** PASS
- **目标与前置条件：** 确保未来 Server 状态持久化不保存可直接兑换的邀请码明文。
- **改动：** 新增 `sha2` 依赖；Server 仅以邀请码 SHA-256 摘要作为内部索引，明文只在创建响应中返回一次。
- **关键命令：** `cargo fmt --all`；`cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`。
- **验证证据：** Rust 10 项测试通过，新增 `store_does_not_index_invites_by_plaintext_code`。
- **风险/阻塞：** Server 状态仍未持久化；摘要化为持久化前置安全措施，不代表已实现重启恢复。
- **下一步：** 为房间、成员、邀请码元数据和凭据 epoch 实现版本化状态文件与恢复测试。

### WL-0008 — Server 状态快照与恢复

- **状态：** PASS
- **目标与前置条件：** 为控制面重启恢复提供版本化、非明文邀请码的状态基础。
- **改动：** `Store` 增加 schema 版本和 JSON 序列化；新增 `save_snapshot`/`load_snapshot`，使用临时文件后重命名写入。
- **关键命令：** `cargo fmt --all`；`cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`。
- **验证证据：** Rust 11 项测试通过，新增 `snapshot_restores_room_state_without_plaintext_invite_code`。
- **风险/阻塞：** 快照尚未自动接入 Server 的每次写操作与启动参数；Windows 上覆盖既有状态文件的原子替换策略需单独验证。
- **下一步：** 接入状态路径配置、启动加载和写操作持久化，并执行重启 API 集成测试。

### WL-0009 — Server 自动状态持久化

- **状态：** PASS
- **目标与前置条件：** 让固定房间、邀请码和凭据 epoch 在 Server 重启后恢复。
- **改动：** 新增 `WHALELINK_STATE_PATH`；Server 启动时加载或创建状态文件，并在创建/兑换/撤销邀请码及轮换凭据后自动保存。
- **关键命令：** `cargo fmt --all`；`cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`。
- **验证证据：** Rust 12 项测试通过，新增 `configured_state_file_persists_new_invites`。
- **风险/阻塞：** Linux 服务目录权限与 Windows 覆盖既有状态文件的原子替换仍需在目标平台验证；TLS、限流和审计未实现。
- **下一步：** 实现 daemon 的受限 Named Pipe 服务端与 Windows 凭据保护。

### WL-0010 — Windows 本地 Named Pipe 传输

- **状态：** PASS
- **目标与前置条件：** 让 GUI 能通过本地 IPC 调用 daemon，而不开放 TCP 管理端口。
- **改动：** `whalelinkd serve` 在 Windows 创建 `\\\\.\\pipe\\WhaleLink.v1`，拒绝远程客户端，处理单个 JSON IPC 请求并返回版本或健康状态；进程控制命令继续拒绝。
- **关键命令：** `cargo fmt --all`；`cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`。
- **验证证据：** Windows Rust workspace 编译通过；12 项测试全绿、Clippy 无警告。
- **风险/阻塞：** 尚未强制当前用户 SID ACL，尚未绑定 EasyTier 运行状态，尚未实现 DPAPI 凭据库。
- **下一步：** 添加用户 SID 访问控制、凭据保护和 GUI 端到端 Pipe 冒烟测试。

### WL-0011 — Windows DPAPI 入网凭据库

- **状态：** PASS
- **目标与前置条件：** 防止客户端将入网材料写入 TOML、日志或明文磁盘文件。
- **改动：** 桌面端新增当前用户范围 DPAPI 加密库；房间文件名使用 SHA-256 摘要，内容经 `ProtectedData` 加密后以临时文件替换写入；项目目标框架改为 `net8.0-windows`。
- **关键命令：** `dotnet build apps/WhaleLink.Desktop/WhaleLink.Desktop.csproj --configuration Release`。
- **验证证据：** Release 构建通过，0 warnings、0 errors。
- **风险/阻塞：** 尚未从 GUI 邀请兑换流程调用该库；尚无跨用户/损坏文件的自动化测试。
- **下一步：** 实现邀请码兑换客户端、写入 DPAPI 库并显示可操作错误。

### WL-0012 — GUI 邀请码兑换与安全保存

- **状态：** PASS
- **目标与前置条件：** 让客户端能从自建控制面兑换邀请码，并不以明文保存入网材料。
- **改动：** 桌面端新增 Server 地址/邀请码输入与兑换操作；成功后调用 `/api/v1/invites/{code}/redeem`，立即清空邀请码输入，只将入网材料保存到 DPAPI 库。
- **关键命令：** `dotnet build apps/WhaleLink.Desktop/WhaleLink.Desktop.csproj --configuration Release --no-restore`。
- **验证证据：** Release 构建通过，0 warnings、0 errors。
- **风险/阻塞：** 尚未以真实 Server 进行 GUI 端到端测试；已保存材料尚未下发给 daemon 以生成 EasyTier 连接配置。
- **下一步：** 添加 daemon 的安全入网材料导入命令并完成兑换到连接的本地闭环。

### WL-0013 — EasyTier 锁定资产下载复测

- **状态：** BLOCKED
- **目标与前置条件：** 下载并校验官方锁定二进制，验证真实 CLI 和数据面配置。
- **改动：** 无源码变更；已按锁定 URL 启动带重试的下载。
- **关键命令：** `curl --retry 3` 下载 Windows x86_64 v2.6.4 ZIP。
- **验证证据：** `docs/verification/2026-09-17-easytier-download-retry.md`。
- **风险/阻塞：** 远端连接重置，未取得可校验二进制；因此真实网络和 daemon→EasyTier 配置闭环不能实现或验证。
- **下一步：** 外部网络恢复后重新下载、核验 SHA-256，并在隔离环境执行双节点矩阵。

### WL-0014 — 固定房间配置化

- **状态：** PASS
- **目标与前置条件：** 使自建 Server 能管理多个管理员预配置房间，而非仅硬编码默认房间。
- **改动：** 新增 `WHALELINK_CONFIG` 和 `whalelink-server.example.toml`；启动时解析 `[[rooms]]` TOML 配置并校验每个房间的 ID 与显示名称。
- **关键命令：** `cargo fmt --all`；`cargo test --workspace`；`cargo clippy --workspace --all-targets -- -D warnings`。
- **验证证据：** Rust 13 项测试通过，新增 `loads_fixed_rooms_from_toml`。
- **风险/阻塞：** 房间的实际 EasyTier 数据面凭据仍受上游二进制验证阻塞；配置变更后的受控迁移尚未实现。
- **下一步：** 在取得已校验 EasyTier 资产后将固定房间映射到已验证的数据面配置。

### WL-0015 — 控制面回环冒烟测试

- **状态：** PASS
- **目标与前置条件：** 验证编译通过以外的实际 Server 启动、鉴权与房间查询行为。
- **改动：** 新增 `scripts/server-smoke.ps1`，使用临时状态、回环地址和测试令牌启动后自动清理。
- **关键命令：** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/server-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-17-server-smoke.md`。
- **风险/阻塞：** 未覆盖 TLS、Linux、容器或真实 EasyTier 数据面。
- **下一步：** 在隔离环境完成数据面验证，并补充 Windows GUI 到控制面的端到端测试。

### WL-0016 — 控制面邀请码重启恢复测试

- **状态：** PASS
- **目标与前置条件：** 验证自动状态持久化不仅能保存快照，也能支持真实 HTTP 流程后的重启恢复。
- **改动：** 扩展 `server-smoke.ps1`：创建邀请码、停止/重启 Server、兑换邀请码并验证成员状态。
- **关键命令：** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/server-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-17-server-smoke.md`。
- **风险/阻塞：** 仅在 Windows 回环环境验证；未覆盖 TLS、Linux 进程管理或 EasyTier 数据面。
- **下一步：** 运行 GUI→Server 兑换→DPAPI 保存的端到端验收，并等待上游二进制网络恢复。

### WL-0017 — Windows 发行打包门禁

- **状态：** PARTIAL
- **目标与前置条件：** 生成可校验 Windows 发行物，且不允许遗漏锁定的数据面依赖。
- **改动：** 新增 `scripts/package-windows.ps1`，校验 EasyTier SHA-256 后构建 Rust、发布桌面端并生成 `SHA256SUMS.txt`；发行清单加入该步骤。
- **关键命令：** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/package-windows.ps1`。
- **验证证据：** 脚本按预期因缺少锁定 EasyTier ZIP 失败，未产生不完整 artifacts。
- **风险/阻塞：** 上游资产持续不可下载，故无法验证成功打包、资产校验或实际发行物。
- **下一步：** 网络恢复后下载并核验资产，重跑脚本以生成 Windows 发行物。

### WL-0018 — daemon 本地 Pipe 端到端验证

- **状态：** PASS
- **目标与前置条件：** 验证 GUI 所依赖的本地 IPC 不是仅能编译的代码路径。
- **改动：** 新增 `scripts/daemon-pipe-smoke.ps1`，通过 .NET NamedPipeClientStream 调用 `whalelinkd serve` 的 IPC v1 `health`。
- **关键命令：** `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/daemon-pipe-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-17-daemon-pipe-smoke.md`。
- **风险/阻塞：** 未覆盖当前用户 SID ACL、GUI 自动化或 EasyTier 进程状态。
- **下一步：** 加强 Pipe 访问控制，并在取得 EasyTier 资产后绑定真实隧道状态。

### WL-0019 — Windows 进程级 CI 门禁

- **状态：** PASS
- **目标与前置条件：** 防止后续修改破坏 Server 或 daemon 的真实进程链路而单元测试未发现。
- **改动：** GitHub Actions 新增 `windows-smoke` job，运行控制面重启/邀请码冒烟与 daemon Named Pipe 冒烟。
- **关键命令：** `cargo test --workspace`；`scripts/server-smoke.ps1`；`scripts/daemon-pipe-smoke.ps1`。
- **验证证据：** 本机三项命令均通过；CI workflow 已配置为 Windows runner 复现。
- **风险/阻塞：** 尚无远程 CI 运行记录；真实 EasyTier 数据面继续受官方资产下载阻塞。
- **下一步：** 获取锁定资产后加入数据面集成 job，并执行成功发行打包。

### WL-0020 — EasyTier 资产对账与双节点回环

- **状态：** PASS
- **目标与前置条件：** 取得锁定官方资产，验证真实 CLI 与不改系统网络的数据面握手。
- **改动：** 使用本地官方 ZIP 完成三重 SHA-256 对账，解压至忽略的 vendor 目录；新增 `easytier-loopback-smoke.ps1`。
- **关键命令：** `easytier-cli --help`；`easytier-core --check-config`；`scripts/easytier-loopback-smoke.ps1`。
- **验证证据：** `docs/verification/2026-09-18-easytier-archive-verification.md`、`docs/verification/2026-09-18-easytier-loopback.md`。
- **风险/阻塞：** 尚未验证 TUN、跨机器 NAT、业务流量、SSH 或 Windows 防火墙行为。
- **下一步：** 为受管 EasyTier 生成已验证配置，并在隔离环境进行有 TUN 的双节点业务连通验收。

### WL-0021 — EasyTier 房间配置模板

- **状态：** PASS
- **目标与前置条件：** 将已验证的 v2.6.4 CLI 配置参数转为可供自建 Relay 部署使用的安全模板。
- **改动：** 新增 `config/easytier-room.example.toml`，包含 network name、secret、relay peer、DHCP、UPnP 禁用和本机 RPC 门户占位配置；工程规格明确真实密钥不得提交。
- **验证证据：** 已由 `easytier-core --help` 确认所用参数在 v2.6.4 中受支持；本地双节点验证见 WL-0020。
- **风险/阻塞：** 未提供实际可访问 Relay URL、房间网络名和密钥，不能生成或验证真实业务入网配置。
- **下一步：** 等待部署者提供 Relay 输入后生成每房间受控配置并执行有 TUN 验收。

### WL-0001 — 初始化工程追溯体系

- **状态：** PASS
- **目标与前置条件：** 空工作区；建立工程规格、决策、验证证据和任务日志。
- **改动：** 已建立 Git `main` 仓库、Rust workspace、Avalonia 项目、部署样例、CI、规格、ADR、验证目录和发行检查清单。
- **关键命令：** `git init --initial-branch=main`；本地构建与测试。
- **验证证据：** `docs/verification/m0-environment-update.md`、`docs/verification/m1-m2-build.md`。
- **风险/阻塞：** 无。
- **下一步：** 维护工程门禁并完成首版缺口。

### WL-0002 — M0 EasyTier 版本锁定

- **状态：** PARTIAL
- **目标与前置条件：** 使用官方 Release，不维护 fork。
- **改动：** 已写入 `docs/easytier-lock.toml`、ADR 0001 和第三方声明；锁定官方 v2.6.4 Windows/Linux x86_64 资产与 SHA-256。
- **关键命令：** GitHub Releases latest API；Windows 资产下载尝试。
- **验证证据：** `docs/verification/m0-easytier-metadata.md`。
- **风险/阻塞：** Windows 资产下载遭远端连接重置；真实双节点验证未执行。
- **下一步：** 在隔离 Windows+Linux 环境下载、校验、解压并完成真实组网矩阵。
