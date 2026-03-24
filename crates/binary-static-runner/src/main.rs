use clap::Parser;
use sp1_core_executor::{MinimalExecutor, Program};
use sp1_core_machine::io::SP1Stdin;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the RISC-V ELF binary (for memory image)")]
    binary: PathBuf,

    #[arg(help = "Path to the bincode serialized SP1Stdin")]
    stdin: PathBuf,
}

fn main() -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off")))
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(false)
                .compact(),
        )
        .init();

    let args = Args::parse();

    let elf_bytes = std::fs::read(&args.binary)?;
    let program = Program::from(&elf_bytes)?;
    let program = Arc::new(program);

    let stdin_bytes = std::fs::read(&args.stdin)?;
    let stdin: SP1Stdin = bincode::deserialize(&stdin_bytes)?;

    println!("Running {}", args.binary.display());

    let mut executor = MinimalExecutor::from_static_link(program);

    for buf in stdin.buffer {
        executor.with_input(&buf);
    }

    let start = std::time::Instant::now();
    let mut count = 0;
    while executor.execute_chunk().is_some() {
        count += 1;
    }
    let elapsed = start.elapsed();

    println!("Execution complete:");
    println!("  exit code: {}", executor.exit_code());
    println!("  cycles: {}", executor.global_clk());
    println!("  execution time: {:?}", elapsed);
    println!("  chunks: {}", count);
    if elapsed.as_secs_f64() > 0.0 {
        println!(
            "  mhz: {}",
            executor.global_clk() as f64 / (elapsed.as_secs_f64() * 1_000_000.0)
        );
    }

    Ok(())
}
