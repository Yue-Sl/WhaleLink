# Windows 凭据目录 DPAPI 与 ACL（2026-09-18）

## 实现

- Avalonia 客户端保存入网材料前，创建目录并使用当前用户 SID 设置受保护 DACL；目录不继承父目录
  权限，唯一允许项为当前用户完全控制。
- Rust daemon 保存导入材料前，以 Owner Rights SDDL 设置受保护 DACL。目录所有者保持为创建目录的
  当前用户，因此 Owner Rights 是当前用户的唯一允许主体。
- 两条路径都继续以 Windows DPAPI CurrentUser 加密文件内容；ACL 是独立的第二层边界。

## 验证

- Desktop Release 构建通过，0 warnings、0 errors。
- `scripts/desktop-credential-acl-contract.ps1` 在临时 LocalAppData 目录保存 DPAPI 材料，验证目录
  不继承权限、所有者和唯一允许主体为当前用户，且仅产生一个加密文件；随后删除目录。
- `scripts/daemon-pipe-smoke.ps1` 在 daemon 导入后验证受保护 DACL、当前用户所有者和唯一 Owner
  Rights/current-user 允许项，再完成真实 EasyTier 无 TUN 启动/停止。
- Desktop ACL 合同脚本已纳入 Windows CI 的 PowerShell 7 运行步骤。

## 边界

该验证不替代独立 Windows 用户会话对 Named Pipe 拒绝访问的测试；后者仍是发布阻断项。
