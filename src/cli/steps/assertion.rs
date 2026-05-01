use crate::cli::context::Context;
use crate::cli::feature::Step;
use crate::cli::steps::StepHandler;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;

/// Handler for: "the page should contain ..."
pub struct PageContainsStep;

#[async_trait]
impl StepHandler for PageContainsStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type() && step.text.contains("page should contain")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"page should contain "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid page contains format"))?;

        let expected_text = &caps[1];

        // Get page content via JavaScript
        let script = r#"
            document.body.innerText || document.body.textContent || ''
        "#;

        let page_text = ctx.cdp_client.evaluate(script).await?;

        if !page_text.contains(expected_text) {
            return Err(anyhow::anyhow!(
                "Page does not contain expected text: '{}'",
                expected_text
            ));
        }

        println!("✓ Page contains: {}", expected_text);
        Ok(())
    }
}

/// Handler for: "the element ... should be visible"
pub struct ElementVisibleStep;

#[async_trait]
impl StepHandler for ElementVisibleStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type()
            && step.text.contains("element")
            && step.text.contains("visible")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"element "([^"]+)" should be visible"#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid element visible format"))?;

        let selector = &caps[1];

        // Check visibility via JavaScript
        let script = format!(
            r#"
            (function() {{
                let el = document.querySelector('{}');
                if (!el) return 'not found';
                let rect = el.getBoundingClientRect();
                let isVisible = rect.width > 0 && rect.height > 0 &&
                               el.style.visibility !== 'hidden' &&
                               el.style.display !== 'none';
                return isVisible ? 'visible' : 'hidden';
            }})()
            "#,
            selector.replace('"', "\\\"")
        );

        let result = ctx.cdp_client.evaluate(&script).await?;

        if result.contains("not found") {
            return Err(anyhow::anyhow!("Element '{}' not found", selector));
        }

        if result.contains("hidden") {
            return Err(anyhow::anyhow!(
                "Element '{}' is not visible",
                selector
            ));
        }

        println!("✓ Element '{}' is visible", selector);
        Ok(())
    }
}

/// Handler for: "the page title should be ..."
pub struct PageTitleStep;

#[async_trait]
impl StepHandler for PageTitleStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type() && step.text.contains("page title")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"page title should be "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid page title format"))?;

        let expected_title = &caps[1];

        let actual_title = ctx.cdp_client.get_title().await?;

        if actual_title != expected_title {
            return Err(anyhow::anyhow!(
                "Page title mismatch: expected '{}', got '{}'",
                expected_title,
                actual_title
            ));
        }

        println!("✓ Page title is: {}", expected_title);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::feature::Keyword;

    #[test]
    fn test_page_contains_can_handle_then() {
        let handler = PageContainsStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r#"the page should contain "Hello""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_contains_can_handle_and() {
        let handler = PageContainsStep;
        let step = Step {
            keyword: Keyword::And,
            text: r#"the page should contain "World""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_contains_cannot_handle_when() {
        let handler = PageContainsStep;
        let step = Step {
            keyword: Keyword::When,
            text: r#"the page should contain "Hello""#.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_element_visible_can_handle_then() {
        let handler = ElementVisibleStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#header" should be visible"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_visible_can_handle_and() {
        let handler = ElementVisibleStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"the element "#footer" should be visible"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_visible_cannot_handle_when() {
        let handler = ElementVisibleStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the element "#header" should be visible"##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_element_visible_cannot_handle_missing_visible() {
        let handler = ElementVisibleStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#header" should exist"##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_can_handle_then() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r#"the page title should be "My Page""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_can_handle_and() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::And,
            text: r#"the page title should be "Dashboard""#.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_cannot_handle_when() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::When,
            text: r#"the page title should be "My Page""#.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_cannot_handle_unrelated_text() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::Then,
            text: "the page should have a title".to_string(),
        };
        assert!(!handler.can_handle(&step));
    }
}
