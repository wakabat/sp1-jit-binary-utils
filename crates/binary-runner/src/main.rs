use clap::Parser;
use sp1_core_executor::{MinimalExecutor, Program, MINIMAL_TRACE_CHUNK_THRESHOLD};
use sp1_core_machine::io::SP1Stdin;
use sp1_jit::CompiledCode;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the RISC-V ELF binary")]
    binary: PathBuf,

    #[arg(help = "Path to the saved JIT result")]
    jit_binary: PathBuf,

    #[arg(help = "Path to the bincode serialized SP1Stdin")]
    stdin: PathBuf,

    #[arg(long, default_value = "true", action = clap::ArgAction::Set, help = "Enable tracing")]
    trace: bool,
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

    let compiled_code = CompiledCode::load(&args.jit_binary)?;

    let stdin_bytes = std::fs::read(&args.stdin)?;
    let stdin: SP1Stdin = bincode::deserialize(&stdin_bytes)?;

    let max_trace_size = if args.trace {
        Some(MINIMAL_TRACE_CHUNK_THRESHOLD)
    } else {
        None
    };

    println!(
        "Running {} with trace={}, max_trace_size={:?}",
        args.binary.display(),
        args.trace,
        max_trace_size
    );

    let mut executor = MinimalExecutor::from_compiled(program, &compiled_code, max_trace_size);

    for buf in stdin.buffer {
        executor.with_input(&buf);
    }

    let start = std::time::Instant::now();
    while executor.execute_chunk().is_some() {}
    let elapsed = start.elapsed();

    println!("Execution complete:");
    println!("  exit code: {}", executor.exit_code());
    println!("  cycles: {}", executor.global_clk());
    println!("  execution time: {:?}", elapsed);
    if elapsed.as_secs_f64() > 0.0 {
        println!(
            "  mhz: {}",
            executor.global_clk() as f64 / (elapsed.as_secs_f64() * 1_000_000.0)
        );
    }

    Ok(())
}
