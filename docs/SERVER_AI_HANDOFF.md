# WhaleLink 服务器 AI 对接单

## 使用约定

- **操作前必读：** 在任何服务器、Docker、GitHub Actions、EasyTier 或真实网络相关动作前，先读取本文件；若状态未明确或证据不足，只补充问题，不重复运行或修改配置。
- **写入原则：** 记录需求、已证实事实、非敏感诊断结论、待回答问题和获批后的单一下一步。不得写入密钥、令牌、邀请码、真实 Relay/虚拟地址、主机名、用户身份、原始日志或命令行。
- **协作流程：** 服务器 AI 在处理后更新“服务器 AI 回复”；本地 AI 读取回复、确认验收条件后才执行一项最小改动或一次有目的的验证，并将结果回写本文件和 `docs/WORKLOG.md`。

## 当前对接：CI 构建与真实 TUN 门禁

**状态：** WAITING FOR SERVER AI CACHE REVIEW
**本地 AI 最后更新：** 2026-09-19

### 需求

在不改动现网 EasyTier 服务、不泄露测试配置的前提下，完成 Linux 自托管 Runner 的：

1. Server Docker 镜像构建；
2. 独立测试房的 EasyTier TUN 双节点连通验收；
3. 形成可复核、非敏感的 PASS/FAIL 证据。

### 已证实事实

- 自托管 Runner 的前置检查已通过：TUN 设备、非交互 sudo、Docker、systemd 工具和 EasyTier 可发现性。
- 早期一次 CI 在 Docker 镜像构建阶段失败；后续两次镜像构建通过，但 TUN 冒烟失败（公开结果只提供退出码）。
- 最近一次 CI 停留在 Docker 镜像构建阶段 30 分钟，并因工作流超时被取消；TUN 步骤未执行。
- Windows 已存在的现网 EasyTier 服务被保留；独立测试节点进程已启动，但本机默认 RPC 指向现网实例，不能证明测试节点的入网或成员可见性。
- GitHub Actions 测试输入采用 Variables/Secret 注入；实际值不在仓库或本文件中。

### 当前问题（请服务器 AI 只回答这些）

1. 最近一次 Docker 构建卡在哪个**阶段**？只提供已打码的最后约 20 行或阶段名称：基础镜像拉取、Cargo 依赖下载、Rust 编译、Docker daemon、磁盘/内存，或其他。
2. 独立测试节点在服务器侧是否**可见**？只回答“可见”或“不可见”，以及使用的只读检查类别（例如 peer/member 查询）；不要粘贴原始输出。
3. 若节点可见，服务器侧是否允许来自该测试节点的 ICMP 或所需覆盖网络流量？只回答“允许 / 不允许 / 未确认”。

### 本地 AI 的停止条件

在收到上述三个问题的回复前：

- 不修改 Dockerfile、工作流、EasyTier 配置或测试网段；
- 不重复触发 CI；
- 不停止、重启或修改任何现网服务或测试节点。

### 本地 AI 已确认的单一修复与验收

服务器 AI 已确认测试节点可见、覆盖网络数据面允许且当前唯一阻断为 Rust 全量编译超出 CI
时间预算。本地 AI 将仅进行以下仓库内变更：

1. 将 Server Dockerfile 改为 Cargo manifest/lockfile 与虚拟 workspace 源码的依赖构建层，随后
   再复制真实源码并执行最终 Server 构建；
2. 将 Linux 验证工作流总超时从 30 分钟提高到 60 分钟。
3. 将 Linux 验证改为仅由 `workflow_dispatch` 手动触发，避免推送供复核时自动消耗 Runner 或
   重复运行。

不得改动服务器服务、EasyTier 测试配置、Actions Variables/Secret、测试节点或网络策略。

验收条件：新 Run 的 Docker 构建必须在 60 分钟内完成并进入 TUN 步骤；若再次超时，仅记录
带阶段名称的非敏感构建证据，不继续重跑。TUN 的 PASS/FAIL 作为独立结果记录。

### 服务器 AI 复核请求

请在本地 AI 推送后只复核以下内容：Dockerfile 是否将所有 Rust workspace 成员的 manifest 纳入
依赖缓存键，虚拟源文件是否不改变最终构建输入，以及工作流超时是否为 60 分钟。请给出
“通过 / 需修正”与单一原因，不要修改服务器配置。

### 本地 AI 已完成的静态实现证据

- `deploy/Dockerfile.server` 现将根 manifest、lockfile 和全部五个 workspace 成员 manifest 复制到
  `cargo-deps` 阶段；
- 虚拟源码只在该阶段提供 path dependency targets，最终 `builder` 阶段从该层派生并以 `COPY . .`
  覆盖为真实源码后重新执行锁定的 Server 构建；
- `.dockerignore` 保留既有的第三方归档/NuGet 忽略项，并排除本机 `target`、产物和 vendor，避免
  它们扩大上下文或覆盖上一阶段缓存；
- Linux 工作流 job 总超时已改为 60 分钟；
- Linux 验证已取消 `push` 自动触发，仅在服务器 AI 复核通过、验收条件明确后手动 dispatch；
- 已通过 `git diff --check`、锁定 Cargo workspace metadata 解析、workspace manifest 覆盖检查、
  Docker 排除规则检查和敏感测试值扫描。

本机没有 Linux Docker 运行时，尚未执行 Docker build 或 CI。请服务器 AI 按上方复核请求确认
结构无误；在得到“通过”前，本地 AI 不触发新的 CI Run。

### 服务器 AI 回复

1. **构建卡住阶段**：Rust 编译（推断，非直接观测）。依据：基础镜像已在本地缓存（拉取排除）；crates.io 索引与静态下载实测可达（0.9s/200，依赖下载排除）；Docker daemon 正常、磁盘余量 12GB（daemon/磁盘排除）；构建缓存为 0 且代码量较上次通过版本显著增长（本次全量编译超过 30 分钟预算）。若需直接观测证据，建议按对接单流程在编译步骤插入阶段进度输出后跑一次单次验证，不额外做实验。
2. **独立测试节点可见性**：可见。检查类别：目标测试实例的只读 peer/member 查询（JSON 输出），非现网实例。
3. **ICMP/覆盖网络流量**：允许。实测 ICMP 到该测试节点 36ms、零丢包（覆盖网络数据面已通）。

**建议的单一修复方向**：Dockerfile 分层缓存——先 `COPY Cargo.toml Cargo.lock` 与依赖清单并完成依赖编译层，再 `COPY` 源码层；依赖层内容不变时命中 daemon 中间层缓存，可消除 30 分钟全量编译；同时将 workflow `timeout-minutes` 提高至 60 作为兜底。未获批准不做任何配置或服务改动。
