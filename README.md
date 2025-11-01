# Keycrate SDK - Rust Examples

Simple and full examples for the Keycrate license authentication SDK in Rust.

## Prerequisites

-   Rust 1.56 or higher

## Setup

Install dependencies:

```bash
cargo build
```

## Running

```bash
cargo run
```

## Examples

-   **Simple Example** - Basic authentication with license key or username/password, plus registration
-   **Full Example** - Includes HWID detection, detailed error handling, and a post-login menu

## Configuration

Update the app ID in `src/main.rs`:

```rust
let client = LicenseAuthClient::new(
    "https://api.keycrate.dev",
    "YOUR_APP_ID",
);
```

## Dependencies

-   **keycrate** - Keycrate SDK
-   **tokio** - Async runtime
-   **chrono** - Date/time parsing
-   **serde_json** - JSON handling
-   **sha2** - Hashing
