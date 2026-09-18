# Linux 通用部署源包（2026-09-18）

## 范围

生成不绑定任何用户服务器的 Linux x86_64 Server 部署源包。该包包含 Docker 和 systemd 部署
文件、锁定源码、示例配置、已校验 EasyTier Linux 上游归档以及 SPDX SBOM；它不是在 Windows
主机上伪造的 Linux 可执行二进制。

## 结果

- `scripts/package-linux-deployment.ps1`：通过。
- EasyTier Linux v2.6.4 ZIP SHA-256 已与 `docs/easytier-lock.toml` 对账。
- 部署 ZIP 包含 `deploy/build-server.sh`、Docker Compose、systemd unit、配置样例、部署说明和
  SPDX 2.3 SBOM。
- SBOM：155 个组件，包含 EasyTier `2.6.4`。
- 包体积：25,596,579 bytes；未包含 `bin/`、`obj/` 或 `target/` 本机构建输出。
- `SHA256SUMS.txt`：70 个条目全部复算一致。

## 限制

本机无 Docker/Podman、Linux Rust target 或 Linux 运行环境。GitHub Actions 已配置 Linux Docker
build 与原生 systemd 载荷构建，但尚无远程 CI 运行记录；因此容器启动和 systemd 服务实际运行
仍是待 Linux CI/目标环境完成的发布门禁。
