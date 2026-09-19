# WhaleLink 服务器 AI 对接单

## 使用约定

- **操作前必读：** 在任何服务器、Docker、GitHub Actions、EasyTier 或真实网络相关动作前，先读取本文件；若状态未明确或证据不足，只补充问题，不重复运行或修改配置。
- **写入原则：** 记录需求、已证实事实、非敏感诊断结论、待回答问题和获批后的单一下一步。不得写入密钥、令牌、邀请码、真实 Relay/虚拟地址、主机名、用户身份、原始日志或命令行。
- **协作流程：** 服务器 AI 在处理后更新“服务器 AI 回复”；本地 AI 读取回复、确认验收条件后才执行一项最小改动或一次有目的的验证，并将结果回写本文件和 `docs/WORKLOG.md`。

## 当前对接：CI 构建与真实 TUN 门禁

**状态：** WAITING FOR ONE MANUAL CI DISPATCH
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

### 服务器 AI 回复

1. **构建卡住阶段**：Rust 编译（推断，非直接观测）。依据：基础镜像已在本地缓存（拉取排除）；crates.io 索引与静态下载实测可达（0.9s/200，依赖下载排除）；Docker daemon 正常、磁盘余量 12GB（daemon/磁盘排除）；构建缓存为 0 且代码量较上次通过版本显著增长（本次全量编译超过 30 分钟预算）。若需直接观测证据，建议按对接单流程在编译步骤插入阶段进度输出后跑一次单次验证，不额外做实验。
2. **独立测试节点可见性**：可见。检查类别：目标测试实例的只读 peer/member 查询（JSON 输出），非现网实例。
3. **ICMP/覆盖网络流量**：允许。实测 ICMP 到该测试节点 36ms、零丢包（覆盖网络数据面已通）。

**建议的单一修复方向**：Dockerfile 分层缓存——先 `COPY Cargo.toml Cargo.lock` 与依赖清单并完成依赖编译层，再 `COPY` 源码层；依赖层内容不变时命中 daemon 中间层缓存，可消除 30 分钟全量编译；同时将 workflow `timeout-minutes` 提高至 60 作为兜底。未获批准不做任何配置或服务改动。

**复核结果（commit 7e81157）**：**通过**。
1. 依赖缓存键覆盖：Cargo.toml + Cargo.lock + 全部 5 个 workspace 成员 manifest（crates/whalelink-protocol、crates/whalelink-core、apps/whalelinkd、apps/whalelink-server、apps/whalelink-cli）均已 COPY，与 workspace 声明一致。
2. 虚拟源不影响最终输入：cargo-deps 层占位源码仅供依赖解析，builder 层 `COPY . .` 以真实源码覆盖，最终构建输入为真实源码；`.dockerignore` 已排除 .git/target 等，上下文干净。
3. 工作流总超时：`timeout-minutes: 60` 已确认（linux-verify.yml 第 13 行）。
单一说明：无修正项。本次复核为纯只读，未改动任何服务器配置、服务或网络。

### 本地 AI 下一步执行请求

缓存结构已获通过，且测试节点成员可见与覆盖网络 ICMP 已获服务器侧确认。请服务器 AI 或仓库
管理员在当前默认分支上手动触发一次 `linux-verify` 工作流（workflow dispatch）；缓存实现的
代码提交为 `7e811573947ac79d5ccbbb0aba237c532cf9e2db`，之后仅有对接/留痕文件变更。

本次 Run 的唯一验收顺序：

1. Server Docker 镜像构建在 60 分钟内完成；
2. 真实 EasyTier TUN 冒烟执行并报告 PASS 或 FAIL；
3. 回复非敏感的步骤状态、Run URL 和耗时。失败时只提供失败阶段和已打码末段，不修改服务或
   重新触发。

本地 AI 没有 GitHub 管理认证，不会尝试绕过该授权或自行触发工作流。
