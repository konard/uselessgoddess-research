mod config;
mod container;
mod container_deps;
mod container_display;
mod container_exec;
mod container_session;
mod container_spoof;
mod container_update;
mod container_verify;
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
#[command(name = "vmctl", about = "KVM/QEMU VM and Docker container worker for CS2 case farming")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ====================================================================
    // VM commands (original)
    // ====================================================================
    /// Check that all required host dependencies are present (VM mode)
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

    // ====================================================================
    // Container commands (new)
    // ====================================================================
    /// Check host dependencies for container mode
    ContainerCheckDeps {
        /// Docker image name to check
        #[arg(long, default_value = "cs2-farm:latest")]
        image: String,

        /// Shared CS2 directory
        #[arg(long, default_value = "/opt/cs2-shared")]
        cs2_dir: String,
    },

    /// Create and start a new CS2 farming container
    ContainerCreate {
        /// Container name (used as seed for hardware spoofing)
        #[arg(short, long)]
        name: String,

        /// Docker image to use
        #[arg(long, default_value = "cs2-farm:latest")]
        image: String,

        /// RAM limit (e.g., "2g")
        #[arg(short, long, default_value = "2g")]
        ram: String,

        /// CPU limit (e.g., "2.0" for 2 cores)
        #[arg(short, long, default_value = "2.0")]
        cpus: String,

        /// VNC port to expose on host
        #[arg(long, default_value = "5901")]
        vnc_port: u16,

        /// Host path for shared CS2 directory
        #[arg(long)]
        cs2_shared_dir: Option<String>,

        /// Host directory to store spoof files
        #[arg(long, default_value = "/var/lib/vmctl/container-spoof")]
        spoof_dir: String,
    },

    /// Start a stopped container
    ContainerStart {
        /// Container name
        name: String,
    },

    /// Stop a running container (graceful)
    ContainerStop {
        /// Container name
        name: String,
    },

    /// Force-kill and remove a container
    ContainerDestroy {
        /// Container name
        name: String,

        /// Also clean up spoof files
        #[arg(long, default_value = "/var/lib/vmctl/container-spoof")]
        spoof_dir: String,
    },

    /// List all CS2 farming containers
    ContainerList {
        /// Filter by name prefix
        #[arg(long, default_value = "cs2-farm")]
        prefix: String,
    },

    /// Create multiple containers in batch
    ContainerSetup {
        /// Docker image
        #[arg(long, default_value = "cs2-farm:latest")]
        image: String,

        /// Number of containers
        #[arg(short, long, default_value = "4")]
        count: u32,

        /// Name prefix
        #[arg(long, default_value = "cs2-farm")]
        prefix: String,

        /// RAM per container
        #[arg(short, long, default_value = "2g")]
        ram: String,

        /// CPU per container
        #[arg(long, default_value = "2.0")]
        cpus: String,

        /// Starting VNC port
        #[arg(long, default_value = "5901")]
        vnc_start: u16,

        /// Shared CS2 directory
        #[arg(long)]
        cs2_shared_dir: Option<String>,

        /// Spoof files directory
        #[arg(long, default_value = "/var/lib/vmctl/container-spoof")]
        spoof_dir: String,
    },

    /// Execute a command inside a container
    ContainerExec {
        /// Container name
        name: String,

        /// Command to execute
        #[arg(short, long)]
        cmd: String,
    },

    /// Verify hardware spoofing inside a running container
    ContainerVerify {
        /// Container name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show spoofed identity for a container name
    ContainerShowIdentity {
        /// Container name
        name: String,
    },

    /// Inject a Steam session into a running container
    ContainerInjectSession {
        /// Container name
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

    /// Switch a container to a different Steam account
    ContainerSwitchAccount {
        /// Container name
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

    /// Update CS2 and notify containers
    ContainerCs2Update {
        /// Shared CS2 directory
        #[arg(long, default_value = "/opt/cs2-shared")]
        shared_dir: String,

        /// Container names to notify
        #[arg(long)]
        containers: Vec<String>,
    },

    /// Check if Wayland display is ready in a container
    ContainerDisplayStatus {
        /// Container name
        name: String,
    },

    /// Take a screenshot from a container
    ContainerScreenshot {
        /// Container name
        name: String,

        /// Output file path on host
        #[arg(short, long, default_value = "/tmp/cs2-screenshot.png")]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // ================================================================
        // VM commands (original)
        // ================================================================
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

        // ================================================================
        // Container commands (new)
        // ================================================================
        Commands::ContainerCheckDeps { image, cs2_dir } => {
            cmd_container_check_deps(&image, &cs2_dir)
        }
        Commands::ContainerCreate {
            name,
            image,
            ram,
            cpus,
            vnc_port,
            cs2_shared_dir,
            spoof_dir,
        } => cmd_container_create(
            &name,
            &image,
            &ram,
            &cpus,
            vnc_port,
            cs2_shared_dir.as_deref(),
            &spoof_dir,
        ),
        Commands::ContainerStart { name } => cmd_container_start(&name),
        Commands::ContainerStop { name } => cmd_container_stop(&name),
        Commands::ContainerDestroy { name, spoof_dir } => {
            cmd_container_destroy(&name, &spoof_dir)
        }
        Commands::ContainerList { prefix } => cmd_container_list(&prefix),
        Commands::ContainerSetup {
            image,
            count,
            prefix,
            ram,
            cpus,
            vnc_start,
            cs2_shared_dir,
            spoof_dir,
        } => cmd_container_setup(
            &image,
            count,
            &prefix,
            &ram,
            &cpus,
            vnc_start,
            cs2_shared_dir.as_deref(),
            &spoof_dir,
        ),
        Commands::ContainerExec { name, cmd } => cmd_container_exec(&name, &cmd),
        Commands::ContainerVerify { name, json } => cmd_container_verify(&name, json),
        Commands::ContainerShowIdentity { name } => cmd_container_show_identity(&name),
        Commands::ContainerInjectSession {
            name,
            account,
            token,
            steam_id,
            persona,
        } => cmd_container_inject_session(&name, &account, &token, &steam_id, &persona),
        Commands::ContainerSwitchAccount {
            name,
            account,
            token,
            steam_id,
            persona,
        } => cmd_container_switch_account(&name, &account, &token, &steam_id, &persona),
        Commands::ContainerCs2Update {
            shared_dir,
            containers,
        } => cmd_container_cs2_update(&shared_dir, &containers),
        Commands::ContainerDisplayStatus { name } => cmd_container_display_status(&name),
        Commands::ContainerScreenshot { name, output } => {
            cmd_container_screenshot(&name, &output)
        }
    }
}

// ========================================================================
// VM command implementations (original)
// ========================================================================

fn cmd_check_deps() {
    println!("Checking host dependencies (VM mode)...\n");
    let (ok, errors) = deps::check_all();
    for check in &ok {
        println!("  [ok] {}: {}", check.name, check.detail);
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
    println!(
        "  SMBIOS: {} / {}",
        hw.smbios_manufacturer, hw.smbios_product
    );
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

    if let Err(e) = image::resize_image(&overlay_path, "25G") {
        eprintln!("Failed to resize disk: {e}");
        std::process::exit(1);
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
        cloud_init_iso: Some("/var/lib/vmctl/base/cloud-init.iso".into()),
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

// ========================================================================
// Container command implementations (new)
// ========================================================================

fn cmd_container_check_deps(image_name: &str, cs2_dir: &str) {
    println!("Checking host dependencies (container mode)...\n");
    let (ok, errors) = container_deps::check_all(image_name, cs2_dir);
    for check in &ok {
        let status = if check.ok { "ok" } else { "warn" };
        println!("  [{status}] {}: {}", check.name, check.detail);
    }
    for err in &errors {
        println!("  [FAIL] {err}");
    }
    println!();
    if errors.is_empty() {
        println!("All container dependencies satisfied.");
    } else {
        println!("{} check(s) passed, {} failed.", ok.len(), errors.len());
        std::process::exit(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_container_create(
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
    println!("  Machine-ID: {}", container_spoof::generate_machine_id(name));
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

fn cmd_container_start(name: &str) {
    match container::start(name) {
        Ok(_) => println!("Container '{name}' started."),
        Err(e) => {
            eprintln!("Failed to start container '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_container_stop(name: &str) {
    match container::stop(name) {
        Ok(_) => println!("Container '{name}' stopped."),
        Err(e) => {
            eprintln!("Failed to stop container '{name}': {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_container_destroy(name: &str, spoof_dir: &str) {
    // Force-remove container
    match container::remove(name, true) {
        Ok(_) => println!("Container '{name}' destroyed."),
        Err(e) => {
            eprintln!("Failed to destroy container '{name}': {e}");
            std::process::exit(1);
        }
    }

    // Clean up spoof files
    if let Err(e) = container_spoof::cleanup_spoof_files(spoof_dir, name) {
        eprintln!("Warning: failed to clean up spoof files: {e}");
    }
}

fn cmd_container_list(prefix: &str) {
    match container::list_all(prefix) {
        Ok(containers) => {
            if containers.is_empty() {
                println!("No containers found with prefix '{prefix}'.");
                return;
            }
            println!(
                "{:<20} {:<12} {:<20} VNC",
                "Name", "State", "Image"
            );
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
fn cmd_container_setup(
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
        cmd_container_create(
            &name,
            image_name,
            ram,
            cpus,
            vnc_port,
            cs2_shared_dir,
            spoof_dir,
        );
        println!();
    }
    println!("Setup complete. {count} containers created.");
}

fn cmd_container_exec(name: &str, cmd: &str) {
    match container_exec::exec(name, cmd) {
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

fn cmd_container_verify(name: &str, json: bool) {
    let hw = spoof::generate_identity(name);
    match container_verify::verify_spoofing(name, &hw) {
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

fn cmd_container_show_identity(name: &str) {
    let hw = spoof::generate_identity(name);
    let machine_id = container_spoof::generate_machine_id(name);
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

fn cmd_container_inject_session(
    name: &str,
    account: &str,
    token: &str,
    steam_id: &str,
    persona: &str,
) {
    let sess = session::SteamSession {
        account_name: account.to_string(),
        refresh_token: token.to_string(),
        steam_id: steam_id.to_string(),
        persona_name: persona.to_string(),
    };
    match container_session::inject_session(name, &sess, None) {
        Ok(()) => println!("Session injected for account '{account}' in container '{name}'."),
        Err(e) => {
            eprintln!("Session injection failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_container_switch_account(
    name: &str,
    account: &str,
    token: &str,
    steam_id: &str,
    persona: &str,
) {
    let sess = session::SteamSession {
        account_name: account.to_string(),
        refresh_token: token.to_string(),
        steam_id: steam_id.to_string(),
        persona_name: persona.to_string(),
    };
    match container_session::switch_account(name, &sess, None) {
        Ok(()) => println!("Switched container '{name}' to account '{account}'."),
        Err(e) => {
            eprintln!("Account switch failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_container_cs2_update(shared_dir: &str, containers: &[String]) {
    let cfg = update::UpdateConfig {
        shared_dir: shared_dir.to_string(),
        lock_file: format!("{shared_dir}/.update.lock"),
        ..Default::default()
    };

    println!("Starting CS2 update in {shared_dir}...");
    match container_update::perform_update(&cfg, containers) {
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

fn cmd_container_display_status(name: &str) {
    match container_display::is_display_ready(name) {
        Ok(true) => {
            println!("Display is ready in container '{name}'.");
            let (host, port) = container_display::vnc_address(5900);
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

fn cmd_container_screenshot(name: &str, output: &str) {
    match container_display::capture_frame(name) {
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
