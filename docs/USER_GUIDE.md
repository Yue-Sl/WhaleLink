# WhaleLink v3 首版使用说明

## 适用范围

本首版用于连接已经部署好的 EasyTier 网络。它不预置任何服务器地址、网络名或密钥；这些参数由你的服务器管理员提供。

## 使用前准备

- Windows 10/11 x64。
- 可使用管理员权限启动 PowerShell。创建虚拟网卡（TUN）是 Windows 的系统级操作，必须提升权限。
- 从管理员取得：网络名、Relay 地址、网络密钥，以及可选的固定虚拟 IPv4 地址。
- 确认本机未运行另一个使用同一虚拟地址的 EasyTier 节点。

## 校验下载包

桌面交付目录提供 `SHA256SUMS.txt`。在 PowerShell 中运行：

```powershell
Get-FileHash .\WhaleLink-v3-win-x64-portable.zip -Algorithm SHA256
```

将输出哈希与 `SHA256SUMS.txt` 中对应条目比对。

## 连接网络

1. 解压 `WhaleLink-v3-win-x64-portable.zip`。
2. 在解压目录中右键 `Connect-WhaleLink.ps1`，选择“使用 PowerShell 运行”，并接受管理员权限提示。
3. 输入服务器管理员提供的参数。例如：

```powershell
.\Connect-WhaleLink.ps1 `
  -NetworkName "你的网络名" `
  -Relay "tcp://relay.example.net:11010" `
  -IPv4 "10.0.0.20/24"
```

`-IPv4` 是可选项；仅在管理员为该设备分配固定地址时填写。随后按提示输入网络密钥。密钥不会写进配置文件、工作日志或发行包。

4. 保持 PowerShell 窗口打开即保持连接。窗口显示连接已启动后，可访问管理员指定的虚拟网段对端或业务服务。
5. 按 `Ctrl+C` 正常断开连接。

## 图形界面

包内的 `desktop\WhaleLink.Desktop.exe` 是桌面管理界面。它的“邀请码兑换”功能需要已经部署并使用 HTTPS 的 WhaleLink 控制面；如果你当前只需要直连 EasyTier 网络，优先使用 `Connect-WhaleLink.ps1`。

## 常见问题

- **无法创建虚拟网卡或出现访问被拒绝：** 确认以管理员身份运行脚本；关闭冲突的 EasyTier 实例后重试。
- **已启动但无法访问对端：** 依次检查网络名、密钥、Relay、固定 IPv4 是否由管理员正确分配，并确认对端在线及防火墙允许业务端口。
- **网络参数变更：** 按 `Ctrl+C` 断开，再使用新参数重新运行脚本。不要把密钥保存到批处理文件或提交到 Git。

## 安全提示

网络密钥等同于该虚拟网络的准入凭据。仅通过受控渠道交付；不要截图、粘贴到聊天群、写入脚本或上传到仓库。
