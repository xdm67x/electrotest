mod cdp;
mod cli;
mod electron;

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(cli::run())
}
