// returns true if and only if for all proofs P in {the inputted proof, the inputted proof after
// being formatted}, P is correct, and P is not correct anymore if any
// non-whitespace non-dash character gets removed from P. This is a super pedantic sanity check, which is
// also intended to help make sure that the parser does not panic in unexpected cases,
// and to make sure that the proof formatter works as intended. So it is like 3 tests in one.
// Note that it is possible to think of proofs which are correct, but not
// ultra-pedantically-correct. However, for most practical tests this is not an issue. If it ever
// is an issue, then just use a normal test for that proof and not an ultra-pedantic one.
//
// Note that if you want to assert that a proof is NOT correct, it is better not to use this
// function!! Because this function might return false even if the proof seen as correct by the
// program, which you don't want. So only use this function is you want to assert that a proof is
// correct.
fn proof_is_correct_ultra_pedantic(proof: &str) -> bool {
    (0..proof.chars().count()).all(|i| {
        proof.chars().nth(i) == Some('-')
            || proof.chars().nth(i).unwrap_or('a').is_whitespace()
            || {
                !fitch_proof::proof_is_correct(
                    proof
                        .chars()
                        .enumerate()
                        .filter(|(j, _)| *j != i)
                        .map(|(_, c)| c)
                        .collect::<String>()
                        .as_str(),
                )
            }
            || {
                println!("{i}:{} can be removed", proof.chars().nth(i).unwrap());
                false
            }
    }) && fitch_proof::proof_is_correct(proof)
        && fitch_proof::proof_is_correct(&fitch_proof::format_proof(proof))
}

// this is what you should call if you want to assert that a proof is not correct. It tests that
// the proof itself is not correct, and tests also that it is still not correct if it has been gone
// through the proof formatter. So it's two tests in one (also tests that formatter does not do
// weird things).
fn proof_is_not_correct_ultra_pedantic(proof: &str) -> bool {
    !fitch_proof::proof_is_correct(proof)
        && !fitch_proof::proof_is_correct(&fitch_proof::format_proof(proof))
}

#[test]
fn test_equals_elim_1() {
    let proof = "
1 | a=b
  | ---
2 | b=b   =Intro
3 | b=a   =Elim:2,1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_2() {
    let proof = "
1 | a=b
  | ---
2 | a=a   =Intro
3 | b=a   =Elim:2,1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_3() {
    let proof = "
1 | a=b
  | ---
2 | a=a   =Intro
3 | b=a   =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_4() {
    let proof = "
1 | a=b
  | ---
2 | a=a   =Intro
3 | a=a   =Elim:2,1
";
    // not correct, because substitution has to be applied ONE or more times
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_5() {
    let proof = "
1 | a=a
  | ---
2 | a=a   =Intro
3 | a=a   =Elim:2,1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_6() {
    let proof = "
1 | b=b
  | ---
2 | a=a   =Intro
3 | a=a   =Elim:2,1
";
    // not correct, because substitution has to be applied ONE or more times
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_7() {
    let proof = "
1 | P(a,b,c,a,b,c,a,b,c,a,b,c)
2 | a=b
  | ---
3 | P(a,b,c,a,b,c,a,b,c,a,b,c)  =Elim: 1,2
";
    // not correct, because substitution has to be applied ONE or more times
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_8() {
    let proof = "
1 | P(a,b,c,a,b,c,a,b,c,a,b,c)
2 | a=b
  | ---
3 | P(a,b,c,a,b,c,b,b,c,a,b,c)  =Elim: 1,2
";
    // correct: substituting once
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_9() {
    let proof = "
1 | P(a,b,c,a,b,c,a,b,c,a,b,c)
2 | a=b
  | ---
3 | P(a,b,c,a,b,c,b,b,c,b,b,c)  =Elim: 1,2
";
    // correct: substituting twice
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_10() {
    let proof = "
1 | P(a,b,c,a,b,c,a,b,c,a,b,c)
2 | a=b
  | ---
3 | P(b,b,c,a,b,c,b,b,c,b,b,c)  =Elim: 1,2
";
    // correct: substituting thrice
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_11() {
    let proof = "
1 | P(a,b,c,a,b,c,a,b,c,a,b,c)
2 | a=b
  | ---
3 | P(b,b,c,b,b,c,b,b,c,b,b,c)  =Elim: 1,2
";
    // correct: substituting four times
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_12() {
    let proof = "
1 | P(a,b,c,a,b,c,a,b,c,a,b,c)
2 | a=b
  | ---
3 | P(b,b,c,a,b,c,b,a,c,b,b,c)  =Elim: 1,2
";
    // not correct: substituting a->b three times, but also substituting b->a once
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_13() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | a=b
  | ---
3 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    // not correct: substitution needs to be applied at least once
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_14() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | a=a
  | ---
3 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_15() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | d=d
  | ---
3 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    // not correct: substitution needs to be applied at least once
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_16a() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,c,b)=f(a,c,b)
  | ---
3 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_16b() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,b,c)=f(a,b,c)
  | ---
3 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_17() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,c,b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    // not correct: substitution needs to be applied at least once
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_18() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,c,b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_19() {
    let proof = "
1 | P(f(a,f(a,c,b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,c,b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,h(a),c),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_20() {
    let proof = "
1 | P(f(a,f(a,h(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,h(a),b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_21() {
    let proof = "
1 | P(f(a,f(a,hello(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,h(a),b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_22() {
    let proof = "
1 | P(f(a,f(a,h(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,h(a),b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_23() {
    let proof = "
1 | P(f(a,f(a,h(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,h(a),b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(b)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_24() {
    let proof = "
1 | P(f(a,f(a,h(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | f(a,h(a),b)=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,hello(a)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c))))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_25() {
    let proof = "
1 | P(f(a,f(a,h(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),f(a,c,h(a)))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_26() {
    let proof = "
1 | P(a, g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))=f(a,c,h(a))
  | ---
3 | P(a, g(f(c,b,a),f(a,c,h(a)))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_27() {
    let proof = "
1 | P(f(a,f(a,c,h(a),b),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),f(a,c,h(a)))) =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_28() {
    let proof = "
1 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))))
2 | g(f(g(a,b),g(b,a),g(a,a)),h(g(a,c)))=f(a,c,h(a))
  | ---
3 | P(f(a,f(a,c,h(a)),f(a,c,f(a,a,b))), g(f(c,b,a),f(a,c,h(a)))) =Elim:1,2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_closed_term_substitution_1() {
    let proof = "
1 | ∀x ∃y P(x,y)
  | ----
2 | ∃y P(f(a),y)     ∀Elim:1
3 | ∃z ∃y P(f(a),z)  ∃Intro:2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_closed_term_substitution_2() {
    let proof = "
1 | ∀x ∃y P(x,y)
  | ----
2 | ∃y P(f(a),y)     ∀Elim:1
3 | ∃z ∃y P(z,y)  ∃Intro:2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_closed_term_substitution_3() {
    let proof = "
1 | ∀x ∃y P(x,y)
  | ----
2 | ∃y P(f(a),y)     ∀Elim:1
3 | ∃z ∃y P(f(z),y)  ∃Intro:2
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_random_stuff() {
    let proof = "

1 | A → (a=b)
2 | B → (f(g(a),h(a,b))=b)
3 | ¬¬(a=b) → ¬(P∨¬P)
4 | ¬(b=f(g(a),h(a,b)))
  | -----
5 | | A ∨ B
  | |----
6 | | | A
  | | |---
7 | | | a=b                          →Elim:1,6
8 | | | | ¬(a=b)        
  | | | | -----
9 | | | | ⊥                           ⊥Intro: 7,8
10| | | ¬¬(a=b)                       ¬Intro: 8-9
11| | | ¬(P∨¬P)                       →Elim: 3, 10
12| | | | P
  | | | |---
13| | | | P∨¬P                        ∨Intro: 12
14| | | | ⊥                           ⊥Intro:13,11
15| | | ¬P                            ¬Intro:12-14
16| | | P∨¬P                           ∨Intro: 15
17| | |⊥                               ⊥Intro: 16,11
  | |
18| | | B
  | | | ---
19| | | f(g(a),h(a,b))=b               →Elim:2,18
20| | | f(g(a),h(a,b))=f(g(a),h(a,b))  =Intro
21| | | b=f(g(a),h(a,b))               =Elim: 20,19
22| | | ⊥                              ⊥Intro: 21,4
23| | ⊥                                ∨Elim:  5,6-17, 18-22
24| ¬(A∨ B)                           ¬Intro: 5-23";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_stuff() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | [c] P(c)
  | | --
3 | | ∃y P(y)     ∃Intro: 2
4 | ∃y P(y)       ∃Elim: 1,2-3";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_conclusion_does_not_match() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | [c] P(c)
  | | --
3 | | ∃y P(y)     ∃Intro: 2
4 | ∃z P(z)       ∃Elim: 1,2-3";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_no_boxed_const() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | P(c)
  | | --
3 | | ∃y P(y)     ∃Intro: 2
4 | ∃y P(y)       ∃Elim: 1,2-3";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_no_boxed_variable() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | [z] P(z)
  | | --
3 | | ∃y P(y)     ∃Intro: 2
4 | ∃y P(y)       ∃Elim: 1,2-3";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_more_exists_stuff() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | [c] P(x)
  | | --
3 | | ∃y P(y)     ∃Intro: 2
4 | ∃y P(y)       ∃Elim: 1,2-3";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_stuff_wrong_constant_in_box() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | [d] P(c)
  | | --
3 | | ∃y P(y)     ∃Intro: 2
4 | ∃y P(y)       ∃Elim: 1,2-3";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_boxed_constant_subproof() {
    let proof = "
1 | ∃x P(x)
  | --
2 | | [c] P(c)
  | | --
3 | | P(c)        Reit: 2
4 | P(c)       ∃Elim: 1,2-3";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_boxed_const() {
    let proof = "
1 | ⊥
  | ---
2 | | [a]
  | | ---
3 | | ⊥     Reit:1
4 | ⊥       Reit:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_boxed_proposition() {
    let proof = "
1 | ⊥
  | ---
2 | | [A]
  | | ---
3 | | ⊥     Reit:1
4 | ⊥       Reit:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_boxed_var() {
    let proof = "
1 | ⊥
  | ---
2 | | [x]
  | | ---
3 | | ⊥     Reit:1
4 | ⊥       Reit:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_boxed_funcapp() {
    let proof = "
1 | ⊥
  | ---
2 | | [f(a)]
  | | ---
3 | | ⊥     Reit:1
4 | ⊥       Reit:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_boxed_funcapp_2() {
    let proof = "
1 | ⊥
  | ---
2 | | [f(x)]
  | | ---
3 | | ⊥     Reit:1
4 | ⊥       Reit:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_1() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | ∃z P(a,z)         ∃Elim:3,4-5
7 | | ∃u∃z P(u,z)       ∃Intro:6
8 |  ∃u∃z P(u,z)          ∃Elim:1,3-7
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_2() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | ∃z P(a,z)         ∃Elim:3,4-5
7 | | ∃u∃z P(u,z)       ∃Intro:6
8 |  ∃u∃z P(u,z)          ∃Elim:1,3-7
9 | a=a                =Intro
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_3() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | ∃z P(a,z)         ∃Elim:3,4-5
7 | | ∃u∃z P(u,z)       ∃Intro:6
8 |  ∃u∃z P(u,z)          ∃Elim:1,3-7
9 | b=b                =Intro
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_4() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | ∃z P(a,z)         ∃Elim:3,4-5
7 | | ∃u∃z P(u,z)       ∃Intro:6
8 |  ∃u∃z P(u,z)          ∃Elim:1,3-7
9 | a=b                ⊥Elim:2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_5() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | ∃z P(a,z)         ∃Elim:3,4-5
7 | | ∃u∃z P(u,z)       ∃Intro:6
8 | a=a                  =Intro
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-7
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_6() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | a=a              =Intro
7 | | ∃z P(a,z)         ∃Elim:3,4-5
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-8
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_7() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | ∃z P(a,z)         ∃Intro:4
6 | | b=b              =Intro
7 | | ∃z P(a,z)         ∃Elim:3,4-5
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-8
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_8() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | a=a         =Intro
6 | | | ∃z P(a,z)         ∃Intro:4
7 | | ∃z P(a,z)         ∃Elim:3,4-6
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-8
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_9() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | | [b] P(a,b)
  | | | --
5 | | | b=b         =Intro
6 | | | ∃z P(a,z)         ∃Intro:4
7 | | ∃z P(a,z)         ∃Elim:3,4-6
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-8
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_10() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | a=a                =Intro
5 | | | [b] P(a,b)
  | | | --
6 | | | ∃z P(a,z)         ∃Intro:5
7 | | ∃z P(a,z)         ∃Elim:3,5-6
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-8
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_11() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
  | ------
3 | | [a] ∃y P(a,y)
  | | --
4 | | b=b                =Intro
5 | | | [b] P(a,b)
  | | | --
6 | | | ∃z P(a,z)         ∃Intro:5
7 | | ∃z P(a,z)         ∃Elim:3,5-6
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,3-8
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_12() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
3 | a=a
  | ------
4 | | [a] ∃y P(a,y)
  | | --
5 | | | [b] P(a,b)
  | | | --
6 | | | ∃z P(a,z)         ∃Intro:5
7 | | ∃z P(a,z)         ∃Elim:4,5-6
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,4-8
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_multiple_exists_13() {
    let proof = "
1 | ∃x∃y P(x,y)
2 | ⊥
3 | b=b
  | ------
4 | | [a] ∃y P(a,y)
  | | --
5 | | | [b] P(a,b)
  | | | --
6 | | | ∃z P(a,z)         ∃Intro:5
7 | | ∃z P(a,z)         ∃Elim:4,5-6
8 | | ∃u∃z P(u,z)       ∃Intro:7
9 |  ∃u∃z P(u,z)          ∃Elim:1,4-8
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_1() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀x P(x)    ∀Intro:3-4
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_2() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀y P(y)    ∀Intro:3-4
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_3() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀y P(d)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_4() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀d P(y)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_5() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀d P(d)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_6() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | d=d      =Intro
5 | ∀x (x=x)    ∀Intro:3-4
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_7() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d]
  | | --
4 | | d=d      =Intro
5 | ∀x (d=x)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_8() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [c]
  | | --
4 | | d=d      =Intro
5 | ∀x (x=x)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_9() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  | ---
3 | | [d] d=d
  | | --
4 | | d=d      =Intro
5 | ∀x (x=x)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_1() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃u S(c,c,c)   ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_2() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃u S(u,c,c)   ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_3() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃u S(u,c,u)   ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_4() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃u S(u,u,u)   ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_5() {
    let proof = "
1 | S(c,c,u)
  | ----
2 | ∃u S(c,c,c)   ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_6() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃x S(c,c,c)   ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_7() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃a S(c,c,c)   ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_zero_or_more_8() {
    let proof = "
1 | S(c,c,c)
  | ----
2 | ∃c S(c,c,c)   ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_1() {
    let proof = "
1 | P(f(a))
  | ---
2 | ∃x P(f(x)) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_2() {
    let proof = "
1 | P(f(a))
  | ---
2 | ∃x P(x) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_3() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(x) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_4() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(x,h(a))) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_5() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(x,x)) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_6() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(a),x)) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_7() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(x)) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_8() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(x) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_9() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(x),h(a))) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_10() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(a),h(x))) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_11() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(x),h(x))) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_12() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(x),x)) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_13() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(x,h(x))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_14() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(a),h(a))) ∃Intro:1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_15() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(h(g(a),h(a))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_16() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(k(g(a),h(a))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_17() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(h(a),g(a))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_18() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(a),g(a))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_19() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(x),g(a))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_complex_term_20() {
    let proof = "
1 | P(f(g(a),h(a)))
  | ---
2 | ∃x P(f(g(x))) ∃Intro:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_elim_1() {
    let proof = "
1 | A↔B
2 | B
3 | A
  | --
4 | A     ↔ Elim:1,2
5 | B     ↔ Elim:1,4
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_elim_2() {
    let proof = "
1 | A↔B
2 | B
3 | A
  | --
4 | A     ↔ Elim:1,3
5 | B     ↔ Elim:1,4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_elim_3() {
    let proof = "
1 | A↔B
2 | B
3 | A
  | --
4 | A     ↔ Elim:2,1
5 | B     ↔ Elim:1,4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_elim_4() {
    let proof = "
1 | A↔B
2 | B
3 | A
  | --
4 | A     ↔ Elim:1,5
5 | B     ↔ Elim:1,4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_intro_1() {
    let proof = "
1 | A→B
2 | B→A
  | ---
3 | | A
  | | ---
4 | | B     →Elim: 1,3
  |
5 | | B
  | | ---
6 | | A     →Elim: 2,5
7 | A↔B     ↔Intro:3-4,5-6
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_intro_2() {
    let proof = "
1 | A→B
2 | B→A
  | ---
3 | | A
  | | ---
4 | | B     →Elim: 1,3
  |
5 | | B
  | | ---
6 | | A     →Elim: 2,5
7 | A↔B     ↔Intro:5-6,3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_intro_3() {
    let proof = "
1 | A→B
2 | B→A
  | ---
3 | | A
  | | ---
4 | | B     →Elim: 1,3
  |
5 | | B
  | | ---
6 | | A     →Elim: 2,5
7 | B↔A     ↔Intro:3-4,5-6
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_intro_4() {
    let proof = "
1 | A→B
2 | B→A
  | ---
3 | | A
  | | ---
4 | | B     →Elim: 1,3
  |
5 | | B
  | | ---
6 | | A     →Elim: 2,5
7 | B↔A     ↔Intro:5-6,3-4
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_bicond_intro_5() {
    let proof = "
1 | A→B
2 | B→A
  | ---
3 | | [a] A
  | | ---
4 | | B     →Elim: 1,3
  |
5 | | B
  | | ---
6 | | A     →Elim: 2,5
7 | B↔A     ↔Intro:5-6,3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_implies_intro_1() {
    let proof = "
1 | A→B
  | ---
2 | | A
  | | ---
3 | | B      →Elim:1,2
4 | A→B      →Intro:2-3
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_implies_intro_2() {
    let proof = "
1 | A→B
  | ---
2 | | [a] A
  | | ---
3 | | B      →Elim:1,2
4 | A→B      →Intro:2-3
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_implies_intro_3() {
    let proof = "
1 | A→B
  | ---
2 | | [A] A
  | | ---
3 | | B      →Elim:1,2
4 | A→B      →Intro:2-3
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_implies_intro_4() {
    let proof = "
1 | A→B
  | ---
2 | | A
  | | ---
3 | | B      →Elim:2,1
4 | A→B      →Intro:2-3
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}

#[test]
fn test_bonus_ai_2018_1() {
    // this proof is from the bonus question of the intro to logic exam for AI (2018)
    // see pdf here:
    // https://studysupport.svcover.nl/data//2%20Bachelor%20Courses/Introduction%20to%20Logic/Exams/AI/2018-01-29%20Solutions%20%28AI%29.pdf
    //
    // Note that there are actually 9 mistakes in the pdf:
    //
    // line 7, 8, 24, 28, 31, 32, 35, 41 and 50 have a (sometimes
    // only slightly) wrong or missing justification
    //
    // Listed hereunder is the corrected version:
    let proof = "
1  | ∀x∃y R(x,y) → ∃y∀x R(x,y)
   | ---
2  | | ¬(∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y))
   | | ---
3  | | | ∀x∃y R(x,y)
   | | | ---
4  | | | ∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y)           ∨ Intro: 3
5  | | | ⊥                                    ⊥ Intro: 4,2
6  | | ¬∀x∃y R(x,y)                           ¬ Intro: 3-5
7  | | ∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y)             ∨ Intro: 6
8  | | ⊥                                      ⊥ Intro: 7,2
9  | ¬¬(∀x∃y R(x, y) ∨ ¬∀x∃y R(x, y))         ¬ Intro: 2-8
10 | ∀x∃y R(x, y) ∨ ¬∀x∃y R(x, y)             ¬ Elim: 9
11 | | ∀x∃y R(x,y)
   | | ---
12 | | ∃y∀x R(x, y)                           → Elim: 1,11
13 | | ∃y∀x R(x, y) ∨ ∃x∀y¬R(x, y)            ∨ Intro: 12
   |
14 | | ¬∀x∃y R(x, y)
   | | ---
15 | | | ¬∃x∀y ¬R(x, y)
   | | | ---
16 | | | | [a]
   | | | | ---
17 | | | | | ∀y¬R(a, y)
   | | | | | ---
18 | | | | | ∃x∀y ¬R(x, y)                    ∃ Intro: 17
19 | | | | | ⊥                                ⊥ Intro: 18,15
20 | | | | ¬∀y ¬R(a, y)                       ¬ Intro: 17-19
21 | | | | | ¬∃y R(a, y)
   | | | | | ---
22 | | | | | | [b]
   | | | | | | ---
23 | | | | | | | R(a,b)
   | | | | | | | ---
24 | | | | | | | ∃y R(a, y)                   ∃ Intro: 23
25 | | | | | | | ⊥                            ⊥ Intro: 24, 21
26 | | | | | | ¬R(a, b)                       ¬ Intro: 23-25
27 | | | | | ∀y ¬R(a, y)                      ∀ Intro: 22-26
28 | | | | | ⊥                                ⊥ Intro: 27,20
29 | | | | ¬¬∃y R(a, y)                       ¬ Intro: 21-28
30 | | | | ∃y R(a, y)                         ¬ Elim: 29
31 | | | ∀x∃y R(x, y)                         ∀ Intro: 16-30
32 | | | ⊥                                    ⊥ Intro: 31,14
33 | | ¬¬∃x∀y ¬R(x, y)                        ¬ Intro: 15-32
34 | | ∃x∀y ¬R(x, y)                          ¬ Elim: 33
35 | | ∃y∀x R(x, y) ∨ ∃x∀y ¬R(x, y)           ∨ Intro: 34
36 | ∃y∀x R(x, y) ∨ ∃x∀y ¬R(x, y)             ∨ Elim: 10, 11-13, 14-35
37 | | ∃x∀y¬R(x, y)
   | | ---
38 | | | [c] ∀y ¬R(c, y)
   | | | ---
39 | | | ∀y¬R(c, y) ∨ ∀y R(y, c)              ∨ Intro: 38
40 | | | ∃x(∀y¬R(x, y) ∨ ∀y R(y, x))          ∃ Intro: 39
41 | | ∃x(∀y¬R(x, y) ∨ ∀y R(y, x))            ∃ Elim: 37, 38-40
   |
42 | | ∃y∀x R(x, y)
   | | ---
43 | | | [d] ∀x R(x, d)
   | | | ---
44 | | | | [e]
   | | | | ---
45 | | | | R(e,d)                             ∀ Elim: 43
46 | | | ∀y R(y, d)                           ∀ Intro: 44-45
47 | | | ∀y¬R(d, y) ∨ ∀y R(y, d)              ∨ Intro: 46
48 | | | ∃x(  ∀y¬R(x, y) ∨ ∀y R(y, x))        ∃ Intro: 47
49 | | ∃x(∀y ¬R(x, y) ∨  ∀y R(y, x))          ∃ Elim: 42, 43-48
50 | ∃x(∀y¬R(x,y) ∨ ∀y R(y,x))                ∨ Elim: 36, 42-49, 37-41
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}

#[test]
fn test_bonus_ai_2018_2_boxed_constant() {
    // this proof is from the bonus question of the intro to logic exam for AI (2018)
    // see pdf here:
    // https://studysupport.svcover.nl/data//2%20Bachelor%20Courses/Introduction%20to%20Logic/Exams/AI/2018-01-29%20Solutions%20%28AI%29.pdf
    //
    // This specific test contains a solution that is correct, except that at the end, the boxed
    // constant [d] is introduced in a subproof in which it is already introduced.
    let proof = "
1  | ∀x∃y R(x,y) → ∃y∀x R(x,y)
   | ---
2  | | ¬(∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y))
   | | ---
3  | | | ∀x∃y R(x,y)
   | | | ---
4  | | | ∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y)           ∨ Intro: 3
5  | | | ⊥                                    ⊥ Intro: 4,2
6  | | ¬∀x∃y R(x,y)                           ¬ Intro: 3-5
7  | | ∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y)             ∨ Intro: 6
8  | | ⊥                                      ⊥ Intro: 7,2
9  | ¬¬(∀x∃y R(x, y) ∨ ¬∀x∃y R(x, y))         ¬ Intro: 2-8
10 | ∀x∃y R(x, y) ∨ ¬∀x∃y R(x, y)             ¬ Elim: 9
11 | | ∀x∃y R(x,y)
   | | ---
12 | | ∃y∀x R(x, y)                           → Elim: 1,11
13 | | ∃y∀x R(x, y) ∨ ∃x∀y¬R(x, y)            ∨ Intro: 12
   |
14 | | ¬∀x∃y R(x, y)
   | | ---
15 | | | ¬∃x∀y ¬R(x, y)
   | | | ---
16 | | | | [a]
   | | | | ---
17 | | | | | ∀y¬R(a, y)
   | | | | | ---
18 | | | | | ∃x∀y ¬R(x, y)                    ∃ Intro: 17
19 | | | | | ⊥                                ⊥ Intro: 18,15
20 | | | | ¬∀y ¬R(a, y)                       ¬ Intro: 17-19
21 | | | | | ¬∃y R(a, y)
   | | | | | ---
22 | | | | | | [b]
   | | | | | | ---
23 | | | | | | | R(a,b)
   | | | | | | | ---
24 | | | | | | | ∃y R(a, y)                   ∃ Intro: 23
25 | | | | | | | ⊥                            ⊥ Intro: 24, 21
26 | | | | | | ¬R(a, b)                       ¬ Intro: 23-25
27 | | | | | ∀y ¬R(a, y)                      ∀ Intro: 22-26
28 | | | | | ⊥                                ⊥ Intro: 27,20
29 | | | | ¬¬∃y R(a, y)                       ¬ Intro: 21-28
30 | | | | ∃y R(a, y)                         ¬ Elim: 29
31 | | | ∀x∃y R(x, y)                         ∀ Intro: 16-30
32 | | | ⊥                                    ⊥ Intro: 31,14
33 | | ¬¬∃x∀y ¬R(x, y)                        ¬ Intro: 15-32
34 | | ∃x∀y ¬R(x, y)                          ¬ Elim: 33
35 | | ∃y∀x R(x, y) ∨ ∃x∀y ¬R(x, y)           ∨ Intro: 34
36 | ∃y∀x R(x, y) ∨ ∃x∀y ¬R(x, y)             ∨ Elim: 10, 11-13, 14-35
37 | | ∃x∀y¬R(x, y)
   | | ---
38 | | | [c] ∀y ¬R(c, y)
   | | | ---
39 | | | ∀y¬R(c, y) ∨ ∀y R(y, c)              ∨ Intro: 38
40 | | | ∃x(∀y¬R(x, y) ∨ ∀y R(y, x))          ∃ Intro: 39
41 | | ∃x(∀y¬R(x, y) ∨ ∀y R(y, x))            ∃ Elim: 37, 38-40
   |
42 | | ∃y∀x R(x, y)
   | | ---
43 | | | [d] ∀x R(x, d)
   | | | ---
44 | | | | [d]
   | | | | ---
45 | | | | R(d,d)                             ∀ Elim: 43
46 | | | ∀y R(y, d)                           ∀ Intro: 44-45
47 | | | ∀y¬R(d, y) ∨ ∀y R(y, d)              ∨ Intro: 46
48 | | | ∃x(  ∀y¬R(x, y) ∨ ∀y R(y, x))        ∃ Intro: 47
49 | | ∃x(∀y ¬R(x, y) ∨  ∀y R(y, x))          ∃ Elim: 42, 43-48
50 | ∃x(∀y¬R(x,y) ∨ ∀y R(y,x))                ∨ Elim: 36, 42-49, 37-41
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_bonus_ai_2018_3() {
    // this proof is from the bonus question of the intro to logic exam for AI (2018)
    // see pdf here:
    // https://studysupport.svcover.nl/data//2%20Bachelor%20Courses/Introduction%20to%20Logic/Exams/AI/2018-01-29%20Solutions%20%28AI%29.pdf
    //
    // This specific test contains a whole bunch of mistakes.
    let proof = "
1  | ∀x∃y R(x,y) → ∃y∀x R(x,y)
   | ---
2  | | ¬(∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y))
   | | ---
3  | | | ∀x∃y R(x,y)
   | | | ---
4  | | | ∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y)           ∨ Intro: 1
5  | | | ⊥                                    ⊥ Intro: 4,2
6  | | ¬∀x∃y R(x,y)                           ¬ Intro: 3-5
7  | | ∀x∃y R(x,y) ∨ ¬∀x∃y R(x,y)             ∨ Intro: 6
8  | | ⊥                                      ⊥ Intro: 7,2
9  | ¬¬(∀x∃y R(x, y) ∨ ¬∀x∃y R(x, y))         ¬ Intro: 2-8
10 | ∀x∃y R(x, y) ∨ ¬∀x∃y R(x, y)             ¬ Elim: 9
11 | | ∀x∃y R(x,y)
   | | ---
12 | | ∃y∀x R(x, y)                           → Elim: 11,1
13 | | ∃y∀x R(x, y) ∨ ∃x∀y¬R(x, y)            ∨ Intro: 12
   |
14 | | ¬∀x∃y R(x, y)
   | | ---
15 | | | ¬∃x∀y ¬R(x, y)
   | | | ---
16 | | | | [a]
   | | | | ---
17 | | | | | ∀y¬R(a, y)
   | | | | | ---
18 | | | | | ∃x∀y ¬R(x, y)                    ∃ Intro: 17
19 | | | | | ⊥                                ⊥ Intro: 18,15
20 | | | | ¬∀y ¬R(a, y)                       ¬ Intro: 17-19
21 | | | | | ¬∃y R(a, y)
   | | | | | ---
22 | | | | | | [b]
   | | | | | | ---
23 | | | | | | | R(a,b)
   | | | | | | | ---
24 | | | | | | | ∃y R(a, y)                   ∃ Intro: 23
25 | | | | | | | ⊥                            ⊥ Intro: 24, 21
26 | | | | | | ¬R(a, b)                       ¬ Intro: 23-25
27 | | | | | ∀y ¬R(a, y)                      ∀ Intro: 23-25
28 | | | | | ⊥                                ⊥ Intro: 27,20
29 | | | | ¬¬∃y R(a, y)                       ¬ Intro: 21-28
30 | | | | ∃y R(a, y)                         ¬ Elim: 25
31 | | | ∀x∃y R(x, y)                         ∀ Intro: 16-30
32 | | | ⊥                                    ⊥ Intro: 31,14
33 | | ¬¬∃x∀y ¬R(x, y)                        ¬ Intro: 15-32
34 | | ∃x∀y ¬R(x, y)                          ¬ Elim: 33
35 | | ∃y∀x R(x, y) ∨ ∃x∀y ¬R(x, y)           ∨ Intro: 34
36 | ∃y∀x R(x, y) ∨ ∃x∀y ¬R(x, y)             ∨ Elim: 10, 11-13, 14-35
37 | | ∃x∀y¬R(x, y)
   | | ---
38 | | | [c] ∀y ¬R(c, y)
   | | | ---
39 | | | ∀y¬R(c, y) ∨ ∀y R(y, c)              ∨ Intro: 38
40 | | | ∃x(∀y¬R(x, y) ∨ ∀y R(y, x))          ∃ Intro: 39
41 | | ∃x(∀y¬R(x, y) ∨ ∀y R(y, x))            ∃ Elim: 37, 38-41
   |
42 | | ∃y∀x R(x, y)
   | | ---
43 | | | [d] ∀x R(x, d)
   | | | ---
44 | | | | [d]
   | | | | ---
45 | | | | R(d,d)                             ∀ Elim: 43
46 | | | ∀y R(y, d)                           ∀ Intro: 44-45
47 | | | ∀y¬R(d, y) ∨ ∀y R(y, d)              ∨ Intro: 46
48 | | | ∃x(  ∀y¬R(x, y) ∨ ∀y R(y, x))        ∃ Intro: 47
49 | | ∃x(∀y ¬R(x, y) ∨  ∀y R(y, x))          ∃ Elim: 42, 43-48
50 | ∃x(∀y¬R(x,y) ∨ ∀y R(y,x))                ∨ Elim: 36, 37-41, 42-49
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_funny_bicond_and_no_premises() {
    let proof = "
  |--
1 | | A
  | | ---
2 | | A    Reit: 1
3 | A ↔ A  ↔Intro:1-2,1-2
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_fitch_bar_1() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
  |
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀x P(x)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_fitch_bar_2() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
3 | | [d]
  | | --
4 | | P(d)     ∀Elim:1
5 | ∀x P(x)    ∀Intro:3-4
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_fitch_bar_3() {
    let proof = "
1 | ∀x P(x)
2 | ⊥
3 | ∀x P(x)    Reit:1
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_simple() {
    let proof = "
1 | P
  | --
2 | P   Reit: 1
";

    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_1() {
    let proof = "
1 | a=b(a)
2 | ⊥
  | ---
3 | a=b    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_2() {
    let proof = "
1 | a=a(a)
2 | ⊥
  | ---
3 | a=b    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_3() {
    let proof = "
1 | a=a(a)
2 | ⊥
  | ---
3 | a=a(a)    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_4() {
    let proof = "
1 | f(a,b)=f(b,a)
2 | ⊥
  | ---
3 | f(a,b,c)=f(c,b,a)    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_5() {
    let proof = "
1 | P(a,b,c,d,e,f,g,h,i,j)
2 | ⊥
  | ---
3 | P    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_6() {
    let proof = "
1 | P(a,b,c,d,e,f,g,h,i,j)
2 | ⊥
  | ---
3 | P(a)    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_7() {
    let proof = "
1 | p(a)=p(a)
2 | ⊥
  | ---
3 | P(a,a,a,a)    ⊥Elim: 2
";

    assert!(fitch_proof::proof_is_correct(proof));
}
#[test]
fn test_arities_8() {
    let proof = "
1 | P(a,b,c,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,e,f,g,h,i,j)
2 | ⊥
  | ---
3 | f(a)=a    ⊥Elim: 2
";

    // not correct, because f is both const and funcsymb
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_arities_9() {
    let proof = "
1 | P(a,b,c,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | f(a)=a    ⊥Elim: 2
";

    // correct
    assert!(fitch_proof::proof_is_correct(proof));
}
#[test]
fn test_no_function_symbol_has_variable_name_1() {
    let proof = "
1 | P(a,b,c,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | x(a)=a    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_function_symbol_has_variable_name_2() {
    let proof = "
1 | P(a,b,c,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | x(a)=y(a)    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_function_symbol_has_variable_name_3() {
    let proof = "
1 | P(a,b,c,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | x(a)=x(a)    ⊥Elim: 2
";

    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_function_symbol_has_variable_name_4() {
    let proof = "
1 | P(a,b,c,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | x(a)=x(a,a)    ⊥Elim: 2
";
    // check that the thing does not panic, because here there are two things wrong: not only a
    // function symbol with a variable name, but also an arity mismatch.
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_function_symbol_has_variable_name_5() {
    let proof = "
1 | ∀x P(x,b,x,d,x,d,d,d,x,d,d,d,d,d,x,d,d,d,d,x,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | x(a)=x(a,a)    ⊥Elim: 2
";
    // check that the thing does not panic, because here there are two things wrong: not only a
    // function symbol with a variable name, but also an arity mismatch.
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_no_function_symbol_has_variable_name_6() {
    let proof = "
1 | ∀x P(x,b,x,d,x,d,d,d,x,d,d,d,d,d,x,d,d,d,d,x,d,d,d,d,e,g,g,h,i,j)
2 | ⊥
  | ---
3 | x(a)=f(a,a)    ⊥Elim: 2
";
    // check that the thing does not panic, because here there are two things wrong: not only a
    // function symbol with a variable name, but also an arity mismatch.
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_diff_number_of_conjs_1() {
    let proof = "
1 | a=b ∧ c=d ∧ e=f
2 | b=c
  | ---
3 | a=c ∧ c=d           =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_diff_number_of_conjs_2() {
    let proof = "
1 | a=b ∧ c=d
2 | b=c
  | ---
3 | a=c ∧ c=d ∧ e=f           =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_diff_number_of_disjs_1() {
    let proof = "
1 | a=b ∨ c=d ∨ e=f
2 | b=c
  | ---
3 | a=c ∨ c=d           =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_equals_elim_diff_number_of_disjs_2() {
    let proof = "
1 | a=b ∨ c=d 
2 | b=c
  | ---
3 | a=c ∨ c=d ∨ e=f         =Elim:1,2
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_1() {
    let proof = "
1 | ¬¬P
  |-
2 | Q ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_2() {
    let proof = "
1 | ¬¬P
  |-
2 | P ¬Elim:1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_3() {
    let proof = "
1 | ¬P
  |-
2 | P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_4() {
    let proof = "
1 | P
  |-
2 | P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_5() {
    let proof = "
1 | P
  |-
2 | ¬P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_6() {
    let proof = "
1 | P
  |-
2 | ¬¬P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_7() {
    let proof = "
1 | Q
  |-
2 | ¬¬P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_8() {
    let proof = "
1 | ¬P
  |-
2 | ¬¬P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_9() {
    let proof = "
1 | ¬¬P
  |-
2 | ¬¬P ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_10() {
    let proof = "
1 | ¬¬P(a)
  |-
2 | ¬¬P(a) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_11() {
    let proof = "
1 | ¬¬¬P(a)
  |-
2 | ¬¬P(a) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_12() {
    let proof = "
1 | ¬¬¬¬P(a)
  |-
2 | ¬¬P(a) ¬Elim:1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_13() {
    let proof = "
1 | ¬¬¬¬¬P(a)
  |-
2 | ¬¬P(a) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_14() {
    let proof = "
1 | ¬¬¬P(a)
  |-
2 | ¬P(a) ¬Elim:1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_15() {
    let proof = "
1 | ¬¬¬P(a)
  |-
2 | ¬P(b) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_16() {
    let proof = "
1 | ¬¬¬P(a)
  |-
2 | ¬P(f(a)) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_17() {
    let proof = "
1 | ¬¬¬P(f)
  |-
2 | ¬P(f(a)) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_18() {
    let proof = "
1 | ¬¬¬P(f(a))
  |-
2 | ¬P(f(a)) ¬Elim:1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_19() {
    let proof = "
1 | ¬¬¬P(f(f(a)))
  |-
2 | ¬P(f(a)) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_20() {
    let proof = "
1 | ¬¬¬P(f(f(a)))
  |-
2 | ¬P(f(f)) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_21() {
    let proof = "
1 | ¬¬¬P(f(f(a)))
  |-
2 | ¬P(f(f(b))) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_22() {
    let proof = "
1 | ¬¬¬P(f(f(a)))
  |-
2 | ¬P(f(f(f(a)))) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_23() {
    let proof = "
1 | ¬¬¬P(f(f(a)))
  |-
2 | ¬P(f(f(f(a)))) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_24() {
    let proof = "
1 | ¬¬¬P(f(f(f(a))))
  |-
2 | ¬P(f(f(f(a)))) ¬Elim:1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_25() {
    let proof = "
1 | ¬¬¬P(f(f(f(b))))
  |-
2 | ¬P(f(f(f(a)))) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_26() {
    let proof = "
1 | ¬¬¬P(f(f(f())))
  |-
2 | ¬P(f(f(f(a)))) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_not_elim_27() {
    let proof = "
1 | ¬¬¬P(f(g(f(a))))
  |-
2 | ¬P(f(f(f(a)))) ¬Elim:1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_bottom_1() {
    let proof = "
1 | A ∧ ¬A
  |----
2 | A              ∧ Elim: 1
3 | ¬A             ∧ Elim: 1
4 | ⊥              ⊥ Intro: 2, 3
5 | | [a]
  | |----
6 | | ⊥            Reit: 4
7 | ∀x ⊥           ∀ Intro: 5-6
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_bottom_2() {
    let proof = "
1 | A ∧ ¬A
  |----
2 | A              ∧ Elim: 1
3 | ¬A             ∧ Elim: 1
4 | ⊥              ⊥ Intro: 2, 3
5 | | [a]
  | |----
6 | | ⊥            Reit: 4
7 | ∀x ⊥           ∀ Intro: 5-6
8 | ⊥              ∀ Elim: 7
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_unused_variable_1() {
    let proof = "
1 | ∀x P
  |----
2 | P            ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_unused_variable_2() {
    let proof = "
1 | ∀x P
  |----
2 | Q            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_unused_variable_3() {
    let proof = "
1 | ∀x P(a)
  |----
2 | P(a)            ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_unused_variable_4() {
    let proof = "
1 | ∀x P(x)
  |----
2 | P(a)            ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_unused_variable_5() {
    let proof = "
1 | ∀x P(x)
  |----
2 | P(x)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_unused_variable_6() {
    let proof = "
1 | ∀x P(x)
  |----
2 | Q(a)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_1() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(c,a,c,b,c,a,c)           ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_2() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(d,a,d,b,d,a,d)           ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_3() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(a,a,a,b,a,a,a)           ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_4() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(b,a,b,b,b,a,b)           ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_5() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(f,a,f,b,f,a,f)           ∀ Elim: 1
";
    // fine: f is normally a function name, but in this case there is no function called f in the
    // same proof, so this is OK.
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_6() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(a,a,b,b,b,a,b)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_7() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(a,a,a,b,b,a,b)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_8a() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(a,a,a,b,a,a,b)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_8b() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(a,a,a,a,a,a,a)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_9() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(x,a,x,b,x,a,x)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_10() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(x,a,x,b,x,b,x)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_11() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(b,a,b,b,b,b,b)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_complex_12() {
    let proof = "
1 | ∀x P(x,a,x,b,x,a,x)
  |----
2 | P(a,a,b,b,c,a,d)           ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_unused_variable_1() {
    let proof = "
1 | P
  |----
2 | ∃x P         ∃ Intro: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_bottom() {
    let proof = "
1 | ⊥
  |----
2 | ∃x ⊥         ∃ Intro: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_forall_elim_needs_forall() {
    let proof = "
1 | ⊥
  |----
2 | ⊥            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_intro_needs_exists() {
    let proof = "
1 | ⊥
  |----
2 | ⊥            ∃ Intro: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_complex_forall_elim_1() {
    let proof = "
1 | ∀x∀y P(x,x,y,y,a)
  |----
2 | ∀y P(a,a,y,y,a)            ∀ Elim: 1
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_complex_forall_elim_2() {
    let proof = "
1 | ∀x∀y P(x,x,y,y,a)
  |----
2 | ∀y P(a,a,y,a,a)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_complex_forall_elim_3() {
    let proof = "
1 | ∀x∀y P(x,x,y,y,a)
  |----
2 | ∀y P(a,a,a,a,a)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_complex_forall_elim_4() {
    let proof = "
1 | ∀x∀y P(x,x,y,y,a)
  |----
2 | ∀y P(x,x,a,a,a)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_complex_forall_elim_5() {
    let proof = "
1 | ∀x∀y P(x,x,y,y,a)
  |----
2 | ∀x P(x,x,a,a,a)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_complex_forall_elim_6() {
    let proof = "
1 | ∀x∀y P(x,x,y,y,a)
  |----
2 | ∀y P(a,b,y,y,a)            ∀ Elim: 1
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_1() {
    let proof = "
1 | ∃x ⊥
  |----
2 | | [a] ⊥
  | |----
3 | | ⊥             Reit: 2
4 | ⊥               ∃ Elim: 1, 2-3
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_2() {
    let proof = "
1 | ∃x P
  |----
2 | | [a] P
  | |----
3 | | P             Reit: 2
4 | ⊥               ∃ Elim: 1, 2-3
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_3() {
    let proof = "
1 | ∃x P
  |----
2 | | [a] P
  | |----
3 | | P             Reit: 2
4 | P               ∃ Elim: 1, 2-3
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_4() {
    let proof = "
1 | ∃x P(x)
  |----
2 | | [a] P(a)
  | |----
3 | | P(a)             Reit: 2
4 | P(a)               ∃ Elim: 1, 2-3
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_5() {
    let proof = "
1 | ∃x P(x)
  |----
2 | | [a] P(a)
  | |----
3 | | P(a)             Reit: 2
4 | | ∃y P(y)          ∃ Intro: 3
5 | ∃y P(y)            ∃ Elim: 1, 2-4
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_6() {
    let proof = "
1 | ∃x P(x)
  |----
2 | | [b] P(a)
  | |----
3 | | P(a)             Reit: 2
4 | | ∃y P(y)          ∃ Intro: 3
5 | ∃y P(y)            ∃ Elim: 1, 2-4
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_7() {
    let proof = "
1 | ∃x P(x)
  |----
2 | | [b] P(b)
  | |----
3 | | P(a)             Reit: 2
4 | | ∃y P(y)          ∃ Intro: 3
5 | ∃y P(y)            ∃ Elim: 1, 2-4
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_8() {
    let proof = "
1 | ∃x ∃y P(x,x,y,y)
  |----
2 | | [a] ∃y P(a,a,y,y)
  | |----
3 | | | [b] P(a,a,b,b)
  | | |----
4 | | | ∃y P(y,y,b,b)            ∃ Intro: 3
5 | | | ∃x ∃y P(y,y,x,x)         ∃ Intro: 4
6 | | ∃x ∃y P(y,y,x,x)           ∃ Elim: 2, 3-5
7 | ∃x ∃y P(y,y,x,x)             ∃ Elim: 1, 2-6
";
    assert!(proof_is_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_9() {
    let proof = "
1 | ∃x ∃y P(x,x,y,y)
  |----
2 | | [a] ∃y P(a,a,a,a)
  | |----
3 | | | [b] P(a,a,a,a)
  | | |----
4 | | | ∃y P(y,y,a,a)            ∃ Intro: 3
5 | | | ∃x ∃y P(y,y,x,x)         ∃ Intro: 4
6 | | ∃x ∃y P(y,y,x,x)           ∃ Elim: 2, 3-5
7 | ∃x ∃y P(y,y,x,x)             ∃ Elim: 1, 2-6
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
#[test]
fn test_exists_elim_10() {
    let proof = "
1 | ∃x ∃y P(x,x,y,y)
  |----
2 | | [d] ∃y P(a,a,y,y)
  | |----
3 | | | [b] P(a,a,b,b)
  | | |----
4 | | | ∃y P(y,y,b,b)            ∃ Intro: 3
5 | | | ∃x ∃y P(y,y,x,x)         ∃ Intro: 4
6 | | ∃x ∃y P(y,y,x,x)           ∃ Elim: 2, 3-5
7 | ∃x ∃y P(y,y,x,x)             ∃ Elim: 1, 2-6
";
    assert!(proof_is_not_correct_ultra_pedantic(proof));
}
