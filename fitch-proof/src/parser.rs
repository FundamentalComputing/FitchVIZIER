use std::collections::HashSet;
use std::iter;
use std::iter::from_fn;

use crate::data::*;

/// This function takes a string slice and tries to parse it as a full proof.
///
/// If it succeeds, a vector of [ProofNode]s is returned. If it does not succeed, then a nice error
/// message is returned.
///
/// For a specification of the grammar that is used for parsing, see the documentation of the
/// functions [parse_proof_line] and [parse_logical_expr].
pub fn parse_fitch_proof(proof: &str) -> Result<Vec<ProofNode>, String> {
    let mut last_line_num = 0;
    proof
        .lines()
        .filter(|s| !s.is_empty())
        .map(|x| match lex(x) {
            Ok(toks) => match parse_proof_line(&toks) {
                Ok(node) => {
                    if let Some(numbered) = node.as_numbered() {
                        last_line_num = numbered.line_num;
                    }
                    Ok(node)
                }
                Err(err) => Err(format!("parser failure near line {}: {}", last_line_num + 1, err)),
            },
            Err(err) => Err(format!("lexer failure near line {}: {}", last_line_num + 1, err)),
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
    if toks.iter().any(|tok| !matches!(tok, Token::Name(_) | Token::Comma)) {
        return Err(err_str);
    }
    let mut allowed_variable_names: HashSet<String> = HashSet::from([]);
    let mut rem_toks = toks.as_slice();

    loop {
        let Some(Token::Name(var_name)) = rem_toks.first() else {
            return Err(err_str);
        };
        if !var_name.chars().next().unwrap().is_ascii_lowercase() {
            return Err("the list of allowed variable names could not be parsed: a variable name must start with a lowercase letter".to_string());
        }
        if allowed_variable_names.contains(var_name) {
            return Err("the list of allowed variable names contains duplicates".to_string());
        }
        allowed_variable_names.insert(var_name.to_string());
        if rem_toks.len() == 1 {
            break;
        }
        if !matches!(rem_toks[1], Token::Comma) {
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
pub fn parse_logical_expression_string(expr: &str) -> Option<Wff> {
    if let Ok(toks) = lex(expr) {
        return match parse_logical_expr(&toks) {
            Ok(expr) => Some(expr),
            _ => None,
        };
    }
    None
}

/* ----------------- PRIVATE -------------------*/

/// This is an enum containing tokens. The lexer converts a [String] to a vector of [Token]s, which
/// can then be used by the parser.
#[derive(PartialEq, Debug)]
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
fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut toks: Vec<Token> = Vec::new();
    let mut input_iter = input.chars().peekable();

    while let Some(ch) = input_iter.next() {
        match ch {
            ' ' => {} // ignore spaces
            '(' => toks.push(Token::LPar),
            ')' => toks.push(Token::RPar),
            '\u{2200}' => toks.push(Token::Forall),
            '\u{2203}' => toks.push(Token::Exists),
            '\u{2227}' => toks.push(Token::And),
            '\u{2228}' => toks.push(Token::Or),
            '\u{2192}' => toks.push(Token::Implies),
            '\u{2194}' => toks.push(Token::Bicond),
            '\u{00AC}' => toks.push(Token::Not),
            ',' => toks.push(Token::Comma),
            '=' => toks.push(Token::Equals),
            'a'..='z' | 'A'..='Z' => {
                let name = iter::once(ch)
                    .chain(from_fn(|| input_iter.by_ref().next_if(|c| c.is_ascii_alphabetic())))
                    .collect::<String>();
                toks.push(Token::Name(name));
            }
            '1'..='9' => {
                let num: Result<usize, _> = iter::once(ch)
                    .chain(from_fn(|| input_iter.by_ref().next_if(|c| c.is_ascii_digit())))
                    .collect::<String>()
                    .parse();
                let err = "there was an integer bigger than 999999999".to_string();
                if let Ok(n) = num {
                    if n > 999999999 {
                        return Err(err);
                    }
                    toks.push(Token::Number(n));
                } else {
                    return Err(err);
                }
            }
            '|' => {
                let num: usize = iter::once(ch)
                    .chain(from_fn(|| input_iter.by_ref().next_if(|c| c == &'|' || c == &' ')))
                    .filter(|c| c == &'|')
                    .collect::<String>()
                    .len();
                toks.push(Token::ConseqVertBar(num));
            }
            ':' => toks.push(Token::Colon),
            '-' => toks.push(Token::Dash),
            '[' => toks.push(Token::LSqBracket),
            ']' => toks.push(Token::RSqBracket),
            '⊥' => toks.push(Token::Bottom),
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
fn parse_logical_expr(toks: &[Token]) -> Result<Wff, String> {
    if let Some((wff, rem_toks)) = parse_e1(toks) {
        if rem_toks.is_empty() {
            // there should be no remaining tokens!
            return Ok(wff);
        } else {
            return Err("failed to parse logical expression".to_string());
        }
    }
    Err("failed to parse logical expression".to_string())
}

/// Parse an `<E1>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_e1(toks: &[Token]) -> Option<(Wff, &[Token])> {
    // always accept the first <E2>
    if let Some((wff, mut rem_toks)) = parse_e2(toks) {
        if rem_toks.is_empty() {
            return Some((wff, rem_toks));
        }
        return match rem_toks[0] {
            // <E2> implies <E2>
            Token::Implies => {
                if let Some((wff2, rem_rem_toks)) = parse_e2(rem_toks.get(1..)?) {
                    return Some((Wff::Implies(Box::new(wff), Box::new(wff2)), rem_rem_toks));
                }
                None
            }
            // <E2> implies <E2>
            Token::Bicond => {
                if let Some((wff2, rem_rem_toks)) = parse_e2(rem_toks.get(1..)?) {
                    return Some((Wff::Bicond(Box::new(wff), Box::new(wff2)), rem_rem_toks));
                }
                None
            }
            // <E2> and <E2> {and <E2>}
            Token::And => {
                let mut conjs: Vec<Wff> = vec![wff];
                while !rem_toks.is_empty() && rem_toks[0] == Token::And {
                    if let Some((new_wff, rem_rem_toks)) = parse_e2(rem_toks.get(1..)?) {
                        rem_toks = rem_rem_toks;
                        conjs.push(new_wff);
                    } else {
                        return None;
                    }
                }
                Some((Wff::And(conjs), rem_toks))
            }
            // <E2> or <E2> {or <E2>}
            Token::Or => {
                let mut disjs: Vec<Wff> = vec![wff];
                while !rem_toks.is_empty() && rem_toks[0] == Token::Or {
                    if let Some((new_wff, rem_rem_toks)) = parse_e2(rem_toks.get(1..)?) {
                        rem_toks = rem_rem_toks;
                        disjs.push(new_wff);
                    } else {
                        return None;
                    }
                }
                Some((Wff::Or(disjs), rem_toks))
            }
            // found just a single <E2>
            _ => Some((wff, rem_toks)),
        };
    }

    None
}

/// Parse an `<E2>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_e2(toks: &[Token]) -> Option<(Wff, &[Token])> {
    // just <E3>
    if let Some((wff, rem_toks)) = parse_e3(toks) {
        return Some((wff, rem_toks));
    }

    // <Term> equals <Term>
    if let Some((term1, rem_toks1)) = parse_term(toks) {
        if rem_toks1.first()? == &Token::Equals {
            if let Some((term2, rem_toks2)) = parse_term(rem_toks1.get(1..)?) {
                return Some((Wff::Equals(term1, term2), rem_toks2));
            }
        }
    }

    None
}

/// Parse an `<E3>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_e3(toks: &[Token]) -> Option<(Wff, &[Token])> {
    match toks.first()? {
        Token::Name(name) if name.chars().next()?.is_uppercase() => {
            if let Some((terms, rem_toks)) = parse_arg_list(toks.get(1..)?) {
                Some((Wff::PredApp(name.to_string(), terms), rem_toks))
            } else {
                Some((Wff::Atomic(name.to_string()), &toks[1..]))
            }
        }
        Token::Not => {
            if let Some((wff, rem_toks)) = parse_e3(&toks[1..]) {
                Some((Wff::Not(Box::new(wff)), rem_toks))
            } else {
                None
            }
        }
        Token::LPar => {
            if let Some((wff, rem_toks)) = parse_e1(&toks[1..]) {
                if rem_toks.first()? == &Token::RPar {
                    return Some((wff, &rem_toks[1..]));
                }
            }
            None
        }
        Token::Forall => match toks.get(1)? {
            Token::Name(name) if name.chars().next()?.is_lowercase() => {
                if let Some((wff, rem_toks)) = parse_e3(toks.get(2..)?) {
                    return Some((Wff::Forall(name.to_owned(), Box::new(wff)), rem_toks));
                }
                None
            }
            _ => None,
        },
        Token::Exists => match toks.get(1)? {
            Token::Name(name) if name.chars().next()?.is_lowercase() => {
                if let Some((wff, rem_toks)) = parse_e3(toks.get(2..)?) {
                    return Some((Wff::Exists(name.to_owned(), Box::new(wff)), rem_toks));
                }
                None
            }
            _ => None,
        },
        Token::Bottom => Some((Wff::Bottom, &toks[1..])),
        _ => None,
    }
}

/// Parse a `<Term>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_term(toks: &[Token]) -> Option<(Term, &[Token])> {
    match toks.first()? {
        Token::Name(name) => {
            if let Some((terms, rem_toks)) = parse_arg_list(&toks[1..]) {
                Some((Term::FuncApp(name.to_string(), terms), rem_toks))
            } else {
                Some((Term::Atomic(name.to_string()), &toks[1..]))
            }
        }
        _ => None,
    }
}

/// Parse an `<ArgList>` as defined by the grammar specified in the documentation of [parse_logical_expr].
fn parse_arg_list(toks: &[Token]) -> Option<(Vec<Term>, &[Token])> {
    if toks.first()? != &Token::LPar {
        return None;
    }

    let mut terms: Vec<Term> = vec![];

    if let Some((term, mut rem_toks)) = parse_term(&toks[1..]) {
        terms.push(term);
        while rem_toks.first()? == &Token::Comma {
            if let Some((term2, rem_rem_toks)) = parse_term(rem_toks.get(1..)?) {
                terms.push(term2);
                rem_toks = rem_rem_toks;
            } else {
                return None;
            }
        }

        if rem_toks.first()? == &Token::RPar {
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

fn parse_proof_line(toks: &[Token]) -> Result<ProofNode, String> {
    if toks.contains(&Token::Colon)
        || (toks.last() == Some(&Token::Name("Intro".to_string())) // special check for =Intro
            && toks.get(toks.len() - 2) == Some(&Token::Equals))
    {
        // we know that in this case, <FitchProofLine> ::= <num> '|' { '|' } <E1> <Justification>
        // since only a Justification can legally contain a colon token or end with =Intro
        let colon_index: usize = toks.iter().position(|t| t == &Token::Colon).unwrap_or(toks.len());
        // note that we set colon_index to toks.len() in case of =Intro

        if colon_index < 4 {
            return Err("failed to parse proof line. The proof line contains a colon, but this colon appears so early that it cannot possibly be a justification".to_string());
            // colon cannot appear this early in a sentence
        }
        let toks_before_justification: &[Token];
        let toks_justification: &[Token];
        if let Token::Name(name) = &toks[colon_index - 1] {
            match name.as_str() {
                "Reit" => {
                    toks_before_justification = &toks[..colon_index - 1];
                    toks_justification = &toks[colon_index - 1..];
                }
                "Intro" | "Elim" => {
                    toks_before_justification = &toks[..colon_index - 2];
                    toks_justification = &toks[colon_index - 2..];
                }
                _ => {
                    return Err(format!("failed to parse justification. Expected \'Reit\', \'Intro\' or \'Elim\', found \'{name}\'. Note that capitalization matters!"));
                }
            }

            if let (
                Some(Token::Number(line_num)),
                Some(Token::ConseqVertBar(depth)),
                justific,
                wff,
            ) = (
                toks_before_justification.first(),
                toks_before_justification.get(1),
                parse_justification(toks_justification)?,
                parse_logical_expr(toks_before_justification.get(2..).unwrap_or(&[]))?,
            ) {
                Ok(ProofNode::Numbered(NumberedLine {
                    line_num: *line_num,
                    depth: *depth,
                    sentence: Some(wff),
                    justification: Some(justific),
                    boxed_constant: None,
                }))
            } else {
                Err("a line with an inference should always start with a line number (integer), followed by at least one vertical bar.".to_string())
            }
        } else {
            Err("sentence contains a colon, which was expected to be preceded by \'Intro\', \'Elim\' or \'Reit\' (with that capitalization), but the parser did not find any of these.".to_string())
        }
    } else {
        // Now we must be in one if these cases:
        //  <num> '|' { '|' } <E1>
        //  <num> '|' { '|' } '[' <ConstantName> ']' [ <E1> ]
        //  '|' { '|' } - { - }
        //  '|' { '|' }
        if toks.first().is_none() {
            return Err("one proof line appears to be empty".to_string());
        }
        match toks.first().unwrap() {
            Token::Number(num) => {
                let Some(Token::ConseqVertBar(depth)) = toks.get(1) else {
                    return Err("after the line number, there should be at least one vertical bar"
                        .to_string());
                };
                let mut const_betw_sqbr: Option<Term> = None;
                let expression_start_index: usize = if let (
                    Some(Token::LSqBracket),
                    Some(Token::Name(name)),
                    Some(Token::RSqBracket),
                ) = (toks.get(2), toks.get(3), toks.get(4))
                {
                    const_betw_sqbr = Some(Term::Atomic(name.to_string()));
                    if !name.chars().next().unwrap_or('U').is_lowercase() {
                        return Err("a boxed constant must be a constant; it should start with a lowercase letter".to_string());
                    }
                    if toks.len() == 5 {
                        // this premise contains only a boxed constant, no further expression:
                        // early exit
                        return Ok(ProofNode::Numbered(NumberedLine {
                            line_num: *num,
                            depth: *depth,
                            sentence: None,
                            justification: None,
                            boxed_constant: const_betw_sqbr,
                        }));
                    }
                    5
                } else {
                    2
                };

                // give error if it seems like the user tried to make a boxed constant but failed
                if (toks.contains(&Token::LSqBracket) || toks.contains(&Token::RSqBracket))
                    && expression_start_index != 5
                {
                    return Err("failed when trying to read boxed constant (if \
                        you did not intend to introduce a boxed constant in this \
                        proof line, remove the characters \'[\' and \']\' from this line)"
                        .to_string());
                }

                let wff = parse_logical_expr(toks.get(expression_start_index..).unwrap_or(&[]))?;

                Ok(ProofNode::Numbered(NumberedLine {
                    line_num: *num,
                    depth: *depth,
                    sentence: Some(wff),
                    justification: None,
                    boxed_constant: const_betw_sqbr,
                }))
            }
            Token::ConseqVertBar(depth) => {
                if toks[1..].iter().all(|t| t == &Token::Dash) {
                    if toks[1..].contains(&Token::Dash) {
                        Ok(ProofNode::FitchBar {
                            depth: *depth,
                        })
                    } else {
                        Ok(ProofNode::Empty {
                            depth: *depth,
                        })
                    }
                } else {
                    Err("when you have a line without line number, then that line can only possibly contain some minuses to indicate a Fitch bar, but it may contain no other tokens than minuses after the vertical bar(s)".to_string())
                }
            }
            _ => {
                Err("each text line must start either with a line number or a vertical bar"
                    .to_string())
            }
        }
    }
}

/// Parse a justification, as specified by the grammar defined in the documentation for
/// [parse_proof_line].
fn parse_justification(toks: &[Token]) -> Result<Justification, String> {
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
            lex("   THIs is SoMe  SiLLY  test  "),
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
            lex("   THIs is SoMe  S∀LLY  test  "),
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
            lex("   THIs is SoMe  S∀ ∀∀LLY  test  "),
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
            lex("∀x(P(x,a)→P(a,x))∨ (A∧ ItIsSunny∧¬∃y P(y,y))∨a=c"),
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

    #[test]
    fn test_parser_1() {
        assert_eq!(
            parse_logical_expression_string("A∧B"),
            Some(Wff::And(vec![Wff::Atomic("A".to_string()), Wff::Atomic("B".to_string())]))
        );
    }
    #[test]
    fn test_parser_2() {
        assert_eq!(parse_logical_expression_string("AB"), Some(Wff::Atomic("AB".to_string()),));
    }
    #[test]
    fn test_parser_3() {
        assert_eq!(parse_logical_expression_string("a∧B"), None);
    }
    #[test]
    fn test_parser_4() {
        assert_eq!(parse_logical_expression_string("aAAA"), None);
    }
    #[test]
    fn test_parser_5() {
        assert_eq!(
            parse_logical_expression_string("A∨B"),
            Some(Wff::Or(vec![Wff::Atomic("A".to_string()), Wff::Atomic("B".to_string())]))
        );
    }
    #[test]
    fn test_parser_6() {
        assert_eq!(parse_logical_expression_string("A∨∧B"), None);
    }
    #[test]
    fn test_parser_7() {
        assert_eq!(
            parse_logical_expression_string("A→B"),
            Some(Wff::Implies(
                Box::new(Wff::Atomic("A".to_string())),
                Box::new(Wff::Atomic("B".to_string()))
            ))
        );
    }
    #[test]
    fn test_parser_8() {
        assert_eq!(parse_logical_expression_string("A→→→B"), None);
    }
    #[test]
    fn test_parser_9() {
        assert_eq!(
            parse_logical_expression_string("∀x(∀y P(x,y))"),
            Some(Wff::Forall(
                "x".to_string(),
                Box::new(Wff::Forall(
                    "y".to_string(),
                    Box::new(Wff::PredApp(
                        "P".to_string(),
                        vec![Term::Atomic("x".to_string()), Term::Atomic("y".to_string())]
                    ))
                ))
            ))
        );
        assert_eq!(
            parse_logical_expression_string("∀x(∀y P(x,y))"),
            parse_logical_expression_string("∀x∀y P(x,y)")
        );
        assert_eq!(
            parse_logical_expression_string("∀x(∀y P(x,y))"),
            parse_logical_expression_string("(∀x∀y P(x,y))")
        );
    }
    #[test]
    fn test_parser_10() {
        assert_eq!(parse_logical_expression_string("∀(x∀y P(x,y))"), None);
    }
    #[test]
    fn test_parser_11() {
        // the most insane test ever
        let expected_result = Some(Wff::Or(vec![
            (Wff::Forall(
                "x".to_string(),
                Box::new(Wff::Implies(
                    Box::new(Wff::PredApp(
                        "P".to_string(),
                        vec![
                            Term::Atomic("a".to_string()),
                            Term::Atomic("b".to_string()),
                            Term::Atomic("x".to_string()),
                        ],
                    )),
                    Box::new(Wff::PredApp(
                        "Q".to_string(),
                        vec![
                            Term::FuncApp("f".to_string(), vec![Term::Atomic("a".to_string())]),
                            Term::FuncApp(
                                "f".to_string(),
                                vec![
                                    Term::Atomic("b".to_string()),
                                    Term::Atomic("c".to_string()),
                                    Term::Atomic("d".to_string()),
                                ],
                            ),
                            Term::FuncApp("g".to_string(), vec![Term::Atomic("x".to_string())]),
                        ],
                    )),
                )),
            )),
            (Wff::Equals(
                Term::FuncApp(
                    "f".to_string(),
                    vec![Term::Atomic("a".to_string()), Term::Atomic("b".to_string())],
                ),
                Term::FuncApp(
                    "f".to_string(),
                    vec![Term::Atomic("bla".to_string()), Term::Atomic("c".to_string())],
                ),
            )),
            (Wff::Not(Box::new(Wff::Exists(
                "x".to_string(),
                Box::new(Wff::Not(Box::new(Wff::Not(Box::new(Wff::Not(Box::new(Wff::Exists(
                    "y".to_string(),
                    Box::new(Wff::Not(Box::new(Wff::Not(Box::new(Wff::Forall(
                        "z".to_string(),
                        Box::new(Wff::Not(Box::new(Wff::Not(Box::new(Wff::Implies(
                            Box::new(Wff::PredApp(
                                "P".to_string(),
                                vec![
                                    Term::FuncApp(
                                        "f".to_string(),
                                        vec![Term::Atomic("x".to_string())],
                                    ),
                                    Term::FuncApp(
                                        "f".to_string(),
                                        vec![Term::Atomic("y".to_string())],
                                    ),
                                    Term::FuncApp(
                                        "f".to_string(),
                                        vec![Term::Atomic("z".to_string())],
                                    ),
                                ],
                            )),
                            Box::new(Wff::Not(Box::new(Wff::And(vec![
                                Wff::PredApp("A".to_string(), vec![Term::Atomic("x".to_string())]),
                                Wff::PredApp("B".to_string(), vec![Term::Atomic("y".to_string())]),
                            ])))),
                        )))))),
                    )))))),
                )))))))),
            )))),
        ]));

        // correct
        let expr1 = "∀x(P(a,b,x)→Q(f(a),f(b,c,d),g(x)))∨f(a,b)=f(bla,c)∨¬∃x¬¬¬∃y¬¬∀z¬¬(P(f(x),f(y),f(z))→¬(A(x)∧B(y)))";

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

        assert_eq!(parse_logical_expression_string(expr1), expected_result);
        assert_eq!(parse_logical_expression_string(expr2), expected_result);
        assert_eq!(parse_logical_expression_string(expr3), None);
        assert_eq!(parse_logical_expression_string(expr4), expected_result);
        assert_eq!(parse_logical_expression_string(expr5), None);
        assert_eq!(parse_logical_expression_string(expr6), None);
        assert_eq!(parse_logical_expression_string(expr7), None);
        assert_eq!(parse_logical_expression_string(expr8), None);
        assert_eq!(parse_logical_expression_string(expr9), None);
    }
    #[test]
    fn test_parser_12() {
        assert_eq!(parse_logical_expression_string("A∨B∧C"), None);
    }
    #[test]
    fn test_parser_13() {
        assert_eq!(parse_logical_expression_string("A∧B∨B"), None);
    }
    #[test]
    fn test_parser_14() {
        assert_eq!(parse_logical_expression_string("a=b=b"), None);
    }
    #[test]
    fn test_parser_15() {
        assert_eq!(
            parse_logical_expression_string("a=b"),
            Some(Wff::Equals(Term::Atomic("a".to_string()), Term::Atomic("b".to_string())))
        );
    }

    #[test]
    fn test_justification_parser_or_elim() {
        assert_eq!(
            parse_justification(&lex("∨Elim:42,43-44").unwrap()),
            Ok(Justification::OrElim(42, vec![(43, 44)]))
        );
        assert_eq!(
            parse_justification(&lex("∨Elim:42,43-44,45-46,47-48").unwrap()),
            Ok(Justification::OrElim(42, vec![(43, 44), (45, 46), (47, 48)]))
        );
        assert!(parse_justification(&lex("∨Elim:42,43-44,45-46,47,48").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim:42,43-44,45-46-47-48").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim:42-43-44,45-46,47-48").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim-42,43-44,45-46,47-48").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim:42,43-44,45-46,47-48,").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim:42,43-44,45-46,47-48,49").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim:42,43-44,45-46,47-48,49-").unwrap()).is_err());
        assert!(parse_justification(&lex("∨Elim:42").unwrap()).is_err());
    }
    #[test]
    fn test_justification_parser_and_intro() {
        assert_eq!(
            parse_justification(&lex("∧Intro:42,43,44").unwrap()),
            Ok(Justification::AndIntro(vec![42, 43, 44]))
        );
        assert_eq!(
            parse_justification(&lex("∧Intro:42,43").unwrap()),
            Ok(Justification::AndIntro(vec![42, 43]))
        );
        assert_eq!(
            parse_justification(&lex("∧Intro:42").unwrap()),
            // TODO: decide whether i want to keep behavior like this (a "unary conjunction")
            Ok(Justification::AndIntro(vec![42]))
        );
        assert!((parse_justification(&lex("∧Intro:42-43").unwrap()).is_err()));
        assert!((parse_justification(&lex("∧Intro:").unwrap()).is_err()));
    }
    #[test]
    fn test_justification_parser_exists_elim() {
        assert_eq!(
            parse_justification(&lex("∃Elim:42,43-44").unwrap()),
            Ok(Justification::ExistsElim(42, (43, 44)))
        );
    }
    #[test]
    fn test_justification_parser_implies_elim() {
        assert_eq!(
            parse_justification(&lex("→Elim:42,43").unwrap()),
            Ok(Justification::ImpliesElim(42, 43))
        );
        assert!((parse_justification(&lex("→Elim:42,43,").unwrap()).is_err()));
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
