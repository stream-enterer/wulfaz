use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Easing function for animations (UI-W05).
/// Minimal set: linear (constant speed), ease-out (decelerate),
/// ease-in-out (smooth start and end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// Constant speed interpolation.
    Linear,
    /// Cubic ease-in-out: slow start, fast middle, slow end.
    EaseInOut,
    /// Cubic ease-out: fast start, slow end (deceleration).
    EaseOut,
}

/// A single active animation interpolating an f32 value over time.
struct Animation {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
    easing: Easing,
}

/// Time-driven f32 interpolation keyed by string (UI-W05).
///
/// Animations tick on wall-clock delta (Instant), not simulation ticks.
/// Stores a small set of named animations; values are queried each frame
/// by the UI builders to apply fade, slide, and highlight effects.
pub struct Animator {
    animations: HashMap<String, Animation>,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }

    /// Start (or restart) an animation. Overwrites any existing animation
    /// with the same key.
    pub fn start(
        &mut self,
        key: &str,
        from: f32,
        to: f32,
        duration: Duration,
        easing: Easing,
        now: Instant,
    ) {
        self.animations.insert(
            key.to_string(),
            Animation {
                from,
                to,
                start: now,
                duration,
                easing,
            },
        );
    }

    /// Get the current interpolated value. Returns `None` if no animation
    /// exists for this key. Returns the `to` value once complete.
    pub fn get(&self, key: &str, now: Instant) -> Option<f32> {
        let anim = self.animations.get(key)?;
        let elapsed = now.duration_since(anim.start);
        if anim.duration.is_zero() || elapsed >= anim.duration {
            return Some(anim.to);
        }
        let t = elapsed.as_secs_f32() / anim.duration.as_secs_f32();
        let eased = ease(t, anim.easing);
        Some(anim.from + (anim.to - anim.from) * eased)
    }

    /// Returns true if the animation exists and has not yet completed.
    pub fn is_active(&self, key: &str, now: Instant) -> bool {
        if let Some(anim) = self.animations.get(key) {
            !anim.duration.is_zero() && now.duration_since(anim.start) < anim.duration
        } else {
            false
        }
    }

    /// Remove an animation by key.
    pub fn remove(&mut self, key: &str) {
        self.animations.remove(key);
    }

    /// The target value of an animation, if it exists.
    pub fn target(&self, key: &str) -> Option<f32> {
        self.animations.get(key).map(|a| a.to)
    }

    /// Remove completed animations to prevent unbounded growth.
    /// Call once per frame.
    pub fn gc(&mut self, now: Instant) {
        self.animations.retain(|_, anim| {
            anim.duration.is_zero() || now.duration_since(anim.start) < anim.duration
        });
    }
}

/// Apply an easing function to a linear progress value `t` in [0, 1].
fn ease(t: f32, easing: Easing) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseInOut => {
            // Cubic ease-in-out: 4t³ for t<0.5, 1-(-2t+2)³/2 for t>=0.5
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let f = -2.0 * t + 2.0;
                1.0 - f * f * f / 2.0
            }
        }
        Easing::EaseOut => {
            // Cubic ease-out: 1-(1-t)³
            let f = 1.0 - t;
            1.0 - f * f * f
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_linear() {
        assert!((ease(0.0, Easing::Linear)).abs() < 1e-6);
        assert!((ease(0.5, Easing::Linear) - 0.5).abs() < 1e-6);
        assert!((ease(1.0, Easing::Linear) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_out_endpoints() {
        assert!((ease(0.0, Easing::EaseInOut)).abs() < 1e-6);
        assert!((ease(1.0, Easing::EaseInOut) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_out_midpoint() {
        // At t=0.5, ease-in-out should be exactly 0.5 (symmetric).
        assert!((ease(0.5, Easing::EaseInOut) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn ease_in_out_slow_start() {
        // At t=0.25, value should be less than 0.25 (slow start).
        assert!(ease(0.25, Easing::EaseInOut) < 0.25);
    }

    #[test]
    fn ease_out_endpoints() {
        assert!((ease(0.0, Easing::EaseOut)).abs() < 1e-6);
        assert!((ease(1.0, Easing::EaseOut) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_fast_start() {
        // At t=0.25, ease-out should produce a value > 0.25 (fast start).
        assert!(ease(0.25, Easing::EaseOut) > 0.25);
    }

    #[test]
    fn ease_out_monotonic() {
        let mut prev = 0.0_f32;
        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let v = ease(t, Easing::EaseOut);
            assert!(v >= prev, "ease_out must be monotonically increasing");
            prev = v;
        }
    }

    #[test]
    fn animator_start_and_get() {
        let mut anim = Animator::new();
        let t0 = Instant::now();

        anim.start(
            "fade",
            0.0,
            1.0,
            Duration::from_millis(100),
            Easing::Linear,
            t0,
        );

        // At start: value should be 0.0
        let v = anim.get("fade", t0).unwrap();
        assert!((v - 0.0).abs() < 1e-6);

        // At midpoint: value should be ~0.5
        let t1 = t0 + Duration::from_millis(50);
        let v = anim.get("fade", t1).unwrap();
        assert!((v - 0.5).abs() < 0.01);

        // After completion: value should be 1.0
        let t2 = t0 + Duration::from_millis(200);
        let v = anim.get("fade", t2).unwrap();
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn animator_reverse_animation() {
        let mut anim = Animator::new();
        let t0 = Instant::now();

        // Animate from 1.0 to 0.0 (fade out).
        anim.start(
            "fade",
            1.0,
            0.0,
            Duration::from_millis(100),
            Easing::Linear,
            t0,
        );

        let v = anim.get("fade", t0).unwrap();
        assert!((v - 1.0).abs() < 1e-6);

        let t1 = t0 + Duration::from_millis(50);
        let v = anim.get("fade", t1).unwrap();
        assert!((v - 0.5).abs() < 0.01);

        let t2 = t0 + Duration::from_millis(150);
        let v = anim.get("fade", t2).unwrap();
        assert!((v - 0.0).abs() < 1e-6);
    }

    #[test]
    fn animator_is_active() {
        let mut anim = Animator::new();
        let t0 = Instant::now();

        assert!(!anim.is_active("fade", t0));

        anim.start(
            "fade",
            0.0,
            1.0,
            Duration::from_millis(100),
            Easing::Linear,
            t0,
        );
        assert!(anim.is_active("fade", t0));
        assert!(anim.is_active("fade", t0 + Duration::from_millis(50)));
        assert!(!anim.is_active("fade", t0 + Duration::from_millis(100)));
    }

    #[test]
    fn animator_missing_key_returns_none() {
        let anim = Animator::new();
        assert!(anim.get("nonexistent", Instant::now()).is_none());
    }

    #[test]
    fn animator_remove() {
        let mut anim = Animator::new();
        let t0 = Instant::now();
        anim.start(
            "fade",
            0.0,
            1.0,
            Duration::from_millis(100),
            Easing::Linear,
            t0,
        );
        assert!(anim.get("fade", t0).is_some());
        anim.remove("fade");
        assert!(anim.get("fade", t0).is_none());
    }

    #[test]
    fn animator_overwrite() {
        let mut anim = Animator::new();
        let t0 = Instant::now();
        anim.start(
            "x",
            0.0,
            100.0,
            Duration::from_millis(200),
            Easing::Linear,
            t0,
        );
        // Overwrite with different values.
        anim.start(
            "x",
            50.0,
            60.0,
            Duration::from_millis(100),
            Easing::Linear,
            t0,
        );
        let v = anim.get("x", t0).unwrap();
        assert!((v - 50.0).abs() < 1e-6);
    }

    #[test]
    fn animator_target() {
        let mut anim = Animator::new();
        let t0 = Instant::now();
        anim.start(
            "x",
            0.0,
            42.0,
            Duration::from_millis(100),
            Easing::Linear,
            t0,
        );
        assert_eq!(anim.target("x"), Some(42.0));
        assert_eq!(anim.target("missing"), None);
    }

    #[test]
    fn animator_gc_removes_completed() {
        let mut anim = Animator::new();
        let t0 = Instant::now();
        anim.start("a", 0.0, 1.0, Duration::from_millis(50), Easing::Linear, t0);
        anim.start(
            "b",
            0.0,
            1.0,
            Duration::from_millis(200),
            Easing::Linear,
            t0,
        );

        let t1 = t0 + Duration::from_millis(100);
        anim.gc(t1);
        // "a" completed (50ms < 100ms elapsed) — removed.
        assert!(anim.get("a", t1).is_none());
        // "b" still active (200ms > 100ms elapsed) — kept.
        assert!(anim.get("b", t1).is_some());
    }

    #[test]
    fn zero_duration_returns_to_immediately() {
        let mut anim = Animator::new();
        let t0 = Instant::now();
        anim.start("snap", 0.0, 1.0, Duration::ZERO, Easing::Linear, t0);
        let v = anim.get("snap", t0).unwrap();
        assert!((v - 1.0).abs() < 1e-6);
    }
}
