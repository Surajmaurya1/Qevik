use std::env;

#[derive(Debug, Default, Clone)]
pub struct LaunchArgs {
    pub is_startup: bool,
}

impl LaunchArgs {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let is_startup = args.iter().any(|arg| arg == "--startup" || arg == "-s");
        Self { is_startup }
    }
}
