# 上游许可证资产与发行门禁（2026-09-18）

## 范围

复核 EasyTier v2.6.4 的锁定 Windows/Linux 归档、发行时必须携带的许可证文本，
并重新生成 Windows 与 Linux 发行物。本报告不含任何部署者的网络参数、令牌、邀请码或
凭据。

## 输入锁定

- Windows EasyTier ZIP 按 `docs/easytier-lock.toml` 校验通过。
- Linux EasyTier ZIP 按 `docs/easytier-lock.toml` 校验通过。
- EasyTier LGPL 文本 SHA-256：
  `e3a994d82e644b03a792a930f574002658412f62407f5fee083f2555c5f23118`。
- GNU GPL v3 文本 SHA-256：
  `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`。

两份文本由 `scripts/stage-easytier-license.ps1` 按
`docs/easytier-license-lock.toml` 锁定；缓存目录被 `.gitignore` 排除，发行脚本在复制前
重新校验哈希。

## 生成与验证结果

- `scripts/package-windows.ps1` 通过，生成便携 ZIP 和当前用户安装 ZIP。两个归档均含
  EasyTier LGPL、GNU GPL v3、SPDX 2.3 SBOM 与部署说明。
- Windows 便携包 SHA-256：
  `3bed65c6ccf80716ec9db71810dd8ef6d32d7a9714aecdb3e25dd8132e8bb3a5`。
- Windows 安装包 SHA-256：
  `5532ff419117f7a1d1c4b2a2ba6f229beae5e186a617f85b185c704c56b43761`。
- Windows `SHA256SUMS.txt` 有 102 条，抽取的两份归档哈希均与文件实际哈希一致。
- `scripts/package-linux-deployment.ps1` 通过，生成 Linux 源码部署 ZIP。
- Linux 部署包 SHA-256：
  `0797018dee3c763a01bee1966a8bb8732b53394feacde8592f8b37b57a64c5f4`。
- Linux `SHA256SUMS.txt` 有 51 条；归档有 50 个条目，含 Dockerfile、systemd unit、部署说明、
  两份许可证、锁定 EasyTier ZIP 和 SPDX 2.3 SBOM。扫描确认归档内没有
  `bin/`、`obj/`、`target/`、`.nuget/`、`artifacts/` 或 `vendor/` 构建缓存。
- 两种 SBOM 各列 155 个组件，均包含 EasyTier `2.6.4`。Linux 包的可复现性修复见
  `docs/verification/2026-09-18-linux-package-reproducibility.md`。

## 当前用户安装验收

从 Windows 安装 ZIP 解压后，在唯一、临时的当前用户 `LocalAppData` 子目录运行安装器，
验证 `whalelinkd.exe`、CLI、桌面程序、两份许可证和卸载器，再由归档内卸载器移除该目录。
验收目录在完成后不存在。管理员控制台纳入载荷后的重建版本已再次执行该验收。该测试不接触
既有安装、运行态或用户凭据。

## 边界

这证明 Windows 打包和当前用户安装/卸载路径可用，且证明 Linux 源包结构正确；它不证明
Linux Docker/systemd 已实际启动，也不替代真实 TUN、跨主机、Relay/NAT 和跨用户 Pipe ACL
验收。这些项目仍保持为发布阻断项。
