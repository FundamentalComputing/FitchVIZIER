import {
  check_proof,
  check_proof_diagnostics_js as checkProofDiagnostics,
  export_to_latex,
  fix_line_numbers_in_proof,
  format_proof,
} from '../wasm/index';

export interface Location {
  file: string | null;
  line: number;
  column: number;
}

export interface Diagnostic {
  message: string;
  location: Location | null;
}

export interface ProofResult {
  status: "correct" | "error" | "fatal";
  diagnostics: Diagnostic[];
}

export function check_proof_diagnostics(
  proof: string,
  allowedVariableNames: string,
): ProofResult {
  return checkProofDiagnostics(proof, allowedVariableNames) as ProofResult;
}

export { check_proof, format_proof, fix_line_numbers_in_proof, export_to_latex };
