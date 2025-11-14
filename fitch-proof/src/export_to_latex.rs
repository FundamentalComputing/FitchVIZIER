use crate::data::*;
use crate::formatter::format_wff;
use std::fmt::Write;

/// Exports a proof to a string that can be put in a LaTeX document.
pub fn proof_to_latex(proof: &[ProofNode]) -> String {
    let mut prev_depth = 1;
    let mut is_hypo = true;
    let proof_str = proof.iter().filter(|node| !node.is_structural()).fold(
        String::new(),
        |mut output, node| {
            if matches!(node, ProofNode::FitchBar { .. }) {
                is_hypo = false;
            }

            let depth = node.depth();
            let part1 = if depth == prev_depth + 1 {
                prev_depth += 1;
                is_hypo = true;
                "\\open\n"
            } else if depth + 1 == prev_depth {
                prev_depth -= 1;
                "\\close\n"
            } else {
                ""
            };

            match node {
                ProofNode::Numbered(line) => {
                    let part2 = format!(
                        "{}{{{}}}{{{}{}}}",
                        if is_hypo {
                            "\\hypo"
                        } else {
                            "\\have"
                        },
                        line.line_num,
                        match &line.boxed_constant {
                            Some(Term::Atomic(t)) => format!(" \\boxed{{{}}}~ ", t),
                            Some(_) => panic!("boxed constant cannot be function application"),
                            None => "".to_string(),
                        },
                        match &line.sentence {
                            Some(wff) => wff_to_latex(wff),
                            None => "".to_string(),
                        },
                    );
                    let part3 = match &line.justification {
                        Some(just) => justification_to_latex(just),
                        None => "".to_string(),
                    };
                    let _ = writeln!(output, "{}{}{}", part1, part2, part3);
                }
                ProofNode::FitchBar {
                    ..
                }
                | ProofNode::Empty {
                    ..
                } => {
                    let _ = write!(output, "{}", part1);
                }
                _ => {}
            }
            output
        },
    );
    format!("{}{}{}", "$\n\\begin{nd}\n", proof_str, "\\end{nd}\n$")
        .replace("  ", " ")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace(" \\", "\\")
}

/* ------------------ PRIVATE -------------------- */

/// Converts a [Wff] to a LaTeX string. This uses [format_wff] under the hood.
fn wff_to_latex(wff: &Wff) -> String {
    let formatted = format_wff(wff);

    // better too many spaces then not enough...
    // we will eliminate duplicate spaces later
    formatted
        .replace('∧', " \\land ")
        .replace('∨', " \\lor ")
        .replace('¬', " \\neg ")
        .replace('→', " \\rightarrow ")
        .replace('↔', " \\leftrightarrow ")
        .replace('⊥', " \\bot ")
        .replace('∀', "\\forall ")
        .replace('∃', "\\exists ")
}

/// Converts a [Justification] to a LaTeX string.
fn justification_to_latex(just: &Justification) -> String {
    match just {
        Justification::Reit(n) => format!("\\r{{{n}}}"),
        Justification::AndIntro(ns) => {
            format!(
                "\\ai{{{}}}",
                ns.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(",")
            )
        }
        Justification::AndElim(n) => format!("\\ae{{{n}}}"),
        Justification::OrIntro(n) => format!("\\oi{{{n}}}"),
        Justification::OrElim(n, subs) => format!(
            "\\oe{{{},{}}}",
            n,
            subs.iter().map(|(a, b)| format!("{}-{}", a, b)).collect::<Vec<String>>().join(","),
        ),
        Justification::NotIntro((a, b)) => format!("\\ni{{{a}-{b}}}"),
        Justification::NotElim(n) => format!("\\ne{{{n}}}"),
        Justification::EqualsIntro => "\\idi".to_owned(),
        Justification::EqualsElim(n, m) => format!("\\ide{{{n},{m}}}"),
        Justification::ImpliesIntro((n, m)) => format!("\\ii{{{n}-{m}}}"),
        Justification::ImpliesElim(n, m) => format!("\\ie{{{n},{m}}}"),
        Justification::BicondIntro((a, b), (c, d)) => format!("\\bci{{{a}-{b},{c}-{d}}}"),
        Justification::BicondElim(n, m) => format!("\\bce{{{n},{m}}}"),
        Justification::BottomIntro(n, m) => format!("\\bi{{{n},{m}}}"),
        Justification::BottomElim(n) => format!("\\be{{{n}}}"),
        Justification::ForallIntro((a, b)) => format!("\\Ai{{{a}-{b}}}"),
        Justification::ForallElim(n) => format!("\\Ae{{{n}}}"),
        Justification::ExistsIntro(n) => format!("\\Ei{{{n}}}"),
        Justification::ExistsElim(n, (a, b)) => format!("\\Ee{{{n},{a}-{b}}}"),
    }
}
