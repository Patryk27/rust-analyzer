use crate::diagnostics::{Diagnostic, DiagnosticsContext};

// Diagnostic: macro-error
//
// This diagnostic is shown for macro expansion errors.
pub(super) fn macro_error(ctx: &DiagnosticsContext<'_>, d: &hir::MacroError) -> Diagnostic {
    Diagnostic::new(
        "macro-error",
        d.message.clone(),
        ctx.sema.diagnostics_display_range(d.node.clone()).range,
    )
    .experimental()
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::tests::{check_diagnostics, check_no_diagnostics};

    #[test]
    fn builtin_macro_fails_expansion() {
        check_diagnostics(
            r#"
#[rustc_builtin_macro]
macro_rules! include { () => {} }

  include!("doesntexist");
//^^^^^^^^^^^^^^^^^^^^^^^^ failed to load file `doesntexist`
            "#,
        );
    }

    #[test]
    fn include_macro_should_allow_empty_content() {
        check_diagnostics(
            r#"
//- /lib.rs
#[rustc_builtin_macro]
macro_rules! include { () => {} }

include!("foo/bar.rs");
//- /foo/bar.rs
// empty
"#,
        );
    }

    #[test]
    fn good_out_dir_diagnostic() {
        check_diagnostics(
            r#"
#[rustc_builtin_macro]
macro_rules! include { () => {} }
#[rustc_builtin_macro]
macro_rules! env { () => {} }
#[rustc_builtin_macro]
macro_rules! concat { () => {} }

  include!(concat!(env!("OUT_DIR"), "/out.rs"));
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `OUT_DIR` not set, enable "run build scripts" to fix
"#,
        );
    }

    #[test]
    fn register_attr_and_tool() {
        cov_mark::check!(register_attr);
        cov_mark::check!(register_tool);
        check_no_diagnostics(
            r#"
#![register_tool(tool)]
#![register_attr(attr)]

#[tool::path]
#[attr]
struct S;
"#,
        );
        // NB: we don't currently emit diagnostics here
    }

    #[test]
    fn macro_diag_builtin() {
        check_diagnostics(
            r#"
#[rustc_builtin_macro]
macro_rules! env {}

#[rustc_builtin_macro]
macro_rules! include {}

#[rustc_builtin_macro]
macro_rules! compile_error {}

#[rustc_builtin_macro]
macro_rules! format_args { () => {} }

fn main() {
    // Test a handful of built-in (eager) macros:

    include!(invalid);
  //^^^^^^^^^^^^^^^^^ could not convert tokens
    include!("does not exist");
  //^^^^^^^^^^^^^^^^^^^^^^^^^^ failed to load file `does not exist`

    env!(invalid);
  //^^^^^^^^^^^^^ could not convert tokens

    env!("OUT_DIR");
  //^^^^^^^^^^^^^^^ `OUT_DIR` not set, enable "run build scripts" to fix

    compile_error!("compile_error works");
  //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ compile_error works

    // Lazy:

    format_args!();
  //^^^^^^^^^^^^^^ no rule matches input tokens
}
"#,
        );
    }

    #[test]
    fn macro_rules_diag() {
        check_diagnostics(
            r#"
macro_rules! m {
    () => {};
}
fn f() {
    m!();

    m!(hi);
  //^^^^^^ leftover tokens
}
      "#,
        );
    }
    #[test]
    fn dollar_crate_in_builtin_macro() {
        check_diagnostics(
            r#"
#[macro_export]
#[rustc_builtin_macro]
macro_rules! format_args {}

#[macro_export]
macro_rules! arg { () => {} }

#[macro_export]
macro_rules! outer {
    () => {
        $crate::format_args!( "", $crate::arg!(1) )
    };
}

fn f() {
    outer!();
} //^^^^^^^^ leftover tokens
"#,
        )
    }
}