//! Literal copy API operations for native ownership and input-order tests.
use copy_mode::{request::Operation, Label, Labels, Request, Target};
use zellij_tile::prelude::*;

pub struct Control {
    token: u64,
    column: u64,
}

fn call(operation: Operation) -> Result<Option<copy_mode::Snapshot>, String> {
    copy_mode(Request {
        operation: Some(operation),
    })
}

impl Control {
    pub fn begin(column: u64) -> Option<Self> {
        let snapshot = call(Operation::Read(true)).ok()??;
        let control = Self {
            token: snapshot.session_id,
            column,
        };
        intercept_key_presses();
        if control.labels().is_err() {
            clear_key_presses_intercepts();
            return None;
        }
        Some(control)
    }

    fn labels(&self) -> Result<Option<copy_mode::Snapshot>, String> {
        call(Operation::SetLabels(Labels {
            session_id: self.token,
            labels: vec![Label {
                row: 0,
                column: self.column,
                text: "x".into(),
            }],
        }))
    }

    pub fn handle_key(&self, payload: &str) -> bool {
        let Some((token, key)) = payload.split_once(':') else {
            return false;
        };
        if token.parse::<u64>().ok() != Some(self.token) {
            return false;
        }
        let finished = match key {
            "a" => {
                let _ = self.labels();
                false
            },
            "x" => {
                let _ = call(Operation::MoveCursor(Target {
                    session_id: self.token,
                    row: 0,
                    column: self.column,
                }));
                true
            },
            _ => {
                let _ = call(Operation::ClearLabels(self.token));
                true
            },
        };
        if finished {
            clear_key_presses_intercepts();
        }
        finished
    }
}
