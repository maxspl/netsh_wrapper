use std::process::Command;
use std::time::Duration;
use std::thread;
use log::{debug, error, info, warn};
use std::process;
use std::env;

#[cfg(debug_assertions)]
fn init_logging() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
}

#[cfg(not(debug_assertions))]
fn init_logging() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
}

fn check_output(output: std::process::Output, message: &str) {
    if output.status.success() {
        info!("PowerShell command executed successfully {}", message);
    } else {
        error!("PowerShell command failed with exit code {:?}", output.status);
        // Print stderr if available
        if !output.stderr.is_empty() {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            error!("PowerShell stderr: {}", stderr_str);
        }
        if !output.stdout.is_empty() {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            error!("PowerShell stdout_str: {}", stdout_str);
        }
        process::exit(0);
    }
}


fn main() {
    init_logging();

    // Parse command-line arguments
    let mut duration: Option<u64> = None;
    let mut output: Option<String> = None;
    let mut maxsize: Option<String> = None;

    let mut args = env::args().skip(1); // Skip the program name
    while let Some(arg) = args.next() {
        if arg.starts_with("duration=") {
            duration = Some(arg["duration=".len()..].parse().expect("Invalid duration"));
        } else if arg.starts_with("output=") {
            output = Some(arg["output=".len()..].to_string());
        } else if arg.starts_with("maxsize=") {
            maxsize = Some(arg["maxsize=".len()..].to_string());
        }
    }

    // Check if all required arguments are provided
    if duration.is_none() || output.is_none() || maxsize.is_none() {
        error!("Usage: {} duration=<duration in secondes> output=<path> maxsize=<maximum size in MB>", env::args().next().unwrap());
        return;
    }

    let duration_int = duration.unwrap();
    let output_string = output.unwrap();
    let maxsize_string = maxsize.unwrap();

    // Start netsh
    let powershell_command = format!(
        "netsh trace start capture=yes tracefile={} maxSize={}",
        output_string,
        maxsize_string
    );
    debug!("Powershell command : {}", powershell_command);
    let output = Command::new("powershell")
        .args(&["-Command", &powershell_command])
        .output()
        .expect("Failed to execute PowerShell command");

    let message = format!("capture start. Duration : {}. Maxsize : {}", duration_int, maxsize_string);
    check_output(output, &message); //message allows to print the successful args

    // Sleep for the specified duration
    thread::sleep(Duration::from_secs(duration_int));

    // Stop netsh
    let output = Command::new("powershell")
        .args(&["-Command", "netsh trace stop"])
        .output()
        .expect("Failed to execute PowerShell command");

    check_output(output, "capture stop");
}