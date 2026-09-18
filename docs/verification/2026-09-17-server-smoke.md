# 控制面回环冒烟测试

**日期：** 2026-09-17
**状态：** PASS

执行 `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/server-smoke.ps1`。
脚本以随机临时状态文件和回环地址启动 `whalelink-server`，验证：

- `GET /api/v1/health` 返回成功；
- 管理员令牌可访问 `GET /api/v1/rooms`；
- 管理员可创建短期邀请码；
- 管理员可撤销邀请码，且被撤销的邀请码得到 410 响应、不能兑换；
- Server 重启后，邀请码仍可兑换，且成员状态可由管理员查询；
- 管理员可将房间凭据 epoch 轮换并从状态快照恢复该 epoch；
- 后台进程被停止，临时状态文件被删除。

未覆盖 TLS、反向代理、真实网络数据面或 EasyTier 网络密钥的跨节点协调轮换。
