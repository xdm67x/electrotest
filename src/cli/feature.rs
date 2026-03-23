/// Represents a Gherkin Feature file
#[derive(Debug, Clone)]
pub struct Feature {
    pub name: String,
    pub scenarios: Vec<Scenario>,
}

/// Represents a Scenario within a Feature
#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

/// Represents a single Step in a Scenario
#[derive(Debug, Clone)]
pub struct Step {
    pub keyword: Keyword,
    pub text: String,
}

/// Gherkin step keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Given,
    When,
    Then,
    And,
    But,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Given => write!(f, "Given"),
            Keyword::When => write!(f, "When"),
            Keyword::Then => write!(f, "Then"),
            Keyword::And => write!(f, "And"),
            Keyword::But => write!(f, "But"),
        }
    }
}

impl Keyword {
    /// Parse a keyword from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Given" => Some(Self::Given),
            "When" => Some(Self::When),
            "Then" => Some(Self::Then),
            "And" => Some(Self::And),
            "But" => Some(Self::But),
            _ => None,
        }
    }

    /// Returns true if this is a "Given" type keyword (Given, And, But when following Given)
    pub fn is_given_type(&self) -> bool {
        matches!(self, Keyword::Given | Keyword::And | Keyword::But)
    }

    /// Returns true if this is a "When" type keyword
    pub fn is_when_type(&self) -> bool {
        matches!(self, Keyword::When | Keyword::And | Keyword::But)
    }

    /// Returns true if this is a "Then" type keyword
    pub fn is_then_type(&self) -> bool {
        matches!(self, Keyword::Then | Keyword::And | Keyword::But)
    }
}
