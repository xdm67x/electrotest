use crate::cli::context::Context;
use crate::cli::feature::Step;
use crate::cli::steps::StepHandler;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::sync::LazyLock;
use tokio::time::{Duration, sleep};

const BUTTON_CLICK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"click on "([^"]+)""#).unwrap());

/// Handler for: "I click on ..."
pub struct ClickStep;

#[async_trait]
impl StepHandler for ClickStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("click")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        match BUTTON_CLICK_REGEX.captures(&step.text) {
            Some(caps) => {
                let selector = &caps[1];
                let script = format!(
                    r#"
                    (function() {{
                        // 1. Try CSS selector first
                        let el = document.querySelector({selector:?});

                        // 2. If not found, search by text content
                        if (!el) {{
                            const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
                            let node;
                            while (node = walker.nextNode()) {{
                                if (node.textContent.trim() === {selector:?}) {{
                                    el = node.parentElement;
                                    break;
                                }}
                            }}
                        }}

                        // 3. Click the element if found
                        if (el) {{
                            el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                            el.click();
                            return 'clicked';
                        }}

                        return 'not found';
                    }})()
                    "#
                );

                let result = ctx.cdp_client.evaluate(&script).await?;

                if result.contains("not found") {
                    anyhow::bail!("Element '{selector}' not found");
                }

                println!("✓ Clicked on {selector}");
                Ok(())
            }
            None => {
                anyhow::bail!("invalid click format");
            }
        }
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
