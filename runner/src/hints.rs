use crate::memory::Memory;
use crate::runner::Step;
use giza_core::{Felt, StarkField, Word};

use pyo3::conversion::{FromPyObject, ToPyObject};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Default)]
pub struct HintManager {
    pub hints: HashMap<u64, Vec<Hint>>,
}

impl HintManager {
    pub fn push_hint(&mut self, pc: u64, hint: Hint) {
        self.hints.entry(pc).or_default().push(hint);
    }
    pub fn get_hints(&self, pc: Felt) -> Option<&Vec<Hint>> {
        let pc: u64 = pc.as_int().try_into().unwrap();
        self.hints.get(&pc)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Hint {
    code: String,
    accessible_scopes: Vec<String>,
    flow_tracking_data: Option<FlowTrackingData>,
}

impl Hint {
    pub fn new(
        code: String,
        accessible_scopes: Vec<String>,
        flow_tracking_data: Option<FlowTrackingData>,
    ) -> Self {
        Hint {
            code,
            accessible_scopes,
            flow_tracking_data,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FlowTrackingData {
    ap_tracking: ApTracking,
    reference_ids: HashMap<String, u64>,
}

#[derive(Serialize, Deserialize)]
pub struct ApTracking {
    group: u64,
    offset: u64,
}

#[derive(Default, Debug)]
pub struct MemoryUpdate(pub Vec<(u64, Word)>);

/// Data structure containing all register and memory updates effected by hint execution
#[derive(Default, Debug)]
pub struct ExecutionEffect {
    pub pc: Felt,
    pub ap: Felt,
    pub fp: Felt,
    pub mem_updates: Option<MemoryUpdate>,
}

impl Hint {
    /// Run hint code in a Python environment, and return the aggregated effect
    /// on program state
    pub fn exec(&self, step: &Step) -> PyResult<ExecutionEffect> {
        // TODO: Import Cairo toolchain and monkey patch methods
        // (e.g. reference manager setter method) to track memory updates
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            locals.set_item(
                "pc",
                TryInto::<u64>::try_into(step.curr.pc.as_int()).unwrap(),
            )?;
            locals.set_item(
                "ap",
                TryInto::<u64>::try_into(step.curr.ap.as_int()).unwrap(),
            )?;
            locals.set_item(
                "fp",
                TryInto::<u64>::try_into(step.curr.fp.as_int()).unwrap(),
            )?;
            locals.set_item("memory", &*step.mem)?;
            locals.set_item("memory_updates", PyDict::new(py))?;
            py.run(self.code.as_str(), None, Some(&locals))
                .expect("error executing hint code");
            ExecutionEffect::from_locals(locals)
        })
    }
}

impl ExecutionEffect {
    fn from_locals(locals: &PyDict) -> PyResult<ExecutionEffect> {
        let pc = locals.get_item("pc").unwrap().extract::<u64>()?;
        let ap = locals.get_item("ap").unwrap().extract::<u64>()?;
        let fp = locals.get_item("fp").unwrap().extract::<u64>()?;
        let mem_updates: Option<MemoryUpdate> = locals
            .get_item("memory_updates")
            .unwrap()
            .extract::<MemoryUpdate>()
            .ok();
        Ok(ExecutionEffect {
            pc: Felt::from(pc),
            ap: Felt::from(ap),
            fp: Felt::from(fp),
            mem_updates,
        })
    }
}

impl<'a> FromPyObject<'a> for MemoryUpdate {
    fn extract(dict: &PyAny) -> PyResult<Self> {
        let mut mem_update = MemoryUpdate::default();
        for (key, val) in dict.downcast::<PyDict>()?.iter() {
            mem_update.0.push((
                key.extract::<u64>()?,
                Word::new(Felt::from(val.extract::<u128>()?)),
            ));
        }
        Ok(mem_update)
    }
}

impl ToPyObject for Memory {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        dict.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use giza_core::{Felt, RegisterState};

    #[test]
    fn test_hint_execution() {
        let mut memory = Memory::new(vec![]);
        memory.write(Felt::from(memory.size()), Felt::from(1u64));
        memory.write(Felt::from(memory.size()), Felt::from(2u64));
        println!("{}", memory);
        let step = Step::new(
            &mut memory,
            None,
            RegisterState::new(Felt::from(1u64), Felt::from(1u64), Felt::from(1u64)),
        );
        let hint = Hint::new(String::from("pc = 2; ap = 5; memory[1] = 10"), vec![], None);
        let res = hint.exec(&step);
        println!("res {:?}", res);
    }
}
