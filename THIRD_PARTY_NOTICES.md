# Third-party notices

## EasyTier

WhaleLink uses the externally distributed EasyTier binary as a data-plane
dependency. The pinned version, source URLs and SHA-256 values are in
[`docs/easytier-lock.toml`](docs/easytier-lock.toml). EasyTier is distributed
under LGPL-3.0-only according to its upstream repository. A release must ship
the applicable upstream license text and notices, preserve the user's ability
to replace the binary, and pass a license review before publication. The
locked upstream LGPL text and required GNU GPL v3 text are staged by
`scripts/stage-easytier-license.ps1` and copied into each release under
`licenses/`; their checksums are in `docs/easytier-license-lock.toml`.

No EasyTier binary is included in this repository.
