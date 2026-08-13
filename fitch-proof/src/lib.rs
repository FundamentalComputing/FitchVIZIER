use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
mod checker;
mod data;
mod export_to_latex;
mod fix_line_numbers;
mod formatter;
mod loc;
mod parser;
mod proof;
mod util;
use crate::data::Wff;
pub use crate::data::{Diagnostic, Justification, NumberedLine, ProofNode, ProofResult};
pub use crate::loc::{Location, WithLoc};
pub use parser::parse_fitch_proof;
pub use parser::parse_logical_expression_string;

fn set_js_property(object: &Object, name: &str, value: &JsValue) {
    Reflect::set(object, &JsValue::from_str(name), value)
        .expect("setting a property on a newly created JavaScript object should succeed");
}

impl Location {
    fn to_js_value(&self) -> JsValue {
        let object = Object::new();
        set_js_property(
            &object,
            "file",
            &self.file.as_deref().map(JsValue::from_str).unwrap_or(JsValue::NULL),
        );
        set_js_property(&object, "line", &JsValue::from_f64(self.line as f64));
        set_js_property(&object, "column", &JsValue::from_f64(self.column as f64));
        object.into()
    }
}

impl Diagnostic {
    fn to_js_value(&self) -> JsValue {
        let object = Object::new();
        set_js_property(&object, "message", &JsValue::from_str(&self.message));
        set_js_property(
            &object,
            "location",
            &self.location.as_ref().map(Location::to_js_value).unwrap_or(JsValue::NULL),
        );
        object.into()
    }
}

impl ProofResult {
    fn to_js_value(&self) -> JsValue {
        let object = Object::new();
        let diagnostics = Array::new();
        let status = match self {
            ProofResult::Correct => "correct",
            ProofResult::Error(errors) => {
                for diagnostic in errors {
                    diagnostics.push(&diagnostic.to_js_value());
                }
                "error"
            }
            ProofResult::FatalError(diagnostic) => {
                diagnostics.push(&diagnostic.to_js_value());
                "fatal"
            }
        };
        set_js_property(&object, "status", &JsValue::from_str(status));
        set_js_property(&object, "diagnostics", &diagnostics.into());
        object.into()
    }
}

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
    let res = check_proof_diagnostics(proof, allowed_variable_names);
    match res {
        ProofResult::Correct => "The proof is correct!".to_string(),
        ProofResult::Error(errs) => errs
            .iter()
            .map(|diagnostic| diagnostic.format())
            .collect::<Vec<_>>()
            .join("\n\n"),
        ProofResult::FatalError(err) =>
            (Diagnostic
             { message: format!("Fatal error: {}", err.message),
               location: err.location }).format()
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
    let res = check_proof_with_template_diagnostics(proof, &template, allowed_variable_names);
    match res {
        ProofResult::Correct => "The proof is correct!".to_string(),
        ProofResult::Error(errs) => errs
            .iter()
            .map(|diagnostic| diagnostic.format())
            .collect::<Vec<_>>()
            .join("\n\n"),
        ProofResult::FatalError(err) =>
            (Diagnostic
             { message: format!("Fatal error: {}", err.message),
               location: err.location }).format()
    }
}

/// Checks if a string is a fully correct proof.
///
/// This function returns its evaluation of the proof in a [ProofResult].
///
/// See also [parser::parse_fitch_proof] and [checker::check_proof].
///
/// This function never panics.
pub fn check_proof_diagnostics(proof: &str, allowed_variable_names: &str) -> ProofResult {
    match parser::parse_fitch_proof_diagnostic(proof) {
        Err(err) => ProofResult::FatalError(err),
        Ok(proof_nodes) => match parser::parse_allowed_variable_names(allowed_variable_names) {
            Ok(variable_names) => checker::check_proof(proof_nodes, variable_names),
            Err(message) => ProofResult::FatalError(Diagnostic {
                message,
                location: None,
            }),
        },
    }
}

#[wasm_bindgen]
pub fn check_proof_diagnostics_js(proof: &str, allowed_variable_names: &str) -> JsValue {
    check_proof_diagnostics(proof, allowed_variable_names).to_js_value()
}

/// Checks if a string is a fully correct proof that matches a given proof template.
///
/// This function returns its evaluation of the proof in a [ProofResult].
///
/// See also [parser::parse_fitch_proof] and [checker::check_proof].
///
/// This function never panics.
pub fn check_proof_with_template_diagnostics(
    proof: &str,
    template: &[String],
    allowed_variable_names: &str,
) -> ProofResult {
    match parser::parse_fitch_proof_diagnostic(proof) {
        Err(err) => ProofResult::FatalError(err),
        Ok(proof_nodes) => match parser::parse_allowed_variable_names(allowed_variable_names) {
            Ok(variable_names) => {
                let template_wffs: Vec<Wff> = template
                    .iter()
                    .filter_map(|s| {
                        parser::parse_logical_expression_string(s).map(|lwff| lwff.take_value())
                    })
                    .collect();
                if template_wffs.len() != template.len() {
                    return ProofResult::FatalError(Diagnostic {
                        message: "Some sentences in the template file could not be parsed. If you see this as a student on Themis, please contact the course staff as soon as possible; something is wrong on our side. Thanks!".to_owned(),
                        location: None,
                    });
                }
                checker::check_proof_with_template(proof_nodes, template_wffs, variable_names)
            }
            Err(message) => ProofResult::FatalError(Diagnostic {
                message,
                location: None,
            }),
        },
    }
}

#[wasm_bindgen]
pub fn check_proof_with_template_diagnostics_js(
    proof: &str,
    template: Vec<String>,
    allowed_variable_names: &str,
) -> JsValue {
    check_proof_with_template_diagnostics(proof, &template, allowed_variable_names).to_js_value()
}

/// Returns whether a string is a fully correct proof.
///
/// This function never panics.
pub fn proof_is_correct(proof: &str) -> bool {
    matches!(check_proof_diagnostics(proof, default_variable_names!()), ProofResult::Correct)
}

/// Takes in a proof string as input, and tries to format that proof.
///
/// If formatting succeeds, the formatted string is returned. If formatting fails,
/// returns "invalid"
///
/// This function never panics.
#[wasm_bindgen]
pub fn format_proof(proof: &str) -> String {
    match parser::parse_fitch_proof(proof) {
        Ok(nodes) if !nodes.is_empty() => formatter::format_proof(nodes),
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
        Ok(mut nodes) if !nodes.is_empty() => {
            fix_line_numbers::fix_line_numbers(&mut nodes);
            formatter::format_proof(nodes)
        }
        _ => proof.to_owned(),
    }
}

#[wasm_bindgen]
pub fn export_to_latex(proof: &str) -> String {
    match parser::parse_fitch_proof(proof) {
        Ok(nodes) if !nodes.is_empty() => export_to_latex::proof_to_latex(&nodes),
        _ => "Failed to export to latex, because the proof could not be parsed or was empty."
            .to_string(),
    }
}

/// Produce a debug-friendly string that includes locations for every proof node and its contents.
pub fn debug_proof_with_locations(proof: &str) -> String {
    match parser::parse_fitch_proof(proof) {
        Ok(nodes) => nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| format!("{}: {:#?}", idx + 1, node))
            .collect::<Vec<_>>()
            .join("\n"),
        Err(err) => format!("Parse error: {err}"),
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;

    #[test]
    fn semantic_diagnostic_preserves_message_and_has_physical_location() {
        let proof = "\n1 | P\n  | ---\n2 | Q Reit:1";
        let expected = "Line 2: the proof rule Reit is used, but the sentence in this line is not the same as the sentence in the referenced line.";

        assert_eq!(check_proof(proof, default_variable_names!()), expected);
        assert_eq!(
            check_proof_diagnostics(proof, default_variable_names!()),
            ProofResult::Error(vec![Diagnostic {
                message: expected.to_string(),
                location: Some(Location::new(None, 4, 1)),
            }])
        );
    }

    #[test]
    fn structural_diagnostic_has_responsible_physical_location() {
        let proof = "\n1 | P\n  | ---\n2 | | | Q";
        let expected = "Fatal error: near line 2, there is an 'indentation/scope jump' that is too big. You cannot open or close two subproofs in the same line.";

        assert_eq!(check_proof(proof, default_variable_names!()), expected);
        let ProofResult::FatalError(diagnostic) =
            check_proof_diagnostics(proof, default_variable_names!())
        else {
            panic!("expected fatal diagnostic");
        };
        assert_eq!(diagnostic.location, Some(Location::new(None, 4, 1)));
        assert_eq!(format!("Fatal error: {}", diagnostic.message), expected);
    }

    #[test]
    fn configuration_diagnostic_has_no_proof_location() {
        let proof = "1 | P\n  | ---\n2 | P Reit:1";
        let ProofResult::FatalError(diagnostic) = check_proof_diagnostics(proof, "X") else {
            panic!("expected fatal diagnostic");
        };

        assert_eq!(diagnostic.location, None);
        assert_eq!(check_proof(proof, "X"), format!("Fatal error: {}", diagnostic.message));
    }

    #[test]
    fn template_premise_mismatch_has_first_mismatching_premise_location() {
        let proof = "\n1 | P\n2 | Q\n  | ---\n3 | P Reit:1";
        let template = vec!["P".to_string(), "R".to_string(), "P".to_string()];
        let ProofResult::Error(diagnostics) =
            check_proof_with_template_diagnostics(proof, &template, default_variable_names!())
        else {
            panic!("expected template diagnostic");
        };
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.starts_with("The premises"))
            .expect("missing premise mismatch diagnostic");

        assert_eq!(diagnostic.location, Some(Location::new(None, 3, 1)));
    }
}
