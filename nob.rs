use std::process::{Command, Stdio, ExitCode};

fn main() -> ExitCode {
    println!("[SYS] cargo build");
    let cmd = Command::new("cargo")
	.arg("build")
	.stdin(Stdio::null())
	.stdout(Stdio::inherit())
	.output();
    match cmd {
	Err(err) => {
	    eprintln!("[ERROR] Failed to build debug version: {err}");
	    return ExitCode::FAILURE;
	},
	Ok(_) => {
	    println!("[INFO] Built debug version");
	}
    };

    println!("[SYS] cargo build --release");
    let cmd = Command::new("cargo")
	.arg("build")
	.arg("--release")
	.stdin(Stdio::null())
	.stdout(Stdio::inherit())
	.output();
    match cmd {
	Err(err) => {
	    eprintln!("[ERROR] Failed to build release version: {err}");
	    return ExitCode::FAILURE;
	},
	Ok(_) => {
	    println!("[INFO] Built debug version");
	}
    };

    return ExitCode::SUCCESS;
}
