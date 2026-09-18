# WhaleLink v3 工程规格（冻结基线）

**版本：** 0.1.0-draft
**状态：** M0--M3 已实现并完成本地验证；M4 发行物已生成，剩余目标环境发布门禁以
`docs/WORKLOG.md` 和 `docs/verification/release-checklist.md` 为实际状态来源。
**范围：** GitHub 开源工程版；Windows 桌面端与 Linux 自建 Server。

## 1. 产品边界

WhaleLink 为自建虚拟局域网提供控制面和桌面体验。EasyTier 是独立的
数据面二进制：WhaleLink 不修改、不静态链接其源码，也不把它提交到仓库。

首发不含账户体系、用户自助建房、移动端、macOS、自动更新、商店上架或
EasyTier fork。固定房间仅由服务器管理员配置和管理。

## 2. 架构与信任边界

| 组件 | 责任 | 不负责 |
| --- | --- | --- |
| `whalelinkd` | 本地配置、EasyTier 生命周期、状态、诊断 | 远程房间授权 |
| `whalelink-server` | 房间、邀请码、兑换、成员和轮换控制 | 转发玩家流量 |
| EasyTier | 加密虚拟网络与中继/直连数据面 | WhaleLink 用户和邀请逻辑 |
| Avalonia GUI / CLI | 通过本地 RPC 操作守护进程 | 直接启动或配置 EasyTier |

Windows GUI 和 CLI 只通过用户范围 Named Pipe 与 `whalelinkd` 通信；协议
使用 JSON-RPC 风格的版本化消息，绝不开放 TCP 本地管理端口。服务器 API
默认绑定 `127.0.0.1`，需要远程访问时由部署者在反向代理/TLS/白名单之后显式开启。

## 3. 身份、房间与数据

- 管理员令牌只用于管理 API，使用 `X-WhaleLink-Token` 传输；不得记录、导出或显示。
- 邀请码为高熵、短期、单次、可撤销的服务器端凭据；兑换后立即失效。
- 客户端持久化的入网材料在 Windows 上必须由 DPAPI 保护并限制为当前用户 ACL。
- TOML 只存非敏感部署配置；当前 daemon 支持 v0→v1 的无损内存迁移，未知 schema 必须拒绝。每房间密钥只通过受限服务环境变量注入。运行状态使用版本化 JSON、原子替换和进程锁，且不得序列化入网密钥。
- daemon 运维日志使用有界 JSON 事件，只允许组件、事件名和 schema 这类非敏感字段；不得写入路径、Relay、令牌、邀请码或入网材料。
- 控制面离线时，既有数据面会话尽力保持；不允许兑换、创建/撤销邀请、切换房间或轮换凭据。
- `credentials/rotate` 当前轮换控制面登记的 credential epoch。实际 EasyTier 网络密钥变更需要
  部署者在受限 Server 配置中协调全部节点，且必须经独立数据面验收后才能作为产品承诺。

## 4. 稳定接口

### 本地 daemon IPC v1

`version`, `health`, `tunnel.start`, `tunnel.stop`, `status.get`, `diagnostics.export`
为 v1 方法集。每个请求与响应都有 `protocol_version`、`request_id` 和稳定错误码。

### Server HTTP API v1

- `GET /api/v1/health`
- `GET /api/v1/rooms`
- `POST /api/v1/rooms/{room_id}/invites`
- `POST /api/v1/invites/{invite_code}/redeem`
- `DELETE /api/v1/invites/{invite_id}`
- `GET /api/v1/rooms/{room_id}/members`
- `POST /api/v1/rooms/{room_id}/credentials/rotate`

除 `health` 和兑换端点外，所有 Server 路由均要求管理员令牌。响应包络统一为
`{ "ok": boolean, "data"?: T, "error"?: { "code", "message", "request_id" } }`。

## 5. 质量门禁

提交前至少运行格式化检查、Rust 单元测试、Server API 测试和 .NET 构建。发行还需
真实双节点 EasyTier 验证、干净环境安装验收、SBOM/NOTICE、SHA-256 和回滚文档。
未执行验证必须在工作日志标为 **BLOCKED** 或 **PENDING**，不可写成通过。

## 6. EasyTier 房间配置输入

每个固定房间需要由管理员提供唯一的 EasyTier `network_name`、高熵 `network_secret` 和至少一个
可访问 Relay/seed URL。`config/whalelink-server.example.toml` 中只记录 `network_secret_env`
名称；真实 secret 只允许在受限 Server 环境与客户端 DPAPI 存储中出现，不得进入 TOML、状态快照、
日志或 Git。
