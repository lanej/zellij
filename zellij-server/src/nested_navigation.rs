use crate::ClientId;
use std::collections::{BTreeMap, HashSet};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub(crate) const MAX_NESTING_DEPTH: usize = 32;
pub(crate) const DISCOVERY_GRACE: Duration = Duration::from_secs(3);

pub(crate) enum NavigationTarget<'a> {
    New,
    Existing(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NavigationError {
    AncestorTarget,
    NestingContextUnavailable,
    MissingTarget,
    UnsupportedClient,
}
impl NavigationError {
    pub(crate) fn notice(&self) -> &'static str {
        match self {
            Self::AncestorTarget => "This item belongs to a containing session. Return to that session before opening it.",
            Self::NestingContextUnavailable => "Session navigation is unavailable until this attachment's nesting identity is known. Return to the containing session to navigate.",
            Self::MissingTarget => "The destination session could not be verified. Try again after it is available.",
            Self::UnsupportedClient => "Reconnect with the updated Zellij client before switching sessions. Your current session and programs are unchanged.",
        }
    }
}
#[derive(Debug)]
enum ClientNesting {
    Discovering(Option<Instant>),
    Ordinary,
    Nested {
        names: Vec<String>,
        ids: Option<Vec<String>>,
    },
}
#[derive(Debug)]
pub(crate) struct NavigationGuard {
    pub(crate) instance_id: String,
    clients: BTreeMap<ClientId, ClientNesting>,
}
impl NavigationGuard {
    pub(crate) fn new(instance_id: String) -> Self {
        Self {
            instance_id,
            clients: BTreeMap::new(),
        }
    }
    pub(crate) fn add_client(&mut self, client_id: ClientId) {
        self.clients
            .entry(client_id)
            .or_insert(ClientNesting::Discovering(None));
    }
    pub(crate) fn configure_client(
        &mut self,
        client_id: ClientId,
        expected_nested: bool,
        now: Instant,
    ) {
        self.add_client(client_id);
        if let Some(state @ ClientNesting::Discovering(_)) = self.clients.get_mut(&client_id) {
            *state = ClientNesting::Discovering(if expected_nested {
                None
            } else {
                Some(now + DISCOVERY_GRACE)
            });
        }
    }
    pub(crate) fn settle_ordinary(&mut self, client_id: ClientId, now: Instant) {
        if let Some(state @ ClientNesting::Discovering(Some(_))) = self.clients.get_mut(&client_id)
        {
            if matches!(state, ClientNesting::Discovering(Some(deadline)) if now >= *deadline) {
                *state = ClientNesting::Ordinary;
            }
        }
    }
    pub(crate) fn remove_client(&mut self, client_id: ClientId) {
        self.clients.remove(&client_id);
    }
    pub(crate) fn validate_ancestry(&self, names: &[String], ids: Option<&[String]>) -> bool {
        if names.is_empty() || names.len() > MAX_NESTING_DEPTH {
            return false;
        }
        let Some(ids) = ids else {
            return true;
        };
        let mut seen = HashSet::new();
        ids.len() == names.len()
            && ids.iter().all(|id| {
                Uuid::parse_str(id)
                    .is_ok_and(|id| id.to_string() != self.instance_id && seen.insert(id))
            })
    }
    pub(crate) fn record_ancestry(
        &mut self,
        client_id: ClientId,
        names: Vec<String>,
        ids: Option<Vec<String>>,
    ) {
        self.clients.insert(
            client_id,
            ClientNesting::Nested {
                names,
                ids: ids.map(|ids| {
                    ids.into_iter()
                        .map(|id| Uuid::parse_str(&id).unwrap().to_string())
                        .collect()
                }),
            },
        );
    }
    pub(crate) fn note_unresolved_host(&mut self, client_id: ClientId) {
        self.add_client(client_id);
        if matches!(
            self.clients.get(&client_id),
            Some(ClientNesting::Discovering(_)) | Some(ClientNesting::Ordinary)
        ) {
            self.clients
                .insert(client_id, ClientNesting::Discovering(None));
        }
    }
    pub(crate) fn is_nested(&self, client_id: ClientId) -> bool {
        matches!(
            self.clients.get(&client_id),
            Some(ClientNesting::Nested { .. })
        )
    }
    pub(crate) fn contains_ancestor(&self, id: &str) -> bool {
        let Ok(id) = Uuid::parse_str(id) else {
            return false;
        };
        self.clients.values().any(|state| matches!(state, ClientNesting::Nested { ids: Some(ids), .. } if ids.iter().any(|a| Uuid::parse_str(a).ok() == Some(id))))
    }
    pub(crate) fn remaining_context(&self) -> Option<(ClientId, Vec<String>, Option<Vec<String>>)> {
        self.clients.iter().find_map(|(client, state)| match state {
            ClientNesting::Nested { names, ids } => Some((*client, names.clone(), ids.clone())),
            _ => None,
        })
    }
    pub(crate) fn descendant_ancestry(&self) -> Option<Vec<String>> {
        let mut path: Option<&Vec<String>> = None;
        for state in self.clients.values() {
            match state {
                ClientNesting::Discovering(_) | ClientNesting::Nested { ids: None, .. } => {
                    return None
                },
                ClientNesting::Nested { ids: Some(ids), .. } => {
                    if path.is_some_and(|path| path != ids) {
                        return None;
                    }
                    path = Some(ids);
                },
                ClientNesting::Ordinary => {},
            }
        }
        let mut ids = path.cloned().unwrap_or_default();
        ids.push(self.instance_id.clone());
        Some(ids)
    }
    pub(crate) fn check(
        &self,
        client_id: ClientId,
        target: NavigationTarget<'_>,
    ) -> Result<(), NavigationError> {
        match self.clients.get(&client_id) {
            None
            | Some(ClientNesting::Discovering(_))
            | Some(ClientNesting::Nested { ids: None, .. }) => {
                Err(NavigationError::NestingContextUnavailable)
            },
            Some(ClientNesting::Nested {
                ids: Some(ancestry),
                ..
            }) => {
                let NavigationTarget::Existing(target_id) = target else {
                    return Ok(());
                };
                let id = Uuid::parse_str(target_id)
                    .ok()
                    .ok_or(NavigationError::MissingTarget)?
                    .to_string();
                if ancestry.contains(&id) {
                    Err(NavigationError::AncestorTarget)
                } else {
                    Ok(())
                }
            },
            Some(ClientNesting::Ordinary) => Ok(()),
        }
    }
}

pub(crate) fn accepts_switch_target(
    expected: Option<&str>,
    is_switch: bool,
    own: &str,
    is_new: bool,
) -> bool {
    match expected {
        Some(expected) => !is_new && expected == own,
        None => !is_switch || is_new,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const OWN: &str = "00000000-0000-4000-8000-000000000001";
    const HOST: &str = "00000000-0000-4000-8000-000000000002";
    const OTHER: &str = "00000000-0000-4000-8000-000000000003";
    #[test]
    fn delayed_ack_never_promotes_expected_nesting_and_descendants_stay_unknown() {
        let now = Instant::now();
        let mut guard = NavigationGuard::new(OWN.into());
        guard.configure_client(1, true, now);
        guard.settle_ordinary(1, now + Duration::from_secs(30));
        assert_eq!(
            guard.check(1, NavigationTarget::Existing(HOST)),
            Err(NavigationError::NestingContextUnavailable)
        );
        assert_eq!(guard.descendant_ancestry(), None);
        guard.record_ancestry(1, vec!["main".into()], Some(vec![HOST.into()]));
        assert_eq!(
            guard.check(1, NavigationTarget::Existing(HOST)),
            Err(NavigationError::AncestorTarget)
        );
    }
    #[test]
    fn fresh_destination_requires_known_nesting_but_not_an_existing_identity() {
        let now = Instant::now();
        let mut guard = NavigationGuard::new(OWN.into());
        guard.configure_client(1, true, now);
        guard.settle_ordinary(1, now + Duration::from_secs(30));
        assert_eq!(
            guard.check(1, NavigationTarget::New),
            Err(NavigationError::NestingContextUnavailable)
        );
        guard.record_ancestry(1, vec!["main".into()], None);
        assert_eq!(
            guard.check(1, NavigationTarget::New),
            Err(NavigationError::NestingContextUnavailable)
        );
        guard.record_ancestry(1, vec!["main".into()], Some(vec![HOST.into()]));
        assert!(guard.check(1, NavigationTarget::New).is_ok());
        assert_eq!(
            guard.check(1, NavigationTarget::Existing(HOST)),
            Err(NavigationError::AncestorTarget)
        );
        assert_eq!(
            guard.check(1, NavigationTarget::Existing("malformed")),
            Err(NavigationError::MissingTarget)
        );
    }
    #[test]
    fn independent_clients_and_remaining_containment_survive_disconnect() {
        let now = Instant::now();
        let mut guard = NavigationGuard::new(OWN.into());
        guard.record_ancestry(1, vec!["main".into()], Some(vec![HOST.into()]));
        guard.configure_client(2, false, now);
        assert_eq!(guard.descendant_ancestry(), None);
        guard.settle_ordinary(2, now + DISCOVERY_GRACE);
        assert!(guard.check(2, NavigationTarget::Existing(HOST)).is_ok());
        guard.record_ancestry(3, vec!["main".into()], Some(vec![OTHER.into()]));
        assert_eq!(guard.descendant_ancestry(), None);
        guard.remove_client(3);
        assert_eq!(
            guard.descendant_ancestry(),
            Some(vec![HOST.into(), OWN.into()])
        );
        assert_eq!(
            guard.check(1, NavigationTarget::Existing(HOST)),
            Err(NavigationError::AncestorTarget)
        );
    }
    #[test]
    fn cycles_are_rejected_while_equal_names_at_different_servers_are_legal() {
        let guard = NavigationGuard::new(OWN.into());
        let names = vec!["main".into(), "main".into()];
        assert!(guard.validate_ancestry(&names, Some(&[HOST.into(), OTHER.into()])));
        assert!(!guard.validate_ancestry(&names, Some(&[HOST.into(), HOST.into()])));
        assert!(!guard.validate_ancestry(&names, Some(&[HOST.into(), OWN.into()])));
        assert!(!guard.validate_ancestry(&names, Some(&["malformed".into(), OTHER.into()])));
        assert!(!guard.validate_ancestry(&names, Some(&[HOST.into()])));
    }
    #[test]
    fn legacy_identity_remains_unknown_and_depth_is_bounded() {
        let mut guard = NavigationGuard::new(OWN.into());
        guard.record_ancestry(1, vec!["main".into()], None);
        assert_eq!(
            guard.check(1, NavigationTarget::Existing(OTHER)),
            Err(NavigationError::NestingContextUnavailable)
        );
        assert_eq!(guard.descendant_ancestry(), None);
        assert!(!guard.validate_ancestry(&vec!["main".into(); MAX_NESTING_DEPTH + 1], None));
    }
    #[test]
    fn replacement_and_an_old_client_dropping_the_expectation_cannot_attach() {
        assert!(accepts_switch_target(Some(HOST), true, HOST, false));
        assert!(!accepts_switch_target(Some(HOST), true, OTHER, false));
        assert!(!accepts_switch_target(None, true, HOST, false));
        assert!(!accepts_switch_target(Some(HOST), true, OTHER, true));
        assert!(accepts_switch_target(None, true, OTHER, true));
        assert!(accepts_switch_target(None, false, HOST, false));
    }
}
