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
    pub sentence: Option<Wff>,
    pub justification: Option<Justification>,
    pub boxed_constant: Option<Term>,
}

impl NumberedLine {
    pub fn introduces_boxed_constant(&self) -> bool {
        self.boxed_constant.is_some()
    }

    pub fn is_inference(&self) -> bool {
        self.justification.is_some()
    }
}

/// This a logical term. A term can be either a constant, a variable, or a function application
/// (which is a function applied to a positive number of terms).
#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum Term {
    /// A variable or constant.
    Atomic(String),
    // Function application
    FuncApp(String, Vec<Term>),
}

#[derive(PartialEq, Debug, Clone)]
/// A logical sentence. "Wff" stands for "well-formed formula", but this is a slightly incorrect
/// name, since for example, a logical sentence that has predicate ariy mismatches is still
/// expressable in this [Wff]. A [Wff] is a core element of a proof. For example, each proof line
/// that has a line number, will contain a [Wff] (unless it is a line which only introduces a boxed
/// constant).
pub enum Wff {
    /// Conjunction.
    And(Vec<Wff>),
    /// Disjunction.
    Or(Vec<Wff>),
    /// Implication.
    Implies(Box<Wff>, Box<Wff>),
    /// Biconditional.
    Bicond(Box<Wff>, Box<Wff>),
    /// Negation.
    Not(Box<Wff>),
    /// Bottom / contradiction.
    Bottom,
    /// Universal quantification.
    ///
    /// The associated [String] denotes the name of the variable that is quantified over, and the
    /// associated [Wff] is the rest of the sentence.
    Forall(String, Box<Wff>),
    /// Existential quantification.
    ///
    /// The associated [String] denotes the name of the variable that is quantified over, and the
    /// associated [Wff] is the rest of the sentence.
    Exists(String, Box<Wff>),
    /// This is a nullary predicate, for example "P".
    Atomic(String),
    /// This is n-ary predicate application, for n >= 1.
    ///
    /// For example, if you have the predicate application `P(x,y,f(a))`, then the associated [String]
    /// would be "P" and the associated vector of [Term]s would correspond to `x`, `y` and `f(a)`,
    /// respectively.
    PredApp(String, Vec<Term>),
    /// The equality predicate, applied to two [Term]s.
    Equals(Term, Term),
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
