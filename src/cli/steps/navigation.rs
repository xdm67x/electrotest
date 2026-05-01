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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::feature::Keyword;

    #[test]
    fn test_window_size_can_handle_given() {
        let handler = WindowSizeStep;
        let step = Step {
            keyword: Keyword::Given,
            text: "the window size is 1920x1080".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_window_size_can_handle_and() {
        let handler = WindowSizeStep;
        let step = Step {
            keyword: Keyword::And,
            text: "the window size is 800x600".to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_window_size_cannot_handle_when() {
        let handler = WindowSizeStep;
        let step = Step {
            keyword: Keyword::When,
            text: "the window size is 1920x1080".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_window_size_cannot_handle_unrelated_text() {
        let handler = WindowSizeStep;
        let step = Step {
            keyword: Keyword::Given,
            text: "the window is open".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_navigate_can_handle_navigate_to() {
        let handler = NavigateStep;
        let step = Step {
            keyword: Keyword::When,
            text: r#"I navigate to "https://example.com""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_navigate_can_handle_go_to() {
        let handler = NavigateStep;
        let step = Step {
            keyword: Keyword::When,
            text: r#"I go to "https://example.com""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_navigate_can_handle_and() {
        let handler = NavigateStep;
        let step = Step {
            keyword: Keyword::And,
            text: r#"I navigate to "https://example.com""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_navigate_cannot_handle_given() {
        let handler = NavigateStep;
        let step = Step {
            keyword: Keyword::Given,
            text: r#"I navigate to "https://example.com""#.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_navigate_cannot_handle_unrelated_text() {
        let handler = NavigateStep;
        let step = Step {
            keyword: Keyword::When,
            text: "I navigate the file system".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }
}
