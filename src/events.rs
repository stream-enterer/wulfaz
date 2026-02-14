use crate::components::{Entity, Tick};

/// All event types in the simulation. Every variant includes tick: Tick.
#[derive(Debug, Clone)]
pub enum Event {
    Spawned {
        entity: Entity,
        tick: Tick,
    },
    Died {
        entity: Entity,
        tick: Tick,
    },
    Moved {
        entity: Entity,
        x: i32,
        y: i32,
        tick: Tick,
    },
    Ate {
        entity: Entity,
        food: Entity,
        tick: Tick,
    },
    Attacked {
        attacker: Entity,
        defender: Entity,
        damage: f32,
        tick: Tick,
    },
    HungerChanged {
        entity: Entity,
        old: f32,
        new_val: f32,
        tick: Tick,
    },
}

/// Ring buffer for events. Fixed capacity, overwrites oldest entries.
pub struct EventLog {
    buffer: Vec<Option<Event>>,
    capacity: usize,
    write_pos: usize,
    count: usize,
}

impl EventLog {
    /// Create a new EventLog with the given max capacity.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1); // minimum 1
        Self {
            buffer: (0..capacity).map(|_| None).collect(),
            capacity,
            write_pos: 0,
            count: 0,
        }
    }

    /// Create an EventLog with the default capacity of 10,000.
    pub fn default_capacity() -> Self {
        Self::new(10_000)
    }

    /// Push an event into the ring buffer. Overwrites oldest if full.
    pub fn push(&mut self, event: Event) {
        self.buffer[self.write_pos] = Some(event);
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Iterate over all events from oldest to newest.
    #[allow(dead_code)] // Used via lib crate in integration tests
    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        let start = if self.count < self.capacity {
            0
        } else {
            self.write_pos
        };

        (0..self.count).filter_map(move |i| {
            let idx = (start + i) % self.capacity;
            self.buffer[idx].as_ref()
        })
    }

    /// Return the most recent n events (newest last).
    pub fn recent(&self, n: usize) -> Vec<&Event> {
        let n = n.min(self.count);
        let start = if self.count < self.capacity {
            self.count.saturating_sub(n)
        } else {
            (self.write_pos + self.capacity - n) % self.capacity
        };

        (0..n)
            .filter_map(|i| {
                let idx = (start + i) % self.capacity;
                self.buffer[idx].as_ref()
            })
            .collect()
    }

    /// Total number of events currently stored.
    #[allow(dead_code)] // Used via lib crate in integration tests
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Entity, Tick};

    fn make_spawned(id: u64, tick: u64) -> Event {
        Event::Spawned {
            entity: Entity(id),
            tick: Tick(tick),
        }
    }

    fn event_tick(event: &Event) -> u64 {
        match event {
            Event::Spawned { tick, .. } => tick.0,
            Event::Died { tick, .. } => tick.0,
            Event::Moved { tick, .. } => tick.0,
            Event::Ate { tick, .. } => tick.0,
            Event::Attacked { tick, .. } => tick.0,
            Event::HungerChanged { tick, .. } => tick.0,
        }
    }

    #[test]
    fn push_and_iter_returns_events_in_order() {
        let mut log = EventLog::new(10);
        log.push(make_spawned(1, 0));
        log.push(make_spawned(2, 1));
        log.push(make_spawned(3, 2));

        let ticks: Vec<u64> = log.iter().map(|e| event_tick(e)).collect();
        assert_eq!(ticks, vec![0, 1, 2]);
    }

    #[test]
    fn ring_buffer_wraps_and_overwrites_oldest() {
        let mut log = EventLog::new(3);
        log.push(make_spawned(1, 0)); // will be overwritten
        log.push(make_spawned(2, 1)); // will be overwritten
        log.push(make_spawned(3, 2));
        log.push(make_spawned(4, 3)); // overwrites tick 0
        log.push(make_spawned(5, 4)); // overwrites tick 1

        assert_eq!(log.len(), 3);

        let ticks: Vec<u64> = log.iter().map(|e| event_tick(e)).collect();
        assert_eq!(ticks, vec![2, 3, 4]);
    }

    #[test]
    fn recent_returns_correct_events() {
        let mut log = EventLog::new(10);
        for i in 0..5 {
            log.push(make_spawned(i, i));
        }

        let recent = log.recent(3);
        let ticks: Vec<u64> = recent.iter().map(|e| event_tick(e)).collect();
        assert_eq!(ticks, vec![2, 3, 4]);
    }

    #[test]
    fn recent_after_wrap() {
        let mut log = EventLog::new(3);
        for i in 0..7 {
            log.push(make_spawned(i, i));
        }

        let recent = log.recent(2);
        let ticks: Vec<u64> = recent.iter().map(|e| event_tick(e)).collect();
        assert_eq!(ticks, vec![5, 6]);
    }

    #[test]
    fn recent_more_than_available() {
        let mut log = EventLog::new(10);
        log.push(make_spawned(1, 0));
        log.push(make_spawned(2, 1));

        let recent = log.recent(100);
        assert_eq!(recent.len(), 2);
        let ticks: Vec<u64> = recent.iter().map(|e| event_tick(e)).collect();
        assert_eq!(ticks, vec![0, 1]);
    }

    #[test]
    fn default_capacity_is_10000() {
        let log = EventLog::default_capacity();
        assert_eq!(log.capacity, 10_000);
        assert!(log.is_empty());
    }

    #[test]
    fn empty_log() {
        let log = EventLog::new(5);
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert_eq!(log.iter().count(), 0);
        assert!(log.recent(10).is_empty());
    }

    #[test]
    fn single_capacity_ring_buffer() {
        let mut log = EventLog::new(1);
        log.push(make_spawned(1, 0));
        log.push(make_spawned(2, 1));

        assert_eq!(log.len(), 1);
        let ticks: Vec<u64> = log.iter().map(|e| event_tick(e)).collect();
        assert_eq!(ticks, vec![1]);
    }

    #[test]
    fn zero_capacity_clamped_to_one() {
        let log = EventLog::new(0);
        assert_eq!(log.capacity, 1);
    }

    #[test]
    fn all_event_variants_have_tick() {
        let events = vec![
            Event::Spawned {
                entity: Entity(1),
                tick: Tick(0),
            },
            Event::Died {
                entity: Entity(1),
                tick: Tick(1),
            },
            Event::Moved {
                entity: Entity(1),
                x: 5,
                y: 3,
                tick: Tick(2),
            },
            Event::Ate {
                entity: Entity(1),
                food: Entity(2),
                tick: Tick(3),
            },
            Event::Attacked {
                attacker: Entity(1),
                defender: Entity(2),
                damage: 10.0,
                tick: Tick(4),
            },
            Event::HungerChanged {
                entity: Entity(1),
                old: 0.0,
                new_val: 5.0,
                tick: Tick(5),
            },
        ];

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event_tick(event), i as u64);
        }
    }
}
