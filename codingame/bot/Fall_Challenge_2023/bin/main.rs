use std::process::Command;

const JAR_REFEREE_PATH: &str = "./referee.jar";

const LEVEL: &str = "1";

const P1_NAME: &str = "BOT1";
const P2_NAME: &str = "BOT2";

fn main() {
    let mut referee = Command::new("java")
        .arg("-jar")
        .arg(JAR_REFEREE_PATH)
        // player 1
        .arg("-p1name")
        .arg(P1_NAME)
        .arg("-p1")
        .arg("cargo run --bin answer")
        // player 2
        .arg("-p2name")
        .arg(P2_NAME)
        .arg("-p2")
        .arg("cargo run --bin answer")
        // level
        .arg("-lvl")
        .arg(LEVEL)
        //
        // .stdin(Stdio::piped())
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start referee");

    referee.wait().expect("Failed to wait referee");
}
