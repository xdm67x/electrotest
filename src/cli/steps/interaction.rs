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

const TYPE_TEXT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"type "([^"]+)" into "([^"]+)""#).unwrap());

/// Handler for: "I type ... into ..."
pub struct TypeTextStep;

#[async_trait]
impl StepHandler for TypeTextStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("type") && step.text.contains("into")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        match TYPE_TEXT_REGEX.captures(&step.text) {
            Some(caps) => {
                let text = &caps[1];
                let selector = &caps[2];

                let script = format!(
                    r#"
                    (function() {{
                        // Find the input element
                        let el = document.querySelector({selector:?});
                        
                        if (!el) {{
                            // Try finding by placeholder or aria-label
                            el = document.querySelector('[placeholder="' + {selector:?} + '"]');
                        }}
                        
                        if (!el) {{
                            // Try finding by text content of parent
                            const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
                            let node;
                            while (node = walker.nextNode()) {{
                                if (node.textContent.trim() === {selector:?}) {{
                                    el = node.parentElement;
                                    break;
                                }}
                            }}
                        }}

                        if (!el) {{
                            return 'element not found';
                        }}

                        // Focus the element
                        el.focus();
                        
                        // Clear existing value
                        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                            el.value = '';
                        }}
                        
                        // Type the text character by character
                        const textToType = {text:?};
                        for (let i = 0; i < textToType.length; i++) {{
                            const char = textToType[i];
                            
                            // KeyDown event
                            const keyDownEvent = new KeyboardEvent('keydown', {{
                                key: char,
                                code: 'Key' + char.toUpperCase(),
                                bubbles: true
                            }});
                            el.dispatchEvent(keyDownEvent);
                            
                            // KeyPress event
                            const keyPressEvent = new KeyboardEvent('keypress', {{
                                key: char,
                                charCode: char.charCodeAt(0),
                                bubbles: true
                            }});
                            el.dispatchEvent(keyPressEvent);
                            
                            // Input event
                            if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                                el.value += char;
                            }}
                            
                            // KeyUp event
                            const keyUpEvent = new KeyboardEvent('keyup', {{
                                key: char,
                                code: 'Key' + char.toUpperCase(),
                                bubbles: true
                            }});
                            el.dispatchEvent(keyUpEvent);
                        }}
                        
                        // Input event for the whole text
                        const inputEvent = new Event('input', {{ bubbles: true }});
                        el.dispatchEvent(inputEvent);
                        
                        // Change event
                        const changeEvent = new Event('change', {{ bubbles: true }});
                        el.dispatchEvent(changeEvent);
                        
                        return 'typed';
                    }})()
                    "#
                );

                let result = ctx.cdp_client.evaluate(&script).await?;

                if result.contains("not found") {{
                    anyhow::bail!("Element '{selector}' not found");
                }}

                println!("✓ Typed text into {selector}");
                Ok(())
            }
            None => {
                anyhow::bail!("Invalid type format. Expected: type \"<text>\" into \"<selector>\"");
            }
        }
    }
}
