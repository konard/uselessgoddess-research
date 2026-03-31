mod container;
mod spoof;

use clap::{Parser, Subcommand};

use container::session::SteamSession;
use container::steam_auth::SteamCredentials;
use container::update::UpdateConfig;

#[derive(Parser)]
#[command(name = "worker", about = "Docker container worker for CS2 case farming")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check host dependencies (Docker, GPU render nodes, CS2 image)
    CheckDeps {
        #[arg(long, default_value = "cs2-farm:latest")]
        image: String,

        #[arg(long, default_value = "/opt/cs2-shared")]
        cs2_dir: String,
    },

    /// Create and start a new CS2 farming container
    Create {
        #[arg(short, long)]
        name: String,

        #[arg(long, default_value = "cs2-farm:latest")]
        image: String,

        #[arg(short, long, default_value = "2g")]
        ram: String,

        #[arg(short, long, default_value = "2.0")]
        cpus: String,

        #[arg(long, default_value = "5901")]
        vnc_port: u16,

        #[arg(long)]
        cs2_shared_dir: Option<String>,

        #[arg(long, default_value = "/var/lib/vmctl/container-spoof")]
        spoof_dir: String,
    },

    /// Start a stopped container
    Start {
        name: String,
    },

    /// Stop a running container
    Stop {
        name: String,
    },

    /// Force-kill and remove a container
    Destroy {
        name: String,

        #[arg(long, default_value = "/var/lib/vmctl/container-spoof")]
        spoof_dir: String,
    },

    /// List all CS2 farming containers
    List {
        #[arg(long, default_value = "cs2-farm")]
        prefix: String,
    },

    /// Create multiple containers in batch
    Setup {
        #[arg(long, default_value = "cs2-farm:latest")]
        image: String,

        #[arg(short, long, default_value = "4")]
        count: u32,

        #[arg(long, default_value = "cs2-farm")]
        prefix: String,

        #[arg(short, long, default_value = "2g")]
        ram: String,

        #[arg(long, default_value = "2.0")]
        cpus: String,

        #[arg(long, default_value = "5901")]
        vnc_start: u16,

        #[arg(long)]
        cs2_shared_dir: Option<String>,

        #[arg(long, default_value = "/var/lib/vmctl/container-spoof")]
        spoof_dir: String,
    },

    /// Execute a command inside a container
    Exec {
        name: String,

        #[arg(short, long)]
        cmd: String,
    },

    /// Verify hardware spoofing inside a running container
    Verify {
        name: String,

        #[arg(long)]
        json: bool,
    },

    /// Show spoofed hardware identity for a container name
    ShowIdentity {
        name: String,
    },

    /// Inject a Steam session into a running container
    InjectSession {
        name: String,

        #[arg(long)]
        account: String,

        #[arg(long)]
        token: String,

        #[arg(long)]
        steam_id: String,

        #[arg(long, default_value = "FarmBot")]
        persona: String,
    },

    /// Switch a container to a different Steam account
    SwitchAccount {
        name: String,

        #[arg(long)]
        account: String,

        #[arg(long)]
        token: String,

        #[arg(long)]
        steam_id: String,

        #[arg(long, default_value = "FarmBot")]
        persona: String,
    },

    /// Check CS2 update status
    Cs2Status {
        #[arg(long, default_value = "/opt/cs2-shared")]
        shared_dir: String,
    },

    /// Update CS2 in the shared directory and notify containers
    Cs2Update {
        #[arg(long, default_value = "/opt/cs2-shared")]
        shared_dir: String,

        #[arg(long)]
        containers: Vec<String>,
    },

    /// Check if Wayland display is ready in a container
    DisplayStatus {
        name: String,
    },

    /// Take a screenshot from a container
    Screenshot {
        name: String,

        #[arg(short, long, default_value = "/tmp/cs2-screenshot.png")]
        output: String,
    },

    /// Login to Steam with username/password/shared_secret and get a refresh token
    SteamLogin {
        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,

        /// Base64-encoded shared_secret for Steam Guard TOTP
        #[arg(long)]
        shared_secret: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate a Steam Guard TOTP code from a shared secret
    SteamGuardCode {
        /// Base64-encoded shared_secret
        #[arg(long)]
        shared_secret: String,
    },

    /// Full auto-start: login to Steam, inject session, configure library, start CS2
    AutoStart {
        /// Container name
        name: String,

        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,

        /// Base64-encoded shared_secret for Steam Guard TOTP
        #[arg(long)]
        shared_secret: Option<String>,

        #[arg(long, default_value = "FarmBot")]
        persona: String,

        /// Path to CS2 shared directory mount inside container
        #[arg(long, default_value = "/opt/cs2")]
        cs2_mount: String,
    },

    /// Inject Steam library folders config into a container (add /opt/cs2 as library)
    InjectLibrary {
        name: String,

        /// CS2 mount path inside the container
        #[arg(long, default_value = "/opt/cs2")]
        cs2_mount: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::CheckDeps { image, cs2_dir } => cmd_check_deps(&image, &cs2_dir),
        Commands::Create {
            name,
            image,
            ram,
            cpus,
            vnc_port,
            cs2_shared_dir,
            spoof_dir,
        } => cmd_create(
            &name,
            &image,
            &ram,
            &cpus,
            vnc_port,
            cs2_shared_dir.as_deref(),
            &spoof_dir,
        ),
        Commands::Start { name } => cmd_start(&name),
        Commands::Stop { name } => cmd_stop(&name),
        Commands::Destroy { name, spoof_dir } => cmd_destroy(&name, &spoof_dir),
        Commands::List { prefix } => cmd_list(&prefix),
        Commands::Setup {
            image,
            count,
            prefix,
            ram,
            cpus,
            vnc_start,
            cs2_shared_dir,
            spoof_dir,
        } => cmd_setup(
            &image,
            count,
            &prefix,
            &ram,
            &cpus,
            vnc_start,
            cs2_shared_dir.as_deref(),
            &spoof_dir,
        ),
        Commands::Exec { name, cmd } => cmd_exec(&name, &cmd),
        Commands::Verify { name, json } => cmd_verify(&name, json),
        Commands::ShowIdentity { name } => cmd_show_identity(&name),
        Commands::InjectSession {
            name,
            account,
            token,
            steam_id,
            persona,
        } => cmd_inject_session(&name, &account, &token, &steam_id, &persona),
        Commands::SwitchAccount {
            name,
            account,
            token,
            steam_id,
            persona,
        } => cmd_switch_account(&name, &account, &token, &steam_id, &persona),
        Commands::Cs2Status { shared_dir } => cmd_cs2_status(&shared_dir),
        Commands::Cs2Update {
            shared_dir,
            containers,
        } => cmd_cs2_update(&shared_dir, &containers),
        Commands::DisplayStatus { name } => cmd_display_status(&name),
        Commands::Screenshot { name, output } => cmd_screenshot(&name, &output),
        Commands::SteamLogin {
            username,
            password,
            shared_secret,
            json,
        } => cmd_steam_login(&username, &password, shared_secret.as_deref(), json),
        Commands::SteamGuardCode { shared_secret } => cmd_steam_guard_code(&shared_secret),
        Commands::AutoStart {
            name,
            username,
            password,
            shared_secret,
            persona,
            cs2_mount,
        } => cmd_auto_start(
            &name,
            &username,
            &password,
            shared_secret.as_deref(),
            &persona,
            &cs2_mount,
        ),
        Commands::InjectLibrary { name, cs2_mount } => cmd_inject_library(&name, &cs2_mount),
    }
}

fn cmd_check_deps(image_name: &str, cs2_dir: &str) {
    println!("Checking host dependencies...\n");
    let (ok, errors) = container::deps::check_all(image_name, cs2_dir);
    for check in &ok {
        let status = if check.ok { "ok" } else { "warn" };
        println!("  [{status}] {}: {}", check.name, check.detail);
    }
    for err in &errors {
        println!("  [FAIL] {err}");
    }
    println!();
    if errors.is_empty() {
        println!("All dependencies satisfied.");
    } else {
        println!("{} check(s) passed, {} failed.", ok.len(), errors.len());
        std::process::exit(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_create(
    name: &str,
    image_name: &str,
    ram: &str,
    cpus: &str,
    vnc_port: u16,
    cs2_shared_dir: Option<&str>,
    spoof_dir: &str,
) {
    let hw = spoof::generate_identity(name);
    println!("Generated identity for container '{name}':");
    println!("  MAC:        {}", hw.mac_address);
    println!("  Machine-ID: {}", container::spoof::generate_machine_id(name));
    println!(
        "  SMBIOS:     {} / {}",
        hw.smbios_manufacturer, hw.smbios_product
    );
    println!("  Serial:     {}", hw.smbios_serial);

    let cfg = container::ContainerConfig {
        name: name.to_string(),
        image: image_name.to_string(),
        memory_limit: ram.to_string(),
        cpu_limit: cpus.to_string(),
        vnc_port,
        hw,
        cs2_shared_dir: cs2_shared_dir.map(String::from),
        spoof_dir: spoof_dir.to_string(),
        extra_args: Vec::new(),
    };

    match container::create(&cfg) {
        Ok(container_id) => {
            println!("\nContainer '{name}' created: {container_id}");
            println!("VNC available at 127.0.0.1:{vnc_port}");
        }
        Err(e) => {
            eprintln!("Failed to create container: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_start(name: &str) {
    match container::start(name) {
        Ok(_) => println!("Container '{name}' started."),
        Err(e) => {
            eprintln!("Failed to start container '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_stop(name: &str) {
    match container::stop(name) {
        Ok(_) => println!("Container '{name}' stopped."),
        Err(e) => {
            eprintln!("Failed to stop container '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_destroy(name: &str, spoof_dir: &str) {
    match container::remove(name, true) {
        Ok(_) => println!("Container '{name}' destroyed."),
        Err(e) => {
            eprintln!("Failed to destroy container '{name}': {e}");
            std::process::exit(1);
        }
    }

    if let Err(e) = container::spoof::cleanup_spoof_files(spoof_dir, name) {
        eprintln!("Warning: failed to clean up spoof files: {e}");
    }
}

fn cmd_list(prefix: &str) {
    match container::list_all(prefix) {
        Ok(containers) => {
            if containers.is_empty() {
                println!("No containers found with prefix '{prefix}'.");
                return;
            }
            println!("{:<20} {:<12} {:<20} VNC", "Name", "State", "Image");
            println!("{}", "-".repeat(60));
            for c in &containers {
                let vnc = c
                    .vnc_port
                    .map(|p| format!(":{p}"))
                    .unwrap_or_else(|| "-".into());
                println!("{:<20} {:<12} {:<20} {vnc}", c.name, c.state, c.image);
            }
        }
        Err(e) => {
            eprintln!("Failed to list containers: {e}");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_setup(
    image_name: &str,
    count: u32,
    prefix: &str,
    ram: &str,
    cpus: &str,
    vnc_start: u16,
    cs2_shared_dir: Option<&str>,
    spoof_dir: &str,
) {
    println!("Setting up {count} containers with prefix '{prefix}'...\n");

    for i in 0..count {
        let name = format!("{prefix}-{i}");
        let vnc_port = vnc_start + i as u16;
        println!("--- Creating container '{name}' (VNC :{vnc_port}) ---");
        cmd_create(&name, image_name, ram, cpus, vnc_port, cs2_shared_dir, spoof_dir);
        println!();
    }
    println!("Setup complete. {count} containers created.");
}

fn cmd_exec(name: &str, cmd: &str) {
    match container::exec::exec(name, cmd) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            std::process::exit(result.exit_code);
        }
        Err(e) => {
            eprintln!("Container exec failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_verify(name: &str, json: bool) {
    let hw = spoof::generate_identity(name);
    match container::verify::verify_spoofing(name, &hw) {
        Ok(report) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("Spoofing verification for container '{name}':\n");
                for check in &report.checks {
                    let status = if check.passed { "ok" } else { "FAIL" };
                    println!("  [{status}] {}", check.component);
                    if !check.passed {
                        println!("      expected: {}", check.expected);
                        println!("      actual:   {}", check.actual);
                    }
                }
                println!();
                if report.all_passed {
                    println!("All spoofing checks passed.");
                } else {
                    let failed = report.checks.iter().filter(|c| !c.passed).count();
                    println!("{failed} check(s) failed.");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Verification failed for container '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_show_identity(name: &str) {
    let hw = spoof::generate_identity(name);
    let machine_id = container::spoof::generate_machine_id(name);
    println!("{{");
    println!("  \"mac_address\": \"{}\",", hw.mac_address);
    println!("  \"machine_id\": \"{machine_id}\",");
    println!(
        "  \"smbios_manufacturer\": \"{}\",",
        hw.smbios_manufacturer
    );
    println!("  \"smbios_product\": \"{}\",", hw.smbios_product);
    println!("  \"smbios_serial\": \"{}\",", hw.smbios_serial);
    println!("  \"disk_model\": \"{}\",", hw.disk_model);
    println!("  \"disk_serial\": \"{}\"", hw.disk_serial);
    println!("}}");
}

fn cmd_inject_session(name: &str, account: &str, token: &str, steam_id: &str, persona: &str) {
    let sess = SteamSession {
        account_name: account.to_string(),
        refresh_token: token.to_string(),
        steam_id: steam_id.to_string(),
        persona_name: persona.to_string(),
    };
    match container::session::inject_session(name, &sess, None) {
        Ok(()) => println!("Session injected for account '{account}' in container '{name}'."),
        Err(e) => {
            eprintln!("Session injection failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_switch_account(name: &str, account: &str, token: &str, steam_id: &str, persona: &str) {
    let sess = SteamSession {
        account_name: account.to_string(),
        refresh_token: token.to_string(),
        steam_id: steam_id.to_string(),
        persona_name: persona.to_string(),
    };
    match container::session::switch_account(name, &sess, None) {
        Ok(()) => println!("Switched container '{name}' to account '{account}'."),
        Err(e) => {
            eprintln!("Account switch failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_cs2_status(shared_dir: &str) {
    let cfg = UpdateConfig {
        shared_dir: shared_dir.to_string(),
        lock_file: format!("{shared_dir}/.update.lock"),
        ..Default::default()
    };
    let status = container::update::check_status(&cfg);
    println!("{}", serde_json::to_string_pretty(&status).unwrap());
}

fn cmd_cs2_update(shared_dir: &str, containers: &[String]) {
    let cfg = UpdateConfig {
        shared_dir: shared_dir.to_string(),
        lock_file: format!("{shared_dir}/.update.lock"),
        ..Default::default()
    };

    println!("Starting CS2 update in {shared_dir}...");
    match container::update::perform_update(&cfg, containers) {
        Ok(output) => {
            println!("Update completed successfully.");
            if !output.is_empty() {
                println!("\nsteamcmd output:\n{output}");
            }
        }
        Err(e) => {
            eprintln!("Update failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_display_status(name: &str) {
    match container::display::is_display_ready(name) {
        Ok(true) => {
            println!("Display is ready in container '{name}'.");
            let (host, port) = container::display::vnc_address(5900);
            println!("VNC: {host}:{port} (use container's mapped port)");
        }
        Ok(false) => {
            println!("Display is NOT ready in container '{name}'.");
            println!("Sway or wayvnc may not be running yet.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to check display: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_screenshot(name: &str, output: &str) {
    match container::display::capture_frame(name) {
        Ok(data) => {
            if let Err(e) = std::fs::write(output, &data) {
                eprintln!("Failed to write screenshot to {output}: {e}");
                std::process::exit(1);
            }
            println!("Screenshot saved to {output} ({} bytes)", data.len());
        }
        Err(e) => {
            eprintln!("Screenshot failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_steam_login(username: &str, password: &str, shared_secret: Option<&str>, json: bool) {
    let creds = SteamCredentials {
        username: username.to_string(),
        password: password.to_string(),
        shared_secret: shared_secret.map(String::from),
    };

    println!("Logging in to Steam as '{username}'...");

    match container::steam_auth::login(&creds) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("Login successful!");
                println!("  Account:       {}", result.account_name);
                println!("  Steam ID:      {}", result.steam_id);
                println!("  Refresh Token: {}", result.refresh_token);
            }
        }
        Err(e) => {
            eprintln!("Steam login failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_steam_guard_code(shared_secret: &str) {
    match container::steam_auth::generate_steam_guard_code(shared_secret, 0) {
        Ok(code) => println!("{code}"),
        Err(e) => {
            eprintln!("Failed to generate Steam Guard code: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_auto_start(
    name: &str,
    username: &str,
    password: &str,
    shared_secret: Option<&str>,
    persona: &str,
    cs2_mount: &str,
) {
    // Step 1: Login to Steam to get refresh token
    let creds = SteamCredentials {
        username: username.to_string(),
        password: password.to_string(),
        shared_secret: shared_secret.map(String::from),
    };

    println!("[1/3] Logging in to Steam as '{username}'...");
    let login_result = match container::steam_auth::login(&creds) {
        Ok(result) => {
            println!("  Login successful (Steam ID: {})", result.steam_id);
            result
        }
        Err(e) => {
            eprintln!("Steam login failed: {e}");
            std::process::exit(1);
        }
    };

    // Step 2: Inject Steam library folders (add CS2 shared dir)
    println!("[2/3] Configuring Steam library in container '{name}'...");
    match container::steam_library::inject_library_folders(name, Some(cs2_mount)) {
        Ok(()) => println!("  Library folders configured ({cs2_mount})"),
        Err(e) => {
            eprintln!("Warning: library injection failed: {e}");
            eprintln!("  Steam may re-download CS2. Continuing anyway...");
        }
    }

    // Step 3: Inject session (this triggers steam-launcher.sh inside the container)
    println!("[3/3] Injecting session into container '{name}'...");
    let sess = SteamSession {
        account_name: login_result.account_name.clone(),
        refresh_token: login_result.refresh_token,
        steam_id: login_result.steam_id,
        persona_name: persona.to_string(),
    };

    match container::session::inject_session(name, &sess, None) {
        Ok(()) => {
            println!("  Session injected. Steam + CS2 will auto-launch.");
            println!();
            println!("Auto-start complete for account '{}'.", login_result.account_name);
        }
        Err(e) => {
            eprintln!("Session injection failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_inject_library(name: &str, cs2_mount: &str) {
    match container::steam_library::inject_library_folders(name, Some(cs2_mount)) {
        Ok(()) => println!("Library folders configured in container '{name}' ({cs2_mount})"),
        Err(e) => {
            eprintln!("Library injection failed: {e}");
            std::process::exit(1);
        }
    }
}
