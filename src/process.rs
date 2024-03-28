use std::process::Command;

fn exec(cmd: &str, args: &[&str]) -> String {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .expect(&format!("Failed to execute {}", cmd));

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Call to {} {} failed", cmd, args.join(" "));
    }

    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn gh(args: &[&str]) -> String {
    exec("gh", args)
}

pub fn git(args: &[&str]) -> String {
    exec("git", args)
}
