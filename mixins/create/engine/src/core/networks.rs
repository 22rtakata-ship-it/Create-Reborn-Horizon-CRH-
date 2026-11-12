// SPDX-License-Identifier: MIT
//! Event-driven, dimension-scoped kinetic networks. One service owns one world
//! session. Hosts publish loaded node/source/consumer changes; queries never read
//! or load the engine world. This service is not yet wired into the native ticker.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::kinetics::{self, Kind, Network, Node, Pos, Status};

/// Host-assigned unload group. No Horizon chunk size or coordinate API is assumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionId(pub [i32; 2]);

/// Ephemeral identity, meaningful only in the service that returned it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkId(u64);

/// Immutable result. After mutation, query again or call `is_current` before use.
/// Cloning a view shares the result; it does not clone the graph.
#[derive(Clone, Debug)]
pub struct NetworkView {
    id: NetworkId,
    result: Arc<Network>,
}
impl NetworkView {
    pub fn id(&self) -> NetworkId {
        self.id
    }
    pub fn status(&self) -> Status {
        self.result.status
    }
    pub fn rpm(&self, position: Pos) -> f32 {
        self.result.speed(position)
    }
    pub fn port_rpm(&self, position: Pos, direction: Pos) -> f32 {
        self.result.port_speed(position, direction)
    }
    pub fn capacity(&self) -> f32 {
        self.result.capacity
    }
    pub fn stress(&self) -> f32 {
        self.result.stress
    }
    /// Remaining SU; a failed/stopped network supplies no usable stress.
    pub fn available_stress(&self) -> f32 {
        if self.status() == Status::Running {
            (self.capacity() - self.stress()).max(0.0)
        } else {
            0.0
        }
    }
}

/// Invalidation notification returned to the host. No unbounded event queue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Change {
    pub changed: bool,
    pub invalidated: Vec<NetworkId>,
}

/// Work counters, useful to distinguish cached queries from actual solver work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub solves: u64,
    pub cache_hits: u64,
    pub invalidated: u64,
}

#[derive(Clone, Copy)]
struct Registered {
    region: RegionId,
    node: Node,
}
#[derive(Default)]
struct Dimension {
    nodes: BTreeMap<Pos, Registered>,
    regions: BTreeMap<RegionId, BTreeSet<Pos>>,
    vertices: BTreeMap<(Pos, u8), NetworkId>,
    networks: BTreeMap<NetworkId, NetworkView>,
}
impl Dimension {
    fn invalidate_near(&mut self, pos: Pos) -> Vec<NetworkId> {
        // A new edge can join two previously cached components. Invalidate both
        // endpoints, including both independent halves of an open clutch.
        let mut ids = BTreeSet::new();
        for candidate in std::iter::once(pos).chain(kinetics::connection_candidates(pos)) {
            for half in [0, 1] {
                if let Some(id) = self.vertices.get(&(candidate, half)) {
                    ids.insert(*id);
                }
            }
        }
        for id in &ids {
            if let Some(view) = self.networks.remove(id) {
                for vertex in &view.result.vertices {
                    self.vertices.remove(vertex);
                }
            }
        }
        ids.into_iter().collect()
    }
    fn forget_region_entry(&mut self, region: RegionId, pos: Pos) {
        if let Some(positions) = self.regions.get_mut(&region) {
            positions.remove(&pos);
            if positions.is_empty() {
                self.regions.remove(&region);
            }
        }
    }
}

/// Mutable server-side service. No tick method: unchanged networks remain cached.
/// All changes to loaded topology, generated RPM and stress must be published
/// with `upsert`/`remove`/`unload_region`. Views and IDs are never persisted.
#[derive(Default)]
pub struct Networks {
    dimensions: BTreeMap<String, Dimension>,
    next_id: u64,
    statistics: Statistics,
}
impl Networks {
    pub fn statistics(&self) -> Statistics {
        self.statistics
    }

    /// Register a loaded component or publish its new orientation/source/load.
    /// Re-publishing an identical finite Node in the same region is a no-op.
    pub fn upsert(&mut self, dimension: &str, region: RegionId, node: Node) -> Change {
        let world = self.dimensions.entry(dimension.to_owned()).or_default();
        if world
            .nodes
            .get(&node.pos)
            .is_some_and(|old| old.region == region && old.node == node)
        {
            return Change::default();
        }
        let invalidated = world.invalidate_near(node.pos);
        if let Some(old) = world.nodes.insert(node.pos, Registered { region, node }) {
            world.forget_region_entry(old.region, node.pos);
        }
        world.regions.entry(region).or_default().insert(node.pos);
        self.statistics.invalidated += invalidated.len() as u64;
        Change {
            changed: true,
            invalidated,
        }
    }

    /// Remove a component. Missing positions do not invalidate their neighbors.
    pub fn remove(&mut self, dimension: &str, pos: Pos) -> Change {
        let Some(world) = self.dimensions.get_mut(dimension) else {
            return Change::default();
        };
        let Some(old) = world.nodes.remove(&pos) else {
            return Change::default();
        };
        let invalidated = world.invalidate_near(pos);
        world.forget_region_entry(old.region, pos);
        self.statistics.invalidated += invalidated.len() as u64;
        Change {
            changed: true,
            invalidated,
        }
    }

    /// Unload only the indexed host region; no scan of all loaded components.
    pub fn unload_region(&mut self, dimension: &str, region: RegionId) -> Change {
        let Some(world) = self.dimensions.get_mut(dimension) else {
            return Change::default();
        };
        let Some(positions) = world.regions.remove(&region) else {
            return Change::default();
        };
        let mut invalidated = BTreeSet::new();
        for pos in positions {
            invalidated.extend(world.invalidate_near(pos));
            world.nodes.remove(&pos);
        }
        self.statistics.invalidated += invalidated.len() as u64;
        Change {
            changed: true,
            invalidated: invalidated.into_iter().collect(),
        }
    }

    /// Release every node and cache in a dimension, for dimension/session teardown.
    pub fn unload_dimension(&mut self, dimension: &str) -> Change {
        let Some(world) = self.dimensions.remove(dimension) else {
            return Change::default();
        };
        let invalidated: Vec<_> = world.networks.into_keys().collect();
        self.statistics.invalidated += invalidated.len() as u64;
        Change {
            changed: true,
            invalidated,
        }
    }

    /// Query a loaded node's network. Half 1 is valid only for a powered clutch;
    /// half 0 selects its positive-axis side. Unknown positions/halves return None.
    pub fn query(&mut self, dimension: &str, pos: Pos, half: u8) -> Option<NetworkView> {
        let world = self.dimensions.get_mut(dimension)?;
        let node = world.nodes.get(&pos)?.node;
        if half > 1 || half == 1 && !matches!(node.kind, Kind::Clutch { powered: true }) {
            return None;
        }
        if let Some(view) = world
            .vertices
            .get(&(pos, half))
            .and_then(|id| world.networks.get(id))
        {
            self.statistics.cache_hits += 1;
            return Some(view.clone());
        }
        let result = kinetics::solve_half(node, half, |p| world.nodes.get(&p).map(|n| n.node));
        self.statistics.solves += 1;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("network identity space exhausted");
        let view = NetworkView {
            id: NetworkId(self.next_id),
            result: Arc::new(result),
        };
        // A bounded traversal omits vertices: a far-away edit could change its
        // outcome without touching this partial frontier. Never cache TooLarge.
        if view.status() != Status::TooLarge {
            for vertex in &view.result.vertices {
                world.vertices.insert(*vertex, view.id);
            }
            world.networks.insert(view.id, view.clone());
        }
        Some(view)
    }

    /// Check whether a snapshot still belongs to this dimension's active cache.
    /// Uncacheable TooLarge results always return false. Use only this service's views.
    pub fn is_current(&self, dimension: &str, view: &NetworkView) -> bool {
        self.dimensions
            .get(dimension)
            .and_then(|world| world.networks.get(&view.id))
            .is_some_and(|current| Arc::ptr_eq(&current.result, &view.result))
    }
}
