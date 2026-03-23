use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=JIT_O_PATH");
    let o_path = env::var("JIT_O_PATH").unwrap_or_else(|_| "src/jit.o".to_string());

    if !Path::new(&o_path).exists() {
        println!(
            "cargo:warning=JIT .o file not found at '{}'. Set JIT_O_PATH env var to specify location.",
            o_path
        );
        println!("cargo:warning=Expected symbols: sp1_jit_code, sp1_jump_table, sp1_pc_start, sp1_memory_size");
    } else {
        println!("cargo:rustc-link-arg={}", o_path);
    }
}
