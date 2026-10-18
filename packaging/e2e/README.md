# Packaging end-to-end tests

These install the real package on a host running systemd and walk the install
process the way an operator does: ZoneMinder already recording, zm-api
installed beside it in passive mode, takeover, revert, upgrade, removal. Each
step checks that capture and recording carried on.

The host is a container with systemd as PID 1, so maintainer scripts, the unit
file and `systemctl` behave as on a real machine.

## Running

Docker is the only requirement.

```bash
packaging/e2e/build.sh ubuntu-24.04     # builds the package inside ubuntu:24.04
packaging/e2e/run.sh ubuntu-24.04       # about 10 minutes
```

`build.sh` packages the working tree as it stands, uncommitted changes
included. After editing only shell scripts or the unit, the rebuild is under a
minute because the compiled binaries are cached.

Useful options for `run.sh`:

| Option | Effect |
| --- | --- |
| `--keep` | Leave the container up. `docker exec -it zm-api-e2e-<distro> bash` |
| `--stages install,passive` | Run only these stages, in this order |
| `--no-zoneminder` | Use the package-only flow on a distro that has ZoneMinder |
| `--package DIR` | Test a package built elsewhere (the release workflow does) |

Logs, the zm-api and ZoneMinder journals, and each stage's output land in
`target/e2e/<distro>/logs/`.

On Docker Desktop with the default 8 GB of memory the optimised build of the
zm-api crate is killed for running out. Build unoptimised there:

```bash
CARGO_PROFILE_RELEASE_OPT_LEVEL=0 packaging/e2e/build.sh ubuntu-24.04
```

## What runs where

`distros.sh` is the table. A distro where upstream publishes a ZoneMinder
master package gets the full flow against it. The others get the package flow.

| Stage | Full flow | Package flow | Checks |
| --- | --- | --- | --- |
| `database` | yes | yes | MariaDB installed and running |
| `zoneminder` | yes | | ZoneMinder master from zmrepo, a synthetic RTSP camera, one monitor recording continuously |
| `layout` | | yes | The web account, an empty database and a `zm.conf` with upstream's ownership, no ZoneMinder daemons |
| `install` | yes | yes | Files, the account drop-in, keys, service up in passive mode, ZoneMinder's processes and sockets untouched |
| `passive` | yes | yes | Login, monitors, events, a snapshot and live HLS off the stream socket, a restart that leaves ZoneMinder's sockets alone |
| `takeover` | yes | | A takeover that fails is rolled back; a real one moves `zmc` under zm-api, still recording, files still owned by ZoneMinder's account |
| `revert` | yes | | `zmdc.pl` supervises again, still recording |
| `upgrade` | yes | yes | Reinstall keeps keys and mode; an install from the old fixed `zoneminder` account moves to ZoneMinder's |
| `remove` | yes | yes | Removal in takeover mode hands control back; purge clears state |
| `standalone` | | yes | With no ZoneMinder at all the package creates its own account and starts |

The camera is `mediamtx` serving a 20 second H.264 and AAC clip that `ffmpeg`
loops without re-encoding, so `zmc` has an encoded stream to put on its stream
socket.

## Adding a distro

Add a case to `distros.sh` with its image, package family, where ZoneMinder
master comes from, and the account upstream's packaging runs ZoneMinder as.
Add it to the matrix in `.github/workflows/packaging.yml`. If the builder or
host image needs different packages, that is in `builder_dockerfile` in
`build.sh` and `host_dockerfile` in `run.sh`.

## Adding a check

Stages are in `guest/stages/`, helpers in `guest/lib.sh`. A stage runs all its
checks and reports every failure at the end:

```bash
check "zmc is capturing" wait_until 90 capturing
check_eq "zmc runs as" "$E2E_WEB_USER" "$(zmc_user)"
```
