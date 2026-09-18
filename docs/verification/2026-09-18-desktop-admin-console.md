# Windows 桌面管理员控制台（2026-09-18）

## 实现范围

Avalonia 桌面端新增“管理员”工作区。管理员在该界面临时输入控制面地址和管理员令牌后可：

- 查询固定房间及成员数、凭据 epoch；
- 创建限定为 15 分钟、1 小时或 24 小时的一次性邀请码；
- 显示邀请码一次并提供其 ID 以便撤销；
- 撤销邀请码、查询成员状态、轮换房间 credential epoch；
- 客户端工作区仍保留邀请码兑换、DPAPI 保存、启动/停止连接和脱敏诊断摘要。

管理员令牌只作为单次 HTTP 请求头保留在进程内；不传入 DPAPI 凭据库、TOML、日志或诊断。
邀请码仅在创建时显示，兑换后原输入会立即清空。

## 验证

- `dotnet build apps/WhaleLink.Desktop/WhaleLink.Desktop.csproj --configuration Release --no-restore`：通过，
  0 warnings、0 errors。
- `scripts/server-smoke.ps1`：通过，覆盖鉴权的房间查询、邀请码创建、撤销后的 410 拒绝、重启后
  兑换、成员查询和 credential epoch 轮换持久化。
- 控制面客户端的每个管理员操作均映射到既有 `/api/v1` 版本化端点，并统一读取
  `{ ok, data, error }` 包络。

## 边界

这是一套桌面管理控制，不能替代部署端的 TLS、访问控制或审计。当前 `credentials/rotate`
端点轮换的是控制面登记 epoch；实际 EasyTier `network_secret` 的跨节点协调轮换必须由部署者在
受限 Server 配置和隔离网络验收中完成，尚未作为已通过的自动化能力声明。
