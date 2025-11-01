use keycrate::{AuthenticateOptions, LicenseAuthClient, RegisterOptions};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    println!("=== Keycrate – Simple Demo ===\n");

    let client = LicenseAuthClient::new(
        "https://api.keycrate.dev",
        "YOUR_APP_ID",
    );

    print!(" (1) Authenticate   (2) Register   → ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    if choice == "1" {
        print!("License key (or ENTER for username): ");
        io::stdout().flush().unwrap();

        let mut key = String::new();
        io::stdin().read_line(&mut key).unwrap();
        let key = key.trim();

        if !key.is_empty() {
            let opts = AuthenticateOptions {
                license: Some(key.to_string()),
                ..Default::default()
            };
            match client.authenticate(opts).await {
                Ok(result) => print_result(result.success, &result.message),
                Err(e) => print_result(false, &e.to_string()),
            }
        } else {
            prompt_username_password(&client).await;
        }
    } else if choice == "2" {
        print!("License key to bind: ");
        io::stdout().flush().unwrap();
        let mut lic = String::new();
        io::stdin().read_line(&mut lic).unwrap();

        print!("Username: ");
        io::stdout().flush().unwrap();
        let mut user = String::new();
        io::stdin().read_line(&mut user).unwrap();

        print!("Password: ");
        io::stdout().flush().unwrap();
        let mut pass = String::new();
        io::stdin().read_line(&mut pass).unwrap();

        let opts = RegisterOptions {
            license: lic.trim().to_string(),
            username: user.trim().to_string(),
            password: pass.trim().to_string(),
        };

        match client.register(opts).await {
            Ok(result) => print_result(result.success, &result.message),
            Err(e) => print_result(false, &e.to_string()),
        }
    } else {
        println!("Invalid choice – exiting.");
    }
}

async fn prompt_username_password(client: &LicenseAuthClient) {
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut u = String::new();
    io::stdin().read_line(&mut u).unwrap();

    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut p = String::new();
    io::stdin().read_line(&mut p).unwrap();

    let opts = AuthenticateOptions {
        username: Some(u.trim().to_string()),
        password: Some(p.trim().to_string()),
        ..Default::default()
    };

    match client.authenticate(opts).await {
        Ok(result) => print_result(result.success, &result.message),
        Err(e) => print_result(false, &e.to_string()),
    }
}

fn print_result(ok: bool, msg: &str) {
    let status = if ok { "SUCCESS" } else { "FAILED" };
    let color = if ok { "\x1b[32m" } else { "\x1b[31m" };
    let reset = "\x1b[0m";

    println!("\n{}{}: {}{}", color, status, msg, reset);
}