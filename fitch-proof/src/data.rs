use crate::loc::{Location, WithLoc};

pub type LProofNode = WithLoc<ProofNode>;
pub type LWff = WithLoc<Wff>;
pub type LTerm = WithLoc<Term>;
pub type LJustification = WithLoc<Justification>;

// Temp code for plugging in type errors
pub fn wrap_nodes_with_dummy_locations(nodes: Vec<ProofNode>) -> Vec<LProofNode> {
    nodes.into_iter().map(WithLoc::dummy).collect()
}

pub fn dummy_lwff(wff: Wff) -> LWff {
    WithLoc::dummy(wff)
}

pub fn dummy_lterm(term: Term) -> LTerm {
    WithLoc::dummy(term)
}

pub fn dummy_ljustification(just: Justification) -> LJustification {
    WithLoc::dummy(just)
}

pub fn boxed_lwff(wff: Wff) -> Box<LWff> {
    Box::new(dummy_lwff(wff))
}

pub fn vec_lwff(wffs: Vec<Wff>) -> Vec<LWff> {
    wffs.into_iter().map(dummy_lwff).collect()
}

pub fn vec_lterm(terms: Vec<Term>) -> Vec<LTerm> {
    terms.into_iter().map(dummy_lterm).collect()
}

/// A `ProofNode` represents every relevant element of a Fitch-style proof in document order.
///
/// Roughly, it correponds to either a physical line in a text-based
/// proof, or it represens opening/closing a new subproof.
#[derive(PartialEq, Debug, Clone)]
pub enum ProofNode {
    /// A numbered line (premise or inference). These are the only nodes that carry a line number.
    Numbered(NumberedLine),
    /// A Fitch bar line (`| ---`) separating premises from a subproof body or the initial derivation.
    FitchBar {
        depth: usize,
    },
    /// An empty line that contains only scope markers (vertical bars). These are rare but allowed.
    Empty {
        depth: usize,
    },
    /// Synthetic element inserted when a new subproof scope is opened. It immediately precedes the
    /// numbered line that serves as the subproof premise.
    SubproofOpen {
        depth: usize,
    },
    /// Synthetic element inserted when one or more subproof scopes close. It precedes the next
    /// textual node at the shallower depth.
    SubproofClose {
        depth: usize,
    },
}

impl ProofNode {
    pub fn depth(&self) -> usize {
        match self {
            ProofNode::Numbered(line) => line.depth,
            ProofNode::FitchBar {
                depth,
            }
            | ProofNode::Empty {
                depth,
            }
            | ProofNode::SubproofOpen {
                depth,
            }
            | ProofNode::SubproofClose {
                depth,
            } => *depth,
        }
    }

    pub fn as_numbered(&self) -> Option<&NumberedLine> {
        if let ProofNode::Numbered(line) = self {
            Some(line)
        } else {
            None
        }
    }

    pub fn is_fitch_bar(&self) -> bool {
        matches!(self, ProofNode::FitchBar { .. })
    }

    pub fn is_structural(&self) -> bool {
        matches!(self, ProofNode::SubproofOpen { .. } | ProofNode::SubproofClose { .. })
    }
}

/// Numbered proof lines carry the logical content of the user's proof.
///
/// A numbered line may be a premise (no justification), an inference
/// (justification present), or a placeholder line where the user has
/// not yet written the justification -- we want to be able to deal
/// with those since we want to provide feedback on imcomplete proofs.
/// Additionaly, when a boxed constant is introduced, it is stored in
/// `boxed_constant`.
#[derive(PartialEq, Debug, Clone)]
pub struct NumberedLine {
    pub line_num: usize,
    pub depth: usize,
    pub sentence: Option<LWff>,
    pub justification: Option<LJustification>,
    pub boxed_constant: Option<LTerm>,
}

#[derive(PartialEq, Debug, Clone)]
/// A logical sentence. "Wff" stands for "well-formed formula", but this is a slightly incorrect
/// name, since for example, a logical sentence that has predicate ariy mismatches is still
/// expressable in this [Wff]. A [Wff] is a core element of a proof. For example, each proof line
/// that has a line number, will contain a [Wff] (unless it is a line which only introduces a boxed
/// constant).
pub enum Wff {
    /// Conjunction.
    And(Vec<LWff>),
    /// Disjunction.
    Or(Vec<LWff>),
    /// Implication.
    Implies(Box<LWff>, Box<LWff>),
    /// Biconditional.
    Bicond(Box<LWff>, Box<LWff>),
    /// Negation.
    Not(Box<LWff>),
    /// Bottom / contradiction.
    Bottom,
    /// Universal quantification.
    ///
    /// The associated [String] denotes the name of the variable that is quantified over, and the
    /// associated [Wff] is the rest of the sentence.
    Forall(String, Box<LWff>),
    /// Existential quantification.
    ///
    /// The associated [String] denotes the name of the variable that is quantified over, and the
    /// associated [Wff] is the rest of the sentence.
    Exists(String, Box<LWff>),
    /// This is a nullary predicate, for example "P".
    Atomic(String),
    /// This is n-ary predicate application, for n >= 1.
    ///
    /// For example, if you have the predicate application `P(x,y,f(a))`, then the associated [String]
    /// would be "P" and the associated vector of [Term]s would correspond to `x`, `y` and `f(a)`,
    /// respectively.
    PredApp(String, Vec<LTerm>),
    /// The equality predicate, applied to two [Term]s.
    Equals(LTerm, LTerm),
}

/// This a logical term. A term can be either a constant, a variable, or a function application
/// (which is a function applied to a positive number of terms).
#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum Term {
    /// A variable or constant.
    Atomic(String),
    // Function application
    FuncApp(String, Vec<LTerm>),
}

/// This enum represents the justification rules for an inference. The associated [usize]s denote
/// the line numbers being represented.
#[derive(PartialEq, Debug, Clone)]
pub enum Justification {
    AndIntro(Vec<usize>),
    AndElim(usize),
    OrIntro(usize),
    OrElim(usize, Vec<(usize, usize)>),
    NotIntro((usize, usize)),
    NotElim(usize),
    BottomIntro(usize, usize),
    BottomElim(usize),
    ImpliesIntro((usize, usize)),
    ImpliesElim(usize, usize),
    BicondIntro((usize, usize), (usize, usize)),
    BicondElim(usize, usize),
    EqualsIntro,
    EqualsElim(usize, usize),
    ForallIntro((usize, usize)),
    ForallElim(usize),
    ExistsIntro(usize),
    ExistsElim(usize, (usize, usize)),
    Reit(usize),
}

impl NumberedLine {
    pub fn introduces_boxed_constant(&self) -> bool {
        self.boxed_constant.is_some()
    }

    pub fn is_inference(&self) -> bool {
        self.justification.is_some()
    }

    pub fn sentence(&self) -> Option<&Wff> {
        self.sentence.as_ref().map(|w| w.value())
    }

    pub fn sentence_with_loc(&self) -> Option<&LWff> {
        self.sentence.as_ref()
    }

    pub fn justification(&self) -> Option<&Justification> {
        self.justification.as_ref().map(|j| j.value())
    }

    pub fn justification_with_loc(&self) -> Option<&LJustification> {
        self.justification.as_ref()
    }

    pub fn boxed_constant(&self) -> Option<&Term> {
        self.boxed_constant.as_ref().map(|t| t.value())
    }

    pub fn boxed_constant_with_loc(&self) -> Option<&LTerm> {
        self.boxed_constant.as_ref()
    }

    pub fn sentence_loc(&self) -> Option<&Location> {
        self.sentence.as_ref().map(|w| w.location())
    }

    pub fn justification_loc(&self) -> Option<&Location> {
        self.justification.as_ref().map(|j| j.location())
    }

    pub fn boxed_constant_loc(&self) -> Option<&Location> {
        self.boxed_constant.as_ref().map(|t| t.location())
    }

    pub fn sentence_owned(&self) -> Option<Wff> {
        self.sentence().cloned()
    }

    pub fn justification_owned(&self) -> Option<Justification> {
        self.justification().cloned()
    }

    pub fn boxed_constant_owned(&self) -> Option<Term> {
        self.boxed_constant().cloned()
    }
}

impl Justification {
    pub fn rule_used(self: &Justification) -> (&'static str, &'static str) {
        match self {
            Justification::AndIntro(_) => ("∧", "Intro"),
            Justification::AndElim(_) => ("∧", "Elim"),
            Justification::OrIntro(_) => ("∨", "Intro"),
            Justification::OrElim(_, _) => ("∨", "Elim"),
            Justification::NotIntro(_) => ("¬", "Intro"),
            Justification::NotElim(_) => ("¬", "Elim"),
            Justification::BottomIntro(_, _) => ("⊥", "Intro"),
            Justification::BottomElim(_) => ("⊥", "Elim"),
            Justification::ImpliesIntro(_) => ("→", "Intro"),
            Justification::ImpliesElim(_, _) => ("→", "Elim"),
            Justification::BicondIntro(_, _) => ("↔", "Intro"),
            Justification::BicondElim(_, _) => ("↔", "Elim"),
            Justification::EqualsIntro => ("=", "Intro"),
            Justification::EqualsElim(_, _) => ("=", "Elim"),
            Justification::ForallIntro(_) => ("∀", "Intro"),
            Justification::ForallElim(_) => ("∀", "Elim"),
            Justification::ExistsIntro(_) => ("∃", "Intro"),
            Justification::ExistsElim(_, _) => ("∃", "Elim"),
            Justification::Reit(_) => ("R", "Reit"),
        }
    }
}

pub enum ProofResult {
    /// No mistakes; proof is correct.
    Correct,
    /// An 'error' is a mistake that makes the proof wrong, but still allows
    /// the checker to go on and find other mistakes. This [ProofResult::Error]
    /// variant denotes the list of errors that was obtained during analysis.
    Error(Vec<String>),
    /// A mistake that is so severe that the checker cannot continue its analysis.
    /// When a fatal error occurs, this fatal error will be returned to the user,
    /// with no other error messages along it.
    ///
    /// Note that when the user checks some proof that should match to some proof template,
    /// a [ProofResult::FatalError] will be returned if the proof does
    /// not match the template.
    FatalError(String),
}
