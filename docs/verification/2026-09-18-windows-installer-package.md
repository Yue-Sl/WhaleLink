# Windows 当前用户安装包验收（2026-09-18）

## 范围

验证首发 Windows 安装包，不使用管理员权限，也不写入服务、注册表、驱动或系统目录。

## 结果

- `scripts/package-windows.ps1`：通过。
- 发行载荷校验：`SHA256SUMS.txt` 的 94 个文件均已复算一致。
- 安装包：`artifacts/windows-x64/WhaleLink-v3-win-x64-installer.zip`。
- 安装包 SHA-256：`63010be40579dacc0a49215e15dba24fb6fc2f5b9b45b4f03071e1ae4a14129f`。
- 从解压的 installer ZIP 运行 `Install-WhaleLink.ps1`：通过。
- 已检查安装后的 Desktop 可执行文件和 EasyTier 运行时：通过。
- 从安装目录运行 `Uninstall-WhaleLink.ps1`：通过；验证目录不存在。

## 安全边界

安装器只允许当前用户 `LocalAppData` 下的目标路径，默认拒绝覆盖已存在安装。卸载脚本
在删除前要求目标路径仍在同一范围且有预期 `app/` 载荷。验收使用随机临时安装目录，测试
结束后已清理。

## 未覆盖项

这不是 MSI/MSIX，也不修改 Windows 服务、网络驱动或防火墙规则。启用 EasyTier TUN 时的
管理员权限提示、驱动安装和企业分发策略仍须在目标环境单独验收。
