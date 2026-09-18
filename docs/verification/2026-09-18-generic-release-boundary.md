# 通用发行版边界与 Windows 重打包（2026-09-18）

## 范围

确认 WhaleLink 作为通用发行软件交付，用户服务器设置仅属于部署后的受限配置，且本次 Windows
发行载荷包含这一部署说明。

## 结果

- 发行说明：`docs/DEPLOYMENT.md` 已进入便携 ZIP 的 `docs/DEPLOYMENT.md`。
- Windows 发行：`scripts/package-windows.ps1` 通过，`SHA256SUMS.txt` 中 96 个文件均已复算。
- 便携 ZIP SHA-256：`0a15162609c304dae9354f7e43ad12dabe8c4698383d369eb54e83334f5eb0e5`。
- 当前用户安装 ZIP SHA-256：`64c31a474dacb54f3f19ddd194b15f4b6858b2befea89f50650d699fe41c8232`。
- 对用户本次提供的 Relay、网络名、网络密钥与对端地址进行项目源码扫描：均未命中。

## 设计保证

发行包不包含任何用户服务器、网络或凭据配置。Server 管理员在仓库外的受限目录配置固定房间，
客户端仅在兑换一次性邀请码后获得入网材料，并以当前用户 DPAPI 保存。Docker Compose 要求
`WHALELINK_DEPLOY_DIR` 指向仓库外的受限配置目录；`.dockerignore` 排除了该类目录。

## 未覆盖项

本机没有 Docker/Podman，因此本次只能静态检查 Compose 和 Dockerfile，不能声明 Linux 容器
已运行。真实 TUN 和跨主机通信仍须在独立验收环境中执行。
