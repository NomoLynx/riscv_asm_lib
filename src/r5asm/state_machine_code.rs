use minijinja::{Environment, context};
use parser_lib::mermaid_state::StateGraphProgram;

use crate::r5asm::asm_error::AsmError;

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
    let result = template.render(context! {
        events => event,
    }).unwrap();

    Ok(result)
}