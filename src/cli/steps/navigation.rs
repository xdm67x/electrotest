use crate::cli::context::Context;
use crate::cli::feature::Step;
use crate::cli::steps::StepHandler;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;

/// Handler for: "the window size is <WIDTH>x<HEIGHT>"
pub struct WindowSizeStep;

#[async_trait]
impl StepHandler for WindowSizeStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_given_type() && step.text.contains("window size")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r"window size is (\d+)x(\d+)").unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid window size format"))?;

        let width: u32 = caps[1].parse()?;
        let height: u32 = caps[2].parse()?;

        // Set window size using CDP
        let script = format!("window.resizeTo({}, {})", width, height);
        ctx.cdp_client.evaluate(&script).await?;

        ctx.window_size = Some((width, height));

        println!("✓ Set window size to {}x{}", width, height);
        Ok(())
    }
}

/// Handler for: "I navigate to \"<URL>\""
pub struct NavigateStep;

#[async_trait]
impl StepHandler for NavigateStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type()
            && (step.text.contains("navigate to") || step.text.contains("go to"))
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"(?:navigate|go) to \"([^\"]+)\""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid navigate format"))?;

        let url = &caps[1];

        ctx.cdp_client.navigate(url).await?;

        println!("✓ Navigated to {}", url);
        Ok(())
    }
}
