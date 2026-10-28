//! Exercise the actual OS guard with independent child programs, not parser counters.
#![cfg(target_os = "linux")]
use rhwp::resource_supervisor::{run, ExecutionLimits};
use std::process::Command;
use std::time::{Duration, Instant};

fn python(script: &str) -> Command {
    let mut command = Command::new("python3");
    command.args(["-I", "-c", script]);
    command
}

fn stopped(pid: u32) -> bool {
    // A killed orphan can briefly remain a zombie until the host init reaps it.
    std::fs::read_to_string(format!("/proc/{pid}/stat")).map_or(true, |s| {
        s.rsplit_once(')')
            .is_some_and(|(_, tail)| tail.split_whitespace().next() == Some("Z"))
    })
}

#[test]
fn address_space_failure_is_contained_and_next_child_is_healthy() {
    let limits = ExecutionLimits {
        address_space_bytes: 64 * 1024 * 1024,
        ..ExecutionLimits::default()
    };
    let failed = run(&mut python("x=bytearray(128*1024*1024)"), &[], limits).unwrap();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("MemoryError"));
    let next = run(&mut python("print('HEALTHY')"), &[], limits).unwrap();
    assert!(next.status.success());
    assert_eq!(next.stdout, b"HEALTHY\n");
    assert!(next.metrics.address_space_limit_enforced);
}

#[test]
fn cpu_limit_stops_a_busy_child_before_wall_timeout() {
    let limits = ExecutionLimits {
        cpu_seconds: 1,
        wall_timeout_ms: 5_000,
        ..Default::default()
    };
    let started = Instant::now();
    let failed = run(&mut python("while True: pass"), &[], limits).unwrap();
    assert!(!failed.status.success());
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(failed.metrics.cpu_limit_enforced);
}

#[test]
fn stdout_flood_is_rejected_without_waiting_for_completion() {
    let limits = ExecutionLimits {
        output_bytes: 8,
        diagnostics_bytes: 8,
        ..Default::default()
    };
    let result = run(
        &mut python("import sys; sys.stdout.write('x'*1000000); sys.stdout.flush()"),
        &[],
        limits,
    );
    assert!(matches!(result, Err("PDF_PROCESS_OUTPUT_LIMIT")));
}

#[test]
fn unread_stdin_is_cancelled_at_the_wall_deadline() {
    let input = vec![0; 1024 * 1024];
    let started = Instant::now();
    let result = run(
        &mut python("import time; time.sleep(60)"),
        &input,
        ExecutionLimits {
            wall_timeout_ms: 100,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err("PDF_PROCESS_TIMEOUT")));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn escaped_descendant_pipe_does_not_extend_the_request_lifetime() {
    let script = "import subprocess,sys; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)'],start_new_session=True); print(p.pid,flush=True)";
    let started = Instant::now();
    let result = run(&mut python(script), &[], ExecutionLimits::default()).unwrap();
    let pid: u32 = String::from_utf8(result.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // A process group is deliberately not described as a whole process tree
    // sandbox. Clean up this intentionally escaped test process explicitly.
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGKILL);
    }
    assert!(result.status.success());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn completed_leader_cannot_leave_a_descendant_holding_the_response_pipe() {
    let script = "import subprocess,sys; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); print(p.pid, flush=True)";
    let started = Instant::now();
    let result = run(&mut python(script), &[], ExecutionLimits::default()).unwrap();
    assert!(result.status.success());
    assert!(started.elapsed() < Duration::from_secs(5));
    let pid = String::from_utf8(result.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    for _ in 0..100 {
        if stopped(pid) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("descendant {pid} survived completed request");
}

#[test]
fn timeout_kills_the_whole_child_group_and_followup_works() {
    let path = std::env::temp_dir().join(format!(
        "rhwp-8174-pid-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let script = format!("import subprocess,sys,time; p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); open(sys.argv[1],'w').write(str(p.pid)); time.sleep(60)");
    let mut command = python(&script);
    command.arg(&path);
    let result = run(
        &mut command,
        &[],
        ExecutionLimits {
            wall_timeout_ms: 500,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err("PDF_PROCESS_TIMEOUT")));
    let pid = std::fs::read_to_string(&path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    for _ in 0..100 {
        if stopped(pid) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(stopped(pid), "descendant {pid} survived timeout");
    assert!(run(&mut python("pass"), &[], ExecutionLimits::default())
        .unwrap()
        .status
        .success());
}
