use crate::cli::context::Context;
use crate::cli::feature::Step;
use crate::cli::steps::StepHandler;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::sync::LazyLock;
use tokio::time::{Duration, sleep};

/// Regex to match click step patterns like "click on "Submit""
static BUTTON_CLICK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"click on "([^"]+)""#).unwrap());

/// Handler for: "I click on ..."
pub struct ClickStep;

#[async_trait]
impl StepHandler for ClickStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("click")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        // Use a local variable to avoid borrowing the static LazyLock directly
        let click_regex = &*BUTTON_CLICK_REGEX;
        match click_regex.captures(&step.text) {
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

/// Regex to match type step patterns like "type "hello" into "input""
static TYPE_TEXT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"type "([^"]+)" into "([^"]+)""#).unwrap());

/// Handler for: "I type ... into ..."
pub struct TypeTextStep;

#[async_trait]
impl StepHandler for TypeTextStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("type") && step.text.contains("into")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        // Use a local variable to avoid borrowing the static LazyLock directly
        let type_regex = &*TYPE_TEXT_REGEX;
        match type_regex.captures(&step.text) {
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

                if result.contains("not found") {
                    {
                        anyhow::bail!("Element '{selector}' not found");
                    }
                }

                println!("✓ Typed text into {selector}");
                Ok(())
            }
            None => {
                anyhow::bail!("Invalid type format. Expected: type \"<text>\" into \"<selector>\"");
            }
        }
    }
}

/// Handler for: "I press "<KEY>"" / "I press "<KEY>" on "<SELECTOR>""
pub struct PressKeyStep;

static PRESS_KEY_ON_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"press "([^"]+)" on "([^"]+)""#).unwrap());
static PRESS_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"press "([^"]+)""#).unwrap());

#[async_trait]
impl StepHandler for PressKeyStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("press")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        if let Some(caps) = PRESS_KEY_ON_REGEX.captures(&step.text) {
            let key = &caps[1];
            let selector = &caps[2];
            let script = format!(
                r#"
                (function() {{
                    let el = document.querySelector({selector:?});
                    if (!el) return 'element not found';
                    el.focus();
                    const keyEvent = new KeyboardEvent('keydown', {{ key: {key:?}, bubbles: true }});
                    el.dispatchEvent(keyEvent);
                    const keyUpEvent = new KeyboardEvent('keyup', {{ key: {key:?}, bubbles: true }});
                    el.dispatchEvent(keyUpEvent);
                    return 'pressed';
                }})()
                "#
            );
            let result = ctx.cdp_client.evaluate(&script).await?;
            if result.contains("not found") {
                anyhow::bail!("Element '{selector}' not found");
            }
            println!("✓ Pressed {key} on {selector}");
            Ok(())
        } else if let Some(caps) = PRESS_KEY_REGEX.captures(&step.text) {
            let key = &caps[1];
            let script = format!(
                r#"
                (function() {{
                    const target = document.activeElement || document.body;
                    const opts = {{ key: {key:?}, code: {key:?}, bubbles: true, cancelable: true }};
                    target.dispatchEvent(new KeyboardEvent('keydown', opts));
                    target.dispatchEvent(new KeyboardEvent('keyup', opts));
                    return 'pressed';
                }})()
                "#
            );
            ctx.cdp_client.evaluate(&script).await?;
            println!("✓ Pressed {key}");
            Ok(())
        } else {
            anyhow::bail!(
                "Invalid press format. Expected: press \"<key>\" or press \"<key>\" on \"<selector>\""
            )
        }
    }
}

/// Handler for: "I double-click on "<SELECTOR>""
pub struct DoubleClickStep;

#[async_trait]
impl StepHandler for DoubleClickStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("double-click")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"double-click on "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid double-click format"))?;
        let selector = &caps[1];
        let script = format!(
            r#"
            (function() {{
                let el = document.querySelector({selector:?});
                if (!el) return 'element not found';
                el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                const rect = el.getBoundingClientRect();
                const opts = {{
                    bubbles: true, cancelable: true, view: window,
                    clientX: rect.left + rect.width / 2,
                    clientY: rect.top + rect.height / 2
                }};
                el.dispatchEvent(new MouseEvent('dblclick', opts));
                return 'clicked';
            }})()
            "#
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("not found") {
            anyhow::bail!("Element '{selector}' not found");
        }
        println!("✓ Double-clicked on {selector}");
        Ok(())
    }
}

/// Handler for: "I hover over "<SELECTOR>""
pub struct HoverStep;

#[async_trait]
impl StepHandler for HoverStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type()
            && (step.text.contains("hover over") || step.text.contains("hover on"))
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"hover (?:over|on) "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid hover format"))?;
        let selector = &caps[1];
        let script = format!(
            r#"
            (function() {{
                let el = document.querySelector({selector:?});
                if (!el) return 'element not found';
                el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                const rect = el.getBoundingClientRect();
                const opts = {{
                    bubbles: true, cancelable: true, view: window,
                    clientX: rect.left + rect.width / 2,
                    clientY: rect.top + rect.height / 2
                }};
                el.dispatchEvent(new MouseEvent('mouseover', opts));
                el.dispatchEvent(new MouseEvent('mouseenter', {{ ...opts, bubbles: false }}));
                el.dispatchEvent(new MouseEvent('mousemove', opts));
                return 'hovered';
            }})()
            "#
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("not found") {
            anyhow::bail!("Element '{selector}' not found");
        }
        println!("✓ Hovered over {selector}");
        Ok(())
    }
}

/// Handler for: "I clear "<SELECTOR>""
pub struct ClearInputStep;

#[async_trait]
impl StepHandler for ClearInputStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("clear")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"clear "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid clear format"))?;
        let selector = &caps[1];
        let script = format!(
            r#"
            (function() {{
                const el = document.querySelector({selector:?});
                if (!el) return 'element not found';
                el.focus();
                if ('value' in el) {{
                    el.value = '';
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                return 'cleared';
            }})()
            "#
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("not found") {
            anyhow::bail!("Element '{selector}' not found");
        }
        println!("✓ Cleared {selector}");
        Ok(())
    }
}

/// Handler for: "I select "<VALUE>" from "<SELECTOR>""
pub struct SelectOptionStep;

#[async_trait]
impl StepHandler for SelectOptionStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("select ")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"select "([^"]+)" from "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid select format"))?;
        let value = &caps[1];
        let selector = &caps[2];
        let script = format!(
            r#"
            (function() {{
                const el = document.querySelector({selector:?});
                if (!el) return 'element not found';
                if (el.tagName !== 'SELECT') return 'not a select';
                const option = Array.from(el.options).find(
                    o => o.value === {value:?} || o.text === {value:?}
                );
                if (!option) return 'option not found';
                el.value = option.value;
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return 'selected';
            }})()
            "#
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("element not found") {
            anyhow::bail!("Element '{selector}' not found");
        }
        if result.contains("not a select") {
            anyhow::bail!("Element '{selector}' is not a <select>");
        }
        if result.contains("option not found") {
            anyhow::bail!("Option '{value}' not found in '{selector}'");
        }
        println!("✓ Selected {value} from {selector}");
        Ok(())
    }
}

/// Handler for: "I scroll ..."
pub struct ScrollStep;

static SCROLL_TO_ELEMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"scroll to "([^"]+)""#).unwrap());
static SCROLL_BY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"scroll (?:down|up) by (\d+)(?: pixels| px)?").unwrap());

#[async_trait]
impl StepHandler for ScrollStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("scroll")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        if let Some(caps) = SCROLL_TO_ELEMENT_REGEX.captures(&step.text) {
            let selector = &caps[1];
            let script = format!(
                r#"
                (function() {{
                    const el = document.querySelector({selector:?});
                    if (!el) return 'element not found';
                    el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                    return 'scrolled';
                }})()
                "#
            );
            let result = ctx.cdp_client.evaluate(&script).await?;
            if result.contains("not found") {
                anyhow::bail!("Element '{selector}' not found");
            }
            println!("✓ Scrolled to {selector}");
            return Ok(());
        }
        if let Some(caps) = SCROLL_BY_REGEX.captures(&step.text) {
            let pixels: i64 = caps[1].parse()?;
            let direction = if step.text.contains("scroll up") {
                "-"
            } else {
                ""
            };
            let script = format!("window.scrollBy(0, {direction}{pixels}); 'scrolled'");
            ctx.cdp_client.evaluate(&script).await?;
            println!(
                "✓ Scrolled {} by {pixels} pixels",
                if direction.is_empty() { "down" } else { "up" }
            );
            return Ok(());
        }
        if step.text.contains("scroll to top") {
            ctx.cdp_client
                .evaluate("window.scrollTo(0, 0); 'scrolled'")
                .await?;
            println!("✓ Scrolled to top");
            return Ok(());
        }
        if step.text.contains("scroll to bottom") {
            ctx.cdp_client
                .evaluate("window.scrollTo(0, document.body.scrollHeight); 'scrolled'")
                .await?;
            println!("✓ Scrolled to bottom");
            return Ok(());
        }
        anyhow::bail!(
            "Invalid scroll format. Expected: scroll down/up by <N> pixels, scroll to \"<selector>\", scroll to top, or scroll to bottom"
        )
    }
}

/// Handler for: "I wait for "<SELECTOR>""
pub struct WaitForElementStep;

#[async_trait]
impl StepHandler for WaitForElementStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("wait for")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"wait for "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid wait for format"))?;
        let selector = &caps[1];
        let script = format!(
            r#"
            (function() {{
                return new Promise((resolve) => {{
                    const timeout = setTimeout(() => resolve('timeout'), 10000);
                    const check = () => {{
                        if (document.querySelector({selector:?})) {{
                            clearTimeout(timeout);
                            resolve('found');
                        }}
                    }};
                    check();
                    const observer = new MutationObserver(check);
                    observer.observe(document.documentElement, {{
                        childList: true, subtree: true
                    }});
                }});
            }})()
            "#
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("timeout") {
            anyhow::bail!("Timed out waiting for element '{selector}'");
        }
        println!("✓ Found element {selector}");
        Ok(())
    }
}

/// Handler for: "I reload the page"
pub struct ReloadStep;

#[async_trait]
impl StepHandler for ReloadStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("reload")
    }

    async fn execute(&self, _step: &Step, ctx: &mut Context) -> Result<()> {
        ctx.cdp_client.reload().await?;
        println!("✓ Reloaded the page");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::feature::Keyword;

    #[test]
    fn test_click_can_handle_when() {
        let handler = ClickStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I click on "Submit""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_click_can_handle_and() {
        let handler = ClickStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"I click on "#submit-btn""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_click_cannot_handle_given() {
        let handler = ClickStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I click on "Submit""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_click_cannot_handle_unrelated_text() {
        let handler = ClickStep;
        let step = Step {
            keyword: Keyword::When,
            text: "I press the button".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_screenshot_can_handle_when() {
        let handler = ScreenshotStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I take a screenshot "test.png""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_screenshot_can_handle_and() {
        let handler = ScreenshotStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"I take a screenshot "output.png""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_screenshot_cannot_handle_given() {
        let handler = ScreenshotStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I take a screenshot "test.png""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_wait_can_handle_when() {
        let handler = WaitStep;
        let step = Step {
            keyword: Keyword::When,
            text: "I wait 2 seconds".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_wait_can_handle_and() {
        let handler = WaitStep;
        let step = Step {
            keyword: Keyword::And,
            text: "I wait 1.5 seconds".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_wait_can_handle_singular_second() {
        let handler = WaitStep;
        let step = Step {
            keyword: Keyword::And,
            text: "I wait 1 second".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_wait_cannot_handle_given() {
        let handler = WaitStep;
        let step = Step {
            keyword: Keyword::Given,
            text: "I wait 2 seconds".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_type_text_can_handle_when() {
        let handler = TypeTextStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I type "hello" into "input""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_type_text_can_handle_and() {
        let handler = TypeTextStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"I type "test message" into "#message-input""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_type_text_cannot_handle_given() {
        let handler = TypeTextStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I type "hello" into "input""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_press_key_can_handle_when() {
        let handler = PressKeyStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I press "Enter""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_press_key_on_element_can_handle_when() {
        let handler = PressKeyStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I press "Enter" on "#search-input""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_press_key_cannot_handle_given() {
        let handler = PressKeyStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I press "Enter""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_double_click_can_handle_when() {
        let handler = DoubleClickStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I double-click on "#item""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_double_click_cannot_handle_given() {
        let handler = DoubleClickStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I double-click on "#item""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_hover_can_handle_when() {
        let handler = HoverStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I hover over "#menu""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_hover_cannot_handle_given() {
        let handler = HoverStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I hover over "#menu""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_clear_input_can_handle_when() {
        let handler = ClearInputStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I clear "#name-input""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_select_option_can_handle_when() {
        let handler = SelectOptionStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I select "fr" from "#language""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_select_option_cannot_handle_given() {
        let handler = SelectOptionStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I select "fr" from "#language""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_scroll_by_pixels_can_handle_when() {
        let handler = ScrollStep;
        let step = Step {
            keyword: Keyword::When,
            text: "I scroll down by 500 pixels".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_scroll_to_element_can_handle_when() {
        let handler = ScrollStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I scroll to "#footer""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_scroll_to_top_can_handle_when() {
        let handler = ScrollStep;
        let step = Step {
            keyword: Keyword::When,
            text: "I scroll to top".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_scroll_to_bottom_can_handle_and() {
        let handler = ScrollStep;
        let step = Step {
            keyword: Keyword::And,
            text: "I scroll to bottom".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_scroll_cannot_handle_given() {
        let handler = ScrollStep;
        let step = Step {
            keyword: Keyword::Given,
            text: "I scroll down by 100 pixels".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_wait_for_element_can_handle_when() {
        let handler = WaitForElementStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I wait for "#result""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_wait_for_element_cannot_handle_given() {
        let handler = WaitForElementStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r##"I wait for "#result""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_reload_can_handle_when() {
        let handler = ReloadStep;
        let step = Step {
            keyword: Keyword::When,
            text: "I reload the page".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_type_text_cannot_handle_missing_into() {
        let handler = TypeTextStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"I type "hello""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }
}
