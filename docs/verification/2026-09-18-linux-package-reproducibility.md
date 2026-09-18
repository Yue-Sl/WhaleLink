# Linux 部署包自引用修复（2026-09-18）

## 问题

Linux 源码部署包原先递归复制 `docs/`，其中包含工作日志和验证报告。这些文件会记录发行包
SHA-256；每次更新哈希证据都会改变包内容，从而使刚写入的哈希立刻失效。

## 修复

`scripts/package-linux-deployment.ps1` 继续复制部署所需的静态文档（规格、部署说明、EasyTier
锁定和 ADR），但排除 `docs/verification/`、`docs/WORKLOG.md` 和历史工作日志。验证证据始终
保留在源码仓库，不作为部署运行时输入。

## 结果

- 重新生成的 Linux ZIP 有 49 项，`SHA256SUMS.txt` 有 50 条，且所有条目逐条复算一致。
- Linux ZIP SHA-256：
  `6514a7e2ef6063a27ca7a44f4454640ab9b2e8e185b2cc78bc4249d454a9b8b6`。
- 归档包含 `PROJECT_SPEC.md`、`DEPLOYMENT.md`、EasyTier 锁定、ADR、许可证与 SBOM。
- 归档不含 `docs/verification/`、任何工作日志、`bin/`、`obj/`、`target/`、`.nuget/`、
  `artifacts/` 或 `vendor/`。

该布局使后续在仓库内追加验证记录不会改变 Linux 发行候选的载荷或校验和。
