# WhaleLink v3：计划与当前进度对照

**状态快照：** `ff7b49685daa4ceeb1619d78945ceafe28c310e9`（后续仅留痕文档变更不改变软件交付物）
**结论先行：** Windows 首版的“管理员按自己服务器参数接入已有 EasyTier 网络”已经可用，且有本机发行包与跨机 TUN 数据面验收证据。控制面已部署并经 HTTPS 健康检查通过；GUI 邀请码真实端到端验收受当前 Codex Computer Use 认证模式阻塞。干净 Linux/CI 验收和 GitHub Release 仍未完成。

## 状态图例

| 标记 | 含义 |
| --- | --- |
| ✅ | 已实现并有相应验证证据 |
| 🟡 | 已实现或已有交付物，但尚有部署/目标环境验收缺口 |
| ⏳ | 未完成；列出的前置条件满足后才应开始 |
| 🚫 | 已明确不纳入首版 |

## 总览：原计划与实际进度

| 里程碑 | 原计划交付 | 当前状态 | 对当前用户的影响 | 仍需完成 |
| --- | --- | --- | --- | --- |
| **M0 工程与依赖** | 工程骨架、锁定官方 EasyTier、不维护 fork、真实双节点验证 | ✅ | 发行包使用锁定的外部 EasyTier 运行时；没有把上游二进制提交到源码仓库 | 后续升级 EasyTier 时重新核验版本、SHA-256 和许可证 |
| **M1 本地 Core** | `whalelinkd`、版本化 Named Pipe、配置/诊断、DPAPI 与 ACL | ✅ | 直连脚本可受管地启动 EasyTier；本地机密不进入日志或普通配置 | 独立 Windows 用户会话下的 Pipe 拒绝访问验收 |
| **M2 Linux 控制面** | 固定房间、管理员邀请、兑换、成员状态、凭据轮换 | 🟡 | 控制面已部署在服务器回环地址，并经受信 HTTPS 入口健康检查 | 执行 GUI 邀请码兑换→DPAPI 保存→daemon 连接的真实验收 |
| **M3 Windows GUI** | 管理端房间/邀请/诊断/轮换，客户端邀请导入与连接状态 | 🟡 | `WhaleLink.Desktop.exe` 可作为界面交付；邀请码主流程取决于 M2 已部署 | M2 就绪后完成真实 GUI 邀请码端到端验收；当前联机优先走脚本 |
| **M4 发行版** | Windows 安装/便携包、Linux Docker/systemd、SBOM/许可证/SHA、GitHub Release | 🟡 | Windows 首版便携包和安装包、哈希、SBOM、NOTICE 和说明书已生成 | 在干净 Linux 验收 Docker/systemd/回滚；获得 CI 绿色记录；创建 GitHub Release 并复核第三方义务 |

## 已可用的首版能力

这不是演示版：以下路径已经有验证证据，可用于接入**管理员已经部署好的** EasyTier 网络。

```text
管理员提供本次连接参数
        ↓
Windows 管理员运行 Connect-WhaleLink.ps1
        ↓
脚本仅在当前进程内交给 whalelinkd / EasyTier
        ↓
创建虚拟网卡并加入管理员的网络
        ↓
访问同一虚拟网段内已在线的对端或业务服务
```

- ✅ 参数均在运行时输入：网络名、Relay、可选虚拟 IPv4；密钥交互读取。发行包、源码和日志均不预置任何用户服务器地址、网络名、密钥、邀请码或令牌。
- ✅ `--no-listener` 会转发给 EasyTier；已修复未指定时强制固定本机 RPC 端口导致的冲突。多个正常直连实例不应争用默认数据监听端口。
- ✅ Windows 发行包包含便携 ZIP、当前用户安装 ZIP、SHA-256、SBOM、第三方声明及中文说明。
- ✅ 已有非敏感跨机验收：TUN 就绪、加入中继、三节点互见、零丢包，以及服务器到客户端 ICMP 通过。

证据入口：[`2026-09-19-v1-connection-release.md`](verification/2026-09-19-v1-connection-release.md)、[`2026-09-19-local-release-smoke.md`](verification/2026-09-19-local-release-smoke.md)、[`2026-09-19-cross-machine-acceptance.md`](verification/2026-09-19-cross-machine-acceptance.md)、[`release-checklist.md`](verification/release-checklist.md)。

## 当前精简版怎么使用

### 适用场景

你已经有自己的 EasyTier Relay/网络，管理员能够安全地给每台设备分发网络名、网络密钥，以及（如果需要）固定虚拟 IPv4。此流程**不要求**先部署 WhaleLink Linux 控制面。

### 连接步骤

1. 本机当前交付目录为 `C:\Users\a1311\Desktop\WhaleLink-v3-First-Release`。在其中校验 `WhaleLink-v3-win-x64-portable.zip` 的 SHA-256，解压它；交付给其他电脑时只需复制该目录中的发行物。
2. 右键以管理员方式打开 PowerShell，进入解压根目录。管理员权限是创建 Windows TUN 虚拟网卡的系统要求。
3. 使用管理员提供的实际参数运行；示例中的值只是占位符：

   ```powershell
   .\Connect-WhaleLink.ps1 `
     -NetworkName "你的网络名" `
     -Relay "tcp://relay.example.net:11010" `
     -IPv4 "10.x.x.x/24"
   ```

   `-IPv4` 仅在管理员为该设备分配固定地址时提供；否则省略。脚本随后会隐藏回显地询问网络密钥。
4. 保持此 PowerShell 窗口开启即可维持连接；再按管理员给出的虚拟地址或业务地址访问对端。按 `Ctrl+C` 正常断开。

### 重要限制

- 不要把密钥写进 `.ps1`、批处理文件、仓库或聊天记录；脚本只把它传给当前子进程环境。
- 同一虚拟 IPv4 不能被两个在线节点同时使用。
- 目前不要把 GUI 的邀请码按钮当作首要入网方式：它要等 M2 的 HTTPS 控制面部署完成。
- 当前首版不提供账户体系、自助建房、移动端、macOS、自动更新、应用商店发布或 EasyTier fork。

完整逐步说明见 [`USER_GUIDE.md`](USER_GUIDE.md)。

## 剩余工作：按依赖顺序执行

| 优先级 | 要完成的事 | 完成判据 | 依赖 |
| --- | --- | --- | --- |
| P0 | 部署 M2 HTTPS 控制面 | 固定房间可查询；管理员可创建/撤销邀请码；客户端可安全兑换 | ✅ 控制面在回环 `8787` 运行，HTTPS 入口健康接口已返回成功响应 |
| P0 | GUI 邀请码端到端验收 | 邀请码兑换、DPAPI 保存、daemon 启动和真实对端连通全部通过 | 🟡 HTTPS 入口已通过；Computer Use 两次初始化均因当前 `apikey` 认证模式无法枚举 GUI，尚未执行 GUI 输入 |
| P1 | Linux 发行验收 | Docker 镜像、systemd、回滚在干净 Linux 环境实际通过 | 可用 Linux 验收主机 |
| P1 | CI 门禁实跑 | Windows/Linux 构建与非敏感冒烟存在可复核绿色 Run | 可用 Runner 与仓库认证 |
| P2 | 正式 GitHub Release | 附件、SHA-256、SBOM、NOTICE、许可证和回滚说明复核完成 | P1 全绿和发行管理员确认 |

在 P0 完成前，产品定位应准确表述为：**已验收的 Windows 配置驱动直连首版**，而不是“已部署的邀请制控制面服务”。当前 P0 的唯一基础设施阻塞是受信 HTTPS 入口；不得用裸 HTTP 或自签名证书替代它。
