use std::collections::HashSet;
use std::iter;
use std::iter::from_fn;

use crate::data::*;
use crate::loc::{Location, WithLoc};

type LToken = WithLoc<Token>;

/// This function takes a string slice and tries to parse it as a full proof.
///
/// If it succeeds, a vector of [LProofNode]s is returned. If it does not succeed, then a nice error
/// message is returned.
///
/// For a specification of the grammar that is used for parsing, see the documentation of the
/// functions [parse_proof_line] and [parse_logical_expr].
pub fn parse_fitch_proof(proof: &str) -> Result<Vec<LProofNode>, String> {
    let mut last_line_num = 0;
    proof
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            if line.is_empty() {
                return None;
            }
            Some(match lex_with_line(line, Some(idx + 1)) {
                Ok(toks) => match parse_proof_line(&toks) {
                    Ok(node) => {
                        last_line_num = node.line_num()
                            .unwrap_or(last_line_num);
                        Ok(node)
                    }
                    Err(err) => Err(format!("parser failure near line {}: {}", last_line_num + 1, err)),
                },
                Err(err) => Err(format!("lexer failure near line {}: {}", last_line_num + 1, err)),
            })
        })
        .collect()
}

/// This function parses the list of strings that should be seen as a variable. This list should
/// simply be a string slice like this: "x,y,z", which means that "x", "y" and "z" are the strings
/// that should be seen as a variable.
///
/// Note that the list should not contain duplicates and that it should not contain a 'variable'
/// of which the name starts with an uppercase letter. If this happens, then an error message is
/// returned. An error message is returned in all cases in which the parsing failed.
///
/// If the parsing is successful, a [HashSet] containing the allowed variable names is returned.
pub fn parse_allowed_variable_names(allowed_var_names: &str) -> Result<HashSet<String>, String> {
    let toks = match lex(allowed_var_names) {
        Ok(toks) => toks,
        Err(err) => {
            return Err(format!("failure when lexing list of allowed variable names: {err}"))
        }
    };
    let err_str = "the list of allowed variable names could not be parsed".to_string();

    // additional check, does not hurt
    if toks.iter().any(|tok| !matches!(tok.value(), Token::Name(_) | Token::Comma)) {
        return Err(err_str);
    }

    let mut allowed_variable_names: HashSet<String> = HashSet::from([]);
    let mut rem_toks = toks.as_slice();

    loop {
        let Some(first_tok) = rem_toks.first() else {
            return Err(err_str);
        };
        let Token::Name(var_name) = first_tok.value() else {
            return Err(err_str);
        };
        if !var_name.chars().next().unwrap().is_ascii_lowercase() {
            return Err(format!("the list of allowed variable names could not be parsed: a variable name must start with a lowercase letter: {}", var_name));
        }
        if allowed_variable_names.contains(var_name) {
            return Err(format!("the list of allowed variable names contains duplicates: {}", var_name));
        }
        allowed_variable_names.insert(var_name.to_string());
        if rem_toks.len() == 1 {
            break;
        }
        if !matches!(rem_toks[1].value(), Token::Comma) {
            return Err(err_str);
        }
        rem_toks = &rem_toks[2..];
    }
    Ok(allowed_variable_names)
}

/// This function parses a *logical expression* from a String.
///
/// If it succeeds, a [Wff] is returned. Otherwise, a nice error message is returned.
///
/// The grammar: (brackets denote tokens; {} is EBNF notation for 0 or more times)
///
/// ```notrust
/// <E1> ::=
///            <E2>
///          | <E2> and <E2> {and <E2>}
///          | <E2> or <E2> {or <E2>}
///          | <E2> implies <E2>
///          | <E2> bicond <E2>
///
/// <E2> ::=
///            <E3>
///          | <Term> equals <Term>
///
/// <E3> ::=
///            <PredicateName> <ArgList>
///          | <AtomicPropositionName>
///          | ( <E1> )
///          | forall <VariableOrConstantName> <E3>
///          | exists <VariableOrConstantName> <E3>
///          | not <E3>
///          | bottom
///
/// <Term> ::=
///              <FunctionName> <ArgList>
///            | <VariableOrConstantName>
///
/// <ArgList> ::= ( <Term> {, <Term>} )
///
/// <FunctionName> : some string starting with a lowercase letter
/// <VariableOrConstantName> : some string starting with a lowercase letter
/// <PredicateName> : some string starting with an UPPERCASE letter
/// <AtomicPropositionName> : some string starting with an UPPERCASE letter
/// ```
pub fn parse_logical_expression_string(expr: &str) -> Option<LWff> {
    lex(expr)
        .and_then(|toks| parse_logical_expr(&toks))
        .ok()
}

/* ----------------- PRIVATE -------------------*/

/// This is an enum containing tokens. The lexer converts a [String] to a vector of [Token]s, which
/// can then be used by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Name(String),
    LPar,
    RPar,
    Forall,
    Exists,
    And,
    Or,
    Implies,
    Bicond,
    Not,
    Bottom,
    Comma,
    Equals,
    Number(usize),
    ConseqVertBar(usize),
    Colon,
    Dash,
    LSqBracket,
    RSqBracket,
}

/// Generate a list of [Token]s from a [String]. If the lexer fails, a nice error message is returned.
fn lex(input: &str) -> Result<Vec<LToken>, String> {
    lex_with_line(input, None)
}

fn lex_with_line(input: &str, line_number: Option<usize>) -> Result<Vec<LToken>, String> {
    let mut toks: Vec<LToken> = Vec::new();
    let mut input_iter = input.chars().peekable();
    let line = line_number.unwrap_or(1);
    let mut pos = Location::new(None, line, 0);

    while let Some(ch) = input_iter.next() {
        pos.next_column();
        match ch {
            ' ' | '\t' => {} // ignore spaces
            '(' => toks.push(WithLoc::new(Token::LPar, pos.clone())),
            ')' => toks.push(WithLoc::new(Token::RPar, pos.clone())),
            '\u{2200}' => toks.push(WithLoc::new(Token::Forall, pos.clone())),
            '\u{2203}' => toks.push(WithLoc::new(Token::Exists, pos.clone())),
            '\u{2227}' => toks.push(WithLoc::new(Token::And, pos.clone())),
            '\u{2228}' => toks.push(WithLoc::new(Token::Or, pos.clone())),
            '\u{2192}' => toks.push(WithLoc::new(Token::Implies, pos.clone())),
            '\u{2194}' => toks.push(WithLoc::new(Token::Bicond, pos.clone())),
            '\u{00AC}' => toks.push(WithLoc::new(Token::Not, pos.clone())),
            ',' => toks.push(WithLoc::new(Token::Comma, pos.clone())),
            '=' => toks.push(WithLoc::new(Token::Equals, pos.clone())),
            //a variable name begins with a letter and contains only other letters
            //TODO: consider using c.is_ascii_alphanumeric())
            'a'..='z' | 'A'..='Z' => {
                let name = iter::once(ch) // push back the read char
                    .chain(from_fn(|| input_iter.by_ref().next_if(|c| c.is_ascii_alphabetic())))
                    .collect::<String>();
                let loc = pos.clone();
                pos.advance_by(name.len() - 1); // advance the position information
                toks.push(WithLoc::new(Token::Name(name), loc));
            }
            '1'..='9' => {
                let num_str = iter::once(ch)
                    .chain(from_fn(|| input_iter.by_ref().next_if(|c| c.is_ascii_digit())))
                    .collect::<String>();
                let err = "there was an integer bigger than 999999999".to_string();
                let loc = pos.clone();
                pos.advance_by(num_str.len() - 1);
                match num_str.parse::<usize>() {
                    Ok(n) if n <= 999_999_999 => {
                        toks.push(WithLoc::new(Token::Number(n), loc))
                    }
                    _ => return Err(err),
                }
            }
            '|' => {
                let full_bar_str: String = iter::once(ch)
                    .chain(from_fn(|| input_iter.by_ref().next_if(|c| *c == '|' || *c == ' ')))
                    .collect();
                // how many bares we actually have
                let count =
                    full_bar_str.chars().filter(|c| *c == '|').count();
                let loc = pos.clone();
                pos.advance_by(full_bar_str.len() - 1);

                toks.push(WithLoc::new(Token::ConseqVertBar(count), loc));
            }
            ':' => toks.push(WithLoc::new(Token::Colon, pos.clone())),
            '-' => toks.push(WithLoc::new(Token::Dash, pos.clone())),
            '[' => toks.push(WithLoc::new(Token::LSqBracket, pos.clone())),
            ']' => toks.push(WithLoc::new(Token::RSqBracket, pos.clone())),
            '⊥' => toks.push(WithLoc::new(Token::Bottom, pos.clone())),
            _ => {
                let mut err: String = "invalid character found: ".to_owned();
                err.push(ch);
                return Err(err);
            }
        }
    }

    Ok(toks)
}

/// This function parses a *logical expression* from a list of [Token]s.
///
/// If it succeeds, a [Wff] is returned. Otherwise, a nice error message is returned.
///
/// The grammar: see documentation of [parser::parse_logical_expression_string].
///
fn parse_logical_expr(toks: &[LToken]) -> Result<LWff, String> {
    if toks.is_empty() {
        return Err("parse_logical_expression: no tokens to parse".to_string())
    }
    let loc = toks.first().unwrap().location();
    if let Some((wff, rem_toks)) = parse_e1(toks) {
        // check that there are no remaining tokens left
        if rem_toks.is_empty() {
            return Ok(wff);
        } else {
            return Err(format!("failed to parse logical expression near {}", loc));
        }
    }
    Err(format!("failed to parse logical expression near {}", loc))
}

/// Parse an `<E1>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_e1(toks: &[LToken]) -> Option<(LWff, &[LToken])> {
    let (first, mut rem) = parse_e2(toks)?;
    if rem.is_empty() {
        return Some((first, rem));
    }

    match rem.first()?.value() {
        Token::Implies => {
            let (rhs, rem_rest) = parse_e2(rem.get(1..)?)?;
            let loc = first.location().clone();
            Some((WithLoc::new(Wff::Implies(Box::new(first), Box::new(rhs)), loc), rem_rest))
        }
        Token::Bicond => {
            let (rhs, rem_rest) = parse_e2(rem.get(1..)?)?;
            let loc = first.location().clone();
            Some((WithLoc::new(Wff::Bicond(Box::new(first), Box::new(rhs)), loc), rem_rest))
        }
        Token::And => {
            let mut items = vec![first];
            while !rem.is_empty() && matches!(rem[0].value(), Token::And) {
                let (next, rest) = parse_e2(rem.get(1..)?)?;
                items.push(next);
                rem = rest;
            }
            let loc = items.first()?.location().clone();
            Some((WithLoc::new(Wff::And(items), loc), rem))
        }
        Token::Or => {
            let mut items = vec![first];
            while !rem.is_empty() && matches!(rem[0].value(), Token::Or) {
                let (next, rest) = parse_e2(rem.get(1..)?)?;
                items.push(next);
                rem = rest;
            }
            let loc = items.first()?.location().clone();
            Some((WithLoc::new(Wff::Or(items), loc), rem))
        }
        _ => Some((first, rem)),
    }
}

/// Parse an `<E2>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_e2(toks: &[LToken]) -> Option<(LWff, &[LToken])> {
    // just <E3>
    if let Some((wff, rem_toks)) = parse_e3(toks) {
        return Some((wff, rem_toks));
    }

    // <Term> equals <Term>
    if let Some((term1, rem_toks1)) = parse_term(toks) {
        if matches!(rem_toks1.first()?.value(), Token::Equals) {
            if let Some((term2, rem_toks2)) = parse_term(rem_toks1.get(1..)?) {
                let loc = term1.location().clone();
                return Some((WithLoc::new(Wff::Equals(term1, term2), loc), rem_toks2));
            }
        }
    }

    None
}

/// Parse an `<E3>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_e3(toks: &[LToken]) -> Option<(LWff, &[LToken])> {
    let first = toks.first()?;
    match first.value() {
        Token::Name(name) if name.chars().next()?.is_uppercase() => {
            if let Some((terms, rem_toks)) = parse_arg_list(toks.get(1..)?) {
                let loc = first.location().clone();
                Some((WithLoc::new(Wff::PredApp(name.to_string(), terms), loc), rem_toks))
            } else {
                let loc = first.location().clone();
                Some((WithLoc::new(Wff::Atomic(name.to_string()), loc), &toks[1..]))
            }
        }
        Token::Not => {
            if let Some((expr, rem_toks)) = parse_e3(&toks[1..]) {
                let loc = first.location().clone();
                Some((WithLoc::new(Wff::Not(Box::new(expr)), loc), rem_toks))
            } else {
                None
            }
        }
        Token::LPar => {
            if let Some((expr, rem_toks)) = parse_e1(&toks[1..]) {
                if matches!(rem_toks.first()?.value(), Token::RPar) {
                    return Some((expr, &rem_toks[1..]));
                }
            }
            None
        }
        Token::Forall => {
            let next = toks.get(1)?;
            let Token::Name(var) = next.value() else {
                return None;
            };
            if !var.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                return None;
            }
            if let Some((expr, rem_toks)) = parse_e3(&toks[2..]) {
                let loc = first.location().clone();
                return Some((
                    WithLoc::new(Wff::Forall(var.to_owned(), Box::new(expr)), loc),
                    rem_toks,
                ));
            }
            None
        }
        Token::Exists => {
            let next = toks.get(1)?;
            let Token::Name(var) = next.value() else {
                return None;
            };
            if !var.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                return None;
            }
            if let Some((expr, rem_toks)) = parse_e3(&toks[2..]) {
                let loc = first.location().clone();
                return Some((
                    WithLoc::new(Wff::Exists(var.to_owned(), Box::new(expr)), loc),
                    rem_toks,
                ));
            }
            None
        }
        Token::Bottom => {
            let loc = first.location().clone();
            Some((WithLoc::new(Wff::Bottom, loc), &toks[1..]))
        }
        _ => None,
    }
}

/// Parse a `<Term>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_term(toks: &[LToken]) -> Option<(LTerm, &[LToken])> {
    let first = toks.first()?;
    match first.value() {
        Token::Name(name) => {
            if let Some((terms, rem_toks)) = parse_arg_list(&toks[1..]) {
                let loc = first.location().clone();
                Some((WithLoc::new(Term::FuncApp(name.to_string(), terms), loc), rem_toks))
            } else {
                let loc = first.location().clone();
                Some((WithLoc::new(Term::Atomic(name.to_string()), loc), &toks[1..]))
            }
        }
        _ => None,
    }
}

/// Parse an `<ArgList>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_arg_list(toks: &[LToken]) -> Option<(Vec<LTerm>, &[LToken])> {
    if !matches!(toks.first()?.value(), Token::LPar) {
        return None;
    }

    let mut terms: Vec<LTerm> = vec![];

    if let Some((term, mut rem_toks)) = parse_term(&toks[1..]) {
        terms.push(term);
        while matches!(rem_toks.first()?.value(), Token::Comma) {
            if let Some((term2, rem_rem_toks)) = parse_term(rem_toks.get(1..)?) {
                terms.push(term2);
                rem_toks = rem_rem_toks;
            } else {
                return None;
            }
        }

        if matches!(rem_toks.first()?.value(), Token::RPar) {
            Some((terms, &rem_toks[1..]))
        } else {
            None
        }
    } else {
        None
    }
}

/// This function parses one proof line from a list of [Token]s.
///
/// The grammar of a Fitch proof:
///
/// ```notrust
/// <FitchProof> is several <FitchProofLine>s separated by newline
/// <FitchProofLine> ::=
///                        <num> '|' { '|' } <E1> <Justification>             // non-premise
///                      | <num> '|' { '|' } <E1>                             // premise
///                      | <num> '|' { '|' } '[' <ConstantName> ']' [ <E1> ]  // premise with box
///                      | '|' { '|' } - { - }                                // fitch bar
///                      | '|' { '|' }                                        // empty line
///
/// <ConstantName> : some string starting with lowercase letter
///
/// <E1> is a full logical expression as parsed by the function parse_logical_expression_string();
/// the grammar for <E1> is defined in logic_expr.parser.rs.
///
/// <num> is a non-negative decinal integer
///
/// <Justification> ::=
///                      | Reit: <num>
///                      | And Intro: <num> {, <num>}
///                      | And Elim: <num>
///                      | Or Intro: <num>
///                      | Or Elim: <num>, <numrange> {, <numrange>}
///                      | Implies Intro: <numrange>
///                      | Implies Elim: <num>, <num>
///                      | Bicond Intro: <numrange>, <numrange>
///                      | Bicond Elim: <num>, <num>
///                      | Not Intro: <numrange>
///                      | Not Elim: <num>
///                      | Equals Intro
///                      | Equals Elim: <num>, <num>
///                      | Bottom Intro: <num>, <num>
///                      | Bottom Elim: <num>
///                      | Forall Intro: <numrange>
///                      | Forall Elim: <num>
///                      | Exists Intro: <num>
///                      | Exists Elim: <num>, <numrange>
///
/// ```
///
/// Note that Fitch proof lines are not very straightforward to parse, because it can be difficult
/// to find the separation between the `<E1>` and the `<Justification>`. However, note that the Colon
/// token only appears in the `<Justification>`, not in `<E1>`, `<num>` or `<ConstantName>`. Hence, if we
/// want to parse a proof line, we first check whether there is a colon token in it. If there is,
/// then we parse the justification first. If the line ends with =Intro, then we also parse the
/// justification first (=Intro is the only justification without colon). For the rest, everything
/// can just be done normally from left to right.

fn parse_proof_line(toks: &[LToken]) -> Result<LProofNode, String> {
    if toks.is_empty() {
        return Err("one proof line appears to be empty".to_string());
    }

    let has_colon = toks.iter().any(|t| matches!(t.value(), Token::Colon));
    let ends_with_intro = toks.len() >= 2
        && matches!(toks.last().unwrap().value(), Token::Name(name) if name == "Intro")
        && matches!(toks[toks.len() - 2].value(), Token::Equals);

    if has_colon || ends_with_intro {
        parse_line_with_justification(toks)
    } else {
        parse_line_without_justification(toks)
    }
}

fn parse_line_with_justification(toks: &[LToken]) -> Result<LProofNode, String> {
    let colon_index =
        toks.iter().position(|t| matches!(t.value(), Token::Colon)).unwrap_or(toks.len());
    if colon_index < 4 {
        return Err(
            "failed to parse proof line. The proof line contains a colon, but this colon appears so early that it cannot possibly be a justification".to_string()
        );
    }

    let (before_just, just_slice) = if let Token::Name(name) = toks[colon_index - 1].value() {
        match name.as_str() {
            "Reit" => (&toks[..colon_index - 1], &toks[colon_index - 1..]),
            "Intro" | "Elim" => (&toks[..colon_index - 2], &toks[colon_index - 2..]),
            _ => {
                return Err(format!(
                    "failed to parse justification. Expected 'Reit', 'Intro' or 'Elim', found '{name}'. Note that capitalization matters!"
                ));
            }
        }
    } else {
        return Err("sentence contains a colon, which was expected to be preceded by 'Intro', 'Elim' or 'Reit', but the parser did not find any of these.".to_string());
    };

    let number_tok = before_just
        .first()
        .ok_or_else(|| "a proof line must start with a line number".to_string())?;
    let depth_tok = before_just.get(1).ok_or_else(|| {
        "after the line number, there should be at least one vertical bar".to_string()
    })?;

    let Token::Number(line_num) = number_tok.value() else {
        return Err("a proof line with justification must start with a line number".to_string());
    };
    let Token::ConseqVertBar(depth) = depth_tok.value() else {
        return Err("after the line number, there should be at least one vertical bar".to_string());
    };

    let sentence_tokens = before_just.get(2..).unwrap_or(&[]);
    let sentence = parse_logical_expr(sentence_tokens)?;
    let justification = parse_justification(just_slice)?;

    let node_loc = number_tok.location().clone();
    Ok(WithLoc::new(
        ProofNode::Numbered(NumberedLine {
            line_num: *line_num,
            depth: *depth,
            sentence: Some(sentence),
            justification: Some(justification),
            boxed_constant: None,
        }),
        node_loc,
    ))
}

fn parse_line_without_justification(toks: &[LToken]) -> Result<LProofNode, String> {
    let first = toks.first().unwrap();
    match first.value() {
        Token::Number(line_num) => {
            let depth_tok = toks.get(1).ok_or_else(|| {
                "after the line number, there should be at least one vertical bar".to_string()
            })?;
            let Token::ConseqVertBar(depth) = depth_tok.value() else {
                return Err(
                    "after the line number, there should be at least one vertical bar".to_string()
                );
            };

            let mut const_between: Option<LTerm> = None;
            let expression_start = if let (Some(lsq), Some(name_tok), Some(rsq)) =
                (toks.get(2), toks.get(3), toks.get(4))
            {
                if !matches!(lsq.value(), Token::LSqBracket)
                    || !matches!(rsq.value(), Token::RSqBracket)
                {
                    2
                } else {
                    let Token::Name(name) = name_tok.value() else {
                        return Err("boxed constants must be names".to_string());
                    };
                    if !name.chars().next().unwrap_or('a').is_ascii_lowercase() {
                        return Err(
                            "a boxed constant must be a constant; it should start with a lowercase letter".to_string()
                        );
                    }
                    let loc = name_tok.location().clone();
                    const_between = Some(WithLoc::new(Term::Atomic(name.to_string()), loc));
                    if toks.len() == 5 {
                        let node_loc = first.location().clone();
                        return Ok(WithLoc::new(
                            ProofNode::Numbered(NumberedLine {
                                line_num: *line_num,
                                depth: *depth,
                                sentence: None,
                                justification: None,
                                boxed_constant: const_between,
                            }),
                            node_loc,
                        ));
                    }
                    5
                }
            } else {
                2
            };

            let has_brackets = toks.iter().any(|t| matches!(t.value(), Token::LSqBracket))
                || toks.iter().any(|t| matches!(t.value(), Token::RSqBracket));
            if has_brackets && expression_start != 5 {
                return Err("failed when trying to read boxed constant (if you did not intend to introduce a boxed constant in this proof line, remove '[' and ']').".to_string());
            }

            let sentence_tokens = toks.get(expression_start..).unwrap_or(&[]);
            let sentence = if sentence_tokens.is_empty() {
                None
            } else {
                Some(parse_logical_expr(sentence_tokens)?)
            };

            if sentence.is_none() && const_between.is_none() {
                return Err("a proof line must contain a sentence or introduce a boxed constant".to_string());
            }

            let node_loc = first.location().clone();
            Ok(WithLoc::new(
                ProofNode::Numbered(NumberedLine {
                    line_num: *line_num,
                    depth: *depth,
                    sentence,
                    justification: None,
                    boxed_constant: const_between,
                }),
                node_loc,
            ))
        }
        Token::ConseqVertBar(depth) => {
            let rest = &toks[1..];
            if rest.iter().all(|t| matches!(t.value(), Token::Dash)) && !rest.is_empty() {
                let loc = first.location().clone();
                Ok(WithLoc::new(
                    ProofNode::FitchBar {
                        depth: *depth,
                    },
                    loc,
                ))
            } else if rest.is_empty() || rest.iter().all(|t| matches!(t.value(), Token::Dash)) {
                let loc = first.location().clone();
                Ok(WithLoc::new(
                    ProofNode::Empty {
                        depth: *depth,
                    },
                    loc,
                ))
            } else {
                Err("when a line starts with only scope markers, it may only contain '-' after those markers.".to_string())
            }
        }
        _ => {
            Err("each text line must start either with a line number or a vertical bar".to_string())
        }
    }
}

/// Parse a justification, as specified by the grammar defined in the documentation for
/// [parse_proof_line].
fn parse_justification(toks: &[LToken]) -> Result<LJustification, String> {
    let plain: Vec<Token> = toks.iter().map(|tok| tok.value().clone()).collect();
    let justification = parse_justification_tokens(&plain)?;
    let loc = toks.first().map(|tok| tok.location().clone()).unwrap_or_else(Location::dummy);
    Ok(WithLoc::new(justification, loc))
}

fn parse_justification_tokens(toks: &[Token]) -> Result<Justification, String> {
    if toks.first().is_none() || toks.get(1).is_none() {
        return Err("failure when parsing justification; it seems not to be there?".to_string());
    }
    match (&toks[0], &toks[1], toks.get(2), toks.get(3)) {
        (Token::Name(name), Token::Colon, Some(Token::Number(num)), None) if name == "Reit" => {
            Ok(Justification::Reit(*num))
        }
        (Token::And, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Intro" =>
        {
            let err_str = "failed to parse ∧Intro justification. It should be of this form: ∧Intro:<num>,<num>{,<num>}".to_string();
            let mut nums: Vec<usize> = vec![*num];
            let mut i = 4;
            while toks.get(i).is_some() {
                if toks[i] == Token::Comma {
                    if toks.get(i + 1).is_none() {
                        return Err(err_str);
                    }
                    if let Token::Number(next_num) = toks.get(i + 1).unwrap() {
                        nums.push(*next_num);
                    } else {
                        return Err(err_str);
                    }
                } else {
                    return Err(err_str);
                }
                i += 2;
            }
            Ok(Justification::AndIntro(nums))
        }
        (Token::And, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Elim" =>
        {
            if toks.get(4).is_none() {
                Ok(Justification::AndElim(*num))
            } else {
                Err("failed to parse ∧Elim justification. It should be of this form: ∧Elim:<num>"
                    .to_string())
            }
        }
        (Token::Or, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Intro" =>
        {
            if toks.get(4).is_none() {
                Ok(Justification::OrIntro(*num))
            } else {
                Err("failed to parse ∨Intro justification. It should be of this form: ∨Intro:<num>"
                    .to_string())
            }
        }
        (Token::Or, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Elim" =>
        {
            let err_str = "failed to parse ∨Elim justification. It should be of this form: ∨Elim:<num>,<num>-<num>,<num>-<num>{,<num>-<num>}".to_string();
            let mut num_pairs: Vec<(usize, usize)> = vec![];
            let mut i = 4;
            if toks.get(i).is_none() {
                // should be at least one num-range provided
                return Err(err_str);
            };
            while toks.get(i).is_some() {
                if toks[i] == Token::Comma {
                    if toks.get(i + 1).is_none()
                        || toks.get(i + 2).is_none()
                        || toks.get(i + 3).is_none()
                    {
                        return Err(err_str);
                    }
                    if let (Token::Number(next_num1), Token::Dash, Token::Number(next_num2)) = (
                        toks.get(i + 1).unwrap(),
                        toks.get(i + 2).unwrap(),
                        toks.get(i + 3).unwrap(),
                    ) {
                        num_pairs.push((*next_num1, *next_num2));
                    } else {
                        return Err(err_str);
                    }
                } else {
                    return Err(err_str);
                }
                i += 4;
            }
            Ok(Justification::OrElim(*num, num_pairs))
        }
        (Token::Implies, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Intro" =>
        {
            let err_str = "failed to parse →Intro justification. It should be of this form: →Intro:<num>-<num>".to_string();
            if toks.len() != 6 {
                return Err(err_str);
            }
            if let (Token::Dash, Token::Number(num2)) = (toks.get(4).unwrap(), toks.get(5).unwrap())
            {
                Ok(Justification::ImpliesIntro((*num1, *num2)))
            } else {
                Err(err_str)
            }
        }
        (Token::Implies, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Elim" =>
        {
            let err_str =
                "failed to parse →Elim justification. It should be of this form: →Elim:<num>,<num>"
                    .to_string();
            if toks.len() != 6 {
                Err(err_str)
            } else if let [Token::Comma, Token::Number(num2)] = &toks[4..6] {
                Ok(Justification::ImpliesElim(*num1, *num2))
            } else {
                Err(err_str)
            }
        }
        (Token::Bicond, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Intro" =>
        {
            let err_str = "failed to parse ↔Intro justification. It should be of this form: ↔Intro:<num>-<num>,<num>-<num>".to_string();
            if toks.len() != 10 {
                Err(err_str)
            } else if let [Token::Dash, Token::Number(num2), Token::Comma, Token::Number(num3), Token::Dash, Token::Number(num4)] =
                &toks[4..10]
            {
                Ok(Justification::BicondIntro((*num1, *num2), (*num3, *num4)))
            } else {
                Err(err_str)
            }
        }
        (Token::Bicond, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Elim" =>
        {
            let err_str =
                "failed to parse ↔Elim justification. It should be of this form: ↔Elim:<num>,<num>"
                    .to_string();
            if toks.len() != 6 {
                Err(err_str)
            } else if let (Token::Comma, Token::Number(num2)) = (&toks[4], &toks[5]) {
                Ok(Justification::BicondElim(*num1, *num2))
            } else {
                Err(err_str)
            }
        }
        (Token::Not, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Intro" =>
        {
            let err_str = "failed to parse ¬Intro justification. It should be of this form: ¬Intro:<num>-<num>".to_string();
            if toks.len() != 6 {
                Err(err_str)
            } else if let (Token::Dash, Token::Number(num2)) = (&toks[4], &toks[5]) {
                Ok(Justification::NotIntro((*num1, *num2)))
            } else {
                Err(err_str)
            }
        }
        (Token::Not, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Elim" =>
        {
            if toks.get(4).is_none() {
                Ok(Justification::NotElim(*num))
            } else {
                Err("failed to parse ¬Elim justification. It should be of this form: ¬Elim:<num>"
                    .to_string())
            }
        }
        (Token::Bottom, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Intro" =>
        {
            let err_str = "failed to parse ⊥Intro justification. It should be of this form: ⊥Intro:<num>,<num>".to_string();
            if toks.len() != 6 {
                Err(err_str)
            } else if let (Token::Comma, Token::Number(num2)) = (&toks[4], &toks[5]) {
                Ok(Justification::BottomIntro(*num1, *num2))
            } else {
                Err(err_str)
            }
        }
        (Token::Bottom, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Elim" =>
        {
            if toks.get(4).is_none() {
                Ok(Justification::BottomElim(*num))
            } else {
                Err("failed to parse ⊥Elim justification. It should be of this form: ⊥Elim:<num>"
                    .to_string())
            }
        }
        (Token::Equals, Token::Name(name), ..) if name == "Intro" => {
            if toks.len() == 2 {
                Ok(Justification::EqualsIntro)
            } else {
                Err("failed to parse =Intro justification. This proof rule goes without colon and without line references, so all you write is just \'=Intro\'".to_string())
            }
        }
        (Token::Equals, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Elim" =>
        {
            let err_str = "failed to parse =Elim justification. It should be of this form: =Elim:<num>,<num>".to_string();
            if toks.len() != 6 {
                Err(err_str)
            } else if let (Token::Comma, Token::Number(num2)) =
                (toks.get(4).unwrap(), toks.get(5).unwrap())
            {
                Ok(Justification::EqualsElim(*num1, *num2))
            } else {
                Err(err_str)
            }
        }
        (Token::Forall, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Intro" =>
        {
            let err_str = "failed to parse ∀Intro justification. It should be of this form: ∀Intro:<num>-<num>".to_string();
            if toks.len() != 6 {
                Err(err_str)
            } else if let (Token::Dash, Token::Number(num2)) =
                (toks.get(4).unwrap(), toks.get(5).unwrap())
            {
                Ok(Justification::ForallIntro((*num1, *num2)))
            } else {
                Err(err_str)
            }
        }
        (Token::Forall, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Elim" && toks.get(4).is_none() =>
        {
            if toks.get(4).is_none() {
                Ok(Justification::ForallElim(*num))
            } else {
                Err("failed to parse ∀Elim justification. It should be of this form: ∀Elim:<num>"
                    .to_string())
            }
        }
        (Token::Exists, Token::Name(name), Some(Token::Colon), Some(Token::Number(num)))
            if name == "Intro" && toks.get(4).is_none() =>
        {
            if toks.get(4).is_none() {
                Ok(Justification::ExistsIntro(*num))
            } else {
                Err("failed to parse ∃Intro justification. It should be of this form: ∃Intro:<num>"
                    .to_string())
            }
        }
        (Token::Exists, Token::Name(name), Some(Token::Colon), Some(Token::Number(num1)))
            if name == "Elim" =>
        {
            let err_str = "failed to parse ∃Elim justification. It should be of this form: ∃Elim:<num>,<num>-<num>".to_string();
            if toks.len() != 8 {
                Err(err_str)
            } else if let (Token::Comma, Token::Number(num2), Token::Dash, Token::Number(num3)) = (
                toks.get(4).unwrap(),
                toks.get(5).unwrap(),
                toks.get(6).unwrap(),
                toks.get(7).unwrap(),
            ) {
                Ok(Justification::ExistsElim(*num1, (*num2, *num3)))
            } else {
                Err(err_str)
            }
        }
        _ => Err("failed to parse justification. Make sure that you have references where necessary, and note that the proper capitalization is \'Intro\'/\'Elim\'/\'Reit\'.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lexer_1() {
        assert_eq!(
            lex_tokens("   THIs is SoMe  SiLLY  test  "),
            Ok(vec![
                Token::Name("THIs".to_owned()),
                Token::Name("is".to_owned()),
                Token::Name("SoMe".to_owned()),
                Token::Name("SiLLY".to_owned()),
                Token::Name("test".to_owned())
            ])
        );
    }

    #[test]
    fn test_lexer_2() {
        assert_eq!(
            lex_tokens("   THIs is SoMe  S∀LLY  test  "),
            Ok(vec![
                Token::Name("THIs".to_owned()),
                Token::Name("is".to_owned()),
                Token::Name("SoMe".to_owned()),
                Token::Name("S".to_owned()),
                Token::Forall,
                Token::Name("LLY".to_owned()),
                Token::Name("test".to_owned())
            ])
        );
    }
    #[test]
    fn test_lexer_3() {
        assert_eq!(
            lex_tokens("   THIs is SoMe  S∀ ∀∀LLY  test  "),
            Ok(vec![
                Token::Name("THIs".to_owned()),
                Token::Name("is".to_owned()),
                Token::Name("SoMe".to_owned()),
                Token::Name("S".to_owned()),
                Token::Forall,
                Token::Forall,
                Token::Forall,
                Token::Name("LLY".to_owned()),
                Token::Name("test".to_owned())
            ])
        );
    }
    #[test]
    fn test_lexer_4() {
        assert_eq!(
            lex_tokens("∀x(P(x,a)→P(a,x))∨ (A∧ ItIsSunny∧¬∃y P(y,y))∨a=c"),
            Ok(vec![
                Token::Forall,
                Token::Name("x".to_owned()),
                Token::LPar,
                Token::Name("P".to_owned()),
                Token::LPar,
                Token::Name("x".to_owned()),
                Token::Comma,
                Token::Name("a".to_owned()),
                Token::RPar,
                Token::Implies,
                Token::Name("P".to_owned()),
                Token::LPar,
                Token::Name("a".to_owned()),
                Token::Comma,
                Token::Name("x".to_owned()),
                Token::RPar,
                Token::RPar,
                Token::Or,
                Token::LPar,
                Token::Name("A".to_owned()),
                Token::And,
                Token::Name("ItIsSunny".to_owned()),
                Token::And,
                Token::Not,
                Token::Exists,
                Token::Name("y".to_owned()),
                Token::Name("P".to_owned()),
                Token::LPar,
                Token::Name("y".to_owned()),
                Token::Comma,
                Token::Name("y".to_owned()),
                Token::RPar,
                Token::RPar,
                Token::Or,
                Token::Name("a".to_owned()),
                Token::Equals,
                Token::Name("c".to_owned()),
            ])
        );
    }

    fn lex_tokens(input: &str) -> Result<Vec<Token>, String> {
        lex(input).map(|toks| toks.into_iter().map(|t| t.value().clone()).collect())
    }

    fn lwff(wff: Wff) -> LWff {
        WithLoc::new(wff, Location::dummy())
    }

    fn lterm(term: Term) -> LTerm {
        WithLoc::new(term, Location::dummy())
    }

    fn strip_term_locations(term: LTerm) -> LTerm {
        let WithLoc {
            value,
            ..
        } = term;
        let value = match value {
            Term::Atomic(name) => Term::Atomic(name),
            Term::FuncApp(name, args) => {
                Term::FuncApp(name, args.into_iter().map(strip_term_locations).collect())
            }
        };
        WithLoc::dummy(value)
    }

    fn strip_wff_locations(wff: LWff) -> LWff {
        let WithLoc {
            value,
            ..
        } = wff;
        let sanitized = match value {
            Wff::And(children) => Wff::And(children.into_iter().map(strip_wff_locations).collect()),
            Wff::Or(children) => Wff::Or(children.into_iter().map(strip_wff_locations).collect()),
            Wff::Implies(lhs, rhs) => {
                Wff::Implies(Box::new(strip_wff_locations(*lhs)), Box::new(strip_wff_locations(*rhs)))
            }
            Wff::Bicond(lhs, rhs) => {
                Wff::Bicond(Box::new(strip_wff_locations(*lhs)), Box::new(strip_wff_locations(*rhs)))
            }
            Wff::Not(inner) => Wff::Not(Box::new(strip_wff_locations(*inner))),
            Wff::Bottom => Wff::Bottom,
            Wff::Forall(var, body) => Wff::Forall(var, Box::new(strip_wff_locations(*body))),
            Wff::Exists(var, body) => Wff::Exists(var, Box::new(strip_wff_locations(*body))),
            Wff::Atomic(name) => Wff::Atomic(name),
            Wff::PredApp(name, terms) => {
                Wff::PredApp(name, terms.into_iter().map(strip_term_locations).collect())
            }
            Wff::Equals(lhs, rhs) => {
                Wff::Equals(strip_term_locations(lhs), strip_term_locations(rhs))
            }
        };
        WithLoc::dummy(sanitized)
    }

    fn parse_expr(expr: &str) -> Option<LWff> {
        parse_logical_expression_string(expr).map(strip_wff_locations)
    }

    #[allow(dead_code)]
    fn func(name: &str, args: Vec<Term>) -> Term {
        Term::FuncApp(name.to_string(), args.into_iter().map(lterm).collect())
    }

    fn atom(name: &str) -> LWff {
        lwff(Wff::Atomic(name.to_string()))
    }

    fn and(children: Vec<LWff>) -> LWff {
        lwff(Wff::And(children))
    }

    fn or(children: Vec<LWff>) -> LWff {
        lwff(Wff::Or(children))
    }

    fn implies(lhs: LWff, rhs: LWff) -> LWff {
        lwff(Wff::Implies(Box::new(lhs), Box::new(rhs)))
    }

    fn forall(var: &str, body: LWff) -> LWff {
        lwff(Wff::Forall(var.to_string(), Box::new(body)))
    }

    fn pred(name: &str, args: Vec<Term>) -> LWff {
        lwff(Wff::PredApp(
            name.to_string(),
            args.into_iter().map(lterm).collect(),
        ))
    }

    fn eq_terms(left: Term, right: Term) -> LWff {
        lwff(Wff::Equals(lterm(left), lterm(right)))
    }

    fn parse_justification_text(input: &str) -> Result<Justification, String> {
        parse_justification(&lex(input).unwrap()).map(|j| j.value().clone())
    }

    #[test]
    fn test_parser_1() {
        assert_eq!(parse_expr("A∧B"), Some(and(vec![atom("A"), atom("B")])));
    }
    #[test]
    fn test_parser_2() {
        assert_eq!(parse_expr("AB"), Some(atom("AB")));
    }
    #[test]
    fn test_parser_3() {
        assert_eq!(parse_expr("a∧B"), None);
    }
    #[test]
    fn test_parser_4() {
        assert_eq!(parse_expr("aAAA"), None);
    }
    #[test]
    fn test_parser_5() {
        assert_eq!(parse_expr("A∨B"), Some(or(vec![atom("A"), atom("B")])));
    }
    #[test]
    fn test_parser_6() {
        assert_eq!(parse_expr("A∨∧B"), None);
    }
    #[test]
    fn test_parser_7() {
        assert_eq!(parse_expr("A→B"), Some(implies(atom("A"), atom("B"))));
    }
    #[test]
    fn test_parser_8() {
        assert_eq!(parse_expr("A→→→B"), None);
    }
    #[test]
    fn test_parser_9() {
        assert_eq!(
            parse_expr("∀x(∀y P(x,y))"),
            Some(forall(
                "x",
                forall("y", pred("P", vec![Term::Atomic("x".into()), Term::Atomic("y".into())]))
            ))
        );
        assert_eq!(parse_expr("∀x(∀y P(x,y))"), parse_expr("∀x∀y P(x,y)"));
        assert_eq!(parse_expr("∀x(∀y P(x,y))"), parse_expr("(∀x∀y P(x,y))"));
    }
    #[test]
    fn test_parser_10() {
        assert_eq!(parse_expr("∀(x∀y P(x,y))"), None);
    }
    #[test]
    fn test_parser_11() {
        let expr1 = "∀x(P(a,b,x)→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y)))";
        let expected_result = parse_expr(expr1);

        // correct

        // correct, same as expr1 but with a lot of spaces
        let expr2 = " ∀ x ( P ( a , b , x )   → Q ( f ( a ) , f ( b , c , d ) , g ( x ) ) ) ∨ f ( a , b ) = f ( bla , c ) ∨ ¬ ∃ x ¬ ¬ ¬ ∃ y ¬ ¬ ∀ z ¬ ¬ ( P ( f ( x ) , f ( y ) , f ( z ) ) → ¬ ( A ( x ) ∧ B ( y ) ) ) ";

        // wrong, misses a bracket in the end
        let expr3 = "∀x(P(a,b,x)→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y))";

        // correct, same as expr1 but with a lot of extra brackets
        let expr4 = "((∀x((P(a,b,x))→((Q(f(a),f(b,c,d),g(x)))))∨((((((f(a,b)=f(bla,c)))))))∨(¬(∃x(¬(¬(¬(∃y(¬(¬(∀z(¬(¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y))))))))))))))))";

        // wrong, same as expr1 but with brackets in a place where they shouldn't be
        let expr5 = "∀x(P((a),b,x)→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y)))";

        // wrong, same as expr1 but with brackets in a place where they shouldn't be
        let expr6 = "∀x(P((a,b,x))→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y)))";

        // wrong, same as expr1 but with brackets in a place where they shouldn't be
        let expr7 = "∀x(P(a,b,x)→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃(x)¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y)))";

        // wrong, same as expr1 but with one ) removed
        let expr8 = "∀x(P(a,b,x→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y)))";

        // wrong, same as expr1 but with one → removed
        let expr9 = "∀x(P(a,b,x)→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))¬(A(x)∧B(y)))";

        assert_eq!(parse_expr(expr1), expected_result);
        assert_eq!(parse_expr(expr2), expected_result);
        assert_eq!(parse_expr(expr3), None);
        assert_eq!(parse_expr(expr4), expected_result);
        assert_eq!(parse_expr(expr5), None);
        assert_eq!(parse_expr(expr6), None);
        assert_eq!(parse_expr(expr7), None);
        assert_eq!(parse_expr(expr8), None);
        assert_eq!(parse_expr(expr9), None);
    }
    #[test]
    fn test_parser_12() {
        assert_eq!(parse_expr("A∨B∧C"), None);
    }
    #[test]
    fn test_parser_13() {
        assert_eq!(parse_expr("A∧B∨B"), None);
    }
    #[test]
    fn test_parser_14() {
        assert_eq!(parse_expr("a=b=b"), None);
    }
    #[test]
    fn test_parser_15() {
        assert_eq!(
            parse_expr("a=b"),
            Some(eq_terms(Term::Atomic("a".to_string()), Term::Atomic("b".to_string())))
        );
    }

    #[test]
    fn test_justification_parser_or_elim() {
        assert_eq!(
            parse_justification_text("∨Elim:42,43-44"),
            Ok(Justification::OrElim(42, vec![(43, 44)]))
        );
        assert_eq!(
            parse_justification_text("∨Elim:42,43-44,45-46,47-48"),
            Ok(Justification::OrElim(42, vec![(43, 44), (45, 46), (47, 48)]))
        );
        assert!(parse_justification_text("∨Elim:42,43-44,45-46,47,48").is_err());
        assert!(parse_justification_text("∨Elim:42,43-44,45-46-47-48").is_err());
        assert!(parse_justification_text("∨Elim:42-43-44,45-46,47-48").is_err());
        assert!(parse_justification_text("∨Elim-42,43-44,45-46,47-48").is_err());
        assert!(parse_justification_text("∨Elim:42,43-44,45-46,47-48,").is_err());
        assert!(parse_justification_text("∨Elim:42,43-44,45-46,47-48,49").is_err());
        assert!(parse_justification_text("∨Elim:42,43-44,45-46,47-48,49-").is_err());
        assert!(parse_justification_text("∨Elim:42").is_err());
    }
    #[test]
    fn test_justification_parser_and_intro() {
        assert_eq!(
            parse_justification_text("∧Intro:42,43,44"),
            Ok(Justification::AndIntro(vec![42, 43, 44]))
        );
        assert_eq!(
            parse_justification_text("∧Intro:42,43"),
            Ok(Justification::AndIntro(vec![42, 43]))
        );
        assert_eq!(
            parse_justification_text("∧Intro:42"),
            // TODO: decide whether i want to keep behavior like this (a "unary conjunction")
            Ok(Justification::AndIntro(vec![42]))
        );
        assert!((parse_justification_text("∧Intro:42-43").is_err()));
        assert!((parse_justification_text("∧Intro:").is_err()));
    }
    #[test]
    fn test_justification_parser_exists_elim() {
        assert_eq!(
            parse_justification_text("∃Elim:42,43-44"),
            Ok(Justification::ExistsElim(42, (43, 44)))
        );
    }
    #[test]
    fn test_justification_parser_implies_elim() {
        assert_eq!(parse_justification_text("→Elim:42,43"), Ok(Justification::ImpliesElim(42, 43)));
        assert!((parse_justification_text("→Elim:42,43,").is_err()));
    }

    #[test]
    fn test_parser_bug_infinite_loop_1() {
        let toks = lex("(f(g(a),=b)").unwrap();
        let _ = parse_e2(&toks);
    }

    #[test]
    fn test_parser_bug_infinite_loop_2() {
        let toks = lex("f(g(a),=b").unwrap();
        let _ = parse_e1(&toks);
    }
    #[test]
    fn test_parser_bug_infinite_loop_3() {
        let toks = lex("f(g(a),=b").unwrap();
        let _ = parse_term(&toks);
    }
    #[test]
    fn test_parser_bug_infinite_loop_4() {
        let toks = lex("(g(a),=b").unwrap();
        let _ = parse_arg_list(&toks);
    }
}
