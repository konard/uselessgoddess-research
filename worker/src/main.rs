mod config;
mod deps;
mod disk;
mod guest_agent;
mod image;
mod session;
mod spoof;
mod update;
mod verify;
mod vm;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vmctl", about = "KVM/QEMU VM worker for CS2 case farming")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check that all required host dependencies are present
    CheckDeps,

    /// Create and define a new VM
    Create {
        /// VM name (used as seed for hardware spoofing)
        #[arg(short, long)]
        name: String,

        /// RAM in MiB
        #[arg(short, long, default_value = "2048")]
        ram: u32,

        /// Number of vCPUs
        #[arg(short, long, default_value = "2")]
        cpus: u32,

        /// VNC port
        #[arg(long, default_value = "5901")]
        vnc_port: u16,

        /// Path to base disk image (qcow2); overlay will be created
        #[arg(long)]
        base_disk: String,

        /// Directory to store VM disk overlays
        #[arg(long, default_value = "/var/lib/vmctl/disks")]
        disk_dir: String,

        /// Host path for virtiofs shared directory (e.g. /opt/cs2)
        #[arg(long)]
        virtiofs_source: Option<String>,

        /// virtiofs mount tag inside guest
        #[arg(long, default_value = "cs2")]
        virtiofs_tag: String,
    },

    /// Start a defined VM
    Start {
        /// VM name
        name: String,
    },

    /// Gracefully shut down a VM
    Stop {
        /// VM name
        name: String,
    },

    /// Force-stop a VM
    Destroy {
        /// VM name
        name: String,
    },

    /// Undefine a VM (remove definition, keep disk)
    Undefine {
        /// VM name
        name: String,
    },

    /// List all VMs
    List,

    /// Show spoofed hardware identity for a VM name
    ShowIdentity {
        /// VM name
        name: String,
    },

    /// Create multiple VMs from a base image in batch
    Setup {
        /// Base disk image (qcow2)
        #[arg(long)]
        base_disk: String,

        /// Number of VMs to create
        #[arg(short, long, default_value = "4")]
        count: u32,

        /// Name prefix for VMs
        #[arg(long, default_value = "cs2-farm")]
        prefix: String,

        /// RAM per VM in MiB
        #[arg(short, long, default_value = "2048")]
        ram: u32,

        /// vCPUs per VM
        #[arg(long, default_value = "2")]
        cpus: u32,

        /// Starting VNC port
        #[arg(long, default_value = "5901")]
        vnc_start: u16,

        /// Disk overlay directory
        #[arg(long, default_value = "/var/lib/vmctl/disks")]
        disk_dir: String,

        /// Host path for virtiofs shared CS2 directory
        #[arg(long)]
        virtiofs_source: Option<String>,
    },

    /// Verify hardware spoofing inside a running VM
    Verify {
        /// VM name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Ping the QEMU guest agent inside a VM
    GaPing {
        /// VM name
        name: String,
    },

    /// Execute a command inside a VM via the guest agent
    GaExec {
        /// VM name
        name: String,

        /// Command to execute
        #[arg(short, long)]
        cmd: String,
    },

    /// Inject a Steam session into a running VM for auto-login
    InjectSession {
        /// VM name
        name: String,

        /// Steam account name
        #[arg(long)]
        account: String,

        /// Steam refresh token
        #[arg(long)]
        token: String,

        /// Steam ID (64-bit)
        #[arg(long)]
        steam_id: String,

        /// Display name
        #[arg(long, default_value = "FarmBot")]
        persona: String,
    },

    /// Switch a VM to a different Steam account
    SwitchAccount {
        /// VM name
        name: String,

        /// Steam account name
        #[arg(long)]
        account: String,

        /// Steam refresh token
        #[arg(long)]
        token: String,

        /// Steam ID (64-bit)
        #[arg(long)]
        steam_id: String,

        /// Display name
        #[arg(long, default_value = "FarmBot")]
        persona: String,
    },

    /// Check CS2 update status
    Cs2Status {
        /// Shared CS2 directory
        #[arg(long, default_value = "/opt/cs2-shared")]
        shared_dir: String,
    },

    /// Update CS2 in the shared directory
    Cs2Update {
        /// Shared CS2 directory
        #[arg(long, default_value = "/opt/cs2-shared")]
        shared_dir: String,

        /// VM names to notify of the update
        #[arg(long)]
        vms: Vec<String>,
    },

    /// Generate cloud-init ISO for VM provisioning
    CloudInit {
        /// Output ISO path
        #[arg(short, long)]
        output: String,

        /// VM name (used for instance metadata)
        #[arg(long)]
        vm_name: String,

        /// Farm user name inside VM
        #[arg(long, default_value = "farmuser")]
        user: String,

        /// Farm user password
        #[arg(long, default_value = "farmpass")]
        password: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::CheckDeps => cmd_check_deps(),
        Commands::Create {
            name,
            ram,
            cpus,
            vnc_port,
            base_disk,
            disk_dir,
            virtiofs_source,
            virtiofs_tag,
        } => cmd_create(
            &name,
            ram,
            cpus,
            vnc_port,
            &base_disk,
            &disk_dir,
            virtiofs_source.as_deref(),
            &virtiofs_tag,
        ),
        Commands::Start { name } => cmd_start(&name),
        Commands::Stop { name } => cmd_stop(&name),
        Commands::Destroy { name } => cmd_destroy(&name),
        Commands::Undefine { name } => cmd_undefine(&name),
        Commands::List => cmd_list(),
        Commands::ShowIdentity { name } => cmd_show_identity(&name),
        Commands::Setup {
            base_disk,
            count,
            prefix,
            ram,
            cpus,
            vnc_start,
            disk_dir,
            virtiofs_source,
        } => cmd_setup(
            &base_disk,
            count,
            &prefix,
            ram,
            cpus,
            vnc_start,
            &disk_dir,
            virtiofs_source.as_deref(),
        ),
        Commands::Verify { name, json } => cmd_verify(&name, json),
        Commands::GaPing { name } => cmd_ga_ping(&name),
        Commands::GaExec { name, cmd } => cmd_ga_exec(&name, &cmd),
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
        Commands::Cs2Update { shared_dir, vms } => cmd_cs2_update(&shared_dir, &vms),
        Commands::CloudInit {
            output,
            vm_name,
            user,
            password,
        } => cmd_cloud_init(&output, &vm_name, &user, &password),
    }
}

fn cmd_check_deps() {
    println!("Checking host dependencies...\n");
    let (ok, errors) = deps::check_all();
    for check in &ok {
        println!("  ✓ {}: {}", check.name, check.detail);
    }
    for err in &errors {
        println!("  ✗ {err}");
    }
    println!();
    if errors.is_empty() {
        println!("All dependencies satisfied.");
    } else {
        println!(
            "{} check(s) passed, {} failed.",
            ok.len(),
            errors.len()
        );
        std::process::exit(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_create(
    name: &str,
    ram: u32,
    cpus: u32,
    vnc_port: u16,
    base_disk: &str,
    disk_dir: &str,
    virtiofs_source: Option<&str>,
    virtiofs_tag: &str,
) {
    let hw = spoof::generate_identity(name);
    println!("Generated identity for '{name}':");
    println!("  MAC:    {}", hw.mac_address);
    println!("  SMBIOS: {} / {}", hw.smbios_manufacturer, hw.smbios_product);
    println!("  Serial: {}", hw.smbios_serial);
    println!("  Disk:   {} ({})", hw.disk_model, hw.disk_serial);

    // Create disk overlay
    let overlay_path = format!("{disk_dir}/{name}.qcow2");
    std::fs::create_dir_all(disk_dir).unwrap_or_else(|e| {
        eprintln!("Failed to create disk directory {disk_dir}: {e}");
        std::process::exit(1);
    });
    match disk::create_overlay(&overlay_path, base_disk) {
        Ok(()) => println!("\nCreated overlay: {overlay_path}"),
        Err(e) => {
            eprintln!("Disk error: {e}");
            std::process::exit(1);
        }
    }

    // Generate and define XML
    let cfg = config::VmConfig {
        name: name.to_string(),
        ram_mb: ram,
        vcpus: cpus,
        disk_path: overlay_path,
        vnc_port,
        hw,
        virtiofs_source: virtiofs_source.map(String::from),
        virtiofs_tag: if virtiofs_source.is_some() {
            Some(virtiofs_tag.to_string())
        } else {
            None
        },
    };

    let xml = cfg.to_xml();
    match vm::define(&xml) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Failed to define VM: {e}");
            std::process::exit(1);
        }
    }
    println!("\nVM '{name}' created. Use `vmctl start {name}` to boot.");
}

fn cmd_start(name: &str) {
    match vm::start(name) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Failed to start VM '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_stop(name: &str) {
    match vm::shutdown(name) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Failed to stop VM '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_destroy(name: &str) {
    match vm::destroy(name) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Failed to destroy VM '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_undefine(name: &str) {
    match vm::undefine(name) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Failed to undefine VM '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_list() {
    match vm::list_all() {
        Ok(vms) => {
            if vms.is_empty() {
                println!("No VMs defined.");
                return;
            }
            println!("{:<6} {:<20} State", "ID", "Name");
            println!("{}", "-".repeat(40));
            for v in &vms {
                let id = v.id.map(|i| i.to_string()).unwrap_or_else(|| "-".into());
                println!("{:<6} {:<20} {}", id, v.name, v.state);
            }
        }
        Err(e) => {
            eprintln!("Failed to list VMs: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_show_identity(name: &str) {
    let hw = spoof::generate_identity(name);
    println!("{}", serde_json::to_string_pretty(&hw).unwrap());
}

#[allow(clippy::too_many_arguments)]
fn cmd_setup(
    base_disk: &str,
    count: u32,
    prefix: &str,
    ram: u32,
    cpus: u32,
    vnc_start: u16,
    disk_dir: &str,
    virtiofs_source: Option<&str>,
) {
    println!("Setting up {count} VMs with prefix '{prefix}'...\n");

    for i in 0..count {
        let name = format!("{prefix}-{i}");
        let vnc_port = vnc_start + i as u16;
        println!("--- Creating VM '{name}' (VNC :{vnc_port}) ---");
        cmd_create(
            &name,
            ram,
            cpus,
            vnc_port,
            base_disk,
            disk_dir,
            virtiofs_source,
            "cs2",
        );
        println!();
    }
    println!("Setup complete. {count} VMs created.");
}

fn cmd_verify(name: &str, json: bool) {
    let hw = spoof::generate_identity(name);
    match verify::verify_spoofing(name, &hw) {
        Ok(report) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("Spoofing verification for VM '{name}':\n");
                for check in &report.checks {
                    let status = if check.passed { "✓" } else { "✗" };
                    println!("  {status} {}", check.component);
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
            eprintln!("Verification failed for VM '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_ga_ping(name: &str) {
    match guest_agent::ping(name) {
        Ok(true) => println!("Guest agent is responding in VM '{name}'."),
        Ok(false) => {
            println!("Guest agent is NOT responding in VM '{name}'.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error pinging guest agent: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_ga_exec(name: &str, cmd: &str) {
    match guest_agent::exec(name, cmd) {
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
            eprintln!("Guest exec failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_inject_session(name: &str, account: &str, token: &str, steam_id: &str, persona: &str) {
    let sess = session::SteamSession {
        account_name: account.to_string(),
        refresh_token: token.to_string(),
        steam_id: steam_id.to_string(),
        persona_name: persona.to_string(),
    };
    match session::inject_session(name, &sess, None) {
        Ok(()) => println!("Session injected for account '{account}' in VM '{name}'."),
        Err(e) => {
            eprintln!("Session injection failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_switch_account(name: &str, account: &str, token: &str, steam_id: &str, persona: &str) {
    let sess = session::SteamSession {
        account_name: account.to_string(),
        refresh_token: token.to_string(),
        steam_id: steam_id.to_string(),
        persona_name: persona.to_string(),
    };
    match session::switch_account(name, &sess, None) {
        Ok(()) => println!("Switched VM '{name}' to account '{account}'."),
        Err(e) => {
            eprintln!("Account switch failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_cs2_status(shared_dir: &str) {
    let cfg = update::UpdateConfig {
        shared_dir: shared_dir.to_string(),
        lock_file: format!("{shared_dir}/.update.lock"),
        ..Default::default()
    };
    let status = update::check_status(&cfg);
    println!("{}", serde_json::to_string_pretty(&status).unwrap());
}

fn cmd_cs2_update(shared_dir: &str, vms: &[String]) {
    let cfg = update::UpdateConfig {
        shared_dir: shared_dir.to_string(),
        lock_file: format!("{shared_dir}/.update.lock"),
        ..Default::default()
    };

    println!("Starting CS2 update in {shared_dir}...");
    match update::perform_update(&cfg, vms) {
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

fn cmd_cloud_init(output: &str, vm_name: &str, user: &str, password: &str) {
    match image::create_cloud_init_iso(output, vm_name, user, password) {
        Ok(()) => println!("Cloud-init ISO created: {output}"),
        Err(e) => {
            eprintln!("Failed to create cloud-init ISO: {e}");
            std::process::exit(1);
        }
    }
}
