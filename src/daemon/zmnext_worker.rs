//! Running zm-next workers outside zm-api's lifetime.
//!
//! A worker must keep recording when zm-api restarts or is upgraded (zm-next
//! `docs/Worker_Control_Protocol.md`, "Worker lifetime"). So it isn't an
//! ordinary child of zm-api:
//!
//! * under systemd, with the packaged `zm-next@.service` template installed,
//!   each worker is `zm-next@<monitor id>.service`, started and stopped with
//!   `systemctl` (a packaged polkit rule allows zm-api's user those verbs on
//!   those units only);
//! * otherwise it is spawned in its own session, without a parent-death
//!   signal, and its pid is written to a pidfile.
//!
//! Either way zm-api writes the worker's pipeline to
//! `<runtime_dir>/<id>.json` (mode 0600; it carries camera credentials) instead
//! of piping it to stdin, which a unit can't receive. When zm-api starts, it
//! finds running workers here and adopts the ones whose pipeline file matches
//! the pipeline it would send now.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;
use tracing::{debug, warn};

/// How workers are started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Launcher {
    /// `zm-next@<id>.service` instances.
    Systemd,
    /// Own session plus a pidfile.
    Session,
}

const UNIT_TEMPLATE_DIRS: &[&str] = &[
    "/etc/systemd/system",
    "/lib/systemd/system",
    "/usr/lib/systemd/system",
];

/// Pick the launcher: `systemd` or `session` when configured, otherwise
/// systemd when it is running and the template unit is installed.
pub fn choose_launcher(configured: &str) -> Launcher {
    match configured {
        "systemd" => Launcher::Systemd,
        "session" => Launcher::Session,
        _ => {
            let systemd = Path::new("/run/systemd/system").is_dir();
            let template = UNIT_TEMPLATE_DIRS
                .iter()
                .any(|d| Path::new(d).join("zm-next@.service").is_file());
            if systemd && template {
                Launcher::Systemd
            } else {
                Launcher::Session
            }
        }
    }
}

pub fn unit_name(monitor_id: u32) -> String {
    format!("zm-next@{monitor_id}.service")
}

/// The files zm-api keeps for its workers.
#[derive(Debug, Clone)]
pub struct RuntimeDir {
    pub dir: PathBuf,
}

impl RuntimeDir {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn pipeline_path(&self, monitor_id: u32) -> PathBuf {
        self.dir.join(format!("{monitor_id}.json"))
    }

    pub fn env_path(&self, monitor_id: u32) -> PathBuf {
        self.dir.join(format!("{monitor_id}.env"))
    }

    pub fn pid_path(&self, monitor_id: u32) -> PathBuf {
        self.dir.join(format!("{monitor_id}.pid"))
    }

    fn ensure_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.dir, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }

    /// Replace a file atomically with `mode`.
    fn write_file(&self, path: &Path, bytes: &[u8], mode: u32) -> std::io::Result<()> {
        self.ensure_dir()?;
        let tmp = path.with_extension("tmp");
        let _ = std::fs::remove_file(&tmp);
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(mode);
        }
        #[cfg(not(unix))]
        let _ = mode;
        let mut f = opts.open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        std::fs::rename(&tmp, path)
    }

    /// Write the pipeline a worker will load. Readable by its owner only.
    pub fn write_pipeline(&self, monitor_id: u32, pipeline: &[u8]) -> std::io::Result<PathBuf> {
        let path = self.pipeline_path(monitor_id);
        self.write_file(&path, pipeline, 0o600)?;
        Ok(path)
    }

    /// The pipeline file a running worker was started with, if any.
    pub fn read_pipeline(&self, monitor_id: u32) -> Option<Vec<u8>> {
        std::fs::read(self.pipeline_path(monitor_id)).ok()
    }

    /// The unit's environment: binary, its directory, and arguments. No secrets.
    pub fn write_env(
        &self,
        monitor_id: u32,
        binary: &Path,
        args: &[String],
    ) -> std::io::Result<()> {
        let dir = binary.parent().unwrap_or_else(|| Path::new("/"));
        let text = format!(
            "ZMNEXT_BIN={}\nZMNEXT_DIR={}\nZMNEXT_ARGS={}\n",
            binary.display(),
            dir.display(),
            args.join(" ")
        );
        self.write_file(&self.env_path(monitor_id), text.as_bytes(), 0o600)
    }

    pub fn write_pid(&self, monitor_id: u32, pid: u32) -> std::io::Result<()> {
        self.write_file(
            &self.pid_path(monitor_id),
            format!("{pid}\n").as_bytes(),
            0o600,
        )
    }

    pub fn read_pid(&self, monitor_id: u32) -> Option<u32> {
        std::fs::read_to_string(self.pid_path(monitor_id))
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    pub fn remove_pid(&self, monitor_id: u32) {
        let _ = std::fs::remove_file(self.pid_path(monitor_id));
    }
}

/// What `systemctl show` says about a unit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnitInfo {
    pub active_state: String,
    pub main_pid: u32,
    /// Exit status of the unit's main process, when it exited normally.
    pub exit_status: Option<i32>,
}

fn parse_show(text: &str) -> UnitInfo {
    let mut info = UnitInfo::default();
    let mut code = String::new();
    let mut status = None;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        match k {
            "ActiveState" => info.active_state = v.to_string(),
            "MainPID" => info.main_pid = v.parse().unwrap_or(0),
            "ExecMainCode" => code = v.to_string(),
            "ExecMainStatus" => status = v.parse().ok(),
            _ => {}
        }
    }
    // ExecMainCode 1 is CLD_EXITED: the status is an exit code. Anything else
    // (killed, dumped) means a signal ended it.
    if code == "1" {
        info.exit_status = status;
    }
    info
}

pub async fn unit_info(monitor_id: u32) -> Option<UnitInfo> {
    let out = Command::new("systemctl")
        .args([
            "show",
            &unit_name(monitor_id),
            "--property=ActiveState,MainPID,ExecMainCode,ExecMainStatus",
        ])
        .output()
        .await
        .ok()?;
    out.status
        .success()
        .then(|| parse_show(&String::from_utf8_lossy(&out.stdout)))
}

async fn systemctl(verb: &str, monitor_id: u32) -> Result<(), String> {
    let out = Command::new("systemctl")
        .args(["--no-ask-password", verb, &unit_name(monitor_id)])
        .output()
        .await
        .map_err(|e| format!("systemctl {verb}: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "systemctl {verb} {}: {}",
            unit_name(monitor_id),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Start the monitor's unit and return its main pid once systemd reports one.
pub async fn start_unit(monitor_id: u32) -> Result<u32, String> {
    systemctl("start", monitor_id).await?;
    for _ in 0..50 {
        match unit_info(monitor_id).await {
            Some(info) if info.main_pid > 0 => return Ok(info.main_pid),
            Some(info) if info.active_state == "failed" || info.active_state == "inactive" => {
                return Err(format!(
                    "{} exited at once (status {:?})",
                    unit_name(monitor_id),
                    info.exit_status
                ))
            }
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    Err(format!("{} reported no main pid", unit_name(monitor_id)))
}

pub async fn stop_unit(monitor_id: u32) -> Result<(), String> {
    systemctl("stop", monitor_id).await
}

/// Process start time (clock ticks since boot, `/proc/<pid>/stat` field 22):
/// with the pid, it identifies one process even after the pid is reused.
pub fn start_time(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The command name is parenthesised and may contain spaces; fields after
    // the last ')' are space-separated, starting at field 3.
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().nth(19)?.parse().ok()
}

/// Whether `pid` is a zombie: exited, waiting to be reaped.
fn is_zombie(pid: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|stat| {
            let rest = stat[stat.rfind(')')? + 1..].trim_start().to_string();
            rest.chars().next()
        })
        == Some('Z')
}

/// Whether `pid` is still the process that had `started`. Where `/proc`
/// isn't available it falls back to `kill(pid, 0)`.
pub fn is_alive(pid: u32, started: Option<u64>) -> bool {
    if !Path::new("/proc/self/stat").exists() {
        #[cfg(unix)]
        return nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid as i32), None).is_ok();
        #[cfg(not(unix))]
        return false;
    }
    if is_zombie(pid) {
        return false;
    }
    match (start_time(pid), started) {
        (Some(now), Some(then)) => now == then,
        (Some(_), None) => true,
        (None, _) => false,
    }
}

/// Whether `pid` is a zm-core worker for `monitor_id`, from its command line.
pub fn is_worker_for(pid: u32, monitor_id: u32) -> bool {
    let Ok(raw) = std::fs::read(format!("/proc/{pid}/cmdline")) else {
        return false;
    };
    let args: Vec<String> = raw
        .split(|b| *b == 0)
        .map(|a| String::from_utf8_lossy(a).into_owned())
        .collect();
    let is_core = args
        .first()
        .and_then(|a| Path::new(a).file_name())
        .and_then(|n| n.to_str())
        == Some("zm-core");
    let mon = monitor_id.to_string();
    is_core
        && args
            .windows(2)
            .any(|w| w[0] == "--monitor-id" && w[1] == mon)
}

/// A worker found running for a monitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningWorker {
    pub pid: u32,
    pub start_time: Option<u64>,
}

/// Look for a live worker for `monitor_id`: the active unit's main pid, or
/// the pidfile's pid when it is still that monitor's zm-core.
pub async fn find_running(
    launcher: Launcher,
    runtime: &RuntimeDir,
    monitor_id: u32,
) -> Option<RunningWorker> {
    if launcher == Launcher::Systemd {
        if let Some(info) = unit_info(monitor_id).await {
            if info.active_state == "active" && info.main_pid > 0 {
                return Some(RunningWorker {
                    pid: info.main_pid,
                    start_time: start_time(info.main_pid),
                });
            }
        }
    }
    let pid = runtime.read_pid(monitor_id)?;
    if is_worker_for(pid, monitor_id) {
        Some(RunningWorker {
            pid,
            start_time: start_time(pid),
        })
    } else {
        debug!("zm-next: stale pidfile for monitor {monitor_id} (pid {pid})");
        runtime.remove_pid(monitor_id);
        None
    }
}

/// Whether a running worker already has the pipeline zm-api would start it
/// with: its pipeline file is byte-identical. (A worker that sends a hello can
/// be compared by `pipeline_hash` instead.)
pub fn pipeline_matches(runtime: &RuntimeDir, monitor_id: u32, wanted: &[u8]) -> bool {
    match runtime.read_pipeline(monitor_id) {
        Some(current) => current == wanted,
        None => {
            warn!("zm-next: running worker for monitor {monitor_id} has no pipeline file");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> RuntimeDir {
        let dir = std::env::temp_dir().join(format!("zmnext_rt_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        RuntimeDir::new(dir)
    }

    #[test]
    fn launcher_honours_configuration() {
        assert_eq!(choose_launcher("systemd"), Launcher::Systemd);
        assert_eq!(choose_launcher("session"), Launcher::Session);
    }

    #[test]
    fn pipeline_file_is_private_and_compared_exactly() {
        let rt = tmp("pipeline");
        let path = rt.write_pipeline(3, br#"{"plugins":[]}"#).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
            let dir_mode = std::fs::metadata(&rt.dir).unwrap().permissions().mode() & 0o777;
            assert_eq!(dir_mode, 0o700);
        }
        assert!(pipeline_matches(&rt, 3, br#"{"plugins":[]}"#));
        assert!(!pipeline_matches(&rt, 3, br#"{"plugins":[1]}"#));
        assert!(!pipeline_matches(&rt, 4, br#"{"plugins":[]}"#));
        let _ = std::fs::remove_dir_all(&rt.dir);
    }

    #[test]
    fn env_file_names_binary_dir_and_args() {
        let rt = tmp("env");
        rt.write_env(
            3,
            Path::new("/opt/zm-next/zm-core"),
            &[
                "--monitor-id".into(),
                "3".into(),
                "--socket".into(),
                "/run/zm/stream_3.sock".into(),
            ],
        )
        .unwrap();
        let text = std::fs::read_to_string(rt.env_path(3)).unwrap();
        assert_eq!(
            text,
            "ZMNEXT_BIN=/opt/zm-next/zm-core\nZMNEXT_DIR=/opt/zm-next\n\
             ZMNEXT_ARGS=--monitor-id 3 --socket /run/zm/stream_3.sock\n"
        );
        let _ = std::fs::remove_dir_all(&rt.dir);
    }

    #[test]
    fn systemctl_show_output_is_parsed() {
        let exited =
            parse_show("ActiveState=failed\nMainPID=0\nExecMainCode=1\nExecMainStatus=3\n");
        assert_eq!(exited.active_state, "failed");
        assert_eq!(exited.exit_status, Some(3));
        let killed =
            parse_show("ActiveState=inactive\nMainPID=0\nExecMainCode=2\nExecMainStatus=15\n");
        assert_eq!(killed.exit_status, None);
        let running =
            parse_show("ActiveState=active\nMainPID=4242\nExecMainCode=0\nExecMainStatus=0\n");
        assert_eq!(running.main_pid, 4242);
    }

    #[test]
    fn pidfile_round_trip() {
        let rt = tmp("pid");
        assert_eq!(rt.read_pid(9), None);
        rt.write_pid(9, 1234).unwrap();
        assert_eq!(rt.read_pid(9), Some(1234));
        rt.remove_pid(9);
        assert_eq!(rt.read_pid(9), None);
        let _ = std::fs::remove_dir_all(&rt.dir);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn own_process_is_alive_and_identified_by_start_time() {
        let me = std::process::id();
        let started = start_time(me).expect("own start time");
        assert!(is_alive(me, Some(started)));
        assert!(
            !is_alive(me, Some(started + 1)),
            "a reused pid has another start time"
        );
        assert!(!is_worker_for(me, 1), "the test binary isn't zm-core");
    }
}
