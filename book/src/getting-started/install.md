# Install

zm-api installs as a systemd service in **passive mode** — it serves the REST
API and leaves ZoneMinder's daemons alone, so it is safe to put on a live box.
That is the starting point; [takeover](../guide/takeover.md) is where it is
meant to end up, whenever you choose to switch.

## Packages

```bash
sudo dpkg -i zm-api_*.deb        # Debian / Ubuntu / Raspberry Pi OS
sudo dnf install zm-api-*.rpm    # Fedora / RHEL / Rocky / Alma
sudo zypper install zm-api-*.rpm # openSUSE
```

Arch users build from [`packaging/arch/PKGBUILD`](https://github.com/SteveGilvarry/zm-api/blob/master/packaging/arch/PKGBUILD)
with `makepkg`.

Installing does three things beyond copying files: it creates the `zoneminder`
service account if it does not exist, generates this install's JWT signing keys
into `/var/lib/zm-api/keys`, and registers the systemd unit. It does **not**
start touching your cameras.

> **If the database already has ZoneMinder in it, do not start the service yet.**
> Run the migration first — see [Upgrading an existing ZoneMinder](upgrading.md).
> zm-api only *warns* when startup migrations fail, so a database left in the
> wrong state gives you a service that looks healthy with features silently
> missing.

## From source

For platforms without a package, or to test a local build.

You need current stable Rust, a MariaDB/MySQL server with a ZoneMinder schema,
and the FFmpeg development libraries:

```bash
sudo apt install pkg-config libssl-dev \
  libavutil-dev libavcodec-dev libavformat-dev libavfilter-dev \
  libavdevice-dev libswscale-dev libswresample-dev
```

Then:

```bash
git clone https://github.com/SteveGilvarry/zm-api.git
cd zm-api
cargo build --release --bins
sudo ./packaging/install.sh
```

`install.sh` lays files out exactly where the packages put them, so a later
package install upgrades cleanly instead of colliding.

## What gets installed

| Path | What |
| --- | --- |
| `/usr/bin/zm-api` | The server |
| `/usr/bin/zm-api-db` | Database migration tool |
| `/usr/bin/zm-api-takeover` | Hands daemon supervision to zm-api, or back |
| `/etc/zm-api/base.toml` | Packaged defaults — replaced on upgrade, don't edit |
| `/etc/zm-api/prod.toml` | Your configuration |
| `/etc/zm-api/zm-api.env` | Environment overrides — wins over both TOML files |
| `/var/lib/zm-api/keys/` | JWT signing keys, generated per install |
| `/var/log/zm-api/` | Logs |

Man pages: `zm-api(8)`, `zm-api.env(5)`, `zm-api-takeover(8)`, `zm-api-db(8)`.

## Next

[First run](first-run.md) covers starting the service and checking it works.
