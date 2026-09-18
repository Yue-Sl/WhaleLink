# 发行检查清单

所有发布阻断项必须通过才能发布 GitHub Release。已勾选项只表示当前有本地、可复查证据，
不替代尚未执行的目标环境验收。

- [x] 本地 Rust 格式、Clippy、20 个测试、Server/daemon 进程冒烟（含异常退出重启）与 Windows GUI Release 构建通过；远程 CI 尚未有运行记录。
- [x] Windows/Linux EasyTier v2.6.4 资产按 `docs/easytier-lock.toml` 校验 SHA-256。
- [x] `scripts/package-windows.ps1` 已生成 Windows 便携 ZIP、当前用户安装 ZIP 与 102 条 `SHA256SUMS.txt`。
- [x] `scripts/package-linux-deployment.ps1` 已生成 Linux 源码部署 ZIP；该包排除本机构建缓存和会自引用包哈希的工作日志/验证报告。
- [x] Server 持久化、Windows owner-only Named Pipe ACL、DPAPI 凭据存储和脱敏诊断已实现并通过当前用户测试。
- [x] Windows 安装/卸载已从实际安装 ZIP 在唯一当前用户目录验收，随后清理完成。
- [x] 当前生成的 Windows 与 Linux 载荷均附带 SHA-256、SPDX SBOM、第三方 NOTICE、EasyTier LGPL、GNU GPL v3 与部署文档。
- [x] 源码、示例、当前发行载荷和留痕文件遵循通用发行边界；不存放部署者的令牌、邀请码、网络密钥或可识别网络信息。
- [ ] 真实双节点 TUN 连通、房间隔离、重启、控制面离线、凭据轮换、Relay/NAT 和防火墙测试均留有打码证据。
- [ ] Linux Docker 镜像实际运行、systemd 服务实际运行、部署回滚已在干净 Linux 环境验收。
- [ ] 跨 Windows 用户的 Named Pipe 拒绝访问已在独立用户会话验收。
- [ ] GitHub Actions 的 Linux/Windows 构建和冒烟有实际绿色运行记录。
- [ ] GitHub Release 已附产物并完成最终第三方许可证复核、发布及回滚演练。
