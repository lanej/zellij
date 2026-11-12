use std::collections::BTreeMap;

use serde::Deserialize;
use zellij_tile::prelude::*;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct TitleOwnership {
    session: Option<String>,
    auto_titles: BTreeMap<String, String>,
}

impl TitleOwnership {
    pub(super) fn from_result(
        code: Option<i32>,
        output: &[u8],
        session: Option<&str>,
    ) -> Option<Self> {
        let view: Self = serde_json::from_slice(output).ok()?;
        (code == Some(0) && session.is_some() && view.session.as_deref() == session).then_some(view)
    }

    pub(super) fn label(&self, tab: &TabInfo, mode: &ModeInfo) -> String {
        let fixed = self.session.as_deref() == mode.session_name.as_deref()
            && !tab.name.is_empty()
            && tab.name != format!("Tab #{}", tab.tab_id + 1)
            && !(tab.active && mode.mode == InputMode::RenameTab)
            && self.auto_titles.get(&tab.tab_id.to_string()) != Some(&tab.name);
        if fixed {
            let lock = if mode.capabilities.arrow_fonts {
                " "
            } else {
                "L "
            };
            format!("{lock}{}", tab.name)
        } else {
            tab.name.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ownership_is_scoped_to_the_current_session_and_successful_response() {
        let output = br#"{"session":"main","auto_titles":{"7":"project"}}"#;
        assert!(TitleOwnership::from_result(Some(0), output, Some("main")).is_some());
        assert!(TitleOwnership::from_result(Some(1), output, Some("main")).is_none());
        assert!(TitleOwnership::from_result(Some(0), output, Some("other")).is_none());
        assert!(TitleOwnership::from_result(Some(0), output, None).is_none());
        assert!(TitleOwnership::from_result(Some(0), b"bad json", Some("main")).is_none());
        assert!(
            TitleOwnership::from_result(Some(0), br#"{"session":"main"}"#, Some("main")).is_none()
        );
    }

    #[test]
    fn only_fixed_titles_get_a_left_lock_without_changing_the_name() {
        let view = TitleOwnership::from_result(
            Some(0),
            br#"{"session":"main","auto_titles":{"7":"project"}}"#,
            Some("main"),
        )
        .unwrap();
        let mut mode = ModeInfo {
            session_name: Some("main".into()),
            ..Default::default()
        };
        mode.capabilities.arrow_fonts = true;
        let mut tab = TabInfo {
            tab_id: 7,
            name: "project".into(),
            active: true,
            ..Default::default()
        };
        assert_eq!(view.label(&tab, &mode), "project");
        tab.name = "review".into();
        assert_eq!(view.label(&tab, &mode), " review");
        assert_eq!(tab.name, "review");
        mode.capabilities.arrow_fonts = false;
        assert_eq!(view.label(&tab, &mode), "L review");
        mode.mode = InputMode::RenameTab;
        assert_eq!(view.label(&tab, &mode), "review");
        mode.mode = InputMode::Normal;
        tab.name.clear();
        assert_eq!(view.label(&tab, &mode), "");
        tab.name = "Tab #8".into();
        assert_eq!(view.label(&tab, &mode), "Tab #8");
        tab.name = "review".into();
        mode.session_name = Some("other".into());
        assert_eq!(view.label(&tab, &mode), "review");
    }
}
