fn main() -> std::io::Result<()> {
    Ok(())
}

mod args {
    use std::{net::SocketAddr, path::PathBuf};

    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(long, short)]
        pub path: PathBuf,

        #[arg(long, short)]
        pub dest: SocketAddr,
    }
}
