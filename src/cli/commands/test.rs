pub async fn run() -> Result<(), crate::Error> {
    let config = crate::config::load_default().await?;
    crate::config::validate_paths(&config)?;
    crate::config::validate_startup(&config)?;

    let request = crate::runner::RunRequest::from_config(config).await?;
    let summary = crate::runner::execute(request).await?;

    for line in &summary.output {
        println!("{line}");
    }
    println!("{} scenario passed, {} failed", summary.passed, summary.failed);
    if summary.failed > 0 {
        return Err(crate::Error::TestFailures(summary.failed));
    }

    Ok(())
}
