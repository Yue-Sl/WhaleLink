# WhaleLink 部署与首次配置

WhaleLink 是通用发行版。安装包和源仓库不预置 Relay 地址、房间名称、网络密钥、管理员令牌、
控制面地址或任何用户服务器 IP。请只在部署后的受限配置中填写这些值。

## 1. 验证上游数据面

从 `docs/easytier-lock.toml` 中的官方 URL 下载目标平台 ZIP，并用其中的 SHA-256 校验。不要把
下载 ZIP 或已解压二进制提交到 Git。Windows 便携包已包含已校验的 EasyTier 运行时；Linux 部署
应由管理员单独保存并校验官方 Linux 资产。

## 2. 部署控制面

在 Linux 主机的受限目录（例如 `/opt/whalelink`）创建以下文件，权限只授予部署服务账户：

1. 从 `config/whalelink-server.example.toml` 复制 `server.toml`。
2. 从 `config/server.example.env` 复制 `server.env`。
3. 为每个固定房间填写唯一 `network_name`、Relay/seed `peers` 与环境变量名称。
4. 在 `server.env` 中设置高熵管理员令牌以及每个 `network_secret_env` 对应的高熵值。密钥值不应
   出现在 TOML、Shell 历史、日志、诊断包或 Git。
5. 配置 TLS 反向代理后才将控制面公开；Docker Compose 默认只发布 `127.0.0.1:8787`。

Docker 部署时，将上述两个文件放在仓库外的受限目录，然后设置
`WHALELINK_DEPLOY_DIR=/opt/whalelink` 并从 `deploy/` 运行 `docker compose up -d --build`。
该 Compose 配置挂载 TOML 为只读，并把状态保存在命名卷。

systemd 部署时，安装 `deploy/whalelink-server.service`，将二进制放入 `/usr/local/bin`，配置文件
放入 `/etc/whalelink/`，状态目录设为 `/var/lib/whalelink` 并由 `whalelink` 服务账户拥有。
`server.env` 与 `server.toml` 必须由服务账户以外的用户不可读。

## 3. 客户端安装和连接

Windows 发行目录提供便携 ZIP 与当前用户安装 ZIP。安装或解压后：

1. 首版便携包可直接以管理员身份运行 `Connect-WhaleLink.ps1`，按提示输入管理员提供的网络名、
   Relay、密钥和可选虚拟 IPv4。脚本只在当前进程环境中传递密钥，不写入文件或日志。也可启动
   `whalelinkd`，传入一个只含 EasyTier 可执行文件绝对路径的本地 TOML；示例见
   `config/whalelink.example.toml`，不要填写网络或服务端信息。
2. 启动桌面端，输入由管理员公布的控制面 HTTPS 地址和一次性邀请码。
3. 兑换后，客户端把材料以当前用户 DPAPI 保存，并可在守护进程运行时立即导入。
4. 桌面端可使用兑换所得房间 ID 启动/停止连接；CLI 也提供 `whalelink status`、
   `whalelink start --room-id <id>`、`whalelink stop`。

客户端也可在“导入二维码/邀请码”中粘贴二维码解码内容。受控发放渠道可使用
`whalelink://invite?server=https%3A%2F%2Fcontrol.example&code=一次性邀请码` 格式；客户端仅在
内存中解析它并将地址、邀请码填入兑换栏，随后仍由用户触发兑换。原始邀请码也可直接导入。
不要把二维码内容、邀请码或控制面真实地址写入仓库、诊断包或工作日志。

管理员可在桌面端“管理员”工作区临时输入控制面地址和管理员令牌，查询固定房间、创建/撤销
邀请码、查看成员以及轮换控制面 credential epoch。令牌不会保存到本地；创建的邀请码只能通过
受控渠道交付。该 epoch 操作不自动改变 EasyTier 网络密钥，真正的数据面密钥轮换必须由部署者
协调所有节点并在隔离环境验收。

首次启用真实 TUN、网络适配器或防火墙规则前，应在组织的 Windows 权限策略和隔离验收环境中
确认所需管理员权限。当前发行验证只自动运行无 TUN 的 EasyTier 生命周期测试。

## 4. 回滚与诊断

停止 `whalelinkd` 后可回滚到上一个已校验发行目录；Server 回滚前备份 `/var/lib/whalelink` 状态，
并保持其配置与密钥环境变量一致。诊断导出必须继续脱敏，禁止包含令牌、邀请码、网络密钥或可识别
网络地址。
