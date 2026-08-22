# Passive and takeover mode

zm_api ships **passive** and most deployments should stay there.

## Passive

`daemon.enabled = false`. zm_api serves the REST API. `zoneminder.service` keeps
supervising `zmdc.pl`, `zmc`, `zmfilter` and the rest, exactly as before zm_api
was installed. Installing the package changes nothing about how ZoneMinder
records.

The daemon-control endpoints (`/api/v3/daemons*`, `/api/v3/system/*`) are still
registered but return 503.

## Takeover

`daemon.enabled = true`. zm_api supervises the ZoneMinder daemons itself,
replacing `zmdc.pl` and `zmwatch.pl` with its own manager and health loop.

**Exactly one supervisor may run.** On startup in takeover mode zm_api runs
`kill_orphan_daemons()`, which `pkill -9`s `zmc`, `zma`, `zmfilter.pl` and
friends before starting its own. Leaving `zoneminder.service` enabled means two
supervisors killing and restarting each other's processes: daemons that restart
in a loop and events that record erratically.

## Switching

```bash
sudo zm_api-takeover            # take over
sudo zm_api-takeover --revert   # hand back
```

The script sequences both services correctly — disable and stop
`zoneminder.service`, flip the flag, restart zm_api; and the reverse on the way
back. Prefer it over editing `APP_DAEMON__ENABLED` by hand.

## Before you take over

- zm_api must already be working in passive mode. Takeover is not a way to fix
  a broken install: if the API cannot reach the database, taking over the
  capture daemons as well means nothing records.
- The service user needs write access to the events directory, and membership
  of `ZM_STREAM_SOCKET_GROUP` for live streaming.
- Nothing else may be starting the ZoneMinder daemons — no cron entry, no
  `zmpkg.pl` invocation, no second host sharing the database with the same
  server ID.

## Verifying

```bash
systemctl is-active zm_api          # active
systemctl is-enabled zoneminder     # disabled (or not-found)
ps -o ppid=,cmd= -C zmc             # one zmc per enabled monitor,
                                    # parented by the zm_api process
journalctl -u zm_api -n 50
```

Then confirm recording actually continues — watch the event count for an active
monitor rise over a few minutes. A clean `systemctl status` proves the
supervisor started, not that video is being captured.

## Reverting

`sudo zm_api-takeover --revert` is the first thing to try if capture misbehaves.
It restores the arrangement the host had before, and the REST API keeps working
throughout because passive is its normal state.

Full details: `man 8 zm_api-takeover`.
