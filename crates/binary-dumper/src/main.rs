use clap::Parser;
use sp1_core_executor::{MinimalTranspiler, Program, MINIMAL_TRACE_CHUNK_THRESHOLD};
use sp1_primitives::consts::MAX_JIT_LOG_ADDR;
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Path to the RISC-V ELF binary")]
    binary: PathBuf,

    #[arg(long, default_value = "false", help = "Enable debug mode")]
    debug: bool,

    #[arg(long, default_value = "true", action = clap::ArgAction::Set, help = "Enable tracing")]
    trace: bool,

    #[arg(short, long, help = "Output file path for JIT result")]
    output: Option<PathBuf>,

    #[arg(short = 's', long, help = "Output file path for assembly source")]
    asm_output: Option<PathBuf>,

    #[arg(
        long,
        default_value = "sp1_ecall_handler",
        help = "ECALL symbol name for assembly"
    )]
    ecall_symbol: String,

    #[arg(
        long,
        default_value = "sp1_unimp_handler",
        help = "UNIMP symbol name for assembly"
    )]
    unimp_symbol: String,
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

    let max_memory_size = 2_u64.pow(MAX_JIT_LOG_ADDR as u32) as usize;
    let max_trace_size = if args.trace {
        Some(MINIMAL_TRACE_CHUNK_THRESHOLD)
    } else {
        None
    };

    println!(
        "Transpiling {} with debug={}, trace={}, max_trace_size={:?}",
        args.binary.display(),
        args.debug,
        args.trace,
        max_trace_size
    );

    let transpiler = MinimalTranspiler::new(max_memory_size, args.debug, max_trace_size);
    let compiled_code = transpiler.transpile_to_compiled(&program)?;

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from("jit_result.bin"));
    compiled_code.save(&output_path)?;

    println!("JIT result saved to {}", output_path.display());

    if let Some(asm_path) = args.asm_output {
        compiled_code.write_asm_to_file(&asm_path, &args.ecall_symbol, &args.unimp_symbol)?;
        println!("Assembly saved to {}", asm_path.display());
    }

    println!("  code size: {} bytes", compiled_code.code.len());
    println!("  instructions: {}", compiled_code.jump_table.len());

    Ok(())
}
