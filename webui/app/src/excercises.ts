export default [
  {
    assumptions: ["A"],
    conclusion: "A ∨ B"
  },
  {
    assumptions: ["(P ∨ Q) → (R ∧ S)"],
    conclusion: "(P ∧ Q) → (R ∨ S)"
  },
  {
    assumptions: ["S(a,b) ∧ ¬S(b,c)", "b=a"],
    conclusion: "¬(b=c)"
  },
  {
    assumptions: ["A ∨ B", "A ∨ C"],
    conclusion: "A ∨ (B ∧ C)"
  },
  {
    assumptions: ["∀x ∀y Cuddles(x,y)"],
    conclusion: "∀x ∃y Cuddles(x,y)"
  },
  {
    assumptions: ["∃x P(x) ∨ ∃x Q(x)"],
    conclusion: "∃x (P(x) ∨ Q(x))"
  },
  {
    assumptions: ["∀x ∀y (p(x,y)=p(y,x))", "∀x (p(x,a)=x)"],
    conclusion: "∃z (p(z,b)=b)"
  },
  {
    assumptions: ["∀x (∃y R(x,y) → P(x))", "∀x ∀y ((x=y) → R(x,y))"],
    conclusion: "P(a)"
  },
  {
    assumptions: [],
    conclusion: "∀ x (x = x)"
  },
  {
    assumptions: ["∃x ∀y P(x,y)"],
    conclusion: "∀y ∃x P(x,y)"
  },
  {
    assumptions: ["R(a,b) ∧ (R(b,a) → R(c,c))", "a=b"],
    conclusion: "¬R(c,a) → ¬(a=c)"
  },
  {
    assumptions: ["(A → B) ∨ ((B → C) → C)", "A ∧ ¬B"],
    conclusion: "C"
  },
  {
    assumptions: ["∀x ∃y (P(x) → R(x,y))", "∀x ∀y ((R(x,y) ∨ (x=a)) → P(x))"],
    conclusion: "∃x R(a,x)"
  },
  {
    assumptions: ["∀x ∀y ¬R(x,y)"],
    conclusion: "¬∃x R(x,f(x))"
  },
  {
    assumptions: ["(P ∨ Q) ∧ R", "¬Q"],
    conclusion: "R ∧ (R → P)"
  },
  {
    assumptions: ["∀x (P(x) → R(a,x))"],
    conclusion: "∀x ((a=x) → ∃y R(x,y))"
  },
];
