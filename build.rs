fn main() {
    // This will execute every time you run `cargo build` or `cargo run`
    println!("cargo:warning=To run the program, use: 'sudo target/debug/supreme-server' (because the program interacts with systemd)");
}

