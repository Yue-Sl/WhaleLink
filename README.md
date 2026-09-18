# WhaleLink v3

WhaleLink is a self-hosted virtual-LAN coordination project. It uses a pinned,
externally managed [EasyTier](https://github.com/EasyTier/EasyTier) release for
the data plane and supplies a Rust control plane, local daemon, CLI, and a
Windows Avalonia desktop shell.

## Project status

WhaleLink is a generic, self-hosted release project. Its packages contain no
user Relay endpoint, room name, network key, administrator token, or server IP.
Those are supplied only by the deploying administrator after installation, then
delivered to clients by one-time invite redemption. The exact completed and
pending work is tracked in [the work log](docs/WORKLOG.md).

Read [deployment instructions](docs/DEPLOYMENT.md) before deploying a Server or
connecting a client.

## Layout

- `crates/` — shared protocol and core process/configuration support.
- `apps/` — daemon, server, CLI, and Avalonia desktop application.
- `docs/` — specification, decisions, verification evidence, and work log.
- `deploy/` — Docker and systemd deployment assets.

## Safety

Do not put administrator tokens, invite codes, or EasyTier credentials in the
repository. All examples use placeholders. Downloaded EasyTier assets must be
verified against [`docs/easytier-lock.toml`](docs/easytier-lock.toml).
