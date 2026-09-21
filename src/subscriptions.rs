//! Persistent, explicit project-watch subscriptions.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Target {
    pub guild_id: u64,
    pub channel_id: u64,
}

impl Target {
    pub fn new(guild_id: u64, channel_id: u64) -> Self {
        Self {
            guild_id,
            channel_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Watch {
    pub guild_id: u64,
    pub channel_id: u64,
    #[serde(default = "EventFilter::all")]
    pub events: EventFilter,
}

impl Watch {
    pub fn new(target: Target, events: EventFilter) -> Self {
        Self {
            guild_id: target.guild_id,
            channel_id: target.channel_id,
            events,
        }
    }

    pub fn target(&self) -> Target {
        Target::new(self.guild_id, self.channel_id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EventFilter {
    #[serde(default = "default_event_names")]
    names: BTreeSet<String>,
}

impl EventFilter {
    pub fn all() -> Self {
        Self {
            names: default_event_names(),
        }
    }

    pub fn from_names(names: impl IntoIterator<Item = String>) -> Self {
        Self {
            names: names.into_iter().collect(),
        }
    }

    pub fn includes(&self, event: &str) -> bool {
        self.names.contains(event)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        ["activity", "release", "ci"]
            .into_iter()
            .filter(|name| self.includes(name))
    }
}

fn default_event_names() -> BTreeSet<String> {
    ["activity", "release", "ci"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct State {
    watches: BTreeMap<String, Vec<Watch>>,
}

#[derive(Clone)]
pub struct Store {
    state: Arc<RwLock<State>>,
    path: PathBuf,
}

impl Store {
    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let state = fs::read_to_string(&path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default();
        Self {
            state: Arc::new(RwLock::new(state)),
            path,
        }
    }

    pub fn watch(&self, repo: &str, watch: Watch) -> io::Result<bool> {
        let mut state = self
            .state
            .write()
            .expect("subscription state lock poisoned");
        let targets = state.watches.entry(repo.to_owned()).or_default();
        if let Some(existing) = targets.iter_mut().find(|existing| {
            existing.guild_id == watch.guild_id && existing.channel_id == watch.channel_id
        }) {
            if existing.events == watch.events {
                return Ok(false);
            }
            *existing = watch;
            targets.sort();
            save_state(&self.path, &state)?;
            return Ok(true);
        }
        if targets.contains(&watch) {
            return Ok(false);
        }
        targets.push(watch);
        targets.sort();
        save_state(&self.path, &state)?;
        Ok(true)
    }

    pub fn unwatch(&self, repo: &str, target: &Target) -> io::Result<bool> {
        let mut state = self
            .state
            .write()
            .expect("subscription state lock poisoned");
        let Some(targets) = state.watches.get_mut(repo) else {
            return Ok(false);
        };
        let original_len = targets.len();
        targets.retain(|candidate| candidate.target() != *target);
        let removed = targets.len() != original_len;
        let empty = targets.is_empty();
        if !removed {
            return Ok(false);
        }
        if empty {
            state.watches.remove(repo);
        }
        save_state(&self.path, &state)?;
        Ok(true)
    }

    pub fn repos_for(&self, target: &Target) -> Vec<String> {
        let state = self.state.read().expect("subscription state lock poisoned");
        state
            .watches
            .iter()
            .filter(|(_, watches)| watches.iter().any(|watch| watch.target() == *target))
            .map(|(repo, _)| repo.clone())
            .collect()
    }

    pub fn watches_for(&self, repo: &str) -> Vec<Watch> {
        let state = self.state.read().expect("subscription state lock poisoned");
        state.watches.get(repo).cloned().unwrap_or_default()
    }

    pub fn count(&self) -> usize {
        let state = self.state.read().expect("subscription state lock poisoned");
        state.watches.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

fn save_state(path: &Path, state: &State) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(state)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, contents)?;
    fs::rename(temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watches_persist_and_can_be_removed() {
        let path =
            std::env::temp_dir().join(format!("hermes-subscriptions-{}.json", std::process::id()));
        let target = Target::new(1, 2);
        let store = Store::load(&path);

        assert!(store
            .watch(
                "darkstardevx/gateflow",
                Watch::new(target.clone(), EventFilter::all()),
            )
            .unwrap());
        assert!(!store
            .watch(
                "darkstardevx/gateflow",
                Watch::new(target.clone(), EventFilter::all()),
            )
            .unwrap());
        assert_eq!(store.repos_for(&target), vec!["darkstardevx/gateflow"]);

        let reloaded = Store::load(&path);
        assert_eq!(
            reloaded.watches_for("darkstardevx/gateflow")[0].target(),
            target.clone()
        );
        assert!(reloaded.unwatch("darkstardevx/gateflow", &target).unwrap());
        assert!(reloaded.is_empty());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn changing_events_updates_an_existing_channel_watch() {
        let path = std::env::temp_dir().join(format!(
            "hermes-subscriptions-events-{}.json",
            std::process::id()
        ));
        let target = Target::new(1, 2);
        let store = Store::load(&path);
        let events = EventFilter::from_names(["ci".to_owned()]);

        assert!(store
            .watch(
                "darkstardevx/gateflow",
                Watch::new(target.clone(), events.clone()),
            )
            .unwrap());
        assert_eq!(store.watches_for("darkstardevx/gateflow")[0].events, events);

        let _ = fs::remove_file(path);
    }
}
