use minijinja::{Environment, context};
use parser_lib::mermaid_state::StateGraphProgram;

use crate::r5asm::asm_error::AsmError;

/// Convert a state graph program into corresponding assembly code using a template.
pub (crate) fn from_state1_to_asm(state_graph:&StateGraphProgram) -> Result<String, AsmError> {
    let template_text = include_str!("../templates/state_machine.template");
    let mut env = Environment::new();
    env.add_template("main", &template_text).unwrap();
    let template = env.get_template("main").unwrap();

    let event = state_graph.get_all_events()
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(i, e)| format!(".equ EVENT_{e} = {i}"))
                                                    .collect::<Vec<_>>();

    let state = state_graph.get_all_states()
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(i, s)| format!(".equ STATE_{s} = {i}"))
                                                    .collect::<Vec<_>>();

    let result = template.render(context! {
        events => event,
        states => state,
    }).unwrap();

    Ok(result)
}