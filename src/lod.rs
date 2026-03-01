use crate::registry::{QuartierData, QuartierId};

/// Level-of-detail zone for a quartier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodZone {
    /// Full entity-level simulation.
    Active,
    /// Reduced-fidelity entity simulation.
    Nearby,
    /// District aggregate model only.
    Statistical,
}

/// Per-speed-level configuration.
#[derive(Debug, Clone, Copy)]
pub struct SpeedConfig {
    /// Accumulator multiplier. 0.0 = bypass accumulator (speed 5).
    pub time_mult: f64,
    /// If true, force all zones to Statistical regardless of camera.
    pub force_statistical: bool,
}

/// A zone transition detected during recompute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodTransition {
    pub quartier: QuartierId,
    pub from: LodZone,
    pub to: LodZone,
}

/// Active zone radius in tiles (meters). Quartiers whose bounding box
/// edge is within this Chebyshev distance of the camera are Active.
pub const ACTIVE_RADIUS: i32 = 150; // ~150m

/// Nearby zone radius in tiles (meters). Quartiers within this distance
/// (but outside ACTIVE_RADIUS) are Nearby; beyond are Statistical.
pub const NEARBY_RADIUS: i32 = 500; // ~500m

/// Speed configs for speeds 1-5. Index by `sim_speed - 1`.
pub const SPEED_CONFIGS: [SpeedConfig; 5] = [
    SpeedConfig {
        time_mult: 1.0,
        force_statistical: false,
    }, // speed 1
    SpeedConfig {
        time_mult: 2.0,
        force_statistical: false,
    }, // speed 2
    SpeedConfig {
        time_mult: 5.0,
        force_statistical: true,
    }, // speed 3
    SpeedConfig {
        time_mult: 20.0,
        force_statistical: true,
    }, // speed 4
    SpeedConfig {
        time_mult: 0.0,
        force_statistical: true,
    }, // speed 5 (bypass accumulator)
];

/// Max simulation ticks per frame for a given speed config.
/// Entity-level speeds (1-2) cap at 5; statistical speeds (3-5) cap at 500.
pub fn max_ticks_for_speed(config: &SpeedConfig) -> u32 {
    if config.force_statistical { 500 } else { 5 }
}

/// Chebyshev distance from point (px, py) to nearest point on AABB.
/// Returns 0 if the point is inside the box.
fn chebyshev_dist_to_aabb(px: i32, py: i32, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> i32 {
    let dx = if px < min_x {
        min_x - px
    } else if px > max_x {
        px - max_x
    } else {
        0
    };
    let dy = if py < min_y {
        min_y - py
    } else if py > max_y {
        py - max_y
    } else {
        0
    };
    dx.max(dy)
}

/// Classify a quartier into an LOD zone based on camera position.
/// Sentinel bounds (min_x == i32::MAX) → Statistical.
pub fn classify_quartier(q: &QuartierData, cam_x: i32, cam_y: i32) -> LodZone {
    // Sentinel bounds = no buildings → always Statistical
    if q.min_x == i32::MAX {
        return LodZone::Statistical;
    }
    let dist = chebyshev_dist_to_aabb(cam_x, cam_y, q.min_x, q.min_y, q.max_x, q.max_y);
    if dist <= ACTIVE_RADIUS {
        LodZone::Active
    } else if dist <= NEARBY_RADIUS {
        LodZone::Nearby
    } else {
        LodZone::Statistical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quartier(id: u8, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> QuartierData {
        QuartierData {
            id: QuartierId(id),
            name: format!("Q{id}"),
            min_x,
            min_y,
            max_x,
            max_y,
            building_count: 1,
            total_building_area_m2: 100.0,
            occupant_count: 10,
            blocks: Vec::new(),
        }
    }

    // 1. Camera inside quartier bbox → Active
    #[test]
    fn classify_camera_inside_bbox() {
        let q = make_quartier(1, 100, 100, 200, 200);
        assert_eq!(classify_quartier(&q, 150, 150), LodZone::Active);
    }

    // 2. Camera on bbox edge → Active (distance 0)
    #[test]
    fn classify_camera_on_edge() {
        let q = make_quartier(1, 100, 100, 200, 200);
        assert_eq!(classify_quartier(&q, 100, 150), LodZone::Active);
    }

    // 3. Camera within ACTIVE_RADIUS of bbox → Active
    #[test]
    fn classify_within_active_radius() {
        let q = make_quartier(1, 100, 100, 200, 200);
        // 50 tiles away from nearest edge
        assert_eq!(classify_quartier(&q, 50, 150), LodZone::Active);
    }

    // 4. Camera just outside ACTIVE_RADIUS → Nearby
    #[test]
    fn classify_just_outside_active_radius() {
        let q = make_quartier(1, 100, 100, 200, 200);
        // 151 tiles away from nearest edge (x=100, cam_x=-51)
        assert_eq!(classify_quartier(&q, -51, 150), LodZone::Nearby);
    }

    // 5. Camera within NEARBY_RADIUS → Nearby
    #[test]
    fn classify_within_nearby_radius() {
        let q = make_quartier(1, 100, 100, 200, 200);
        // 400 tiles from nearest edge
        assert_eq!(classify_quartier(&q, -300, 150), LodZone::Nearby);
    }

    // 6. Camera beyond NEARBY_RADIUS → Statistical
    #[test]
    fn classify_beyond_nearby_radius() {
        let q = make_quartier(1, 100, 100, 200, 200);
        // 501 tiles from nearest edge
        assert_eq!(classify_quartier(&q, -401, 150), LodZone::Statistical);
    }

    // 7. force_statistical overrides zone to Statistical
    #[test]
    fn force_statistical_overrides() {
        // This tests that recompute_lod_zones applies force_statistical.
        // We test via GisTables in the world module; here we just verify
        // the SpeedConfig values.
        assert!(SPEED_CONFIGS[2].force_statistical); // speed 3
        assert!(SPEED_CONFIGS[3].force_statistical); // speed 4
        assert!(SPEED_CONFIGS[4].force_statistical); // speed 5
        assert!(!SPEED_CONFIGS[0].force_statistical); // speed 1
        assert!(!SPEED_CONFIGS[1].force_statistical); // speed 2
    }

    // 8. Sentinel bounds → Statistical
    #[test]
    fn classify_sentinel_bounds() {
        let q = QuartierData {
            id: QuartierId(1),
            name: "Empty".into(),
            min_x: i32::MAX,
            min_y: i32::MAX,
            max_x: i32::MIN,
            max_y: i32::MIN,
            building_count: 0,
            total_building_area_m2: 0.0,
            occupant_count: 0,
            blocks: Vec::new(),
        };
        assert_eq!(classify_quartier(&q, 0, 0), LodZone::Statistical);
    }

    // 9. max_ticks_for_speed returns correct caps
    #[test]
    fn max_ticks_caps() {
        assert_eq!(max_ticks_for_speed(&SPEED_CONFIGS[0]), 5);
        assert_eq!(max_ticks_for_speed(&SPEED_CONFIGS[1]), 5);
        assert_eq!(max_ticks_for_speed(&SPEED_CONFIGS[2]), 500);
        assert_eq!(max_ticks_for_speed(&SPEED_CONFIGS[3]), 500);
        assert_eq!(max_ticks_for_speed(&SPEED_CONFIGS[4]), 500);
    }
}
