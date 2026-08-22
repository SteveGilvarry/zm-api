# Passive and takeover mode

zm-api ships **passive** so that installing it cannot disturb a running
ZoneMinder. That is a starting point, not a destination: **takeover is where
zm-api is meant to end up.** Passive exists so you choose *when* to switch, on
your own schedule, with a one-command way back.

## Passive

`daemon.enabled = false`. zm-api serves the REST API. `zoneminder.service` keeps
supervising `zmdc.pl`, `zmc`, `zmfilter` and the rest, exactly as before zm-api
was installed. Installing the package changes nothing about how ZoneMinder
records, which is what makes it safe to drop onto a live box.

The daemon-control endpoints (`/api/v3/daemons*`, `/api/v3/system/*`) are still
registered but return 503.

## Takeover

`daemon.enabled = true`. zm-api supervises the ZoneMinder daemons itself,
replacing both `zmdc.pl` and `zmwatch.pl` with one native supervisor.

This is an upgrade, not a lateral move. What you gain:

- **One supervisor instead of two processes.** ZoneMinder runs `zmdc.pl` to
  start daemons and `zmwatch.pl` to poll whether the capture daemons are still
  healthy. zm-api does both in its own health-check loop, so there is no second
  Perl daemon whose own death goes unnoticed.
- **Exponential backoff with recovery.** A daemon that crashes repeatedly backs
  off between restarts rather than being restarted at a fixed interval forever,
  and the backoff resets once it has stayed up. A camera that is genuinely
  unreachable stops generating restart churn.
- **Reconciliation against the database.** A loop syncs running daemons with
  what the `Monitors` table says should be running, so enabling or disabling a
  monitor through the API takes effect without a separate restart cycle.
- **Daemon control over the REST API.** Start, stop, and query daemons through
  `/api/v3/daemons*` instead of shelling out to `zmdc.pl`.
- **Structured logging.** Supervision events go to the journal alongside
  everything else zm-api logs, rather than into ZoneMinder's own log tables and
  files.

The legacy `zmdc.sock` IPC shim is still bound, so tooling that talks to
`zmdc.pl` keeps working.

**Exactly one supervisor may run.** On startup in takeover mode zm-api runs
`kill_orphan_daemons()`, which `pkill -9`s `zmc`, `zma`, `zmfilter.pl` and
friends before starting its own. Leaving `zoneminder.service` enabled means two
supervisors killing and restarting each other's processes: daemons that restart
in a loop and events that record erratically. `zm-api-takeover` handles the
ordering for you — that is the whole reason it exists.

## Switching

```bash
sudo zm-api-takeover            # take over
sudo zm-api-takeover --revert   # hand back
```

The script sequences both services correctly — disable and stop
`zoneminder.service`, flip the flag, restart zm-api; and the reverse on the way
back. Prefer it over editing `APP_DAEMON__ENABLED` by hand.

## Before you take over

- zm-api must already be working in passive mode. Takeover is not a way to fix
  a broken install: if the API cannot reach the database, taking over the
  capture daemons as well means nothing records.
- The service user needs write access to the events directory, and membership
  of `ZM_STREAM_SOCKET_GROUP` for live streaming.
- Nothing else may be starting the ZoneMinder daemons — no cron entry, no
  `zmpkg.pl` invocation, no second host sharing the database with the same
  server ID.

## Verifying

```bash
systemctl is-active zm-api          # active
systemctl is-enabled zoneminder     # disabled (or not-found)
ps -o ppid=,cmd= -C zmc             # one zmc per enabled monitor,
                                    # parented by the zm-api process
journalctl -u zm-api -n 50
```

Then confirm recording actually continues — watch the event count for an active
monitor rise over a few minutes. A clean `systemctl status` proves the
supervisor started, not that video is being captured.

## Reverting

`sudo zm-api-takeover --revert` is the first thing to try if capture misbehaves.
It restores the arrangement the host had before, and the REST API keeps working
throughout — passive is a fully supported mode, not a broken state.

Reverting is cheap and reversible in both directions, which is the point: you
can take over on a Tuesday afternoon, watch it for a day, and go back with one
command if anything looks wrong. Treat it as a rollback, not a defeat.

Full details: `man 8 zm-api-takeover`.
