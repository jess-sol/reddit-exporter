use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Reddit username")]
    pub username: String,

    #[arg(short, long, help = "Reddit password")]
    pub password: Option<String>,

    #[arg(long="stdin", help = "Read password from stdin", action = clap::ArgAction::SetTrue)]
    pub password_stdin: bool,

    #[arg(short='v', long, action = clap::ArgAction::Count)]
    pub debug: u8,
}
