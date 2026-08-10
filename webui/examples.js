let ex1 = `1 | A ∧ B
  |----
2 | B             ∧ Elim: 1
3 | A             ∧ Elim: 1
4 | B ∧ A         ∧ Intro: 2, 3`;

let ex2 = `1 | P(a) ∧ P(b)
2 | (a=c) ∨ (b=c)
  |----
3 | | a=c
  | |----
4 | | P(a)                ∧ Elim: 1
5 | | P(c)                = Elim: 4, 3
  |
6 | | b=c
  | |----
7 | | P(b)                ∧ Elim: 1
8 | | P(c)                = Elim: 7, 6
9 | P(c)                  ∨ Elim: 2, 3-5, 6-8`;


let ex3 = `1  | ∀x ∃y R(x,y) → ∃y ∀x R(x,y)
   |----
2  | | ¬∃x (∀y ¬R(x,y) ∨ ∀y R(y,x))
   | |----
3  | | | [a]
   | | |----
4  | | | | ¬∃y R(a,y)
   | | | |----
5  | | | | | [b]
   | | | | |----
6  | | | | | | R(a,b)
   | | | | | |----
7  | | | | | | ∃y R(a,y)                       ∃ Intro: 6
8  | | | | | | ⊥                               ⊥ Intro: 7, 4
9  | | | | | ¬R(a,b)                           ¬ Intro: 6-8
10 | | | | ∀y ¬R(a,y)                          ∀ Intro: 5-9
11 | | | | ∀y ¬R(a,y) ∨ ∀y R(y,a)              ∨ Intro: 10
12 | | | | ∃x (∀y ¬R(x,y) ∨ ∀y R(y,x))         ∃ Intro: 11
13 | | | | ⊥                                   ⊥ Intro: 12, 2
14 | | | ¬¬∃y R(a,y)                           ¬ Intro: 4-13
15 | | | ∃y R(a,y)                             ¬ Elim: 14
16 | | ∀x ∃y R(x,y)                            ∀ Intro: 3-15
17 | | ∃y ∀x R(x,y)                            → Elim: 1,16
18 | | | [c] ∀x R(x,c)
   | | |----
19 | | | | [d]
   | | | |----
20 | | | | R(d,c)                              ∀ Elim: 18
21 | | | ∀y R(y,c)                             ∀ Intro: 19-20
22 | | | ∀y ¬R(c,y) ∨ ∀y R(y,c)                ∨ Intro: 21
23 | | | ∃x (∀y ¬R(x,y) ∨ ∀y R(y,x))           ∃ Intro: 22
24 | | | ⊥                                     ⊥ Intro: 23, 2
25 | | ⊥                                       ∃ Elim: 17, 18-24
26 | ¬¬∃x (∀y ¬R(x,y) ∨ ∀y R(y,x))             ¬ Intro: 2-25
27 | ∃x (∀y ¬R(x,y) ∨ ∀y R(y,x))               ¬ Elim: 26`;
