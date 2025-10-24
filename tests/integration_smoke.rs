use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn simple_server_smoke() {
    // Prefer the platform-specific built binary so tests don't fall back to
    // `cargo run` (which can trigger a build and cause timing/race issues).
    let exe_path = if cfg!(windows) {
        "target/debug/simple_server.exe"
    } else {
        "target/debug/simple_server"
    };
    let exe = std::path::Path::new(exe_path);
    let mut child = if exe.exists() {
        Command::new(exe)
            .spawn()
            .expect("failed to spawn simple_server")
    } else {
        // Fallback: run cargo run --bin simple_server
        Command::new("cargo")
            .args(["run", "--bin", "simple_server"])
            .spawn()
            .expect("failed to spawn cargo run simple_server")
    };

    // give the server a moment to bind
    sleep(Duration::from_millis(500));

    let resp = reqwest::blocking::get("http://127.0.0.1:8080/").expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "expected 200 from simple server"
    );

    // Kill the child process
    let _ = child.kill();
    let _ = child.wait();
}
