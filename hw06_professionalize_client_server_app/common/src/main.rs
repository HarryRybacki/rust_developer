use env_logger::{Builder, Env};

/// Solely used to initialize the logger
fn main() {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();
}
