pub async fn run(_args: crate::cli::TestArgs) -> Result<(), crate::Error> {
    let request = crate::runner::RunRequest::load_default().await?;
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
