// @ts-nocheck
import './style.css'
// @ts-ignore
import { init, check_proof, format_proof, fix_line_numbers_in_proof, export_to_latex } from '@workspace/library'

export function process_user_input() {
  replace_words_by_fancy_symbols();

  console.log(document.getElementById("proof-field").value);
  let res = check_proof(document.getElementById("proof-field").value, document.getElementById("allowed-variable-names").value);
  if (res.startsWith("The proof is correct!")) {
    document.getElementById("feedback").style.color = "green";
  } else if (res.startsWith("Fatal error")) {
    document.getElementById("feedback").style.color = "red";
  } else {
    document.getElementById("feedback").style.color = "#f05a1f";
  }
  document.getElementById("feedback").innerText = res;
  console.log(res);
}

function format() {
  let formatted = format_proof(document.getElementById("proof-field").value);
  document.getElementById("proof-field").value = formatted;
  process_user_input();
}

function fix_line_numbers() {
  let fixed = fix_line_numbers_in_proof(document.getElementById("proof-field").value);
  document.getElementById("proof-field").value = fixed;
  process_user_input();
}

function to_latex() {
  let latex = export_to_latex(document.getElementById("proof-field").value);
  sessionStorage.setItem("latex-exported-proof", latex);
  window.open("latex_export", "_blank");
}

let advanced_settings_are_visible = false;
function toggle_show_advanced_settings() {
  advanced_settings_are_visible = !advanced_settings_are_visible;
  document.getElementById("advanced-settings").hidden = !advanced_settings_are_visible;
};

let examples_are_visible = false;
function show_examples() {
  examples_are_visible = !examples_are_visible;
  document.getElementById("additional-examples").hidden = !examples_are_visible;
}
export function load_example(ex) {
  let rdy = document.getElementById('proof-field').value == "";
  if (!rdy) {
    rdy = confirm("Your proof area is not empty. Loading an example will overwrite your current proof. Are you sure you want to continue?");
  }
  if (rdy) {
    console.log(ex);
    document.getElementById('proof-field').value = ex;
    process_user_input();
  }
}
window.load_example = load_example;

let proof_is_upside_down = false;
function upside_down() {
  proof_is_upside_down = !proof_is_upside_down;
  if (proof_is_upside_down) {
    document.getElementById("proof-field").style.setProperty("-webkit-transform", "rotate(180deg)", null);
  } else {
    document.getElementById("proof-field").style.setProperty("-webkit-transform", "rotate(0deg)", null);
  }
}

document.addEventListener('keydown', function(event) {
  if (event.altKey && event.key === 's') {
    toggle_show_advanced_settings();
  }
});


// when user types e.g. 'forall', replace it instantly with the proper forall unicode symbol, and 
// keep the user's cursor at the correct position so that user can continue typing.
function replace_words_by_fancy_symbols() {
  let proofstr = document.getElementById("proof-field").value;
  let offset = -1;
  if (proofstr.includes("fa")) {
    offset = 1;
  } else if (proofstr.includes("ex")) {
    offset = 1;
  } else if (proofstr.includes("not")) {
    offset = 2;
  } else if (proofstr.includes("impl")) {
    offset = 3;
  } else if (proofstr.includes("bic")) {
    offset = 2;
  } else if (proofstr.includes("and")) {
    offset = 2;
  } else if (proofstr.includes("or")) {
    offset = 1;
  } else if (proofstr.includes("bot")) {
    offset = 2;
  }

  if (offset == -1) {
    return;
  }

  proofstr = proofstr.replace("fa", "∀").replace("ex", "∃").replace("not", "¬").replace("or", "∨")
    .replace("bot", "⊥").replace("bic", "↔").replace("impl", "→").replace("and", "∧");
  let oldSelectionIndex = document.getElementById("proof-field").selectionStart;
  document.getElementById("proof-field").value = proofstr;
  document.getElementById("proof-field").focus();
  document.getElementById("proof-field").setSelectionRange(oldSelectionIndex - offset, oldSelectionIndex - offset);
};

// Download proof as .txt file
function download_proof() {
  const element = document.createElement('a');
  const blob = new Blob([document.getElementById('proof-field').value], { type: 'plain/text' });
  const fileUrl = URL.createObjectURL(blob);
  element.setAttribute("href", fileUrl);
  element.setAttribute("download", "proof.txt");
  element.style.display = 'none';
  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
};

document.getElementById("proof-field").onkeyup = process_user_input;
document.getElementById("format-button").onclick = format;
document.getElementById("latex-button").onclick = to_latex;
document.getElementById("load-example-button").onclick = show_examples;
document.getElementById("download-button").onclick = download_proof;
document.getElementById("upside-down-button").onclick = upside_down;
document.getElementById("fix-line-numbers-button").onclick = fix_line_numbers;
document.getElementById("settings-button").onclick = toggle_show_advanced_settings;
document.getElementById("allowed-variable-names").onkeyup = process_user_input;

document.addEventListener('keydown', function(event) {
  if (event.ctrlKey && event.key === 's') {
    event.preventDefault(); // Prevent default save action
    format()
  }
});
await init();
process_user_input();




