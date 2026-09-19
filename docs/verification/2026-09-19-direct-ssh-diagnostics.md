# 2026-09-19 直接 SSH 服务器诊断

## 范围

本次按部署者要求暂停服务器 AI 文件对接，使用已配置的 `ssh xiaoxing` 进行只读环境检查，并在服务器临时目录对当前 `main` 做一次受控 Server 镜像预热。未停止、重启或修改现网 EasyTier 服务；未触发 GitHub Actions；未读取或保存受保护变量、密钥、邀请码、Relay、节点地址或原始日志。

## 证据摘要

- SSH 身份检查成功；服务器具备 Docker、`easytier-core` 和 `easytier-cli`。
- `gha-runner.service` 为 active；服务器未发现可用 GitHub CLI 认证，因此没有从服务器触发或查询 Actions Run。
- Runner 留存检出仍是早期工作流版本（自动 push 触发、30 分钟上限）；当前本地 `main` 是手动触发、60 分钟上限版本。旧检出不作为当前代码证据。
- 第一次临时构建因诊断命令错误附加 legacy builder 不支持的 `--progress` 参数，以 exit 125 停止；未进入源码构建，不构成 CI 根因。
- 使用与 CI 完全一致的 `docker build --file deploy/Dockerfile.server` 参数后，当前提交成功进入 Docker `cargo-deps` 层，完成索引下载并进入 Rust crate 编译。未观察到代码错误、权限拒绝、基础镜像拉取失败或网络拒绝。
- 服务器可用 CPU 为 2，系统负载高于核数；冷缓存 Rust 依赖编译明显缓慢。该唯一预热构建在本报告生成时仍在后台运行，最终镜像结果待后续只读确认。

## 结论与下一步

当前证据支持“首次冷缓存依赖构建耗时 + 服务器资源有限”是历史构建超时的主要方向，不能据此宣称 Linux TUN 门禁通过。等待该预热完成后，仓库管理员应在认证环境手动 dispatch 当前 `main` 的 Linux 工作流，再分别记录 Server 镜像和真实 TUN 结果。
