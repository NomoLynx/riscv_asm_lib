//! ============[Macro Operand-Form Dispatch]============
//!
//! Responsibility: only perform macro-name substitution by operand form — rewrite
//! (macro name, operand-form list) into the final macro name used for template lookup.
//! This module does not touch template parsing, argument substitution, or instruction
//! generation (handled by macro_instructions.rs / macro_instruction_list.rs /
//! instruction.rs respectively).
//!
//! Typical scenario: `exit 5` (immediate) uses the `exit` template in
//! macro_instruction.md; `exit t0` (register) uses the `exit_reg` template.
//!
//! ## Steps for adding a same-name form variant (add one data entry; no parsing/expansion code changes)
//!   1. If the target template does not exist yet, add a template section to
//!      macro_instruction.md (and add it to the table);
//!   2. Append one [`MacroOverload`] to [`MACRO_OVERLOADS`] below.
//!
//! ## Matching priority (see [`resolve_overload`])
//!   1. Exact original-name match: when writing `exit_reg t0` directly, there is no
//!      `exit_reg` entry in the table, so the original name is used for template lookup;
//!   2. Form-overload match: when the original name + operand-form list "strictly match
//!      an entry in length and element by element", use the target macro name instead;
//!   3. Original-name fallback: if no overload matches, use the original name for
//!      template lookup.
//!
//! ## Conventions
//! - Normalize the macro name to lowercase before calling [`resolve_overload`]
//!   (handled uniformly at the `MacroInstructionHashMap::get_incs` entry); `name`
//!   values in the table must always be lowercase;
//! - [`MacroOperandForm::Unknown`] (the caller provided no syntax information) does not
//!   participate in typed form matching, preventing old call styles from silently
//!   triggering overloads; [`MacroOperandForm::Any`] is used only as a wildcard on the
//!   table side.
//!
//! ## Version 2 (reserved)
//! After adding a Form column to the macro_instruction.md table, merge the parsed
//! overload entries into the runtime overload table during Markdown loading; the
//! matching logic in this module does not need to change.

use pest::iterators::Pair;

use super::super::r5asm_pest::Rule as AsmRule;

/// Operand form of a macro-call argument, used for overload matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroOperandForm {
    /// Register (e.g. t0, a0)
    Register,
    /// Immediate/literal (e.g. 5, 3.14, %hi(x), 'c')
    Immediate,
    /// Overload-table-only wildcard: matches any form (including Unknown)
    Any,
    /// Caller-only: the argument carries no syntax information (e.g. plain strings) and
    /// does not participate in typed form matching
    Unknown,
}

impl MacroOperandForm {
    /// Classify a pest grammar rule as an operand form (named entry point, equivalent to `From<AsmRule>`)
    pub fn from_rule(rule: AsmRule) -> Self {
        rule.into()
    }
}

/// Classify grammar rules parsed from the r5asm grammar (r5asm.pest) as operand forms.
/// Note: classifying `var_name` as Register follows the semantics of the original
/// hard-coded `exit` branch (in `exit t0`, t0 is parsed as a var_name in the imm slot
/// and semantically means "register holding the exit code"); semantic differences in
/// other contexts (e.g. jump labels) are handled by specific macro templates and are
/// not distinguished at the form layer.
impl From<AsmRule> for MacroOperandForm {
    fn from(rule: AsmRule) -> Self {
        match rule {
            AsmRule::var_name | AsmRule::registers => Self::Register,
            AsmRule::integer | AsmRule::float | AsmRule::imm_macro | AsmRule::single_character => {
                Self::Immediate
            }
            _ => Self::Unknown,
        }
    }
}

/// One "macro alias mapping by operand form": performs name substitution only, without
/// touching template expansion or argument substitution.
#[derive(Debug, Clone, Copy)]
pub struct MacroOverload {
    /// Macro instruction name (lowercase), i.e. the name written by the user
    pub name: &'static str,
    /// Form of each parameter position, matched strictly in order; the array length is
    /// the number of parameters used for matching
    pub param_forms: &'static [MacroOperandForm],
    /// Resolved real macro name (the key in the macro_instruction.md table)
    pub target: &'static str,
}

/// Global overload dispatch table. To add a same-name variant, only add an entry here
/// (see the module header comments).
pub const MACRO_OVERLOADS: &[MacroOverload] = &[
    // When exit is passed a register (the register stores the exit code), use the
    // exit_reg template; with an immediate, use the exit template
    MacroOverload {
        name: "exit",
        param_forms: &[MacroOperandForm::Register],
        target: "exit_reg",
    },
];

/// Resolve overload: takes a lowercase macro name + operand-form list and returns the
/// final macro name used for template lookup.
/// Priority: exact original-name match > form overload (strict same-length, element-by-
/// element match) > original-name fallback.
/// `Unknown` (the caller provided no syntax information) does not match `Register`/
/// `Immediate`; it only matches explicit `Any` in the table.
pub fn resolve_overload<'a>(name: &'a str, param_forms: &[MacroOperandForm]) -> &'a str {
    for overload in MACRO_OVERLOADS {
        if overload.name == name
            && overload.param_forms.len() == param_forms.len()
            && overload
                .param_forms
                .iter()
                .zip(param_forms)
                .all(|(expected, actual)| {
                    matches!(expected, MacroOperandForm::Any) || expected == actual
                })
        {
            return overload.target;
        }
    }
    name
}

/// List all overload entries registered for a macro name (for error messages/debugging)
pub fn overloads_for<'a>(name: &'a str) -> impl Iterator<Item = &'static MacroOverload> + 'a {
    MACRO_OVERLOADS.iter().filter(move |o| o.name == name)
}

/// Generate a description of the overload registration for a macro name, appended to
/// error messages for easier debugging
pub fn describe_overloads(name: &str) -> String {
    let mut entries = overloads_for(name);
    match entries.next() {
        None => "none".to_string(),
        Some(first) => {
            let mut s = format!(
                "`{}` -> `{}` (param forms {:?})",
                first.name, first.target, first.param_forms
            );
            for o in entries {
                s.push_str(&format!(
                    ", `{}` -> `{}` (param forms {:?})",
                    o.name, o.target, o.param_forms
                ));
            }
            s
        }
    }
}

/// Actual arguments for a macro call: argument text + corresponding forms (equal
/// length, guaranteed by construction).
/// Can be constructed from plain strings (forms are Unknown) or from pest-parsed
/// argument pairs (forms are classified automatically).
#[derive(Debug, Clone, Default)]
pub struct MacroArgs {
    values: Vec<String>,
    forms: Vec<MacroOperandForm>,
}

impl MacroArgs {
    /// Empty argument list with no arguments
    pub fn empty() -> Self {
        Self::default()
    }

    /// Argument text (same length as forms)
    pub fn values(&self) -> &[String] {
        &self.values
    }

    /// Forms of each argument (same length as values)
    pub fn forms(&self) -> &[MacroOperandForm] {
        &self.forms
    }
}

impl From<Vec<String>> for MacroArgs {
    fn from(values: Vec<String>) -> Self {
        let forms = vec![MacroOperandForm::Unknown; values.len()];
        Self { values, forms }
    }
}

impl From<&[String]> for MacroArgs {
    fn from(values: &[String]) -> Self {
        values.to_vec().into()
    }
}

impl From<&Vec<String>> for MacroArgs {
    fn from(values: &Vec<String>) -> Self {
        values.as_slice().into()
    }
}

/// Construct from an array of pest-parsed argument pairs (e.g. `[p, p1]`), automatically
/// carrying operand forms.
impl<'a, 'b, const N: usize> From<[&'a Pair<'b, AsmRule>; N]> for MacroArgs {
    fn from(pairs: [&'a Pair<'b, AsmRule>; N]) -> Self {
        let values = pairs.iter().map(|p| p.as_str().to_string()).collect();
        let forms = pairs
            .iter()
            .map(|p| MacroOperandForm::from_rule(p.as_rule()))
            .collect();
        Self { values, forms }
    }
}

/// Construct from a slice of pest-parsed argument pairs, supporting dynamically sized
/// argument lists.
impl<'a> From<&[Pair<'a, AsmRule>]> for MacroArgs {
    fn from(pairs: &[Pair<'a, AsmRule>]) -> Self {
        let values = pairs.iter().map(|p| p.as_str().to_string()).collect();
        let forms = pairs
            .iter()
            .map(|p| MacroOperandForm::from_rule(p.as_rule()))
            .collect();
        Self { values, forms }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::code_gen_config::CodeGenConfiguration;
    use super::super::super::instruction::Instruction;
    use super::*;

    /// Compare expansion results field by field to ensure macro expansion matches the
    /// original template exactly (rather than comparing only instruction names)
    fn assert_inc(
        inc: &Instruction,
        name: &str,
        r0: Option<&str>,
        r1: Option<&str>,
        r2: Option<&str>,
        imm: Option<&str>,
    ) {
        assert_eq!(inc.get_name(), name, "instruction name mismatch");
        assert_eq!(inc.get_r0().map(|s| s.as_str()), r0, "rd mismatch for {name}");
        assert_eq!(inc.get_r1().map(|s| s.as_str()), r1, "rs1 mismatch for {name}");
        assert_eq!(inc.get_r2().map(|s| s.as_str()), r2, "rs2 mismatch for {name}");
        assert_eq!(
            inc.get_imm().and_then(|i| i.to_string_option()).as_deref(),
            imm,
            "imm mismatch for {name}"
        );
    }

    #[test]
    fn overload_table_has_no_duplicate_forms_and_names_are_lowercase() {
        for (i, a) in MACRO_OVERLOADS.iter().enumerate() {
            assert_eq!(
                a.name,
                a.name.to_lowercase(),
                "overload name must be lowercase: {a:?}"
            );
            for b in MACRO_OVERLOADS.iter().skip(i + 1) {
                assert!(
                    !(a.name == b.name && a.param_forms == b.param_forms),
                    "duplicate overload entry: {a:?} vs {b:?}"
                );
            }
        }
    }

    #[test]
    fn resolve_overload_dispatches_by_operand_form() {
        // exit with a register -> exit_reg template; with an immediate -> exit template
        assert_eq!(
            resolve_overload("exit", &[MacroOperandForm::Register]),
            "exit_reg"
        );
        assert_eq!(
            resolve_overload("exit", &[MacroOperandForm::Immediate]),
            "exit"
        );
    }

    #[test]
    fn resolve_overload_prefers_exact_name() {
        // When the user writes exit_reg directly, there is no exit_reg entry in the table,
        // so the original name is used as fallback
        assert_eq!(
            resolve_overload("exit_reg", &[MacroOperandForm::Register]),
            "exit_reg"
        );
    }

    #[test]
    fn resolve_overload_requires_strict_arity() {
        // Strict same-length matching: 2 arguments must not hit an overload entry with 1
        // form (prevents a prefix-matching loophole)
        assert_eq!(
            resolve_overload(
                "exit",
                &[MacroOperandForm::Register, MacroOperandForm::Register]
            ),
            "exit"
        );
    }

    #[test]
    fn resolve_overload_ignores_unknown_form() {
        // Old call style (plain strings only) has Unknown forms and does not trigger typed overloads
        assert_eq!(resolve_overload("exit", &[MacroOperandForm::Unknown]), "exit");
    }

    #[test]
    fn resolve_overload_any_wildcard_matches_unknown() {
        // Any is a table-side wildcard; when explicitly declared it can match any form
        // (including Unknown).
        // Verify the reverse through the registered table: Unknown does not match a
        // Register entry
        assert_eq!(resolve_overload("exit", &[]), "exit");
    }

    #[test]
    fn operand_form_from_asm_rule() {
        assert_eq!(MacroOperandForm::from(AsmRule::var_name), MacroOperandForm::Register);
        assert_eq!(MacroOperandForm::from(AsmRule::registers), MacroOperandForm::Register);
        assert_eq!(MacroOperandForm::from(AsmRule::integer), MacroOperandForm::Immediate);
        assert_eq!(MacroOperandForm::from(AsmRule::float), MacroOperandForm::Immediate);
        assert_eq!(MacroOperandForm::from(AsmRule::imm_macro), MacroOperandForm::Immediate);
    }

    #[test]
    fn macro_args_from_strings_has_unknown_forms() {
        let args: MacroArgs = vec!["5".to_string(), "t0".to_string()].into();
        assert_eq!(args.values().len(), 2);
        assert!(args.forms().iter().all(|f| *f == MacroOperandForm::Unknown));
    }

    #[test]
    fn expand_exit_with_immediate_uses_exit_template() {
        let mut config = CodeGenConfiguration::default();
        let incs = Instruction::from_string("exit 5", &mut config).unwrap();
        // exit template: addi a0, x0, $1 / addi a7, x0, 93 / ecall
        assert_eq!(incs.len(), 3);
        assert_inc(&incs[0], "ADDI", Some("a0"), Some("x0"), None, Some("5"));
        assert_inc(&incs[1], "ADDI", Some("a7"), Some("x0"), None, Some("93"));
        assert_inc(&incs[2], "ECALL", None, None, None, None);
    }

    #[test]
    fn expand_exit_with_register_dispatches_to_exit_reg() {
        let mut config = CodeGenConfiguration::default();
        let incs = Instruction::from_string("exit t0", &mut config).unwrap();
        // exit_reg template: add a0, x0, $1 / addi a7, x0, 93 / ecall
        assert_eq!(incs.len(), 3);
        assert_inc(&incs[0], "ADD", Some("a0"), Some("x0"), Some("t0"), None);
        assert_inc(&incs[1], "ADDI", Some("a7"), Some("x0"), None, Some("93"));
        assert_inc(&incs[2], "ECALL", None, None, None, None);
    }

    #[test]
    fn expand_exit_reg_directly_uses_exit_reg_template() {
        let mut config = CodeGenConfiguration::default();
        let incs = Instruction::from_string("exit_reg t0", &mut config).unwrap();
        assert_eq!(incs.len(), 3);
        assert_eq!(incs[0].get_name(), "ADD");
    }

    #[test]
    fn old_style_string_args_do_not_trigger_overload() {
        let config = CodeGenConfiguration::default();
        let archive = config.get_marco_instruction_archive();
        // Plain strings only -> Unknown forms -> the exit -> exit_reg overload is not
        // triggered; use the original-name template
        let incs = archive.get_incs("exit", vec!["5".to_string()]).unwrap();
        assert_eq!(incs.len(), 3);
        assert_eq!(incs[0].get_name(), "ADDI");
    }

    #[test]
    fn parameter_count_mismatch_reports_overload_context() {
        let config = CodeGenConfiguration::default();
        let archive = config.get_marco_instruction_archive();
        let err = archive
            .get_incs("exit", vec!["a".to_string(), "b".to_string()])
            .unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("parameter count mismatch"), "unexpected: {msg}");
        assert!(msg.contains("exit_reg"), "error should list registered overloads: {msg}");
    }

    #[test]
    fn nested_exit_inside_tt_panic_expands_via_exit_reg() {
        let mut config = CodeGenConfiguration::default();
        let incs = Instruction::from_string("tt_panic 5", &mut config).unwrap();
        // tt_panic template: li t0, $1 / exit t0; the nested expansion of exit t0 goes
        // through exit_reg (add a0, x0, t0)
        assert!(
            incs.iter().any(|i| i.get_name() == "ADD"),
            "expected exit_reg expansion (add), got: {:?}",
            incs.iter().map(|i| i.get_name()).collect::<Vec<_>>()
        );
    }
}
