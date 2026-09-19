# 2026-09-19 首版连接功能与 Windows 发行验证

## 范围

首版目标是让安装后的 Windows 用户按自身服务器参数启动 EasyTier 联机，且不把 Relay、网络名、
虚拟地址或密钥硬编码到产品中。

## 实现

- `whalelinkd connect` 接收运行时 EasyTier 可执行文件路径、网络名、Relay、可选 IPv4 与本机 RPC
  地址；密钥仅从环境变量读取。
- `Connect-WhaleLink.ps1` 被放入 Windows 便携包：它要求管理员权限，交互式读取密钥，并只把密钥
  传入当前子进程环境。它不写入配置、日志或仓库。
- 守护进程从 EasyTier 可执行文件目录启动子进程，确保发行包中相邻的原生运行时文件可被找到。
- 控制面只拒绝空密钥，避免以 WhaleLink 自定义长度规则阻断 EasyTier 可接受的部署配置。

## 验证结果

- `cargo test --workspace --locked -- --skip dpapi_round_trip_preserves_enrollment_without_plaintext_storage`：通过。
  跳过项是当前受限会话中 Windows DPAPI 用户配置不可用的环境专属测试，不影响已生成的用户态发行物。
- `scripts/package-windows.ps1`：通过。完成上游 EasyTier 归档校验、Rust Release 构建、Avalonia 发布、
  SBOM 生成、便携包和当前用户安装包生成。
- PowerShell 连接脚本语法校验与发行物内容检查：通过。
- 本机非提升会话对真实 TUN 的启动被 Windows 拒绝，符合虚拟网卡创建需要管理员权限的系统要求；
  连接脚本以 `#Requires -RunAsAdministrator` 显式处理该前置条件。未以该受限会话声称真实 TUN 已通过。

## 产物

- `artifacts/windows-x64/WhaleLink-v3-win-x64-portable.zip`
- `artifacts/windows-x64/WhaleLink-v3-win-x64-installer.zip`
- `artifacts/windows-x64/SHA256SUMS.txt`
