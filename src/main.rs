use clap::Parser;
use twopassword::cli::{Cli, CliRunner};
use twopassword::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the application
    twopassword::init()?;

    // Parse command line arguments
    let cli = Cli::parse();

    // Run the CLI
    let mut runner = CliRunner::new();
    runner.run(cli).await?;

    Ok(())
}
