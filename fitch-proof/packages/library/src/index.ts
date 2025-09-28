import { wasm_test } from '../wasm/index'
import init, {check_proof, format_proof, fix_line_numbers_in_proof, export_to_latex} from '../wasm/index';

const test = (): string => {
  return 'Hello from JS!'
}

export {
 init, check_proof, format_proof, fix_line_numbers_in_proof, export_to_latex
}
