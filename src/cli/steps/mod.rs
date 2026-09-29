use crate::cli::context::Context;
use crate::cli::feature::Step;
use anyhow::Result;
use async_trait::async_trait;

pub mod assertion;
pub mod interaction;
pub mod navigation;

/// Trait for handling Gherkin steps
/// Implement this trait to add new step handlers
#[async_trait]
pub trait StepHandler: Send + Sync {
    /// Check if this handler can handle the given step
    fn can_handle(&self, step: &Step) -> bool;

    /// Execute the step
    /// Returns Ok(()) on success, Err on failure
    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()>;
}

/// Registry of all step handlers
pub struct StepRegistry {
    handlers: Vec<Box<dyn StepHandler>>,
}

impl StepRegistry {
    /// Create a new registry with all default handlers registered
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: Vec::new(),
        };

        // Register all default handlers
        // To add a new handler, create a struct implementing StepHandler
        // and register it here
        registry.register(Box::new(navigation::WindowSizeStep));
        registry.register(Box::new(navigation::NavigateStep));
        registry.register(Box::new(interaction::ClickStep));
        registry.register(Box::new(interaction::ScreenshotStep));
        registry.register(Box::new(interaction::WaitStep));
        registry.register(Box::new(interaction::TypeTextStep));
        registry.register(Box::new(interaction::PressKeyStep));
        registry.register(Box::new(interaction::DoubleClickStep));
        registry.register(Box::new(interaction::HoverStep));
        registry.register(Box::new(interaction::ClearInputStep));
        registry.register(Box::new(interaction::SelectOptionStep));
        registry.register(Box::new(interaction::ScrollStep));
        registry.register(Box::new(interaction::WaitForElementStep));
        registry.register(Box::new(interaction::ReloadStep));
        registry.register(Box::new(assertion::PageContainsStep));
        registry.register(Box::new(assertion::PageNotContainsStep));
        registry.register(Box::new(assertion::ElementExistsStep));
        registry.register(Box::new(assertion::ElementContainsStep));
        registry.register(Box::new(assertion::ElementCountStep));
        registry.register(Box::new(assertion::InputValueStep));
        registry.register(Box::new(assertion::ElementEnabledStep));
        registry.register(Box::new(assertion::UrlStep));
        registry.register(Box::new(assertion::ElementNotVisibleStep));
        registry.register(Box::new(assertion::ElementVisibleStep));
        registry.register(Box::new(assertion::PageTitleStep));

        registry
    }

    /// Register a new handler
    pub fn register(&mut self, handler: Box<dyn StepHandler>) {
        self.handlers.push(handler);
    }

    /// Find a handler that can handle the given step
    pub fn find_handler(&self, step: &Step) -> Option<&dyn StepHandler> {
        self.handlers
            .iter()
            .find(|h| h.can_handle(step))
            .map(|b| b.as_ref())
    }
}

impl Default for StepRegistry {
    fn default() -> Self {
        Self::new()
    }
}
