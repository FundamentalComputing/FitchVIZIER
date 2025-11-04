use crate::data::*;
use std::collections::HashSet;

/// [Scope] is a type which stores scoping information (like which lines can reference which
/// lines).
///
/// It is a `Vec<(Vec<usize>,Vec<(usize,usize)>)>`),
/// such that:
/// ```notrust
/// for all i in <line numbers corresponding to inferences (not premises) found in proof>:
///   scope[i].0 == <the set of the line numbers which are referenceable by line i>
///     and
///   scope[i].1 == <the set of the subproofs i-j (stored as tuple (i,j))
///                   which are referenceable by line i>
///   ```
///
/// The first index `scope[0]` is unused.
pub type Scope = Vec<(Vec<usize>, Vec<(usize, usize)>)>;

/// A [Proof] is a fundamental entity in this program. It contains important information that can
/// be used to assess whether the proof is correct.
///
/// Note that a [Proof] should always be
/// constructed using the [Proof::construct] function!!
///
/// Note that a [Proof] does not necessarily mean "a fully correct proof". If you want to assess
/// the full correctness of a [Proof], use [Proof::is_fully_correct].
pub struct Proof {
    ///  the unified sequence of proof nodes that encode numbered lines and structural markers.
    pub nodes: Vec<ProofNode>,
    ///  a field that contains the [Scope] of the proof (it contains information which lines may
    /// reference which lines)
    pub scope: Scope,
    ///  a field which contains the set of strings that should be seen as a variable.
    pub allowed_variable_names: HashSet<String>,
}

impl Proof {
    fn normalize_nodes(raw_nodes: Vec<ProofNode>) -> Result<Vec<ProofNode>, String> {
        let mut normalized: Vec<ProofNode> = Vec::with_capacity(raw_nodes.len() * 2);
        let mut prev_depth = 1;
        let mut last_line_num = 0;

        for node in raw_nodes {
            if matches!(node, ProofNode::SubproofOpen { .. } | ProofNode::SubproofClose { .. }) {
                return Err("internal error: proof already contains structural markers".to_string());
            }

            let depth = node.depth();

            if depth == prev_depth + 1 {
                normalized.push(ProofNode::SubproofOpen {
                    depth,
                });
            } else if depth + 1 == prev_depth {
                normalized.push(ProofNode::SubproofClose {
                    depth: prev_depth,
                });
            } else if depth != prev_depth {
                return Err(format!("near line {}, there is an 'indentation/scope jump' that is too big. You cannot open or close two subproofs in the same line.", last_line_num + 1));
            }

            if let ProofNode::Numbered(ref numbered) = node {
                last_line_num = numbered.line_num;
            }

            normalized.push(node);
            prev_depth = depth;
        }

        Ok(normalized)
    }

    /// Given a vector of [ProofNode]s, this method constructs the proof. In case this method fails,
    /// it means a fatal error will need to be given, because if this method already fails then the
    /// proof is not even half-well-structured, and further analysis is impossible. After
    /// [Proof::construct]ing the proof, you should [Proof::is_fully_correct]() it. The combination of these two things
    /// allows you to assess the correctness of a proof.
    pub fn construct(
        raw_nodes: Vec<ProofNode>,
        allowed_variable_names: HashSet<String>,
    ) -> Result<Proof, String> {
        let nodes = Self::normalize_nodes(raw_nodes)?;
        Self::is_half_well_structured(&nodes)?;
        let scope = Self::determine_scope(&nodes);

        Ok(Proof { nodes, scope, allowed_variable_names })
    }

    pub fn nodes(&self) -> &[ProofNode] {
        &self.nodes
    }

    pub fn numbered_lines(&self) -> impl Iterator<Item = &NumberedLine> {
        self.nodes.iter().filter_map(|node| node.as_numbered())
    }

    pub fn find_numbered_line(&self, line_num: usize) -> Option<&NumberedLine> {
        self.nodes.iter().find_map(|node| match node {
            ProofNode::Numbered(line) if line.line_num == line_num => Some(line),
            _ => None,
        })
    }

    pub fn last_numbered_line(&self) -> Option<&NumberedLine> {
        self.nodes.iter().rev().find_map(|node| node.as_numbered())
    }

    /// This function computes the [Scope] of a proof.
    /// This function computes the [Scope] of a proof.
    fn determine_scope(nodes: &[ProofNode]) -> Scope {
        let last_line_number: usize = nodes
            .iter()
            .filter_map(|node| match node {
                ProofNode::Numbered(line) => Some(line.line_num),
                _ => None,
            })
            .last()
            .expect("Couldn't find the last numberd lined in a proof - panic");
        let mut scope: Scope = vec![(vec![], vec![]); last_line_number + 1];

        for (idx, node) in nodes.iter().enumerate() {
            let Some(line) = node.as_numbered() else {
                continue;
            };
            if !line.is_inference() {
                continue;
            }
            let line_num = line.line_num;
            let mut depth: i32 = 0;
            let mut stack: Vec<usize> = vec![];

            for j in (0..idx).rev() {
                match &nodes[j] {
                    ProofNode::SubproofOpen {
                        ..
                    } => {
                        if depth > 0 {
                            depth -= 1;
                            let subproof_begin = nodes[j + 1..]
                                .iter()
                                .find_map(|node| node.as_numbered())
                                .map(|premise| premise.line_num)
                                .expect("This really should not happen. This is a mistake by the developer. Please contact me if you get this.");
                            let subproof_end = stack.pop().expect("This is a mistake by the developer. Please contact me if you get this.");
                            if stack.is_empty() {
                                scope[line_num].1.push((subproof_begin, subproof_end));
                            }
                        }
                    }
                    ProofNode::SubproofClose {
                        ..
                    } => {
                        depth += 1;
                        let subproof_end = nodes[..j]
                            .iter()
                            .rev()
                            .find_map(|node| node.as_numbered())
                            .map(|last_line| last_line.line_num)
                            .expect("This really should not happen. This is a mistake by the developer. Please contact me if you get this.");
                        stack.push(subproof_end);
                    }
                    ProofNode::Numbered(prev_line) => {
                        if depth == 0 {
                            scope[line_num].0.push(prev_line.line_num);
                        }
                    }
                    _ => {}
                }
            }
        }

        scope
    }

    /// This function checks if a proof is HALF-well-structured.
    /// The reason that we make this distinction is because the validator algorithm does this:
    /// - (1) parse proof
    /// - (2) check that proof is HALF-well-structured
    /// - (3) check all lines of the proof
    /// - (4) check that proof is fully correct
    ///
    /// The point is that we want to give the user as helpful error messages as possible. We also
    /// want to be able to give the user several meaningful messages at the same time. But if a
    /// proof is not even half-well-structured, then it is not even possible to check all lines of
    /// it, so a FATAL error will be given, in which case all the other analysis does not happen.
    /// If the user has a HALF-structured (but not fully correct) proof, then it is still possible to
    /// perform the more detailed analysis in step 3, so we want that. In this case, the user will
    /// get meaningful error messages from all proof lines that are wrong, and that is better than
    /// only a fatal error when they for example just forget one justification somewhere.
    /// A notable allowed thing in half-well-structured proofs is having premises after the Fitch
    /// bar. Of course, this is not allowed in a fully correct proof, but here it means that we
    /// basically allow the user to not write a justification for the time being. In that case it
    /// will be parsed as a premise, so that's why we allow premises. This function won't complain
    /// about it, but of course, this will be checked when the proof is assessed for full correctness.
    fn is_half_well_structured(nodes: &[ProofNode]) -> Result<(), String> {
        // traverse the structural nodes to check validity of the proof
        // basically, for each node, we check that the nodes after that are allowed.

        // Helper function for grabbing the next non-empty line
        fn next_meaningful(nodes: &[ProofNode], mut idx: usize) -> Option<usize> {
            while idx < nodes.len() {
                if !matches!(nodes[idx], ProofNode::Empty { .. }) {
                    return Some(idx);
                }
                idx += 1;
            }
            None
        }

        // if all the lines are empty then the proof is empty
        let Some(first_idx) = next_meaningful(nodes, 0) else {
            return Err("Your proof appears to be empty.".to_string());
        };

        // a proof can start with a fitch bar or with a numbered premise
        match &nodes[first_idx] {
            ProofNode::FitchBar {
                ..
            } => {}
            ProofNode::Numbered(line) if !line.is_inference() => {}
            _ => return Err(
                "Error: proof should start with premises (or Fitch bar, if there are no premises)."
                    .to_string(),
            ),
        }

        for i in 0..nodes.len() {
            match &nodes[i] {
                ProofNode::Empty {
                    ..
                } => {}

                // in HALF-well-structured proofs, a Fitch bar line may be succeeded by:
                //  - an inference
                //  - a premise without boxed constant (inference for which the user didn't write justification yet)
                //  - a new subproof
                //    and a proof must NOT end with a Fitch bar line       
                ProofNode::FitchBar {
                    ..
                } => {
                    let Some(next_idx) = next_meaningful(nodes, i + 1) else {
                        return Err("The proof ends with a Fitch bar.".to_string());
                    };
                    match &nodes[next_idx] {
                        ProofNode::Numbered(line) if line.is_inference() => {}
                        ProofNode::SubproofOpen {
                            ..
                        } => {}
                        ProofNode::Numbered(line)
                            if !line.is_inference() && !line.introduces_boxed_constant() => {}
                        _ => {
                            return Err("Error: Fitch bars should be followed by either a new subproof or an inference. You might be missing a justification.".to_string());
                        }
                    }
                }

                // in HALF-well-tructure proofs, after a subproof is opened, there must be:
                //  - EXACTLY one numbered premise, FOLLOWED by a Fitch bar
                ProofNode::SubproofOpen {
                    ..
                } => {
                    let Some(prem_idx) = next_meaningful(nodes, i + 1) else {
                        return Err("Error: this proof ends with an opened subproof in a way that should not be.".to_string());
                    };
                    match &nodes[prem_idx] {
                        ProofNode::Numbered(line) if !line.is_inference() => {}
                        _ => {
                            return Err(
                                "Error: the first line on any new subproof should be a premise."
                                    .to_string(),
                            );
                        }
                    }
                    let Some(bar_idx) = next_meaningful(nodes, prem_idx + 1) else {
                        return Err("Error: this proof ends with an opened subproof in a way that should not be.".to_string());
                    };
                    match &nodes[bar_idx] {
                        ProofNode::FitchBar {
                            ..
                        } => {}
                        _ => {
                            return Err("Error: a subproof should have exactly one premise, followed by a Fitch bar.".to_string());
                        }
                    }
                }

                // in HALF-well-structured proofs, after a closed subproof there should be either:
                //  - an inference
                //  - a premise without boxed constant (inference for which user didn't write justification yet)
                //  - a new subproof
                ProofNode::SubproofClose {
                    ..
                } => {
                    if let Some(next_idx) = next_meaningful(nodes, i + 1) {
                        match &nodes[next_idx] {
                            ProofNode::Numbered(line) if line.is_inference() => {}
                            ProofNode::SubproofOpen {
                                ..
                            } => {}
                            ProofNode::Numbered(line)
                                if !line.is_inference() && !line.introduces_boxed_constant() => {}
                            _ => {
                                return Err("Error: after closing a subproof, either you should open a new subproof or there should be an inference. Maybe you are missing some justification.".to_string());
                            }
                        }
                    }
                }


                   // in HALF-well-structured proofs, after an inference there should be either:
                   //  - the end of the subproof
                   //  - the opening of a new subproof
                   //  - another inference
                   //  - a premise without boxed constant (i.e. in this case an inference without justification)
                ProofNode::Numbered(line)
                    if line.is_inference() =>
                {
                    if let Some(next_idx) = next_meaningful(nodes, i + 1) {
                        match &nodes[next_idx] {
                            ProofNode::Numbered(next_line) if next_line.is_inference() => {}
                            ProofNode::SubproofOpen {
                                ..
                            } => {}
                            ProofNode::SubproofClose {
                                ..
                            } => {}
                            ProofNode::Numbered(next_line)
                                if !next_line.is_inference()
                                    && !next_line.introduces_boxed_constant() => {}
                            ProofNode::FitchBar {
                                ..
                            } => {
                                return Err("Error: you cannot have a Fitch bar after an inference. Maybe you are giving justification for a premise?".to_string());
                            }
                            ProofNode::Numbered(next_line)
                                if !next_line.is_inference()
                                    && next_line.introduces_boxed_constant() =>
                            {
                                return Err("Error: a boxed constant can only be introduced in the premise of a subproof".to_owned());
                            }
                            _ => {}
                        }
                    }
                }

               // in HALF-well-structured proofs, after a premise w/out b.c. there should be either:
               //  - a Fitch bar line
               //  - another premise without boxed constant
               //       (only at the beginning of the proof, but we already check for
                //        that in the StructuralNode::SubproofOpen arm of this match expression)
                //  - an inference
                //  - a SubproofOpen
                //  - a SubproofClose
                //    and a proof MAY end directly after a premise without b.c.
                ProofNode::Numbered(line)
                    if !line.is_inference() && !line.introduces_boxed_constant() =>
                {
                    if let Some(next_idx) = next_meaningful(nodes, i + 1) {
                        match &nodes[next_idx] {
                            ProofNode::FitchBar {
                                ..
                            } => {}
                            ProofNode::Numbered(next_line)
                                if !next_line.is_inference()
                                    && !next_line.introduces_boxed_constant() => {}
                            ProofNode::Numbered(next_line) if next_line.is_inference() => {}
                            ProofNode::SubproofOpen {
                                ..
                            }
                            | ProofNode::SubproofClose {
                                ..
                            } => {}
                            ProofNode::Numbered(next_line)
                                if !next_line.is_inference()
                                    && next_line.introduces_boxed_constant() =>
                            {
                                return Err("Error: a boxed constant can only be introduced in the premise of a subproof".to_owned());
                            }
                            _ => {}
                        }
                    }
                }
                
                // in HALF-well-structured proofs, after a premise with b.c. there must be:
                //  - a Fitch bar line
                //    and a proof MUST NOT end directly after a premise with b.c.
                ProofNode::Numbered(line)
                    if line.introduces_boxed_constant() =>
                {
                    let Some(next_idx) = next_meaningful(nodes, i + 1) else {
                        return Err("Error: a proof cannot end with a premise.".to_owned());
                    };
                    match &nodes[next_idx] {
                        ProofNode::FitchBar {
                            ..
                        } => {}
                        _ => {
                            return Err(
                                "Error: after a premise, there should be a Fitch bar".to_owned()
                            );
                        }
                    }
                }
                ProofNode::Numbered(_) => {}
            }
        }

        // last but not least: check that the line numbers are correct...
        // they must start at 1 and increase in steps of 1
        let mut prev_num: usize = 0;
        for node in nodes.iter() {
            if let ProofNode::Numbered(line) = node {
                if line.line_num != prev_num + 1 {
                    return Err(format!(
                        "Line numbers are wrong; discrepancy between line {prev_num} and {num}...",
                        num = line.line_num
                    ));
                }
                prev_num = line.line_num;
            }
        }

        Ok(()) // nice, proof is HALF-well-structured. we can now perform further analysis without
               // yielding a fatal error.
    }
}
