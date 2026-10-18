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

Installing does three things beyond copying files: it works out which account
the service runs as, generates this install's JWT signing keys into
`/var/lib/zm-api/keys`, and registers the systemd unit. It does **not** start
touching your cameras.

## The service account

zm-api runs as the account ZoneMinder runs as. ZoneMinder has no account of its
own. Its daemons run as the web server's user, which differs by distribution,
and it records the choice in `zm.conf`:

| Distribution | `ZM_WEB_USER`:`ZM_WEB_GROUP` |
| --- | --- |
| Debian, Ubuntu, Raspberry Pi OS | `www-data:www-data` |
| Fedora, RHEL, Rocky, Alma | `apache:apache` (`nginx:nginx` on the nginx variant) |
| openSUSE | `wwwrun:www` |
| Arch | `http:http` |

Everything ZoneMinder owns is owned by that account and closed to others:
`zm.conf` with the database password is mode 0640, the events tree and the
socket directory are 0755. A separate account could not read the first or,
after takeover, write the other two. Sharing the account also means a recording
has the same owner whether ZoneMinder or zm-api started the `zmc` that wrote it,
so takeover and revert never leave files the other side cannot delete.

The package reads `ZM_WEB_USER` at install and on every upgrade and writes the
result to a systemd drop-in,
`/etc/systemd/system/zm-api.service.d/10-account.conf`. It never modifies the
account itself. The extra groups the service needs go in the same drop-in. On a
host with no ZoneMinder it creates a dedicated `zoneminder` account instead.

```bash
systemctl show -p User -p Group zm-api      # what it runs as
sudo /usr/share/zm-api/setup-instance.sh    # re-resolve, e.g. after installing
sudo systemctl restart zm-api               # ZoneMinder or changing web server
```

To run as something else, add your own drop-in with `systemctl edit zm-api`. It
sorts after `10-account.conf` and wins.

> **What the first start does to an existing ZoneMinder database.** The `.deb`
> and `.rpm` start the service as part of the install, so this happens then.
>
> - Schema at ZoneMinder 1.39.1 or later: adopted in place. zm-api records which
>   upstream updates the schema already has and creates its own tables. If the
>   schema is behind the version zm-api mirrors, it also applies the upstream
>   updates in between, the ones `zmupdate.pl` would. A schema ahead of it is
>   left as it is.
> - Schema older than 1.39.1: nothing is changed, and the journal says to run
>   `zm-api-db bridge`. See [Upgrading an existing ZoneMinder](upgrading.md).
>
> zm-api only *warns* when startup migrations fail, so a database left in the
> wrong state gives you a service that looks healthy with features silently
> missing. Back up before installing beside anything older than current
> ZoneMinder, and check `journalctl -u zm-api | grep -i migrat` afterwards.

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
