fn main() -> std::io::Result<()> {
    Ok(())
}

mod args {
    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(long, short, default_value_t = 7123)]
        pub port: u16,
    }
}
