//! Hand-rolled CSS engine — tokenizer, parser, selectors, cascade.
//!
//! See `docs/v0.3.35-html-css-pdf-plan.md` for the supported CSS surface
//! and the rationale for hand-rolling instead of depending on the
//! Mozilla stack (MPL-2.0 — denied by `deny.toml`).

pub mod at_rules;
pub mod calc;
pub mod cascade;
pub mod counters;
pub mod matcher;
pub mod parser;
pub mod selectors;
pub mod tokenizer;
pub mod values;
pub mod var;

pub use at_rules::{
    resolve as resolve_stylesheet, FontFaceDescriptor, MediaContext, PageRule, PageSelector,
    ResolvedStylesheet, SrcEntry,
};
pub use calc::{
    evaluate as evaluate_calc_expr, evaluate_function as evaluate_calc_function,
    parse_expr as parse_calc_expr, Context as CalcContext, EvalError as CalcError,
    Expr as CalcExpr, Unit,
};
pub use cascade::{
    apply_inline_declarations, cascade, initial_value, pseudo_content_for, ComputedStyles,
    PseudoKind, ResolvedValue,
};
pub use counters::{
    evaluate_content, parse_content, parse_counter_ops, Content, CounterOp, CounterState, ListStyle,
};
pub use matcher::{match_complex_selector, match_selector_list, Element};
pub use parser::{
    parse_declaration_list, parse_stylesheet, AtRule, AtRuleBlock, ComponentValue, Declaration,
    QualifiedRule, Rule, Stylesheet,
};
pub use selectors::{
    parse_selector_list, AnPlusB, AttributeCase, AttributeOp, AttributeSelector, Combinator,
    ComplexSelector, CompoundSelector, ElementSelector, PseudoClass, PseudoElement, SelectorList,
    SelectorParseError, Specificity, SubclassSelector,
};
pub use tokenizer::{tokenize, SourceLocation, Token, TokenizerError};
pub use values::{
    parse_color, parse_length, parse_property, Color, Length, ParseError as ValueParseError, Value,
};
pub use var::{resolve_custom_properties, substitute as substitute_vars, VarError};
