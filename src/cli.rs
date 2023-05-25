use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub username: String,

    #[arg(short, long)]
    pub password: String,

    #[arg(short='v', long, action = clap::ArgAction::Count)]
    pub debug: u8,
}
