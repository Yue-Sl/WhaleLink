# 文件系统助手阻塞记录

**日期：** 2026-09-17
**状态：** BLOCKED

在准备实现 Server 持久化（邀请码 SHA-256 摘要存储、版本化状态文件和重启恢复）时，
补丁工具连续三次返回 `windows sandbox helper: setup refresh had errors`。该错误发生在读取
既有 `Cargo.toml` 与 Rust 源码之前，未写入任何该功能的半成品修改。

为遵守工程规则，未使用 shell 重定向、脚本写入或其他方式绕过 `apply_patch`。现有构建通过
的源码保持不变。恢复后从以下顺序继续：

1. 添加 `sha2` 依赖，并让邀请码仅以 SHA-256 摘要存储。
2. 为控制面 Store 增加 schema 版本、原子持久化和加载/迁移测试。
3. 将 Server 的固定房间改为配置文件加载，并完成重启恢复 API 测试。
