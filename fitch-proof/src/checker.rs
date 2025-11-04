use crate::data::*;
use crate::formatter;
use crate::proof::*;
use crate::util;
use std::collections::{HashMap, HashSet};
use std::iter::zip;

/// This function checks whether a proof is fully correct. It takes in a vector of [ProofNode]s,
/// which can come straight from the parser (i.e. there are no preconditions about well-formedness
/// of this vector).
///
/// The second argument is the set of strings that should be seen as a variable.
/// For example, if this is the set ["x", "y", "z"], then something like ∀x P(x) will be accepted,
/// but something like ∀a P(a) will not be accepted, because "a" is not listed as a string
/// that should be seen as a variable.
pub fn check_proof(
    proof_nodes: Vec<ProofNode>,
    allowed_variable_names: HashSet<String>,
) -> ProofResult {
    match Proof::construct(proof_nodes, allowed_variable_names) {
        Err(err) => ProofResult::FatalError(err),
        Ok(proof) => proof.is_fully_correct(),
    }
}

/// This function checks whether a proof is fully correct, AND that it matches a given proof
/// template. It takes in a vector of [ProofNode]s,
/// which can come straight from the parser (i.e. there are no preconditions about well-formedness
/// of this vector).
///
/// The second argument is a vector of [Wff]s. These are the proof template that should be matched
/// against. These [Wff]s are, in order, the premises, followed by the conclusion that the proof
/// should lead to.
///
/// The third argument is the set of strings that should be seen as a variable.
/// For example, if this is the set ["x", "y", "z"], then something like ∀x P(x) will be accepted,
/// but something like ∀a P(a) will not be accepted, because "a" is not listed as a string
/// that should be seen as a variable.
pub fn check_proof_with_template(
    proof_nodes: Vec<ProofNode>,
    template: Vec<Wff>,
    allowed_variable_names: HashSet<String>,
) -> ProofResult {
    match Proof::construct(proof_nodes, allowed_variable_names) {
        Err(err) => ProofResult::FatalError(err),
        Ok(proof) => proof.is_fully_correct_and_matches_template(template),
    }
}

/* ------------------ PRIVATE -------------------- */

impl Proof {
    /// Given a [Proof], this function checks if it is fully correct, AND that it matches the given
    /// proof template. A template is a vector of [Wff]s, containing (in order) all the premises,
    /// followed by the final conclusion that the proof should lead to.
    ///
    /// When you want to fully assess the validity of a proof, and
    /// check that it matches the template, you should first
    /// [Proof::construct] the proof, and then run this function.
    fn is_fully_correct_and_matches_template(&self, template: Vec<Wff>) -> ProofResult {
        // Note: don't remove this check on the length of `template`. It would cause some panics
        // below if the length is zero.
        if template.is_empty() {
            return ProofResult::FatalError("The proof template is empty. This should not be! If you see this on Themis as a student, please contact the course staff as soon as possible. Something is wrong on our side. Thanks!".to_owned());
        }

        // template matching errors that we will be accumulating.
        let mut template_errors: Vec<String> = vec![];

        // check premises
        {
            let premises_in_proof: Vec<Wff> = self
                .nodes
                .iter()
                .take_while(|node| !node.is_fitch_bar())
                .filter_map(|node| match node {
                    ProofNode::Numbered(line) => line.sentence.clone(),
                    _ => None,
                })
                .collect();

            // index is within bounds
            if premises_in_proof != template[0..template.len() - 1] {
                template_errors.push(
                    "The premises of your proof do not match the premises in the proof template."
                        .to_owned(),
                );
            }
        }

        // check conclusion
        {
            let conclusion_in_proof = self.nodes.iter().rev().find_map(|node| match node {
                ProofNode::Numbered(line) => line.sentence.as_ref(),
                _ => None,
            });
            match conclusion_in_proof {
                None => {
                    template_errors
                        .push("It seems that your proof has no sentences in it.".to_owned());
                }
                Some(concl) => {
                    // both unwraps work (note that we checked the length of `template`)
                    if concl != template.last().unwrap() {
                        template_errors.push("The conclusion of your proof does not match the conclusion in the proof template.".to_owned());
                    }
                }
            }
        }

        let result_without_template_check = self.is_fully_correct();
        match result_without_template_check {
            // If the proof generates a fatal error by itself, the user is not interested in
            // template matching errors.
            ProofResult::FatalError(_) => result_without_template_check,
            // If there were already errors, just append any template matching errors.
            ProofResult::Error(mut errs) => {
                errs.append(&mut template_errors);
                ProofResult::Error(errs)
            }
            // If there were any template mathing errors, change the Correct into Error. Otherwise
            // it stays Correct.
            ProofResult::Correct => {
                if template_errors.is_empty() {
                    ProofResult::Correct
                } else {
                    ProofResult::Error(template_errors)
                }
            }
        }
    }
    /// Given a [Proof], this function checks if it is fully correct.
    ///
    /// When you want to fully assess the validity of a proof, you should first [Proof::construct] the proof, and then run this function.
    fn is_fully_correct(&self) -> ProofResult {
        let mut errors: Vec<String> = vec![]; // here we accumulate all errors

        // check that user applied proof rule correctly everywhere
        for line in self.numbered_lines() {
            if let Err(err) = self.check_line(line) {
                errors.push(err.to_string());
            }
        }

        // check that proof starts with zero or more premises, followed by a Fitch bar
        let mut seen_fitch_bar = false;
        let mut premises_ok = true;
        for node in self.nodes() {
            match node {
                ProofNode::FitchBar {
                    ..
                } => {
                    seen_fitch_bar = true;
                    break;
                }
                ProofNode::Numbered(line) => {
                    if line.is_inference() || line.introduces_boxed_constant() {
                        premises_ok = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !seen_fitch_bar || !premises_ok {
            errors.push(
                "Each proof should start start with zero or more premises, followed by a Fitch bar"
                    .to_string(),
            );
        }

        // check that all inferences have justification
        errors.extend(
            self.line_numbers_missing_justification()
                .iter()
                .map(|n| format!("Line {n}: missing justification").to_string()),
        );

        // check that all variables are bound, that user doesn't have nested quantifiers over the
        // same variable and that users don't quantify over a constant, and that the user does not make
        // a function with the name of a variable
        errors.extend(self.numbered_lines().filter(|line| line.sentence.is_some()).filter_map(
            |line| {
                self.check_variable_scoping_naming_issues(
                    line.sentence.as_ref().unwrap(),
                    line.line_num,
                )
                .err()
            },
        ));

        // check that user does not use a symbol to denote both a constant and a function, and that
        // arities of function symbols are consistent throughout the proof.
        errors.extend(self.generate_arity_errors());

        // check that user doesn't use boxed constant outside the subproof and that user does not
        // introduce the same boxed constant twice in nested subproofs, and that boxed constants
        // are actually constants (not variables or predicates or ...)
        if let Err(errs) = self.check_boxed_constant_outside_subproof() {
            errors.extend(errs);
        }

        // check that last line is top-level
        if self.last_line_is_inside_subproof() {
            let lln = self.last_line_num();
            errors.push(format!("Line {lln}: last line of proof should not be inside subproof"));
        }

        if errors.is_empty() {
            ProofResult::Correct
        } else {
            util::natural_sort(&mut errors);
            ProofResult::Error(errors)
        }
    }

    /// This function returns a vector containing all line numbers which correspond to "premises"
    /// that are found between a Fitch bar line and a SubproofOpen.
    /// (these would be the inferences with missing justification, but they are parsed as premises)
    fn line_numbers_missing_justification(&self) -> Vec<usize> {
        let mut res = vec![]; // store what we're going to return
        let mut expect_justification = false;
        for node in self.nodes() {
            match node {
                ProofNode::FitchBar {
                    ..
                } => {
                    expect_justification = true;
                }
                ProofNode::SubproofOpen {
                    ..
                } => {
                    expect_justification = false;
                }
                ProofNode::SubproofClose {
                    ..
                }
                | ProofNode::Empty {
                    ..
                } => {}
                ProofNode::Numbered(line) => {
                    if expect_justification && !line.is_inference() {
                        res.push(line.line_num);
                    }
                }
            }
        }

        res
    }

    /// This function returns true if and only if the last line (that has a line number) is inside a subproof
    fn last_line_is_inside_subproof(&self) -> bool {
        self.last_numbered_line().map(|line| line.depth > 1).unwrap_or(false)
    }

    /// This function returns the line number of the last sentence of the proof.
    fn last_line_num(&self) -> usize {
        self.last_numbered_line().map(|line| line.line_num).unwrap()
    }

    /// This function checks that no boxed constants are used outside the subproof. If no boxed
    /// constants are used outside the corresponding subproof, `Ok(())` is returned. Otherwise, a
    /// vector or relevant error messages will be returned, wrapped in an `Err`.
    fn check_boxed_constant_outside_subproof(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = vec![];

        // step 1: check which boxed constants exist within the proof
        let boxed_consts: HashSet<_> =
            self.numbered_lines().filter_map(|line| line.boxed_constant.clone()).collect();

        // step 2: let's also give warnings if the user puts a variable in a box (not a constant)
        errors.extend(
            self.numbered_lines()
                .filter(|line| line.boxed_constant.is_some())
                .filter_map(|line| {
                    if self.term_is_constant(line.boxed_constant.as_ref().unwrap().clone()) {
                        None
                    } else {
                        Some(format!(
                            "Line {}: a boxed constant cannot be a variable (should not have the name of a variable).",
                            line.line_num
                        ))
                    }
                }),
        );

        // step 3: iterate over proof units and see if boxed constants only get used when allowed
        // keep a stack of which boxed constants are in scope:
        let mut currently_in_scope: Vec<Option<Term>> = vec![];
        for node in self.nodes() {
            match node {
                ProofNode::FitchBar {
                    ..
                } => {}
                ProofNode::Numbered(line) if line.introduces_boxed_constant() => {
                    let new_boxed_const = &line.boxed_constant;
                    // before we add the new variable to current scope,
                    // test if it was not already in scope:
                    if currently_in_scope.contains(new_boxed_const) {
                        errors.push(format!("Line {}: you cannot introduce the same boxed constant twice in nested subproofs", line.line_num));
                    }
                    currently_in_scope.push(new_boxed_const.clone());
                    if let Some(wff) = &line.sentence {
                        if let Err(err) = check_wff_not_contain_out_of_scope_boxed_consts(
                            wff,
                            &currently_in_scope,
                            &boxed_consts,
                            line.line_num,
                        ) {
                            errors.push(err);
                        }
                    }
                }
                ProofNode::Numbered(line)
                    if !line.is_inference() && !line.introduces_boxed_constant() =>
                {
                    currently_in_scope.push(None);
                    if let Some(wff) = &line.sentence {
                        if let Err(err) = check_wff_not_contain_out_of_scope_boxed_consts(
                            wff,
                            &currently_in_scope,
                            &boxed_consts,
                            line.line_num,
                        ) {
                            errors.push(err);
                        }
                    }
                }
                ProofNode::Numbered(line) if line.is_inference() => {
                    if let Some(wff) = &line.sentence {
                        if let Err(err) = check_wff_not_contain_out_of_scope_boxed_consts(
                            wff,
                            &currently_in_scope,
                            &boxed_consts,
                            line.line_num,
                        ) {
                            errors.push(err);
                        }
                    }
                }
                ProofNode::SubproofClose {
                    ..
                } => {
                    currently_in_scope.pop();
                }
                ProofNode::SubproofOpen {
                    ..
                }
                | ProofNode::Empty {
                    ..
                } => {}
                ProofNode::Numbered(_) => {}
            }
        }

        // If the inputted Wff contains a *constant* which is in the global set of boxed constants (all_boxeds), but not in
        // the current scope, then return Err. Otherwise Ok.
        fn check_wff_not_contain_out_of_scope_boxed_consts(
            wff: &Wff,
            curr_scope: &[Option<Term>],
            all_boxeds: &HashSet<Term>,
            line_num: usize,
        ) -> Result<(), String> {
            fn check_term_not_contain_out_of_scope_boxed_consts(
                term: &Term,
                curr_scope: &[Option<Term>],
                all_boxeds: &HashSet<Term>,
                line_num: usize,
            ) -> Result<(), String> {
                match term {
                    Term::Atomic(_) => {
                        if all_boxeds.contains(term)
                            && !curr_scope.iter().filter_map(|x| x.as_ref()).any(|t| t == term)
                        {
                            Err(format!("Line {line_num}: it is not allowed to use a boxed constant outside the subproof that defines it"))
                        } else {
                            Ok(())
                        }
                    }
                    Term::FuncApp(_, args) => args.iter().try_for_each(|a| {
                        check_term_not_contain_out_of_scope_boxed_consts(
                            a, curr_scope, all_boxeds, line_num,
                        )
                    }),
                }
            }
            match wff {
                Wff::Bottom => Ok(()),
                Wff::And(li) | Wff::Or(li) => li.iter().try_for_each(|w| {
                    check_wff_not_contain_out_of_scope_boxed_consts(
                        w, curr_scope, all_boxeds, line_num,
                    )
                }),
                Wff::Implies(a, b) | Wff::Bicond(a, b) => {
                    check_wff_not_contain_out_of_scope_boxed_consts(
                        a, curr_scope, all_boxeds, line_num,
                    )
                    .and(check_wff_not_contain_out_of_scope_boxed_consts(
                        b, curr_scope, all_boxeds, line_num,
                    ))
                }
                Wff::Not(w) => check_wff_not_contain_out_of_scope_boxed_consts(
                    w, curr_scope, all_boxeds, line_num,
                ),
                Wff::Forall(_, w) | Wff::Exists(_, w) => {
                    check_wff_not_contain_out_of_scope_boxed_consts(
                        w, curr_scope, all_boxeds, line_num,
                    )
                }
                Wff::Atomic(_) => Ok(()),
                Wff::PredApp(_, args) => args.iter().try_for_each(|t| {
                    check_term_not_contain_out_of_scope_boxed_consts(
                        t, curr_scope, all_boxeds, line_num,
                    )
                }),
                Wff::Equals(s, t) => check_term_not_contain_out_of_scope_boxed_consts(
                    s, curr_scope, all_boxeds, line_num,
                )
                .and(check_term_not_contain_out_of_scope_boxed_consts(
                    t, curr_scope, all_boxeds, line_num,
                )),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// This function checks that all variables are bound, that user doesn't have nested quantifiers over the
    /// same variable and that users don't quantify over a constant, and that the user does not make
    /// a function with the name of a variable. In case none of these conditions are violated,
    /// `Ok(())` is returned. Otherwise, a relevant error message will be returned.
    fn check_variable_scoping_naming_issues(
        &self,
        wff: &Wff,
        line_num: usize,
    ) -> Result<(), String> {
        fn check_variable_scoping_naming_issues_helper(
            proof: &Proof,
            wff: &Wff,
            line_num: usize,
            bound_vars_in_scope: &mut Vec<String>,
        ) -> Result<(), String> {
            match wff {
                Wff::Bottom => Ok(()),
                Wff::Atomic(_) => Ok(()),
                Wff::PredApp(_, args) => args.iter().try_for_each(|a| {
                    check_variable_scoping_naming_issues_helper_term(
                        proof,
                        a,
                        line_num,
                        bound_vars_in_scope,
                    )
                }),
                Wff::And(li) | Wff::Or(li) => li.iter().try_for_each(|x| {
                    check_variable_scoping_naming_issues_helper(
                        proof,
                        x,
                        line_num,
                        bound_vars_in_scope,
                    )
                }),
                Wff::Implies(w1, w2) | Wff::Bicond(w1, w2) => {
                    check_variable_scoping_naming_issues_helper(
                        proof,
                        w1,
                        line_num,
                        bound_vars_in_scope,
                    )
                    .and(check_variable_scoping_naming_issues_helper(
                        proof,
                        w2,
                        line_num,
                        bound_vars_in_scope,
                    ))
                }
                Wff::Not(w) => check_variable_scoping_naming_issues_helper(
                    proof,
                    w,
                    line_num,
                    bound_vars_in_scope,
                ),
                Wff::Equals(t1, t2) => check_variable_scoping_naming_issues_helper_term(
                    proof,
                    t1,
                    line_num,
                    bound_vars_in_scope,
                )
                .and(check_variable_scoping_naming_issues_helper_term(
                    proof,
                    t2,
                    line_num,
                    bound_vars_in_scope,
                )),
                Wff::Forall(var, wff) | Wff::Exists(var, wff) => {
                    if !proof.allowed_variable_names.contains(var) {
                        Err(format!("Line {line_num}: you can only quantify over a variable, not over a constant."))
                    } else if bound_vars_in_scope.contains(var) {
                        Err(format!(
                            "Line {line_num}: this line contains \
                                       two nested quantifiers over the same variable."
                        ))
                    } else {
                        bound_vars_in_scope.push(var.to_string());
                        let res = check_variable_scoping_naming_issues_helper(
                            proof,
                            wff,
                            line_num,
                            bound_vars_in_scope,
                        );
                        bound_vars_in_scope.pop();
                        res
                    }
                }
            }
        }

        fn check_variable_scoping_naming_issues_helper_term(
            proof: &Proof,
            term: &Term,
            line_num: usize,
            bound_vars_in_scope: &mut Vec<String>,
        ) -> Result<(), String> {
            match term {
                Term::Atomic(str) => {
                    if proof.allowed_variable_names.contains(str)
                        && !bound_vars_in_scope.contains(str)
                    {
                        Err(format!("Line {line_num}: this line contains unbound variables."))
                    } else {
                        Ok(())
                    }
                }
                Term::FuncApp(name, args) => args
                    .iter()
                    .try_for_each(|a| {
                        check_variable_scoping_naming_issues_helper_term(
                            proof,
                            a,
                            line_num,
                            bound_vars_in_scope,
                        )
                    })
                    .and(if proof.allowed_variable_names.contains(name) {
                        Err(format!(
                            "Line {line_num}: you cannot have a function called \
                                     {name}, because {name} is a reserved name for variables."
                        ))
                    } else {
                        Ok(())
                    }),
            }
        }
        check_variable_scoping_naming_issues_helper(self, wff, line_num, &mut vec![])
    }

    /// This function first calls [Proof::get_arity_set], and then it returns either `Ok(())` or a bunch of
    /// error messages, if for example from the arity set it can be determined that the user has
    /// functions of inconsistent arity throughout the proof (e.g. they use both f(x) and f(x,x)) or
    /// if for example the user uses some letter both as a constant name and a function name.
    fn generate_arity_errors(&self) -> Vec<String> {
        let mut errors: Vec<String> = vec![];
        let mut arity_map: HashMap<String, Vec<usize>> = HashMap::from([]);
        for (name, arity) in self.get_arity_set() {
            if !arity_map.contains_key(&name) {
                arity_map.insert(name.to_string(), vec![]);
            }
            if !arity_map.get(&name).unwrap().contains(&arity) {
                arity_map.get_mut(&name).unwrap().push(arity);
            }
        }
        for (name, mut arities) in arity_map {
            if arities.is_empty() {
                panic!();
            }
            if arities.len() > 1 {
                arities.sort();
                if arities.contains(&0) {
                    if name.chars().next().unwrap().is_lowercase() {
                        errors.push(format!("Error: it seems like you use the name \'{name}\' both to denote a constant, and to denote a function symbol"));
                    } else {
                        errors.push(format!("Error: it seems like you use the name \'{name}\' both to denote a nullary predicate (\'no inputs\'), and to denote a non-nullary predicate"));
                    }
                } else if name.chars().next().unwrap().is_lowercase() {
                    errors.push(format!("Error: it seems like \'{name}\' is meant to denote a function symbol, but throughout the proof, its arity is inconsistent. The found arities are {arities:?}"))
                } else {
                    errors.push(format!("Error: it seems like \'{name}\' is meant to denote a predicate, but throughout the proof, its arity is inconsistent. The found arities are {arities:?}"))
                }
            }
        }
        errors
    }

    /// This function returns the "arity set" for a proof. This is a HashSet containing instances of
    /// (name,arity), where name can be the name of any constant, funtion symbol, atomic proposition
    /// or predicate, and arity is its arity. Note that the arity of constants and atomic
    /// propositions is defined to be 0. Note that variables are not included in the arity set.
    ///
    /// Note that if you find for example both f(x,x) and f(x,x,x) in the same proof, then BOTH the
    /// entries ("f", 2) and ("f", 3) will be included in the arity set.
    fn get_arity_set(&self) -> HashSet<(String, usize)> {
        fn get_arity_set_term(proof: &Proof, term: &Term) -> HashSet<(String, usize)> {
            match term {
                Term::Atomic(str) => {
                    if proof.allowed_variable_names.contains(str) {
                        HashSet::from([])
                    } else {
                        HashSet::from([(str.to_owned(), 0)])
                    }
                }
                Term::FuncApp(str, args) => args
                    .iter()
                    .map(|t| get_arity_set_term(proof, t))
                    .chain(std::iter::once(HashSet::from([(str.to_owned(), args.len())])))
                    .flatten()
                    .collect(),
            }
        }
        fn get_arity_set_wff(proof: &Proof, wff: &Wff) -> HashSet<(String, usize)> {
            match wff {
                Wff::Bottom => HashSet::from([]),
                Wff::And(li) | Wff::Or(li) => {
                    li.iter().flat_map(|t| get_arity_set_wff(proof, t)).collect()
                }
                Wff::Forall(_, w) | Wff::Exists(_, w) | Wff::Not(w) => get_arity_set_wff(proof, w),
                Wff::Bicond(w1, w2) | Wff::Implies(w1, w2) => get_arity_set_wff(proof, w1)
                    .into_iter()
                    .chain(get_arity_set_wff(proof, w2))
                    .collect(),
                Wff::Equals(t1, t2) => get_arity_set_term(proof, t1)
                    .into_iter()
                    .chain(get_arity_set_term(proof, t2))
                    .collect(),
                Wff::PredApp(str, args) => args
                    .iter()
                    .map(|t| get_arity_set_term(proof, t))
                    .chain(std::iter::once(HashSet::from([(str.to_owned(), args.len())])))
                    .flatten()
                    .collect(),
                Wff::Atomic(str) => HashSet::from([(str.to_owned(), 0)]),
            }
        }
        self.numbered_lines()
            .filter_map(|line| line.sentence.as_ref())
            .flat_map(|t| get_arity_set_wff(self, t))
            .chain(
                // also include boxed constants in arity set!
                self.numbered_lines().filter_map(|line| line.boxed_constant.as_ref()).map(|c| {
                    match c {
                        Term::Atomic(str) => (str.to_owned(), 0),
                        Term::FuncApp(..) => panic!("boxed constant cannot be FuncApp"),
                    }
                }),
            )
            .collect()
    }

    /// This function returns whether line n1 can reference line n2.
    ///
    /// For example, it will return `false` if line `n2` comes after line `n1` or if line `n2` is
    /// inside an already closed subproof. It also returns false if `n1 == n2`, since a proof line
    /// cannot reference itself.
    fn can_reference(&self, n1: usize, n2: usize) -> bool {
        self.scope[n1].0.contains(&n2)
    }

    /// Gets the [Wff] at some requested line number, and if this line does not exist or
    /// does not contain a sentence then this function will return an `Err` containing
    /// a relevant error message. The function will also give
    /// an `Err` if the line is not allowed to be referenced from the referencing line (e.g. because
    /// it is inside an already closed subproof).
    fn get_wff_at_line(
        &self,
        referencing_line: usize,
        requested_line: usize,
    ) -> Result<&Wff, String> {
        let line = self.find_numbered_line(requested_line);
        if let Some(l) = line {
            if let Some(wff) = &l.sentence {
                if self.can_reference(referencing_line, requested_line) {
                    Ok(wff)
                } else if requested_line < referencing_line {
                    Err(format!("Line {referencing_line}: line {requested_line} is referenced in the justification, but this is not allowed, because line {requested_line} is inside an already closed subproof."))
                } else {
                    Err(format!("Line {referencing_line}: line {requested_line} is referenced in the justification, but this is not allowed, because line {requested_line} does not come before line {referencing_line}."))
                }
            } else {
                Err(format!("Line {referencing_line}: line {requested_line} is being referenced in the justification, but that line does not contain a sentence."))
            }
        } else {
            Err(format!("Line {referencing_line}: line {requested_line} is being referenced in the justification, but that line does not exist."))
        }
    }

    /// This function gets the subproof that runs from line `subproof_begin` to line
    /// `subproof_end`. It will return either `Ok(())` if the subproof exists and is allowed to be
    /// referenced from `referencing_line`. Otherwise, a relevant error message will be returned.
    ///
    /// Example: if the requested subproof is inside an already closed subproof, then line
    /// `referencing_line` is not allowed to reference this subproof and an error message will be
    /// returned.
    ///
    /// Precondition: referencing_line is an existing line (and the scope needs to be computed
    /// already, but that is always the case since we are working on an already-instantiated Proof
    /// instance, and those cannot be created if their scope cannot be determined)
    fn get_subproof_at_lines(
        &self,
        referencing_line: usize,
        (subproof_begin, subproof_end): (usize, usize),
    ) -> Result<(&NumberedLine, &NumberedLine), String> {
        if self.scope[referencing_line].1.contains(&(subproof_begin, subproof_end)) {
            let s_begin = self.find_numbered_line(subproof_begin).unwrap();
            // the unwrap should work, since `scope` should refer only to valid line numbers
            let s_end = self.find_numbered_line(subproof_end).unwrap();
            Ok((s_begin, s_end))
        } else {
            Err(format!(
                "Line {referencing_line}: the referenced \
                        subproof {subproof_begin}-{subproof_end} is \
                        not in the scope of line {referencing_line}, \
                        or it does not exist."
            ))
        }
    }

    /// This function checks the logical validity of a particular proof line within a proof
    /// i.e., checks if the proof rule in the given line has been applied correctly.
    ///
    /// This function will return `Ok(())` if the proof rule in the given line has been applied
    /// correctly. It will also return `Ok(())` if the given line is a premise, or a Fitch bar
    /// line, or an empty line, since in those cases there is no justification to check.
    ///
    /// Note that the provided [NumberedLine] should exist in the proof!
    fn check_line(&self, line: &NumberedLine) -> Result<(), String> {
        // this function only checks lines that have a justification...
        if line.justification.is_none() {
            return Ok(());
        }

        let curr_line_num = line.line_num;

        let (curr_wff, just) =
            (line.sentence.as_ref().unwrap(), line.justification.as_ref().unwrap());
        match just {
            Justification::Reit(n) => {
                let ref_wff = self.get_wff_at_line(curr_line_num, *n)?;
                if curr_wff == ref_wff {
                    Ok(())
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the \
                                           proof rule Reit is used, but the sentence \
                                           in this line is not the same as the sentence \
                                           in the referenced line."
                    ))
                }
            }
            Justification::AndIntro(ns) => {
                if let Wff::And(conjs) = curr_wff {
                    if ns.len() != conjs.len() {
                        return Err(format!(
                            "Line {curr_line_num}: the rule ∧Intro is used, but the number of \
                            conjuncts ({}) of the sentence in line {curr_line_num} is not equal \
                            to the number of referenced proof lines ({}).",
                            conjs.len(),
                            ns.len()
                        ));
                    }
                    for i in 0..ns.len() {
                        if &conjs[i] != self.get_wff_at_line(curr_line_num, ns[i])? {
                            return Err(format!(
                                "Line {curr_line_num}: the rule ∧Intro is used, but the {}\'th \
                                conjunct of the sentence in that line is not the same as \
                                the sentence found in line {} (the {}\'th \
                                line referenced in the justification).",
                                i + 1,
                                ns[i],
                                i + 1
                            ));
                        }
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the justification ∧Intro is \
                                used, but the top-level connective of this line is not ∧."
                    ))
                }
            }
            Justification::AndElim(n) => {
                let ref_wff = self.get_wff_at_line(curr_line_num, *n)?;
                if let Wff::And(conjs) = ref_wff {
                    if conjs.iter().any(|conj| conj == curr_wff) {
                        Ok(())
                    } else {
                        Err(format!(
                            "Line {curr_line_num}: the justification \
                            ∧Elim: {n} is used, but none of the \
                            conjuncts in line {n} is identical \
                            to the sentence found in line {curr_line_num}."
                        ))
                    }
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the justification \
                        ∧Elim: {n} is used, but the top-level \
                        connective of line {n} is not a conjunction."
                    ))
                }
            }
            Justification::OrIntro(n) => {
                let ref_wff = self.get_wff_at_line(curr_line_num, *n)?;
                if let Wff::Or(disjs) = curr_wff {
                    if disjs.iter().any(|disj| disj == ref_wff) {
                        Ok(())
                    } else {
                        Err(format!(
                            "Line {curr_line_num}: the justification \
                            ∨Intro: {n} is used, but none of the \
                            disjuncts in line {curr_line_num} is identical \
                            to the sentence found in line {n}."
                        ))
                    }
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the justification \
                        ∨Intro is used, but the top-level \
                        connective of this line is not a disjunction."
                    ))
                }
            }
            Justification::OrElim(n, subproofs) => {
                let Wff::Or(disjs) = self.get_wff_at_line(curr_line_num, *n)? else {
                    return Err(format!(
                        "Line {curr_line_num}: ∨Elim: {n}, ..... \
                        is used, but the top-level connective of \
                        the sentence at line {n} is not ∨."
                    ));
                };
                if disjs.len() != subproofs.len() {
                    return Err(format!(
                        "Line {curr_line_num}: the rule ∨Elim: {n}, ..... \
                            is used, but the number of disjuncts ({}) \
                            of the sentence in line {n} is not equal to \
                            the number of referenced subproofs ({}).",
                        disjs.len(),
                        subproofs.len()
                    ));
                }
                for (disj, subprf) in zip(disjs, subproofs) {
                    let (s_begin, s_end) = self.get_subproof_at_lines(curr_line_num, *subprf)?;
                    let (Some(s_begin_wff), Some(s_end_wff), None) =
                        (&s_begin.sentence, &s_end.sentence, &s_begin.boxed_constant)
                    else {
                        return Err(format!(
                            "Line {curr_line_num}: when using ∨Elim, \
                            you cannot reference subproofs which \
                            introduce a boxed constant."
                        ));
                    };
                    if disj != s_begin_wff {
                        return Err(format!(
                            "Line {curr_line_num}: ∨Elim: {n}, ..... \
                            is used, but the premise of one of the \
                            referenced subproofs does not match the \
                            corresponding disjunct of the sentence at line {n}. \
                            Note that the subproofs should be referenced in the \
                            order in which their corresponding premises \
                            appear as disjuncts in the sentence at line {n}."
                        ));
                    }
                    if s_end_wff != curr_wff {
                        return Err(format!(
                            "Line {curr_line_num}: ∨Elim \
                            is used, but not all referenced subproofs end with \
                            the same sentence as the sentence in line {curr_line_num}."
                        ));
                    }
                }
                Ok(())
            }
            Justification::ImpliesIntro((n, m)) => {
                let Wff::Implies(a, b) = curr_wff else {
                    return Err(format!(
                        "Line {curr_line_num}: →Intro is used, but \
                            the top-level connective of the sentence at this line \
                            is not an implication."
                    ));
                };
                let (s_begin, s_end) = self.get_subproof_at_lines(curr_line_num, (*n, *m))?;
                if let (Some(s_begin_wff), Some(s_end_wff), None) =
                    (&s_begin.sentence, &s_end.sentence, &s_begin.boxed_constant)
                {
                    if **a != *s_begin_wff && **b == *s_end_wff {
                        Err(format!(
                            "Line {curr_line_num}: →Intro is used, but \
                                the premise of the referenced subproof does not match the \
                                antecedent of the implication found in line {curr_line_num}."
                        ))
                    } else if **a == *s_begin_wff && **b != *s_end_wff {
                        Err(format!(
                            "Line {curr_line_num}: →Intro is used, but \
                                the last sentence of the referenced subproof does not match the \
                                consequent of the implication found in line {curr_line_num}."
                        ))
                    } else if **a != *s_begin_wff && **b != *s_end_wff {
                        Err(format!(
                            "Line {curr_line_num}: →Intro is used, but \
                                the premise and last sentence of the referenced subproof \
                                do not match the antecedent and the consequent, respectively, \
                                of the implication found in line {curr_line_num}."
                        ))
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!(
                        "Line {curr_line_num}: when using →Intro, you \
                        cannot reference a subproof that introduces a boxed constant."
                    ))
                }
            }
            Justification::ImpliesElim(n, m) => {
                if let Wff::Implies(wff1, wff2) = self.get_wff_at_line(curr_line_num, *n)? {
                    let wff_m = self.get_wff_at_line(curr_line_num, *m)?;
                    if *wff_m == **wff1 && **wff2 == *curr_wff {
                        Ok(())
                    } else {
                        Err(format!(
                            "Line {curr_line_num}: the rule \
                                               →Elim is wrongly used."
                        ))
                    }
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the rule \
                                           →Elim: {n}, {m} is used, but the top-level \
                                           connective of line {n} is not an implication."
                    ))
                }
            }
            Justification::BicondIntro((sb1, se1), (sb2, se2)) => {
                if let Wff::Bicond(p, q) = curr_wff {
                    let (s_begin1, s_end1) =
                        self.get_subproof_at_lines(curr_line_num, (*sb1, *se1))?;
                    let (s_begin2, s_end2) =
                        self.get_subproof_at_lines(curr_line_num, (*sb2, *se2))?;
                    if let (
                        Some(s_begin_wff1),
                        Some(s_end_wff1),
                        Some(s_begin_wff2),
                        Some(s_end_wff2),
                        None,
                        None,
                    ) = (
                        &s_begin1.sentence,
                        &s_end1.sentence,
                        &s_begin2.sentence,
                        &s_end2.sentence,
                        &s_begin1.boxed_constant,
                        &s_begin2.boxed_constant,
                    ) {
                        if **p == *s_begin_wff1
                            && **q == *s_end_wff1
                            && **p == *s_end_wff2
                            && **q == *s_begin_wff2
                        {
                            Ok(())
                        } else {
                            Err(format!("Line {curr_line_num}: when using ↔Intro to infer P↔Q, you must first cite the subproof that proves P→Q, and then the subproof that proves Q→P."))
                        }
                    } else {
                        Err(format!("Line {curr_line_num}: when using ↔Intro, you cannot reference a subproof that introduces a boxed constant."))
                    }
                } else {
                    Err(format!("Line {curr_line_num}: ↔Intro is used, but the top-level connective of this sentence is not a bi-implication."))
                }
            }
            Justification::BicondElim(n, m) => {
                if let Wff::Bicond(wff1, wff2) = self.get_wff_at_line(curr_line_num, *n)? {
                    let wff_m = self.get_wff_at_line(curr_line_num, *m)?;
                    if (*wff_m == **wff1 && **wff2 == *curr_wff)
                        || (*wff_m == **wff2 && **wff1 == *curr_wff)
                    {
                        Ok(())
                    } else {
                        Err(format!("Line {curr_line_num}: the rule ↔Elim is wrongly used."))
                    }
                } else {
                    Err(format!("Line {curr_line_num}: the rule ↔Elim: {n}, {m} is used, but the top-level connective of line {n} is not a bi-implication."))
                }
            }
            Justification::NotIntro((n, m)) => {
                let (s_begin, s_end) = self.get_subproof_at_lines(curr_line_num, (*n, *m))?;
                if let Wff::Not(negated) = curr_wff {
                    if let (Some(s_begin_wff), Some(s_end_wff)) =
                        (&s_begin.sentence, &s_end.sentence)
                    {
                        if **negated == *s_begin_wff {
                            if *s_end_wff == Wff::Bottom {
                                Ok(())
                            } else {
                                Err(format!(
                                    "Line {curr_line_num}: ¬Intro is used, \
                                    but the last sentence in the referenced \
                                    subproof is not ⊥."
                                ))
                            }
                        } else {
                            Err(format!(
                                "Line {curr_line_num}: ¬Intro is \
                                            used, but the negation of the premise \
                                            of the referenced subproof does \
                                            not match this line."
                            ))
                        }
                    } else {
                        Err(format!(
                            "Line {curr_line_num}: ¬Intro is \
                            used, but the referenced subproof is not \
                            of the proper form. You cannot use ¬Intro \
                            on a subproof that introduces a boxed constant."
                        ))
                    }
                } else {
                    Err(format!(
                        "Line {curr_line_num}: ¬Intro is used, \
                        but the top-level connective of the sentence in this line is not ¬."
                    ))
                }
            }
            Justification::NotElim(n) => {
                if let Wff::Not(negd_wff) = self.get_wff_at_line(curr_line_num, *n)? {
                    if let Wff::Not(negd_negd_wff) = &**negd_wff {
                        if **negd_negd_wff == *curr_wff {
                            return Ok(());
                        }
                    }
                }
                if let Wff::Not(negd_wff) = curr_wff {
                    if let Wff::Not(negd_negd_wff) = &**negd_wff {
                        if *self.get_wff_at_line(curr_line_num, *n)? == **negd_negd_wff {
                            return Err(format!("Line {curr_line_num}: ¬Elim can only be used to go from ¬¬P to P, not the other way around"));
                        }
                    }
                }
                Err(format!("Line {curr_line_num}: ¬Elim is used improperly"))
            }
            Justification::BottomIntro(n, m) => {
                let wff1 = self.get_wff_at_line(curr_line_num, *n)?;
                let wff2 = self.get_wff_at_line(curr_line_num, *m)?;
                if let Wff::Not(negd_wff) = wff2 {
                    if *wff1 == **negd_wff {
                        return Ok(());
                    }
                }
                Err(format!(
                    "Line {curr_line_num}: ⊥Intro: {n}, {m} is used, \
                    but the sentence at line {m} is not the negation \
                    of the sentence at line {n}"
                ))
            }
            Justification::BottomElim(n) => {
                if let Wff::Bottom = self.get_wff_at_line(curr_line_num, *n)? {
                    Ok(())
                } else {
                    Err(format!(
                        "Line {curr_line_num}: ⊥Elim: {n} is \
                        used, but the sentence at line {n} is not ⊥."
                    ))
                }
            }
            Justification::EqualsIntro => {
                if let Wff::Equals(term1, term2) = curr_wff {
                    if term1 == term2 {
                        return Ok(());
                    }
                }
                Err(format!("Line {curr_line_num}: =Intro is wrongly used"))
            }
            Justification::EqualsElim(n, m) => {
                let Wff::Equals(subst_old, subst_new) = self.get_wff_at_line(curr_line_num, *m)?
                else {
                    return Err(format!(
                        "Line {curr_line_num}: the rule =Elim:{n},{m} \
                        is used, but line {m} is not of the form (term1) = (term2)"
                    ));
                };

                if substitution_applied_wff_one_or_more_times(
                    self.get_wff_at_line(curr_line_num, *n)?,
                    curr_wff,
                    (subst_old, subst_new),
                ) {
                    Ok(())
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the rule =Elim:{n},{m} \
                            is used, but is is impossible to obtain line {curr_line_num} \
                            from line {n} by changing one or more occurrences of {} to {}",
                        formatter::format_term(subst_old),
                        formatter::format_term(subst_new),
                    ))
                }
            }
            Justification::ForallIntro((sb, se)) => {
                let Wff::Forall(var, forall_curr_wff) = curr_wff else {
                    return Err(format!(
                        "Line {curr_line_num}: the rule ∀Intro is used, \
                    but the sentence at this line is not universally quantified at the top-level"
                    ));
                };
                let (s_begin, s_end) = self.get_subproof_at_lines(curr_line_num, (*sb, *se))?;
                let Some(boxed_const @ Term::Atomic(bc)) = &s_begin.boxed_constant else {
                    return Err(format!(
                        "Line {curr_line_num}: the rule ∀Intro is used, but the \
                        referenced subproof does not introduce a boxed constant"
                    ));
                };
                if s_begin.sentence.is_some() {
                    return Err(format!(
                            "Line {curr_line_num}: when using ∀Intro, the premise of the referenced subproof \
                            should consist of solely a boxed constant, without a sentence"
                    ));
                }
                if apply_trivial_substitution_everywhere_to_wff(
                    forall_curr_wff,
                    (&Term::Atomic(var.to_string()), boxed_const),
                ) != *s_end.sentence.as_ref().unwrap()
                {
                    return Err(format!("Line {curr_line_num}: the rule ∀Intro:{sb}-{se} is used, but if all occurrences of {var} in the quantified part of line {curr_line_num} are replaced by {bc}, one does not obtain the sentence in line {se}"));
                }

                Ok(())
            }
            Justification::ForallElim(n) => {
                let Wff::Forall(var, ref_wff) = self.get_wff_at_line(curr_line_num, *n)? else {
                    return Err(format!(
                        "Line {curr_line_num}: the justification \
                        ∀Elim:{n} is used, but the sentence at line {n} is not a \
                        universally quantified sentence at the top level"
                    ));
                };
                if let Some((term1, term2)) =
                    find_possible_trivial_substitution_wff(ref_wff, curr_wff)
                {
                    if apply_trivial_substitution_everywhere_to_wff(ref_wff, (&term1, &term2))
                        == *curr_wff
                        && Term::Atomic(var.to_string()) == term1
                    {
                        return if self.is_closed_term(&term2) {
                            Ok(())
                        } else {
                            Err(format!(
                                "Line {curr_line_num}: the rule ∀Elim:{n} is used, \
                                 but {} is not a closed term (so you cannot substitute {}
                                 for all occurences of {var} in line {})",
                                formatter::format_term(&term2),
                                formatter::format_term(&term2),
                                *n
                            ))
                        };
                    }
                }
                if &**ref_wff == curr_wff {
                    return Ok(());
                }
                Err(format!(
                    "Line {curr_line_num}: the rule ∀Elim:{n} is used, but there is no \
                    appropriate substitution between line {n} and line {curr_line_num}"
                ))
            }
            Justification::ExistsIntro(n) => {
                let Wff::Exists(var, exists_curr_wff) = curr_wff else {
                    return Err(format!(
                        "Line {curr_line_num}: the justification \
                        ∃Intro:{n} is used, but the sentence at line {curr_line_num} is not an \
                        existentially quantified sentence at the top level"
                    ));
                };
                let ref_wff = self.get_wff_at_line(curr_line_num, *n)?;
                if let Some((term1, term2)) =
                    find_possible_trivial_substitution_wff(exists_curr_wff, ref_wff)
                {
                    if substitution_applied_wff_zero_or_more_times(
                        exists_curr_wff,
                        ref_wff,
                        (&term1, &term2),
                    ) && Term::Atomic(var.to_string()) == term1
                    {
                        return if self.is_closed_term(&term2) {
                            Ok(())
                        } else {
                            Err(format!(
                                "Line {curr_line_num}: the rule ∃Intro:{n} is \
                                used, but {} in line {} is not a closed term",
                                formatter::format_term(&term2),
                                *n
                            ))
                        };
                    }
                }
                if **exists_curr_wff == *ref_wff {
                    return Ok(());
                }
                Err(format!(
                    "Line {curr_line_num}: the rule ∃Intro:{n} is used, but there is no \
                    appropriate substitution between line {n} and line {curr_line_num}"
                ))
            }
            Justification::ExistsElim(n, (sb, se)) => {
                let ref_wff = self.get_wff_at_line(curr_line_num, *n)?;
                let (s_begin, s_end) = self.get_subproof_at_lines(curr_line_num, (*sb, *se))?;
                let Wff::Exists(var, exists_ref_wff) = ref_wff else {
                    return Err(format!(
                        "Line {curr_line_num}: the rule ∃Elim:{n},{sb}-{se} \
                    is used, but the sentence at line {n} ({}) is not an existentially \
                    quantified sentence at the top-level",
                        formatter::format_wff(ref_wff)
                    ));
                };

                let Some(bc_term @ Term::Atomic(bc)) = &s_begin.boxed_constant else {
                    return Err(format!("Line {curr_line_num}: the rule ∃Elim:{n},{sb}-{se} is used, but the referenced subproof does not introduce a boxed constant in line {sb}."));
                };

                if s_begin.sentence.is_none() {
                    return Err(format!("Line {curr_line_num}: the rule ∃Elim:{n},{sb}-{se} is used, but line {sb} contains only a boxed constant; when using ∃Elim, it should contain both a boxed constant and a sentence"));
                }
                if apply_trivial_substitution_everywhere_to_wff(
                    exists_ref_wff,
                    (&Term::Atomic(var.to_string()), bc_term),
                ) == *s_begin.sentence.as_ref().unwrap()
                {
                    if s_end.sentence.as_ref().unwrap() == curr_wff {
                        Ok(())
                    } else {
                        Err(format!(
                            "Line {curr_line_num}: the rule ∃Elim:{n},{sb}-{se} \
                        is used, but the sentence in line {se} ({}) is not the same as \
                        the sentence in line {curr_line_num} ({})",
                            formatter::format_wff(s_end.sentence.as_ref().unwrap()),
                            formatter::format_wff(curr_wff),
                        ))
                    }
                } else {
                    Err(format!(
                        "Line {curr_line_num}: the rule ∃Elim:{n},{sb}-{se} \
                        is used, but if one substitutes {bc} for all free \
                        occurences of {var} in the quantified part of the sentence \
                        in line {n} ({}), one obtains {}, but this is not equal to the \
                        sentence found in line {sb} ({})",
                        formatter::format_wff(ref_wff),
                        formatter::format_wff(&apply_trivial_substitution_everywhere_to_wff(
                            exists_ref_wff,
                            (&Term::Atomic(var.to_string()), bc_term)
                        )),
                        formatter::format_wff(s_begin.sentence.as_ref().unwrap())
                    ))
                }
            }
        }
    }

    /// This function returns `true` if and only if the provided [Term] is a constant. For example,
    /// if the provided term is a `Term::FuncApp` (function application), then `false` will be
    /// returned. This function will also return `false` in case the provided [Term] is a variable
    /// instead of a constant.
    fn term_is_constant(&self, term: Term) -> bool {
        match term {
            Term::FuncApp(..) => false,
            Term::Atomic(str) => !self.allowed_variable_names.contains(&str),
        }
    }

    /// This function returns `true` if and only if the provided [Term] is a closed term, that is,
    /// it recursively contains no free variables.
    fn is_closed_term(&self, term: &Term) -> bool {
        match term {
            Term::Atomic(str) => !self.allowed_variable_names.contains(str),
            Term::FuncApp(_, args) => args.iter().all(|a| self.is_closed_term(a)),
        }
    }
}

/// This function returns `true` iff [Term] `t2` can be obtained from [Term] `t1` by applying
/// the substitution `subst` *zero or more* times.
fn substitution_applied_term_zero_or_more_times(
    t1: &Term,
    t2: &Term,
    subst: (&Term, &Term),
) -> bool {
    subst == (t1, t2)
        || t1 == t2
        || match (&t1, &t2) {
            (Term::FuncApp(f1_name, args1), Term::FuncApp(f2_name, args2)) => {
                f1_name == f2_name
                    && args1.len() == args2.len()
                    && zip(args1, args2).all(|(arg1, arg2)| {
                        substitution_applied_term_zero_or_more_times(arg1, arg2, subst)
                    })
                // it's almost Haskell <3
            }
            _ => false,
        }
}

/// This function returns `true` iff [Wff] `wff2` can be obtained from [Wff] `wff1` by applying
/// the substitution `subst` *zero or more* times.
fn substitution_applied_wff_zero_or_more_times(
    wff1: &Wff,
    wff2: &Wff,
    subst: (&Term, &Term),
) -> bool {
    let terms1 = terms_from_wff(wff1);
    let terms2 = terms_from_wff(wff2);
    wffs_are_syntactically_equal_except_possibly_the_terms(wff1, wff2)
        && terms1.len() == terms2.len()
        && zip(terms1, terms2)
            .all(|(t1, t2)| substitution_applied_term_zero_or_more_times(t1, t2, subst))
}

/// This function returns `true` iff [Wff] `wff2` can be obtained from [Wff] `wff1` by applying
/// the substitution `subst` *one or more* times.
fn substitution_applied_wff_one_or_more_times(
    wff1: &Wff,
    wff2: &Wff,
    subst: (&Term, &Term),
) -> bool {
    (wff1 != wff2 && substitution_applied_wff_zero_or_more_times(wff1, wff2, subst))
        || (wff1 == wff2 && subst.0 == subst.1 && wff_contains_term(wff1, subst.0))
}

/// Returns `true` iff [Term] `t1` contains [Term] `t2` at least once (or `t1` and `t2` are syntactically equal)
fn term_contains_term(t1: &Term, t2: &Term) -> bool {
    match &t1 {
        Term::Atomic(_) => t1 == t2,
        Term::FuncApp(_, args) => t1 == t2 || args.iter().any(|arg| term_contains_term(arg, t2)),
    }
}

// returns `true` iff [Wff] `wff` contains [Term] `t` at least once
fn wff_contains_term(wff: &Wff, t: &Term) -> bool {
    terms_from_wff(wff).iter().any(|t_| term_contains_term(t_, t))
}

/// This function returns the [Wff] which would result from applying a trivial substitution everywhere in given [Wff].
/// The inputted [Wff] is not changed.
/// Note that this substitution must be trivial: that is, the substitution
/// must be of the form term1 -> term2 where term1 is an atomic term,
/// i.e. a constant or variable, so not a function application.
fn apply_trivial_substitution_everywhere_to_wff(wff: &Wff, subst: (&Term, &Term)) -> Wff {
    match subst.0 {
        Term::FuncApp(..) => panic!("Substitution is not trivial"),
        Term::Atomic(_) => {}
    }

    // applies a trivial substitution everywhere and returns the resulting term.
    // Note that this substitution must be trivial: that is, the substitution
    // must be of the form term1 -> term2 where term1 is an atomic term,
    // i.e. a constant or variable, so not a function application.
    fn apply_trivial_substitution_everywhere_to_term(term: &Term, subst: (&Term, &Term)) -> Term {
        match subst.0 {
            Term::FuncApp(..) => panic!("Substitution is not trivial"),
            Term::Atomic(_) => {}
        }

        match term {
            Term::FuncApp(f, args) => Term::FuncApp(
                f.to_string(),
                args.iter()
                    .map(|arg| apply_trivial_substitution_everywhere_to_term(arg, subst))
                    .collect(),
            ),
            Term::Atomic(_) if term == subst.0 => subst.1.clone(),
            Term::Atomic(_) => term.clone(),
        }
    }

    let mut new_wff = wff.clone();
    terms_from_wff_mut(&mut new_wff)
        .iter_mut()
        .for_each(|term| **term = apply_trivial_substitution_everywhere_to_term(term, subst));
    new_wff
}

/// Given two [Wff]s, `wff1` and `wff2`, this function tries to determine if there exists a *trivial*
/// substitution of the form A -> B where A is not equal to B such that `wff2` can be
/// obtained from `wff1` only by applying that substitution at
/// least one time. If such a substitution exists, then this function returns it. If such a
/// substitution does not exist, then the return value of this function is garbage! So, if this
/// function returns some substitution, you should always check it yourself that it is actually
/// possible to obtain `wff2` from `wff1` by only applying the substitution. If this function returns
/// None, then you know for sure that there exists no trivial substitution that can convert `wff1`
/// into `wff2` by applying it at least once.
fn find_possible_trivial_substitution_wff(wff1: &Wff, wff2: &Wff) -> Option<(Term, Term)> {
    fn find_possible_trivial_substitution_term(term1: &Term, term2: &Term) -> Option<(Term, Term)> {
        if term1 == term2 {
            None
        } else if let Term::Atomic(..) = term1 {
            Some((term1.clone(), term2.clone()))
        } else if let (Term::FuncApp(_, args1), Term::FuncApp(_, args2)) = (term1, term2) {
            zip(args1, args2).find_map(|(a1, a2)| find_possible_trivial_substitution_term(a1, a2))
        } else {
            None
        }
    }
    let terms1 = terms_from_wff(wff1);
    let terms2 = terms_from_wff(wff2);
    zip(terms1, terms2).find_map(|(t1, t2)| find_possible_trivial_substitution_term(t1, t2))
}

/// Consider the abstract syntax trees of two well-formed formulas `wff1` and `wff2`. Now remove
/// from that tree all the nodes which are a [Term]. This function returns `true` if the syntax
/// trees are then equal (in which case we call the original wffs s-equivalent), `false` otherwise.
///
/// Notice that in a universal or existential
/// quantification, the variable that is being quantified over, still needs to be the same. That
/// is, this function will say that ∀x(P∨Q) and ∀y(P∨Q) are *not* s-equivalent,
/// because "x" is not "y".
///
/// Notice that this function still considers the number of arguments to a predicate. For example,
/// P(a,b,f(a)) and P(d,e,f(a,b,c,d)) are s-equivalent, but P(a,b,c) and P(a,b) are *not* s-equivalent.
fn wffs_are_syntactically_equal_except_possibly_the_terms(wff1: &Wff, wff2: &Wff) -> bool {
    fn s_eq(wff1: &Wff, wff2: &Wff) -> bool {
        // just a shorthand
        wffs_are_syntactically_equal_except_possibly_the_terms(wff1, wff2)
    }
    match (wff1, wff2) {
        (Wff::Bottom, Wff::Bottom) => true,
        (Wff::Bottom, _) => false,

        (Wff::Or(li1), Wff::Or(li2)) | (Wff::And(li1), Wff::And(li2)) => {
            li1.len() == li2.len() && zip(li1, li2).all(|(w1, w2)| s_eq(w1, w2))
        }
        (Wff::Or(..) | Wff::And(..), _) => false,

        (Wff::Implies(w11, w12), Wff::Implies(w21, w22))
        | (Wff::Bicond(w11, w12), Wff::Bicond(w21, w22)) => s_eq(w11, w21) && s_eq(w12, w22),
        (Wff::Implies(..) | Wff::Bicond(..), _) => false,

        (Wff::Not(w1), Wff::Not(w2)) => s_eq(w1, w2),
        (Wff::Not(..), _) => false,

        (Wff::Forall(x1, w1), Wff::Forall(x2, w2)) | (Wff::Exists(x1, w1), Wff::Exists(x2, w2)) => {
            x1 == x2 && s_eq(w1, w2)
        }
        (Wff::Forall(..) | Wff::Exists(..), _) => false,

        (Wff::Atomic(p), Wff::Atomic(q)) => p == q,
        (Wff::Atomic(..), _) => false,

        (Wff::PredApp(p, args1), Wff::PredApp(q, args2)) => p == q && args1.len() == args2.len(),
        (Wff::PredApp(..), _) => false,

        (Wff::Equals(..), Wff::Equals(..)) => true,
        (Wff::Equals(..), _) => false,
    }
}

// This is a macro to duplicate the same code for both mutable and immutable references...
// It's really cursed, but I saw no better way, other than having duplicate code, since Rust doesn't
// have an inbuilt way of parametrizing over mutability (yet?).
macro_rules! terms_from_wff_macro {
    ($func_name:ident, $($mut_:tt)?) => {
        /// Generates a vector of (((mutable))) references to the [Term]s that are present in a certain [Wff], in a
        /// deterministic order. For recursive terms, such as f(f(f(x))), only the topmost [Term] is
        /// included in the output vector (but this [Term] still recursively contains the sub[Term]s).
        fn $func_name(wff: & $($mut_)? Wff) -> Vec<& $($mut_)? Term> {
            fn helper<'a>(wff: &'a $($mut_)? Wff, ts: &mut Vec<&'a $($mut_)? Term>) {
                match wff {
                    Wff::Equals(t1, t2) => ts.extend([t1, t2]),
                    Wff::And(li) | Wff::Or(li) => {
                        for w in li {
                            helper(w, ts);
                        }
                    }
                    Wff::Implies(t1, t2) | Wff::Bicond(t1, t2) => {
                        helper(t1, ts);
                        helper(t2, ts);
                    }
                    Wff::PredApp(_, args) => ts.extend(args),
                    Wff::Forall(_, w) | Wff::Exists(_, w) => helper(w, ts),
                    Wff::Not(w) => helper(w, ts),
                    Wff::Bottom | Wff::Atomic(_) => {}
                }
            }
            let mut terms: Vec<& $($mut_)? Term> = vec![];
            helper(wff, &mut terms);
            terms
        }
    }
}
terms_from_wff_macro!(terms_from_wff,);
terms_from_wff_macro!(terms_from_wff_mut, mut);
