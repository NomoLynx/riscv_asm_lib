use std::collections::HashMap;

use minijinja::{Environment, context};
use parser_lib::mermaid_state::StateGraphProgram;
use rust_macro::Accessors;

use crate::r5asm::asm_error::AsmError;

/// Convert a state graph program into corresponding assembly code using a template.
pub fn from_state1_to_asm(state_graph:&StateGraphProgram) -> Result<String, AsmError> {
    let template_text = include_str!("../templates/state_machine.template");
    let mut env = Environment::new();
    env.add_template("main", &template_text).unwrap();
    let template = env.get_template("main").unwrap();

    let event_prefix = "EVENT_";
    let state_prefix = "STATE_";

    let event = state_graph.get_all_events()
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(i, e)| format!(".equ {event_prefix}{e} = {i}"))
                                                    .collect::<Vec<_>>();

    let state = state_graph.get_all_states()
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(i, s)| format!(".equ {state_prefix}{s} = {i}"))
                                                    .collect::<Vec<_>>();

    let state_fn = state_graph.get_all_states()
                                                    .iter()
                                                    .map(|s| format!(".word {state_prefix}{s}"))
                                                    .collect::<Vec<_>>();

    let mut result = template.render(context! {
        events => event,
        states => state,
        state_fns => state_fn,
    }).unwrap();

    for state in state_graph.get_all_states() {
        
        let state_code = StateMachineStateCode::new_from_state_diagram(state_graph, &state);
        let current_state_name = format!("{}{}", state_code.get_state_prefix(), state);
        let transitions = state_code.to_transitions();
        let states = state_code.to_states();
        let event_states = state_code.get_target_event_state()
                                                .iter()
                                                .map(|(event, target) | format!("{}{event} ->{}{target}", state_code.get_event_prefix(), state_code.get_state_prefix()))
                                                .collect::<Vec<_>>();

        let template_text = include_str!("../templates/state_machine_event.template");
        let mut env = Environment::new();
        env.add_template("state", &template_text).unwrap();
        let template = env.get_template("state").unwrap();

        let result_state = template.render(context! {
            event_states => event_states,
            current_state_name => current_state_name,
            transitions => transitions,
            states => states,
        }).unwrap();

        result.push_str(&result_state);
    }

    Ok(result)
}

#[derive(Accessors)]
pub struct StateMachineStateCode {
    state_name : String,
    target_event_state : HashMap<String, String>,
    state_prefix : String,
    event_prefix : String,
}

impl StateMachineStateCode {
    pub fn new(state_name:&str, target_event_states:HashMap<String, String>) -> Self {
        Self { 
            state_name : state_name.to_string(), 
            target_event_state : target_event_states,
            state_prefix : "STATE_".to_string(),
            event_prefix : "EVENT_".to_string(),
        }
    }

    pub fn new_from_state_diagram(diagram:&StateGraphProgram, state_name:&str) -> Self {
        let stmts = diagram.find_stmt_by_state(state_name);
        
        let mut target_event_states = HashMap::default();
        for stmt in stmts {
            if let Some((event, target)) = stmt.get_event_target_state() {
                target_event_states.insert(event, target);
            }
        }

        Self::new(state_name, target_event_states)
    }

    fn get_current_state_event_name(&self, event:&str) -> String {
        format!("{}_{}", self.get_state_name(), event)
    }

    pub fn to_transitions(&self) -> Vec<String> {
        let mut result = vec![];
        for (event, _target_state) in self.get_target_event_state() {
            let event_name = format!("{}{}", self.get_event_prefix(), event);
            let current_state_event = self.get_current_state_event_name(event);
            let code = format!("li   t1, {event_name}\n\tbeq  t0, t1, {current_state_event}");
            result.push(code);
        }

        result
    }

    pub fn to_states(&self) -> Vec<String> {
        let mut result = vec![];
        for (event, target_state) in self.get_target_event_state() {
            let current_state_event = self.get_current_state_event_name(event);
            let target_state_name = format!("{}{}", self.get_state_prefix(), target_state);
            let code = format!("{current_state_event}:\n\tli s0, {target_state_name}\n\tj dispatch");
            result.push(code);
        }

        result
    }
}