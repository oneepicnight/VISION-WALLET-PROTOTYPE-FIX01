use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

#[test]
fn vision_market_smoke() {
    // Start the real vision-market binary (or fallback to `cargo run`)
    let exe = std::path::Path::new("target/debug/vision-market.exe");
    let mut child: Child = if exe.exists() {
        Command::new(exe)
            .spawn()
            .expect("failed to spawn vision-market executable")
    } else {
        Command::new("cargo")
            .args(["run", "--bin", "vision-market"])
            .spawn()
            .expect("failed to spawn `cargo run --bin vision-market`")
    };

    // Give the server a moment to start and bind
    sleep(Duration::from_millis(800));

    let resp = reqwest::blocking::get("http://127.0.0.1:8080/market/land/listings")
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "expected 200 from /market/land/listings"
    );

    // Tear down
    let _ = child.kill();
    let _ = child.wait();
}
