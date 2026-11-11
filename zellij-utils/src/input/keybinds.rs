use std::collections::{BTreeMap, HashMap};

use super::actions::Action;
use crate::data::{BareKey, InputMode, KeyWithModifier, KeybindsVec};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Used in the config struct
#[derive(Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct Keybinds(pub HashMap<InputMode, HashMap<KeyWithModifier, Vec<Action>>>);

impl fmt::Debug for Keybinds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut stable_sorted = BTreeMap::new();
        for (mode, keybinds) in self.0.iter() {
            let mut stable_sorted_mode_keybinds = BTreeMap::new();
            for (key, actions) in keybinds {
                stable_sorted_mode_keybinds.insert(key, actions);
            }
            stable_sorted.insert(mode, stable_sorted_mode_keybinds);
        }
        write!(f, "{:#?}", stable_sorted)
    }
}

impl Keybinds {
    pub fn get_actions_for_key_in_mode(
        &self,
        mode: &InputMode,
        key: &KeyWithModifier,
    ) -> Option<&Vec<Action>> {
        self.0
            .get(mode)
            .and_then(|normal_mode_keybindings| normal_mode_keybindings.get(key))
    }
    pub fn get_actions_for_key_in_mode_or_default_action(
        &self,
        mode: &InputMode,
        key_with_modifier: &KeyWithModifier,
        raw_bytes: Vec<u8>,
        default_input_mode: InputMode,
        key_is_kitty_protocol: bool,
    ) -> Vec<Action> {
        self.0
            .get(mode)
            .and_then(|mode_keybindings| {
                if raw_bytes == &[10] {
                    handle_ctrl_j(&mode_keybindings, &raw_bytes, key_is_kitty_protocol)
                } else {
                    mode_keybindings.get(key_with_modifier).cloned()
                }
            })
            .unwrap_or_else(|| {
                vec![self.default_action_for_mode(
                    mode,
                    Some(key_with_modifier),
                    raw_bytes,
                    default_input_mode,
                    key_is_kitty_protocol,
                )]
            })
    }
    pub fn get_input_mode_mut(
        &mut self,
        input_mode: &InputMode,
    ) -> &mut HashMap<KeyWithModifier, Vec<Action>> {
        self.0.entry(*input_mode).or_insert_with(HashMap::new)
    }
    pub fn default_action_for_mode(
        &self,
        mode: &InputMode,
        key_with_modifier: Option<&KeyWithModifier>,
        raw_bytes: Vec<u8>,
        default_input_mode: InputMode,
        key_is_kitty_protocol: bool,
    ) -> Action {
        match *mode {
            InputMode::Copy => Action::CopyModeInput {
                key: key_with_modifier
                    .map(|key| {
                        if let BareKey::Char(c) = key.bare_key {
                            if key.key_modifiers.len() == 1
                                && key.key_modifiers.contains(&crate::data::KeyModifier::Shift)
                            {
                                return c.to_uppercase().collect();
                            }
                        }
                        key.to_string()
                    })
                    .unwrap_or_default(),
            },
            InputMode::Locked => Action::Write {
                key_with_modifier: key_with_modifier.cloned(),
                bytes: raw_bytes,
                is_kitty_keyboard_protocol: key_is_kitty_protocol,
            },
            mode if mode == default_input_mode => Action::Write {
                key_with_modifier: key_with_modifier.cloned(),
                bytes: raw_bytes,
                is_kitty_keyboard_protocol: key_is_kitty_protocol,
            },
            InputMode::RenameTab => Action::TabNameInput { input: raw_bytes },
            InputMode::RenamePane => Action::PaneNameInput { input: raw_bytes },
            InputMode::RenameSession => Action::SessionNameInput { input: raw_bytes },
            InputMode::EnterSearch => Action::SearchInput { input: raw_bytes },
            _ => Action::NoOp,
        }
    }
    pub fn to_keybinds_vec(&self) -> KeybindsVec {
        let mut ret = vec![];
        for (mode, mode_binds) in &self.0 {
            let mut mode_binds_vec: Vec<(KeyWithModifier, Vec<Action>)> = vec![];
            for (key, actions) in mode_binds {
                mode_binds_vec.push((key.clone(), actions.clone()));
            }
            ret.push((*mode, mode_binds_vec))
        }
        ret
    }
    pub fn merge(&mut self, mut other: Keybinds) {
        for (other_input_mode, mut other_input_mode_keybinds) in other.0.drain() {
            let input_mode_keybinds = self
                .0
                .entry(other_input_mode)
                .or_insert_with(|| Default::default());
            for (other_action, other_action_keybinds) in other_input_mode_keybinds.drain() {
                input_mode_keybinds.insert(other_action, other_action_keybinds);
            }
        }
    }
}

const MAX_SHORTCUT_SEARCH_DEPTH: usize = 4;

pub fn shortcut_for_action(
    keybinds: &KeybindsVec,
    base_mode: InputMode,
    is_target_action: impl Fn(&Action) -> bool,
) -> Option<Vec<KeyWithModifier>> {
    let mode_binds = |mode: InputMode| -> Vec<(KeyWithModifier, Vec<Action>)> {
        let mut binds = keybinds
            .iter()
            .find(|(bind_mode, _)| *bind_mode == mode)
            .map(|(_, binds)| binds.clone())
            .unwrap_or_default();
        binds.sort_by(|(a, _), (b, _)| a.cmp(b));
        binds
    };
    let mut visited: Vec<InputMode> = vec![base_mode];
    let mut queue: Vec<(InputMode, Vec<KeyWithModifier>)> = vec![(base_mode, vec![])];
    for _ in 0..MAX_SHORTCUT_SEARCH_DEPTH {
        let mut next_queue: Vec<(InputMode, Vec<KeyWithModifier>)> = vec![];
        for (mode, path) in &queue {
            for (key, actions) in mode_binds(*mode) {
                if actions.iter().any(&is_target_action) {
                    let mut shortcut = path.clone();
                    shortcut.push(key);
                    return Some(shortcut);
                }
            }
        }
        for (mode, path) in &queue {
            for (key, actions) in mode_binds(*mode) {
                for action in &actions {
                    if let Action::SwitchToMode {
                        input_mode: next_mode,
                    } = action
                    {
                        if !visited.contains(next_mode) {
                            visited.push(*next_mode);
                            let mut next_path = path.clone();
                            next_path.push(key.clone());
                            next_queue.push((*next_mode, next_path));
                        }
                    }
                }
            }
        }
        if next_queue.is_empty() {
            return None;
        }
        queue = next_queue;
    }
    None
}

// we need to do this because [10] in standard STDIN, [10] is both Enter (without a carriage
// return) and ctrl-j - so here, if ctrl-j is bound we return its bound action, and otherwise we
// just write the raw bytes to the terminal and let whichever program is there decide what they are
fn handle_ctrl_j(
    mode_keybindings: &HashMap<KeyWithModifier, Vec<Action>>,
    raw_bytes: &[u8],
    key_is_kitty_protocol: bool,
) -> Option<Vec<Action>> {
    let ctrl_j = KeyWithModifier::new(BareKey::Char('j')).with_ctrl_modifier();
    if mode_keybindings.get(&ctrl_j).is_some() {
        mode_keybindings.get(&ctrl_j).cloned()
    } else {
        Some(vec![Action::Write {
            key_with_modifier: Some(ctrl_j),
            bytes: raw_bytes.to_vec().clone(),
            is_kitty_keyboard_protocol: key_is_kitty_protocol,
        }])
    }
}

// The unit test location.
#[cfg(test)]
#[path = "./unit/keybinds_test.rs"]
mod keybinds_test;

#[cfg(test)]
mod keyboard_copy_tests {
    use super::*;
    use crate::{data::KeyModifier, input::config::Config};
    use std::str::FromStr;

    #[test]
    fn keyboard_copy_normalizes_shifted_protocol_characters() {
        for (character, expected) in [('g', "G"), ('V', "V"), ('$', "$")] {
            let key = KeyWithModifier {
                bare_key: BareKey::Char(character),
                key_modifiers: [KeyModifier::Shift].into_iter().collect(),
            };
            assert_eq!(
                Keybinds::default().default_action_for_mode(
                    &InputMode::Copy,
                    Some(&key),
                    vec![],
                    InputMode::Normal,
                    true
                ),
                Action::CopyModeInput {
                    key: expected.to_owned()
                }
            );
        }
    }

    #[test]
    fn keyboard_copy_presets_keep_page_up_and_selection_escape() {
        for preset in [
            include_str!("../../assets/keybinds/default.kdl"),
            include_str!("../../assets/keybinds/unlock-first.kdl"),
        ] {
            let preset = preset
                .replace("{primary}", "Ctrl")
                .replace("{secondary}", "Alt")
                .replace("{unlock}", "Ctrl g");
            let config = Config::from_kdl(&preset, None).expect("parse shipped keymap");
            for (key, expected) in [("Ctrl b", "Ctrl b"), ("Esc", "ESC")] {
                let key = KeyWithModifier::from_str(key).unwrap();
                let actions = config
                    .keybinds
                    .get_actions_for_key_in_mode_or_default_action(
                        &InputMode::Copy,
                        &key,
                        vec![],
                        InputMode::Normal,
                        false,
                    );
                assert_eq!(
                    actions,
                    vec![Action::CopyModeInput {
                        key: expected.to_owned()
                    }]
                );
            }
        }
    }
}
