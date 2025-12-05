//! Interpreter for Draca.
//!
//! ```lisp
//! (define-global-fn ('+ first second)
//!   (rs::builtin/comp-add first second)
//! )
//!
//! (define-global-fn ('first list)
//!   (rs::builtin/cons-first list)
//! )
//!
//! (define-fn-namespace ('std 'io) ('puts char)
//!   (rs::builtin/io-puts char)
//! )
//! ```

pub mod env;

pub mod interp_sexpr;
