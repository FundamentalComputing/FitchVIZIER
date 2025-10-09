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
];
