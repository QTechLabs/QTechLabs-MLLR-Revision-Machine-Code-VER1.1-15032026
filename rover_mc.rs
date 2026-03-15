use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const SOLVER_REPO: &str = "https://github.com/RetiredC/RCKangaroo.git";
const SOLVER_DIR: &str = "rckangaroo_solver";
const BINARY_PATH: &str = "rckangaroo_solver/RCKangaroo";
const PTX_OUT: &str = "kernels.ptx";
const SASS_OUT: &str = "kernels.sass";
const BINARY_DUMP: &str = "binary_dump.bin";

const PUZZLE: &str = "135";
const PUBKEY: &str = "02145d2611c823a396ef6712ce0f712f09b9b4f3135e3e0aa3230fb9b6d08d1e16";
const RANGE_START: &str = "4000000000000000000000000000000000";
const RANGE_END: &str = "7fffffffffffffffffffffffffffffffff";
const ADDRESS: &str = "16RGFo6hjq9ym6Pj7N5H7L1NR1rVPJyw2v";

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let extract_only = args.iter().any(|a| a == "--extract-kernels");
    let dry_run = args.iter().any(|a| a == "--dry-run");

    println!("══════════════════════════════════════════════════════════════");
    println!("      1% LOW-LEVEL MACHINE-CODE EXECUTION HUNTER — #{}", PUZZLE);
    println!("══════════════════════════════════════════════════════════════\n");
    println!("Target: {} (~13.5 BTC)  |  Pubkey exposed → Kangaroo OK", ADDRESS);
    println!("Range: 2^134 … 2^135-1\n");

    if !Path::new(SOLVER_DIR).exists() {
        println!("Cloning RCKangaroo (SOTA negation-map Kangaroo)...");
        run_command_safe("git", &["clone", SOLVER_REPO, SOLVER_DIR])?;
    }

    if !Path::new(BINARY_PATH).exists() && !extract_only {
        println!("Compiling → raw x86 + CUDA SASS machine code...");
        let ccap = env::var("CCAP").unwrap_or_else(|_| "89".to_string());
        run_command_in_dir_safe(SOLVER_DIR, "make", &["gpu=1", &format!("ccap={}", ccap)])?;
        println!("✅ Raw binary ready: {}\n", BINARY_PATH);
    }

    if extract_only {
        extract_ptx()?;
        extract_sass()?;
        println!("\n✅ You now hold raw PTX + SASS machine code.");
        return Ok(());
    }

    println!("Dumping full binary → {}", BINARY_DUMP);
    let mut bin_file = File::open(BINARY_PATH)?;
    let mut bin_data = Vec::new();
    bin_file.read_to_end(&mut bin_data)?;
    fs::write(BINARY_DUMP, &bin_data)?;
    println!("✅ Full binary written ({} bytes)", bin_data.len());

    println!("\n🔍 RAW x86 MACHINE CODE (first 512 bytes):\n");
    for (i, chunk) in bin_data.iter().take(512).collect::<Vec<_>>().chunks(16).enumerate() {
        let hex = chunk.iter().map(|&b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
        let ascii: String = chunk.iter().map(|&&b| if (32..=126).contains(&b) { b as char } else { '.' }).collect();
        println!("{:08x}: {:<47} |{}|", i * 16, hex, ascii);
    }
    println!("\n...this is the machine code that will take over the process.\n");

    let target_file = "puzzle_135.txt";
    fs::write(target_file, format!("{}\n{}:{}\n", PUBKEY, RANGE_START, RANGE_END))?;
    println!("Target file created → {}", target_file);

    println!("\n🚀 DIRECT MACHINE-CODE EXECUTION (execve syscall)...\n");
    let mut cmd = Command::new(format!("./{}", BINARY_PATH));
    cmd.args([
        "-gpu",
        "-d", "28",
        "-w", "work_p135.save",
        "-ws",
        "-wi", "1200",
        target_file,
    ]);

    let gpus = env::var("GPU_IDS").unwrap_or_else(|_| "0".to_string());
    for gpu_id in gpus.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cmd.args(["-gpuId", gpu_id]);
    }
    println!("→ Multi-GPU: {}", gpus);

    if dry_run {
        let program = cmd.get_program();
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        println!("DRY-RUN mode: would exec {} with args {:?}", program.to_string_lossy(), args);
        return Ok(());
    }

    let err = cmd.exec();
    eprintln!("execve failed: {}", err);
    Ok(())
}

fn run_command_safe(cmd: &str, args: &[&str]) -> io::Result<()> {
    println!("Running: {} {}", cmd, args.join(" "));
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Command failed: {} {:?}", cmd, args)));
    }
    Ok(())
}

fn run_command_in_dir_safe(dir: &str, cmd: &str, args: &[&str]) -> io::Result<()> {
    println!("Running in {}: {} {}", dir, cmd, args.join(" "));
    let status: ExitStatus = Command::new(cmd).current_dir(dir).args(args).status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Command failed in {}: {} {:?}", dir, cmd, args)));
    }
    Ok(())
}

fn extract_ptx() -> io::Result<()> {
    println!("Extracting PTX...");
    let out = File::create(PTX_OUT)?;
    let status = Command::new("cuobjdump")
        .args(["-ptx", BINARY_PATH])
        .stdout(Stdio::from(out))
        .status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "PTX extraction failed"));
    }
    println!("→ PTX: {}", PTX_OUT);
    Ok(())
}

fn extract_sass() -> io::Result<()> {
    println!("Extracting SASS...");
    let out = File::create(SASS_OUT)?;
    let status = Command::new("cuobjdump")
        .args(["-sass", BINARY_PATH])
        .stdout(Stdio::from(out))
        .status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "SASS extraction failed"));
    }
    println!("→ SASS (native GPU machine code): {}", SASS_OUT);
    Ok(())
}
