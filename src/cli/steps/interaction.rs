use crate::cli::context::Context;
use crate::cli::feature::Step;
use crate::cli::steps::StepHandler;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use tokio::time::{sleep, Duration};

/// Handler for: "I click on ..."
pub struct ClickStep;

#[async_trait]
impl StepHandler for ClickStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("click")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        // Try to match "click on button \"<text>\""
        let button_re = Regex::new(r#"click on button "([^"]+)""#).unwrap();
        // Try to match "click on \"<selector>\""
        let selector_re = Regex::new(r#"click on "([^"]+)""#).unwrap();

        let (selector, description) = if let Some(caps) = button_re.captures(&step.text) {
            let text = &caps[1];
            let sel = format!("button:has-text('{}')", text);
            (sel, format!("button with text '{}'", text))
        } else if let Some(caps) = selector_re.captures(&step.text) {
            let sel = caps[1].to_string();
            let desc = sel.clone();
            (sel, desc)
        } else {
            return Err(anyhow::anyhow!("Invalid click format"));
        };

        // Use JavaScript to click the element
        let script = format!(
            r#"
            (function() {{
                let el = document.querySelector('{}');
                if (el) {{
                    el.click();
                    return 'clicked';
                }}
                return 'not found';
            }})()
            "#,
            selector.replace('"', "\\\"")
        );

        let result = ctx.cdp_client.evaluate(&script).await?;

        if result.contains("not found") {
            return Err(anyhow::anyhow!("Element '{}' not found", description));
        }

        println!("✓ Clicked on {}", description);
        Ok(())
    }
}

/// Handler for: "I take a screenshot ..."
pub struct ScreenshotStep;

#[async_trait]
impl StepHandler for ScreenshotStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("screenshot")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"screenshot "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid screenshot format"))?;

        let filename = &caps[1];
        let path = ctx.screenshot_path(filename);

        // Ensure output directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        ctx.cdp_client.screenshot(&path).await?;

        println!("✓ Screenshot saved to {}", path.display());
        Ok(())
    }
}

/// Handler for: "I wait ..."
pub struct WaitStep;

#[async_trait]
impl StepHandler for WaitStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("wait")
    }

    async fn execute(&self, step: &Step, _ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r"wait (\d+(?:\.\d+)?) seconds?").unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid wait format"))?;

        let seconds: f64 = caps[1].parse()?;
        let duration = Duration::from_secs_f64(seconds);

        println!("  Waiting {} seconds...", seconds);
        sleep(duration).await;

        println!("✓ Waited {} seconds", seconds);
        Ok(())
    }
}
