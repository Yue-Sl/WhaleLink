# Windows 客户端二维码/邀请码导入（2026-09-18）

## 实现

客户端新增“导入二维码/邀请码”操作，支持：

- 原始一次性邀请码；
- `whalelink://invite?server=<HTTPS 地址>&code=<邀请码>` 的二维码解码载荷。

解析不发起网络请求、不保存载荷且会立即清空导入栏；它仅将地址和邀请码填入已有兑换栏，
实际兑换仍需要用户点击操作。缺少邀请码、无效地址和 HTTP 控制面地址会被本地拒绝。控制面
客户端同样在所有管理与兑换请求前强制 HTTPS，避免管理员令牌或邀请码经明文传输。

## 验证

- Desktop Release 构建通过，0 warnings、0 errors。
- `scripts/desktop-invite-payload-contract.ps1` 由 PowerShell 7 从已编译程序集调用解析器，验证 HTTPS 载荷和
  原始邀请码成功；缺邀请码和 HTTP 载荷失败。
- 该脚本已纳入 Windows `desktop` CI job。

## 边界

此功能处理已经由相机、扫码器或受控渠道解码为文本的二维码内容；首发不申请相机权限，也不
包含二维码生成或图像识别功能。
