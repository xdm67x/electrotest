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
            return Err(anyhow::anyhow!("Element '{}' is not visible", selector));
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

/// Handler for: "the element "<SELECTOR>" should not be visible"
pub struct ElementNotVisibleStep;

#[async_trait]
impl StepHandler for ElementNotVisibleStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type()
            && step.text.contains("element")
            && step.text.contains("not be visible")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"element "([^"]+)" should not be visible"#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid element not visible format"))?;
        let selector = &caps[1];
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
        if result.contains("visible") && !result.contains("not found") {
            return Err(anyhow::anyhow!(
                "Element '{}' is visible but should not be",
                selector
            ));
        }
        println!("✓ Element '{}' is not visible", selector);
        Ok(())
    }
}

/// Handler for: "the element "<SELECTOR>" should contain "<TEXT>""
pub struct ElementContainsStep;

#[async_trait]
impl StepHandler for ElementContainsStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type()
            && step.text.contains("element ")
            && step.text.contains("contain")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"element "([^"]+)" should contain "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid element contains format"))?;
        let selector = &caps[1];
        let expected_text = &caps[2];
        let script = format!(
            r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return 'element not found';
                return el.innerText || el.textContent || '';
            }})()
            "#,
            selector.replace('"', "\\\"")
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("element not found") {
            return Err(anyhow::anyhow!("Element '{}' not found", selector));
        }
        if !result.contains(expected_text) {
            return Err(anyhow::anyhow!(
                "Element '{}' does not contain expected text: '{}'",
                selector,
                expected_text
            ));
        }
        println!("✓ Element '{}' contains: {}", selector, expected_text);
        Ok(())
    }
}

/// Handler for: "the value of "<SELECTOR>" should be "<VALUE>""
pub struct InputValueStep;

#[async_trait]
impl StepHandler for InputValueStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type()
            && (step.text.contains("value of") || step.text.contains("input value"))
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"(?:value of|input value) "([^"]+)" should be "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid input value format"))?;
        let selector = &caps[1];
        let expected_value = &caps[2];
        let script = format!(
            r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return 'element not found';
                return el.value !== undefined ? String(el.value) : '';
            }})()
            "#,
            selector.replace('"', "\\\"")
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("element not found") {
            return Err(anyhow::anyhow!("Element '{}' not found", selector));
        }
        if result != expected_value {
            return Err(anyhow::anyhow!(
                "Value mismatch for '{}': expected '{}', got '{}'",
                selector,
                expected_value,
                result
            ));
        }
        println!("✓ Value of '{}' is: {}", selector, expected_value);
        Ok(())
    }
}

/// Handler for: "the element "<SELECTOR>" should be enabled/disabled"
pub struct ElementEnabledStep;

#[async_trait]
impl StepHandler for ElementEnabledStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type()
            && step.text.contains("element")
            && (step.text.contains("enabled") || step.text.contains("disabled"))
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"element "([^"]+)" should be (enabled|disabled)"#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid element enabled format"))?;
        let selector = &caps[1];
        let expected = &caps[2];
        let expected_enabled = &caps[2] == "enabled";
        let script = format!(
            r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return 'element not found';
                return el.disabled ? 'disabled' : 'enabled';
            }})()
            "#,
            selector.replace('"', "\\\"")
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        if result.contains("element not found") {
            return Err(anyhow::anyhow!("Element '{}' not found", selector));
        }
        let is_enabled = result == "enabled";
        if is_enabled != expected_enabled {
            return Err(anyhow::anyhow!(
                "Element '{}' should be {} but is {}",
                selector,
                if expected_enabled {
                    "enabled"
                } else {
                    "disabled"
                },
                result
            ));
        }
        println!("✓ Element '{}' is {}", selector, expected);
        Ok(())
    }
}

/// Handler for: "the page should not contain "<TEXT>""
pub struct PageNotContainsStep;

#[async_trait]
impl StepHandler for PageNotContainsStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type() && step.text.contains("page should not contain")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"page should not contain "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid page not contains format"))?;
        let unexpected_text = &caps[1];
        let script = r#"
            document.body.innerText || document.body.textContent || ''
        "#;
        let page_text = ctx.cdp_client.evaluate(script).await?;
        if page_text.contains(unexpected_text) {
            return Err(anyhow::anyhow!(
                "Page contains unexpected text: '{}'",
                unexpected_text
            ));
        }
        println!("✓ Page does not contain: {}", unexpected_text);
        Ok(())
    }
}

/// Handler for: "the element "<SELECTOR>" should exist" / "should not exist"
pub struct ElementExistsStep;

#[async_trait]
impl StepHandler for ElementExistsStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type()
            && step.text.contains("element")
            && (step.text.contains("exist") || step.text.contains("exists"))
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"element "([^"]+)" should( not)? exist"#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid element exists format"))?;
        let selector = &caps[1];
        let should_exist = caps.get(2).is_none();
        let script = format!(
            r#"
            (function() {{
                return document.querySelector('{}') ? 'exists' : 'missing';
            }})()
            "#,
            selector.replace('"', "\\\"")
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        let exists = result == "exists";
        if exists != should_exist {
            return Err(anyhow::anyhow!(
                "Element '{}' {} but should {}",
                selector,
                if exists { "exists" } else { "does not exist" },
                if should_exist { "exist" } else { "not exist" }
            ));
        }
        println!(
            "✓ Element '{}' {}",
            selector,
            if should_exist {
                "exists"
            } else {
                "does not exist"
            }
        );
        Ok(())
    }
}

/// Handler for: "I should see "<N>" elements "<SELECTOR>""
pub struct ElementCountStep;

#[async_trait]
impl StepHandler for ElementCountStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type() && step.text.contains("elements ")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let re = Regex::new(r#"(?:see|have) (\d+) elements "([^"]+)""#).unwrap();
        let caps = re
            .captures(&step.text)
            .ok_or_else(|| anyhow::anyhow!("Invalid element count format"))?;
        let expected_count: usize = caps[1].parse()?;
        let selector = &caps[2];
        let script = format!(
            r#"
            (function() {{
                return String(document.querySelectorAll('{}').length);
            }})()
            "#,
            selector.replace('"', "\\\"")
        );
        let result = ctx.cdp_client.evaluate(&script).await?;
        let actual_count: usize = result
            .parse()
            .map_err(|_| anyhow::anyhow!("Failed to parse element count"))?;
        if actual_count != expected_count {
            return Err(anyhow::anyhow!(
                "Element count mismatch for '{}': expected {}, got {}",
                selector,
                expected_count,
                actual_count
            ));
        }
        println!("✓ Found {expected_count} elements '{selector}'");
        Ok(())
    }
}

/// Handler for: "the URL should be "<URL>"" / "the URL should contain "<TEXT>""
pub struct UrlStep;

#[async_trait]
impl StepHandler for UrlStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_then_type() && (step.text.contains("url") || step.text.contains("URL"))
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        let current_url = ctx.cdp_client.evaluate("window.location.href").await?;
        let equals_re = Regex::new(r#"[Uu][Rr][Ll] should be "([^"]+)""#).unwrap();
        if let Some(caps) = equals_re.captures(&step.text) {
            let expected_url = &caps[1];
            if current_url != expected_url {
                return Err(anyhow::anyhow!(
                    "URL mismatch: expected '{}', got '{}'",
                    expected_url,
                    current_url
                ));
            }
            println!("✓ URL is: {expected_url}");
            return Ok(());
        }
        let contains_re = Regex::new(r#"[Uu][Rr][Ll] should contain "([^"]+)""#).unwrap();
        if let Some(caps) = contains_re.captures(&step.text) {
            let expected = &caps[1];
            if !current_url.contains(expected) {
                return Err(anyhow::anyhow!(
                    "URL '{}' does not contain '{}'",
                    current_url,
                    expected
                ));
            }
            println!("✓ URL contains: {expected}");
            return Ok(());
        }
        Err(anyhow::anyhow!(
            "Invalid URL format. Expected: the URL should be \"<url>\" or the URL should contain \"<text>\""
        ))
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
            text: r##"the page should contain "Hello""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_contains_can_handle_and() {
        let handler = PageContainsStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"the page should contain "World""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_contains_cannot_handle_when() {
        let handler = PageContainsStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the page should contain "Hello""##.to_string(),
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
    fn test_element_not_visible_can_handle_then() {
        let handler = ElementNotVisibleStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#modal" should not be visible"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_not_visible_cannot_handle_when() {
        let handler = ElementNotVisibleStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the element "#modal" should not be visible"##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_element_contains_can_handle_then() {
        let handler = ElementContainsStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#status" should contain "Active""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_input_value_can_handle_then() {
        let handler = InputValueStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the value of "#name-input" should be "Alice""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_input_value_cannot_handle_when() {
        let handler = InputValueStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the value of "#name-input" should be "Alice""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_element_enabled_can_handle_then() {
        let handler = ElementEnabledStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#submit-btn" should be enabled"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_disabled_can_handle_then() {
        let handler = ElementEnabledStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#submit-btn" should be disabled"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_not_contains_can_handle_then() {
        let handler = PageNotContainsStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the page should not contain "Error""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_exists_can_handle_then() {
        let handler = ElementExistsStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#logo" should exist"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_not_exists_can_handle_then() {
        let handler = ElementExistsStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the element "#deleted-item" should not exist"##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_element_exists_cannot_handle_when() {
        let handler = ElementExistsStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the element "#logo" should exist"##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_element_count_can_handle_then() {
        let handler = ElementCountStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"I should see 3 elements ".item""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_url_can_handle_then() {
        let handler = UrlStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the URL should be "https://example.com""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_url_contains_can_handle_and() {
        let handler = UrlStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"the URL should contain "dashboard""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_url_cannot_handle_when() {
        let handler = UrlStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the URL should be "https://example.com""##.to_string(),
        };
        assert!(!handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_can_handle_then() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::Then,
            text: r##"the page title should be "My Page""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_can_handle_and() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::And,
            text: r##"the page title should be "Dashboard""##.to_string(),
        };
        assert!(handler.can_handle(&step));
    }

    #[test]
    fn test_page_title_cannot_handle_when() {
        let handler = PageTitleStep;
        let step = Step {
            keyword: Keyword::When,
            text: r##"the page title should be "My Page""##.to_string(),
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
