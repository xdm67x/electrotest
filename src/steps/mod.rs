mod builtin;
mod registry;
mod selector;

pub use registry::{Registry, ResolvedStep};
pub use selector::{normalize_target, Locator, StepTarget, WindowTarget};
