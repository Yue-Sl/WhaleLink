# Windows 便携发行门禁（2026-09-18）

## 范围

在 Windows 上从受管的官方 EasyTier v2.6.4 ZIP 生成 WhaleLink 的便携发行物。没有记录
令牌、邀请码、房间密钥、机器名或网络地址。

## 执行结果

- `scripts/package-windows.ps1`：通过。
- Rust Release：`whalelinkd` 与 `whalelink-cli` 构建通过。
- Desktop：锁定的 `win-x64` Runtime Identifier restore 与 Release publish 通过。
- EasyTier：归档哈希与锁文件相符后解压到便携载荷。
- 产物：`artifacts/windows-x64/WhaleLink-v3-win-x64-portable.zip`。
- 便携 ZIP SHA-256：`4ddcfe17e3596bdf999cd14f0e212f96cde880eab77f39f44d2d235d0b1e4e8b`。
- 完整性复算：`SHA256SUMS.txt` 中 46 个发行载荷均复算一致。

## 可复现性说明

脚本使用仓库内的 `NuGet.Config`、忽略的 `.nuget/` 缓存、`packages.lock.json` 和
`--locked-mode`；不读取或修改构建账户的用户级 NuGet 配置。首次还原需要 NuGet 官方源，
随后同一锁定依赖可使用缓存重建。

## 未覆盖项

该证据只覆盖便携发行物。MSI、Linux Docker/systemd 发行物、代码签名、自动更新以及
真实 TUN 跨主机业务流量不在本次验证范围内。
