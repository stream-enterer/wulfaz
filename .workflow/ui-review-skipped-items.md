# UI Review — Skipped Items Rationale

17 items across the gap report were marked SKIPPED. They fall into three categories.

## 1. Intentional Design Divergence (4 items)

Places where CK3 does something we deliberately don't — our visual identity is flat geometric panels, not CK3's textured art style.

| Area | # | Finding | Why skipped |
|------|---|---------|-------------|
| 2 | 6 | No textured window decoration (9-slice frames) | Flat panels with inner shadows — intentional aesthetic |
| 2 | 7 | No window show/hide animations (slide-in/out) | Classified as polish/future work. Animator exists but not wired to panel open/close |
| 10 | 1 | No texture support in panel renderer | Intentional — SpriteRenderer handles textures separately |
| 10 | 2 | No 9-slice, overlay blending, or masks | CK3-specific art features we don't need |

## 2. Adequate Existing Solution (6 items)

CK3 has a more elaborate version, but our simpler implementation is sufficient.

| Area | # | Finding | Why skipped |
|------|---|---------|-------------|
| 3 | 5 | No flex-grow / layoutstretchfactor | `Sizing::Percent` already distributes remaining space proportionally |
| 3 | 7 | Sizing::Fit behavior with Percent children unclear | Confirmed correct as-is — same semantics as CK3's `set_parent_size_to_minimum` |
| 4 | 5 | Single tooltip positioning algorithm vs CK3's 8-template system | We only use cursor-following tooltips |
| 4 | 6 | Single tooltip style vs CK3's GlossaryTooltip variant | No glossary system exists |
| 6 | 6 | No multi-property animation | Multiple named animation keys achieve the same result |
| 8 | 5 | No scrollbar track or edge fade | Minimalist thumb-only is acceptable |

## 3. Deferred as Low Priority (7 items)

Real value, but no concrete screen needs them yet. Each has a backlog entry.

| Area | # | Finding | Backlog | Why deferred |
|------|---|---------|---------|--------------|
| 6 | 5 | No cubic bezier curves | UI-D24 | Fixed easing curves (Linear, EaseOut, EaseIn) suffice |
| 6 | 8 | No animation state machine / multi-step chaining | UI-D16 | No screen requires sequenced animations yet |
| 7 | 3 | Missing 4th font size tier (9/12/16 vs CK3's 13/15/18/23) | UI-D13 | Current 3-tier scale works for existing screens |
| 7 | 7 | No inline text formatting DSL (`#high;bold;size:18`) | UI-D11 | RichText with explicit spans adequate for current screen count |
| 7 | 8 | No glow/shadow text effects | UI-D12 | No screen uses text effects |
| 8 | 4 | No grid layout widget (fixedgridbox) | UI-D17 | No screen requires grid layout yet |
| 8 | 7 | No sort controls on lists | UI-D18 | Low priority until entity counts exceed ~200 |

## Bottom Line

None of these represent bugs or broken functionality. They are either "we chose a different aesthetic", "what we have works fine", or "real but not worth building until a concrete screen needs it".
