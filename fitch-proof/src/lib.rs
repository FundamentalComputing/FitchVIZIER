use wasm_bindgen::prelude::*;
mod checker;
mod data;
mod export_to_latex;
mod fix_line_numbers;
mod formatter;
mod parser;
mod proof;
mod util;
use crate::data::{ProofResult, Wff};

macro_rules! default_variable_names {
    () => {
        "x,y,z,u,v,w"
    };
}

/// Checks if a string is a fully correct proof.
///
/// If the string corresponds to a fully correct proof, then a string will be returned,
/// saying that the proof is correct.
///
/// If the proof is not correct, then a string is returned which (hopefully) contains a nice error
/// message.
///
/// This function never panics.
#[wasm_bindgen]
pub fn check_proof(proof: &str, allowed_variable_names: &str) -> String {
    let res = check_proof_to_proofresult(proof, allowed_variable_names);
    match res {
        ProofResult::Correct => "The proof is correct!".to_string(),
        ProofResult::Error(errs) => errs.join("\n\n"),
        ProofResult::FatalError(err) => format!("Fatal error: {err}"),
    }
}

/// Checks if a string is a fully correct proof that matches a given proof template.
///
/// If the string corresponds to a fully correct proof, then a string will be returned,
/// saying that the proof is correct.
///
/// If the proof is not correct, or does not match the template,
/// then a string is returned which contains a nice error message.
///
/// This function never panics.
#[wasm_bindgen]
pub fn check_proof_with_template(
    proof: &str,
    template: Vec<String>,
    allowed_variable_names: &str,
) -> String {
    let res = check_proof_to_proofresult_with_template(proof, &template, allowed_variable_names);
    match res {
        ProofResult::Correct => "The proof is correct!".to_string(),
        ProofResult::Error(errs) => errs.join("\n\n"),
        ProofResult::FatalError(err) => format!("Fatal error: {err}"),
    }
}

/// Checks if a string is a fully correct proof.
///
/// This function returns its evaluation of the proof in a [ProofResult].
///
/// See also [parser::parse_fitch_proof] and [checker::check_proof].
///
/// This function never panics.
fn check_proof_to_proofresult(proof: &str, allowed_variable_names: &str) -> ProofResult {
    match (
        parser::parse_fitch_proof(proof),
        parser::parse_allowed_variable_names(allowed_variable_names),
    ) {
        (Ok(proof_lines), Ok(variable_names)) => checker::check_proof(proof_lines, variable_names),
        (Err(err), _) | (_, Err(err)) => ProofResult::FatalError(err),
    }
}

/// Checks if a string is a fully correct proof that matches a given proof template.
///
/// This function returns its evaluation of the proof in a [ProofResult].
///
/// See also [parser::parse_fitch_proof] and [checker::check_proof].
///
/// This function never panics.
fn check_proof_to_proofresult_with_template(
    proof: &str,
    template: &[String],
    allowed_variable_names: &str,
) -> ProofResult {
    match (
        parser::parse_fitch_proof(proof),
        parser::parse_allowed_variable_names(allowed_variable_names),
    ) {
        (Ok(proof_lines), Ok(variable_names)) => {
            let template_wffs: Vec<Wff> = template
                .iter()
                .filter_map(|s| parser::parse_logical_expression_string(s))
                .collect();
            if template_wffs.len() != template.len() {
                return ProofResult::FatalError("Some sentences in the template file could not be parsed. If you see this as a student on Themis, please contact the course staff as soon as possible; something is wrong on our side. Thanks!".to_owned());
            }
            checker::check_proof_with_template(proof_lines, template_wffs, variable_names)
        }
        (Err(err), _) | (_, Err(err)) => ProofResult::FatalError(err),
    }
}

/// Returns whether a string is a fully correct proof.
///
/// This function never panics.
pub fn proof_is_correct(proof: &str) -> bool {
    matches!(check_proof_to_proofresult(proof, default_variable_names!()), ProofResult::Correct)
}

/// Takes in a proof string as input, and tries to format that proof.
///
/// If formatting succeeds, the formatted string is returned. If formatting fails, the original
/// string is returned.
///
/// This function never panics.
#[wasm_bindgen]
pub fn format_proof(proof: &str) -> String {
    match parser::parse_fitch_proof(proof) {
        Ok(lines) if !lines.is_empty() => formatter::format_proof(lines),
        _ => "invalid".to_string(),
    }
}

/// This function fixes the line numbers in a proof (in case they are not proper).
///
/// If fixing the line numbers succeeds, the fixed string is returned. If it fails, the original
/// string is returned.
///
/// This function never panics.
#[wasm_bindgen]
pub fn fix_line_numbers_in_proof(proof: &str) -> String {
    match parser::parse_fitch_proof(proof) {
        Ok(mut lines) if !lines.is_empty() => {
            fix_line_numbers::fix_line_numbers(&mut lines);
            formatter::format_proof(lines)
        }
        _ => proof.to_owned(),
    }
}

#[wasm_bindgen]
pub fn export_to_latex(proof: &str) -> String {
    match parser::parse_fitch_proof(proof) {
        Ok(lines) if !lines.is_empty() => export_to_latex::proof_to_latex(&lines),
        _ => "Failed to export to latex, because the proof could not be parsed or was empty."
            .to_string(),
    }
}
