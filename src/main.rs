mod cdp;
mod cli;

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(cli::run())
}
