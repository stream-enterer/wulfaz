# CK3 UI Reference Index

Source: `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Crusader Kings III/game/gui/`
192 root .gui files + 172 in subdirectories. ~258k lines total.

For deep reads of specific files, use the path above + filename.

---

## File Map

### Root — Windows & HUD
| File | What | Lines |
|------|------|-------|
| `window_character.gui` | Character panel (sidebar, 610px, most complex) | ~2500 |
| `window_character_finder.gui` | Character search (floating, 745px) | ~800 |
| `window_council.gui` | Council (main tab, 655px) | ~1200 |
| `window_military.gui` | Military overview (main tab) | ~2000 |
| `window_court.gui` | Court (main tab) | ~1500 |
| `window_title.gui` | Title/holding (sidebar, 650px) | ~1200 |
| `window_intrigue.gui` | Intrigue/schemes (main tab) | ~1800 |
| `window_inventory.gui` | Artifacts (floating dialog, 1222x840) | ~1000 |
| `window_decisions.gui` | Decisions list | ~800 |
| `window_factions.gui` | Factions | ~600 |
| `hud.gui` | Main HUD layout (tabs, portraits, speed) | ~1500 |
| `hud_top.gui` | Top bar (autosave indicator) | ~50 |
| `hud_bottom.gui` | Bottom (mouse blockers, pause text) | ~400 |
| `hud_sidebars.gui` | Right sidebar background/decoration | ~300 |
| `hud_outliner.gui` | Outliner (expandable tree, 300px wide) | ~500 |
| `interaction_templates.gui` | Shared interaction layout | ~800 |
| `interaction_confirmation.gui` | Confirmation dialog | ~400 |
| `interaction_marriage.gui` | Complex interaction example | ~1200 |
| `interaction_declare_war.gui` | War declaration | ~1500 |
| `console.gui` | Debug console | ~700 |

### Subdirectories
| Dir | Count | What |
|-----|-------|------|
| `shared/` | 28 | **Core widgets** — backgrounds, buttons, progressbars, windows, tooltips, colors |
| `preload/` | 6 | Defaults, fonts, text formatting, tooltip config |
| `event_windows/` | 12 | Event presentation (character, letter, fullscreen, scheme) |
| `event_window_widgets/` | 25 | Injected event widgets (stress, scheme info, naming) |
| `activity_window_widgets/` | 35 | Activity UI components |
| `activity_locale_widgets/` | 33 | Activity locale visuals |
| `decision_view_widgets/` | 12 | Decision detail panels |
| `settings/` | 2 | Settings/game rules UI |
| `debug/` | 10 | Debug tools |
| `notifications/` | 1 | Notification templates |

---

## DSL Quick Reference

```
widget "name" {
    size = { 400 300 }                    # fixed WxH
    layoutpolicy_horizontal = expanding   # stretch to fill
    parentanchor = top|right              # absolute position anchor
    widgetanchor = top|right              # widget's own anchor point
    position = { -50 65 }                 # offset from anchor
    visible = "[SomeCondition]"           # data binding
    alpha = 0.8
    using = TemplateName                  # mixin
    blockoverride "slot_name" { ... }     # override template slot
}

types GroupName {
    type my_widget = button_standard { ... }  # type definition
}

template MyTemplate { size = { 100 100 } }    # reusable property set
block "slot_name" { default_content }          # overridable slot
```

---

## Window Families

### Main Tab (right-side panels)
- `using = Window_Size_MainTab` → 655 x 100%
- `parentanchor = top|right`, `layer = windows_layer`
- Wrapped in `margin_widget` (margins: 30T, 25B, 13R)
- Slide animation: show at x=0, hide at x=40 (slides right)
- Used by: council, military, court, intrigue, factions

### Sidebar (left-side panels)
- `using = Window_Size_Sidebar` → 610 x 100%
- `using = Window_Background_Sidebar`, `layer = middle`
- No margin_widget wrapper, uses `Window_Margins_Sidebar` (R18)
- Slide animation: show at x=0, hide at x=-60 (slides left)
- Used by: character, title

### Floating Dialog (centered)
- Fixed size, `parentanchor = center`, `layer = middle`
- `using = Window_Background`, `using = Window_Decoration_Spike`
- `using = Window_Movable` (draggable, min 200px from edge)
- Used by: inventory, character_finder

### Modal/Confirmation
- `layer = confirmation` (highest), 100% x 100%
- `Background_Full_Dim` (black 0.5 alpha backdrop)
- Click-outside-to-dismiss via invisible button behind content
- Used by: dialogs, settings, game rules

---

## Layer Priority System

```
debug          = 50
confirmation   = 11   (modals, settings)
frontend       = 10
tutorial        = 9
top             = 8   (fullscreen events)
events          = 7   (event popups)
middle          = 6   (sidebars, floating dialogs)
royal_court     = 5
hud_layer       = 4
windows_layer   = 3   (main tab panels)
bottom          = 2
bottom_bottom   = 1
```

---

## Standard Sizes & Spacing

| Element | Value |
|---------|-------|
| Window margin L/R | 40px |
| Window margin T | 18px, B 20px |
| MainTab margin | T50 R50 L40 B45 |
| Sidebar margin | R18 only |
| Scrollbox margin | T15 B15 L15 R20 |
| Standard header | 50-56px tall |
| Illustration header | 120-155px tall |
| Tab button | 100x38 (horizontal), 180x48 (vertical) |
| Button standard | 170x33 |
| Button primary big | 250x42 |
| Button round/icon | 30x30 to 40x40 |
| Checkbox/radio | 30x30 |
| Editbox | 72x30 |
| List item row | varies, typically 92-130px tall |
| Portrait head small | 85x90 |
| Divider | 3px tall |
| Tooltip max width | 400-450px |
| Standard button margin | 3-5px |
| Header margin | 8-12px |

---

## Font System

| Name | Size | Line Height |
|------|------|-------------|
| Tiny | 13 | 18 |
| Small | 15 | 23 |
| Medium | 18 | 26 |
| Big | 23 | 33 |

Typefaces: `StandardGameFont` (body), `TitleFont` (decorative/flavor).

---

## Color Palette (text formatting)

| Name | RGB | Use |
|------|-----|-----|
| white (#high) | 0.87, 0.84, 0.75 | High contrast |
| gray (#medium) | 0.61, 0.60, 0.56 | **Default** text |
| dark_gray (#low) | 0.40, 0.39, 0.37 | Low contrast |
| red (#N) | 0.80, 0.30, 0.30 | Negative/warning |
| green (#P) | 0.40, 0.61, 0.30 | Positive |
| yellow (#M) | 0.82, 0.77, 0.50 | Mixed/highlight |
| light_blue (#E) | 0.51, 0.61, 0.65 | Explanation links |
| goldy_yellow | 0.68, 0.56, 0.41 | Clickable |
| black | 0.10, 0.10, 0.03 | Light-background text |

Contrast hierarchy: `#high` > `#medium` (default) > `#low` > `#weak` (low + italic)
Semantic: `#V`=value, `#N`=negative, `#P`=positive, `#Z`=zero, `#M`=mixed

---

## Background System

All backgrounds use a **sandwich pattern**: base texture (Corneredtiled) + overlay (blend mode) + alpha mask.

| Background | Alpha | Use |
|------------|-------|-----|
| `Background_Area` | 0.2 | Light area fill |
| `Background_Area_Dark` | 0.6 | Dark area fill |
| `Background_Area_ExtraDark` | 0.8 | Very dark fill |
| `Background_Area_Solid` | 1.0 | Solid dark (0.06/0.07/0.077 RGB) |
| `Background_Header` | — | Skinned header with color_theme |
| `Background_Tooltip` | — | Tooltip bg + frame |
| `Background_Full_Dim` | 0.5 | Black modal backdrop |
| `Background_Letter` | — | Parchment with shadow + overlay |
| `Status_Bad` | 0.7 | Red (0.5/0.2/0.15) |
| `Status_Good` | 0.7 | Green (0.24/0.32/0.18) |

---

## Cross-Cutting Patterns

### Header Pattern
`header_pattern` = Background_Header + Background_Header_Pattern + `buttons_window_control` (top-right) + centered text. Alternative: `widget_header_with_picture` = 120px+ illustration with fade mask.

### Tab Pattern
`hbox` of `button_tab` widgets, each `layoutpolicy_horizontal = expanding`. State via `[Window.IsTabShown('name')]` / `[Window.SetTab('name')]`. Content panels use `visible` binding.

### List Pattern
`fixedgridbox` with `addcolumn`/`addrow` for virtual/recycled lists. `datamodel_reuse_widgets = yes`. Wrapping via `datamodel_wrap`. Items bound via `datamodel = "[...]"`.

### Scrollbox Pattern
```
scrollbox {
    layoutpolicy_horizontal = expanding
    layoutpolicy_vertical = expanding
    blockoverride "scrollbox_content" { ... }
    blockoverride "scrollbox_empty" { text = "..." }
}
```

### Close Button
Always via `buttons_window_control` flowcontainer with blockoverride: `blockoverride "button_close" { onclick = "[Window.Close]" }`. Shortcut: `close_window`.

### Expand/Collapse
`button_expandable_toggle_field` with arrow icon that rotates. State via `VariableSystem.Toggle('flag')` / `VariableSystem.HasValue('flag')`.

### Layout Spacer
`expand = {}` — an invisible widget with `layoutpolicy_horizontal = expanding` that fills remaining space. Used for centering and push-to-end layouts.

---

## Animation Patterns

| Pattern | Duration | Curve |
|---------|----------|-------|
| FadeIn Quick | 0.15s | Default bezier |
| FadeIn Standard | 0.25s | Default bezier |
| Tab Switch | 0.15s | Default bezier |
| Window slide (main tab) | — | x: 0→40 on hide |
| Window slide (sidebar) | — | x: 0→-60 on hide |
| Staggered fade-in | 0.5-0.7s per element | Delays: 0.1, 0.2, 0.4, 0.6, 0.8s |
| Shimmer/glow sweep | 2-8s + 5s delay | colordodge translate_uv |
| Notification bounce | 1.35s total | 3-stage size 72→88→72 |
| Screen shake | 0.12-0.2s per state | 3-state loop, offsets -6 to +7px |

Default bezier: `{0.25, 0.1, 0.25, 1}` (CSS ease equivalent)

---

## HUD Layout

- **Top bar**: 88px tall. Alert icons flow R-to-L in 720x116 scissored hbox. Resource bar top-right.
- **Right tab strip**: 50px wide, offset (0, 105). 45x45 buttons in 3 groups separated by spike dividers.
- **Left tab strip**: vertical flowcontainer at bottom-left. 52x52 buttons.
- **Bottom-left**: Player portrait (scale 1.3) + war/raid banners (flowcontainer, spacing 4).
- **Bottom-right**: Time controls 649x65. Pause button 33x33. Speed buttons 34x18 × 5. Date 170px.
- **Outliner**: top-right, 300px wide, max 500px tall. Paper texture alpha 0.9. Sections: Pinned/Players/Units/Holdings/Domicile. 31px tall headers with expand/collapse.

---

## Interaction Window Layout

```
window (center, layer=middle)
  vbox
    header_pattern_interaction (title + close + icon)
    portrait_area (3-4 character layout)
    effects_area (scrollable, tabbed accept/decline)
    options (checkboxes or radio buttons)
    acceptance (score + progress bar)
    cost row (visible if HasCost)
    send button (button_primary_big, enabled=[CanSend])
```

### Confirmation Dialog
`base_dialog` at layer=confirmation with Background_Fade. Cancel = button_standard, Accept = button_primary. 15px spacer between.

---

## Event Window Variants

| Variant | Size | Layer | Key Feature |
|---------|------|-------|-------------|
| Character event | 1120x585 | events | 55% portrait area right, 45% text left |
| Letter event | 675x530 | events | Parchment bg, two-stage open animation |
| Big event (scheme) | 1390x750 | events | 3 large portrait slots (300x650) |
| Fullscreen event | 100% x 100% | top | Staggered fade-in, shake support |

Event options: `fixedgridbox { addcolumn=500, addrow=42 }` with `button_eventoption` items. Support `special` (gold glow) and `dangerous` (red glow) flags.

---

## Template/Type Composition

No class inheritance. Composition via:
1. `using = TemplateName` — mixin property sets
2. `type name = base_widget { ... }` — widget type definitions
3. `block "slot" { default }` / `blockoverride "slot" { override }` — slot-based customization
4. `VariableSystem.Set/Toggle/HasValue` — runtime UI state toggles

This is the entire composition model. No abstract classes, no interfaces, no event bubbling.
