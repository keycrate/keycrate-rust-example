use keycrate::{AuthenticateOptions, LicenseAuthClient, RegisterOptions};
use std::io::{self, Write};
use chrono::DateTime;

#[tokio::main]
async fn main() {
    println!("=== Keycrate – Full Demo ===\n");

    // Get real HWID
    let hwid = get_hwid();
    println!("Your HWID: {}\n", hwid);

    let client = LicenseAuthClient::new(
        "https://api.keycrate.dev",
        "YOUR_APP_ID",
    );

    // Login
    let (logged_in, license_key) = login(&client, &hwid).await;
    if !logged_in {
        println!("\nAccess denied – goodbye.");
        return;
    }

    println!("\nWelcome! You have access.\n");

    // Post-login menu
    loop {
        print!("Type 'register' or 'exit': ");
        io::stdout().flush().unwrap();

        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        let cmd = cmd.trim().to_lowercase();

        match cmd.as_str() {
            "exit" => {
                println!("Bye!");
                break;
            }
            "register" => {
                if let Some(key) = license_key {
                    register(&client, &key).await;
                }
                break;
            }
            _ => println!("Invalid command."),
        }
    }
}

async fn login(client: &LicenseAuthClient, hwid: &str) -> (bool, Option<String>) {
    println!("=== Login ===");
    print!("License key (press ENTER for username/password): ");
    io::stdout().flush().unwrap();

    let mut key = String::new();
    io::stdin().read_line(&mut key).unwrap();
    let key = key.trim();

    let result = if !key.is_empty() {
        let opts = AuthenticateOptions {
            license: Some(key.to_string()),
            hwid: Some(hwid.to_string()),
            ..Default::default()
        };
        client.authenticate(opts).await
    } else {
        print!("Username: ");
        io::stdout().flush().unwrap();
        let mut u = String::new();
        io::stdin().read_line(&mut u).unwrap();

        print!("Password: ");
        io::stdout().flush().unwrap();
        let mut p = String::new();
        io::stdin().read_line(&mut p).unwrap();

        let u = u.trim();
        let p = p.trim();

        if u.is_empty() || p.is_empty() {
            println!("Both fields required.");
            return (false, None);
        }

        let opts = AuthenticateOptions {
            username: Some(u.to_string()),
            password: Some(p.to_string()),
            hwid: Some(hwid.to_string()),
            ..Default::default()
        };
        client.authenticate(opts).await
    };

    match result {
        Ok(auth_result) => {
            if auth_result.success {
                println!("\nLogin successful!\n");
                let license = auth_result
                    .data
                    .as_ref()
                    .and_then(|d| d.get("key").and_then(|k| k.as_str().map(String::from)));
                (true, license)
            } else {
                print_error(&auth_result.message, auth_result.data.as_ref());
                (false, None)
            }
        }
        Err(e) => {
            println!("Connection error: {}", e);
            (false, None)
        }
    }
}

async fn register(client: &LicenseAuthClient, license: &str) {
    println!("\n=== Register Username & Password ===");
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut u = String::new();
    io::stdin().read_line(&mut u).unwrap();

    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut p = String::new();
    io::stdin().read_line(&mut p).unwrap();

    let u = u.trim();
    let p = p.trim();

    if u.is_empty() || p.is_empty() {
        println!("Can't be empty.");
        return;
    }

    let opts = RegisterOptions {
        license: license.to_string(),
        username: u.to_string(),
        password: p.to_string(),
    };

    match client.register(opts).await {
        Ok(result) => {
            let color = if result.success { "\x1b[32m" } else { "\x1b[31m" };
            let reset = "\x1b[0m";
            let status = if result.success { "SUCCESS" } else { "FAILED" };
            println!("\n{}{}: {}{}", color, status, result.message, reset);
        }
        Err(e) => println!("Register failed: {}", e),
    }
}

fn print_error(msg: &str, data: Option<&serde_json::Value>) {
    println!("\x1b[31mAuthentication failed: {}\x1b[0m", msg);

    match msg {
        "LICENSE_NOT_FOUND" => {
            println!("License key not found – double-check it.");
        }
        "INVALID_USERNAME_OR_PASSWORD" => {
            println!("Wrong username or password.");
        }
        "LICENSE_NOT_ACTIVE" => {
            println!("License is not active – contact support.");
        }
        "DEVICE_ALREADY_REGISTERED_WITH_OTHER_LICENSE" => {
            println!("This device is already bound to another license.");
        }
        "LICENSE_EXPIRED" => {
            if let Some(d) = data {
                if let Some(expires_at) = d.get("expires_at").and_then(|v| v.as_str()) {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(expires_at) {
                        println!("License expired on: {} UTC", dt.format("%Y-%m-%d %H:%M:%S"));
                    } else {
                        println!("License has expired (invalid date format).");
                    }
                } else {
                    println!("License has expired.");
                }
            } else {
                println!("License has expired.");
            }
        }
        "HWID_MISMATCH" => {
            println!("HWID does not match the registered device.");
            if let Some(d) = data {
                if let Some(true) = d.get("hwid_reset_allowed").and_then(|v| v.as_bool()) {
                    if let (Some(last_str), Some(cd)) = (
                        d.get("last_hwid_reset_at").and_then(|v| v.as_str()),
                        d.get("hwid_reset_cooldown").and_then(|v| v.as_i64()),
                    ) {
                        if let Ok(last_dt) = DateTime::parse_from_rfc3339(last_str) {
                            let now = chrono::Utc::now();
                            let last_utc = last_dt.with_timezone(&chrono::Utc);
                            let elapsed = (now - last_utc).num_seconds();
                            let left = cd - elapsed;
                            if left > 0 {
                                println!("Reset available in {} seconds.", left);
                            } else {
                                println!("HWID reset is now available.");
                            }
                        } else {
                            println!("Try resetting HWID (invalid timestamp).");
                        }
                    } else {
                        println!("Try resetting HWID.");
                    }
                } else {
                    println!("HWID reset not allowed.");
                }
            } else {
                println!("HWID reset not allowed.");
            }
        }
        _ => {
            println!("Unexpected error: {}. Contact support.", msg);
        }
    }
}

#[cfg(target_os = "windows")]
fn get_hwid() -> String {
    use std::process::Command;
    use sha2::{Sha256, Digest};

    let mut parts = Vec::new();

    // CPU ID
    if let Ok(output) = Command::new("wmic")
        .args(&["cpu", "get", "ProcessorId", "/format:list"])
        .output()
    {
        if let Ok(s) = String::from_utf8(output.stdout) {
            if let Some(line) = s.lines().find(|l| l.starts_with("ProcessorId=")) {
                parts.push(line.replace("ProcessorId=", "").trim().to_string());
            }
        }
    }

    // BIOS Serial
    if let Ok(output) = Command::new("wmic")
        .args(&["bios", "get", "SerialNumber", "/format:list"])
        .output()
    {
        if let Ok(s) = String::from_utf8(output.stdout) {
            if let Some(line) = s.lines().find(|l| l.starts_with("SerialNumber=")) {
                parts.push(line.replace("SerialNumber=", "").trim().to_string());
            }
        }
    }

    // Disk Serial
    if let Ok(output) = Command::new("wmic")
        .args(&["logicaldisk", "get", "SerialNumber", "/format:list"])
        .output()
    {
        if let Ok(s) = String::from_utf8(output.stdout) {
            if let Some(line) = s.lines().find(|l| l.starts_with("SerialNumber=")) {
                parts.push(line.replace("SerialNumber=", "").trim().to_string());
            }
        }
    }

    let combined = parts.join("|");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_hwid() -> String {
    "unsupported-platform".to_string()
}