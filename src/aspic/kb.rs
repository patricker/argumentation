//! Knowledge base: necessary and ordinary premises.

use super::language::Literal;

/// A premise in the knowledge base.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Premise {
    /// Necessary premise (indefeasible): cannot be attacked.
    Necessary(Literal),
    /// Ordinary premise (defeasible): can be attacked by undermining.
    Ordinary(Literal),
}

impl Premise {
    /// The literal content of this premise.
    pub fn literal(&self) -> &Literal {
        match self {
            Premise::Necessary(l) | Premise::Ordinary(l) => l,
        }
    }

    /// Whether this premise can be attacked.
    pub fn is_defeasible(&self) -> bool {
        matches!(self, Premise::Ordinary(_))
    }
}

/// The knowledge base.
#[derive(Debug, Default, Clone)]
pub struct KnowledgeBase {
    premises: Vec<Premise>,
}

impl KnowledgeBase {
    /// Create an empty knowledge base.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a necessary premise.
    pub fn add_necessary(&mut self, l: Literal) {
        self.premises.push(Premise::Necessary(l));
    }

    /// Add an ordinary premise.
    pub fn add_ordinary(&mut self, l: Literal) {
        self.premises.push(Premise::Ordinary(l));
    }

    /// Iterate over all premises.
    pub fn premises(&self) -> &[Premise] {
        &self.premises
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kb_stores_both_types() {
        let mut kb = KnowledgeBase::new();
        kb.add_necessary(Literal::atom("p"));
        kb.add_ordinary(Literal::atom("q"));
        assert_eq!(kb.premises().len(), 2);
        assert!(!kb.premises()[0].is_defeasible());
        assert!(kb.premises()[1].is_defeasible());
    }
}
