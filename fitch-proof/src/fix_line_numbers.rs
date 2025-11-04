use crate::data::*;
use std::collections::HashMap;

/// This function 'fixes' the line numbers in a vector of [ProofNode]s.
///
/// If the line numbers already start at 1 and increase by one at a time, then this function does
/// nothing. However, this function does do something in case the line numbers are not properly
/// starting at 1 and increasing by one at a time. This function modifies the numbered lines such
/// that the line numbers will start at 1 and increase by one at a time.
///
/// This function also updates the justifications with the new line numbers, so those do not get
/// messed up. This can be very useful if you have a big proof and you want to delete one unused
/// line in the middle, or if you want to insert a line in the middle. If you want to insert a line
/// in the middle of a big proof, just give that line the line number 1000 (or any unused value)
/// and then apply this function. It will fix all the line numbers and justifications.
///
/// If the proof contains justifications which have line numbers that do not exist in the proof,
/// these line numbers will be set to zero in the justification.
pub fn fix_line_numbers(proof_nodes: &mut [ProofNode]) {
    let mut line_num_map: HashMap<usize, usize> = HashMap::new();
    let mut next_line = 1usize;

    for node in proof_nodes.iter_mut() {
        if let ProofNode::Numbered(line) = node {
            line_num_map.insert(line.line_num, next_line);
            line.line_num = next_line;
            next_line += 1;
        }
    }

    let remap = |n: &usize| *line_num_map.get(n).unwrap_or(&0);

    for node in proof_nodes.iter_mut() {
        if let ProofNode::Numbered(line) = node {
            if let Some(just) = &line.justification {
                line.justification = Some(remap_justification(just, &remap));
            }
        }
    }
}

fn remap_justification<F>(just: &Justification, remap: &F) -> Justification
where
    F: Fn(&usize) -> usize,
{
    match just {
        Justification::Reit(n) => Justification::Reit(remap(n)),
        Justification::AndIntro(ns) => Justification::AndIntro(ns.iter().map(remap).collect()),
        Justification::AndElim(n) => Justification::AndElim(remap(n)),
        Justification::OrIntro(n) => Justification::OrIntro(remap(n)),
        Justification::OrElim(n, subs) => Justification::OrElim(
            remap(n),
            subs.iter().map(|(a, b)| (remap(a), remap(b))).collect(),
        ),
        Justification::EqualsIntro => Justification::EqualsIntro,
        Justification::EqualsElim(n, m) => Justification::EqualsElim(remap(n), remap(m)),
        Justification::NotIntro((n, m)) => Justification::NotIntro((remap(n), remap(m))),
        Justification::NotElim(n) => Justification::NotElim(remap(n)),
        Justification::BottomIntro(n, m) => Justification::BottomIntro(remap(n), remap(m)),
        Justification::BottomElim(n) => Justification::BottomElim(remap(n)),
        Justification::BicondIntro((a, b), (c, d)) => {
            Justification::BicondIntro((remap(a), remap(b)), (remap(c), remap(d)))
        }
        Justification::BicondElim(n, m) => Justification::BicondElim(remap(n), remap(m)),
        Justification::ForallIntro((a, b)) => Justification::ForallIntro((remap(a), remap(b))),
        Justification::ForallElim(n) => Justification::ForallElim(remap(n)),
        Justification::ExistsIntro(n) => Justification::ExistsIntro(remap(n)),
        Justification::ExistsElim(n, (a, b)) => {
            Justification::ExistsElim(remap(n), (remap(a), remap(b)))
        }
        Justification::ImpliesIntro((n, m)) => Justification::ImpliesIntro((remap(n), remap(m))),
        Justification::ImpliesElim(n, m) => Justification::ImpliesElim(remap(n), remap(m)),
    }
}
