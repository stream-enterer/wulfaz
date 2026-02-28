# CK3 shared/ GUI Index

Source: `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Crusader Kings III/game/gui/shared/`

## Core Framework Concepts

- **`template`** = reusable property mixin (applied via `using = TemplateName`)
- **`types TypeGroup { type name = base_widget { ... } }`** = widget type definitions (instantiated by name)
- **`block "name" {}`** = overridable slots; consumers use `blockoverride "name" {}`
- **Sizing**: `size = { W H }`, `layoutpolicy_horizontal = expanding`, `layoutpolicy_vertical = expanding`
- **Layout**: `vbox`, `hbox` (flex-like), `flowcontainer` (flow wrap), `fixedgridbox` (grid)
- **Sprite 9-slice**: `spriteType = Corneredtiled`, `spriteborder = { L T }` (or `Corneredstretched`)
- **Texture compositing**: `modify_texture { texture blend_mode alpha }` -- blend modes: `overlay`, `alphamultiply`, `colordodge`, `add`

---

## 1. animation.gui
Templates: `Animation_Curve_Default` (bezier 0.25,0.1,0.25,1), `Animation_Transition_Start/End`, `Animation_FadeIn/Out_Standard` (0.25s), `Animation_FadeIn/Out_Quick` (0.15s), `Animation_ShowHide_Standard/Quick` (_show/_hide states), `Animation_Tab_Switch`, `Animation_Refresh_FadeOut/In`, `Animation_Glow_Pulse`, `Animation_MapIcon_Fade`, `Glow_Standard`, `Glow_icon` (shimmer via colordodge translate_uv loop), `Glow_progress_icon`.
Types (Animations): `animation_attention_text`, `animation_attention_text_single`, `animation_glow_event`, `animation_aggressive`, `animation_sonar`, `animation_button_highlight`, `animation_progessbar_center_glow`.
Pattern: State machines with `name/next/trigger_on_create/duration`. Glow uses `modify_texture` with animated `translate_uv` or `rotate_uv`. Standard durations: 0.15s (quick), 0.25s (standard), 0.4-1.6s (glow pulses).

## 2. backgrounds.gui
**KEY FILE.** All background patterns used across the UI.
Templates -- Dark area fills: `Background_Area` (black 0.2 alpha, rough_edges mask, Corneredtiled, spriteborder 20), `Background_Area_Dark` (0.6), `Background_Area_ExtraDark` (0.8), `Background_Area_Solid` (rgb 0.06/0.07/0.077), `Background_Area_Light` (with overlay effect).
Bordered: `Background_Area_Dark_Border` (tile_frame_thin_03), `Background_Area_Border` (tile_dark_area_01, spriteborder 16), `Background_Area_Border_Solid` (tile_dark_area_02).
Headers: `Background_Header` (skinned color_theme + mask_header + noise overlay), `Background_Header_Pattern` (adds pattern mask), `Background_Title`, `Background_Label/Label_Center` (horizontal fade masks).
Frames: `Background_Frame` (mask_frame, color 0.9/0.9/0.9/0.15), `Background_Frame_Gold` (0.9/0.7/0.5/0.7).
Tooltip: `Background_Tooltip` (tooltip_bg + tooltip_frame, both with overlay_window).
Status: `Status_Bad` (red 0.5/0.2/0.15/0.7), `Status_Good` (green 0.24/0.32/0.18/0.7), `Status_Mixed` (yellow), `Status_Highlight` (gold), `Status_Suggestion` variants.
Packaged: `Background_Inset` (Area + scrollbar_fade), `Background_Letter` (shadow + flat + overlay texture + frame), `Background_Fade` (circular radial dim for popups), `Background_Full_Dim` (black 0.5), `Background_DropDown` (Area_Solid + margin 2,6), `Background_Vignette_Button`, `Background_Tab_Area`, `Background_Bottom_Fade`.
Masks: `Mask_Rough_Edges` (alphamultiply, spriteborder 20, texture_density 2), `Mask_Background_Header`.
Skinned pattern: `Background_Color` (tile_window_background_no_edge, alpha 0.8), `Background_Pattern` (color_theme + pattern mask, overlay blend, alpha 0.5).
Alternating rows: `Background_Alternate_Datamodel` (modulo index check).

## 3. buttons.gui
Types (Buttons):
- `button_standard` = THE core button. togglepushbuttongfx, 170x33, Corneredtiled spriteborder 4, framesize 252x80. Has: Background_Color, overlay (button_standard_overlay, overlay blend 0.3), Background_Pattern, highlight_icon for mouseover (button_mouseover, Corneredstretched spriteborder 20), disabled stripes, vignette inner (margin 3). Uses 7-frame spritesheet (up/hover/press/down variants + disable).
- `button_primary` = button_standard + confirm shortcut + frame 4 bg + pattern overlay + #high format.
- `button_primary_big` / `button_standard_big` = 250x42 variants.
- `button_standard_clean` = no vignette/pattern/bg color. `button_standard_list` = clean_weak texture. `button_standard_hover` = upframe 9.
- `button_tab` = 100x38, togglepushbuttongfx, Corneredtiled spriteborder 10, framesize 198x53. `button_tab_dark`, `button_tab_vertical` (180x48), `button_tab_vertical_bookmark` (90x52, slide animation).
- `button_frontend` = 310x55, framedbuttongfx, Corneredtiled spriteborder 50/30.
- `button_event_standard` = 170x33, event.dds, spriteborder 3, framesize 249x78.
- `button_event_letter` / `button_letter` = 260x34, paper texture overlays.
- `button_list` = togglepushbuttongfx, Corneredtiled spriteborder 50, framesize 150x150.
- `button_round` = button_icon 40x40 with round_frame + round_bg + Color_Button_Background + overlay.
- `button_checkbox` = checkbutton 30x30, framesize 80x80. `button_checkbox_label` = flow with checkbox + text_single.
- `button_radio` = button_toggle 30x30, framesize 80x80.
- `button_drop` = dropdown trigger, 100%x33. `button_dropdown` = dropdown option, 225x30.
- `buttons_window_control` = flowcontainer of go_to/pin/me/back/minimize/close icon buttons (margin 8).
- `selection_glow` = pulsing glow widget (alpha 0.3-0.5).
- `button_sidepanel_right/left` = hover button + flow with text + arrow.
Key pattern: `blockoverride` for "button_standard_mouseover", "button_standard_current", "disabled", "vignette", "background_color", "button_pattern".

## 4. buttons_icons.gui
Types (ButtonIcons): ~80 icon button definitions. All inherit `button_icon` (base: button 30x30, togglepushbuttongfx, 4-frame). Examples: button_close, button_back, button_minimize, button_pin, button_search, button_plus, button_minus, etc. `button_icon_custom` = 50x50 base. Frontend buttons (button_account, button_settings, etc.) = 45x45.
`Master_Button_Modify_Texture` = adds colors_textured.dds with `add` blend, framesize 96x96, block "master_color_frame" for frame selection. This is used on ALL icon buttons for tinting.

## 5. cards.gui
Types (ObjectCards): `vbox_generic_object_card` = vbox with header (hbox) + contents. Header has colored background (fade mask + pattern), upper/lower line margin_widgets. `button_clickable_object_card` = button_standard_clean wrapping card.
Templates: `GenericObjectCardDefaultBackground` (Area_Dark + Frame, alpha 0.5), `GenericObjectCardHeaderBackground` (white + rough edges + horizontal fade mask + pattern at 0.2 alpha). Header margins: 12px. Contents spacing: -8.

## 6. coa_designer.gui
Types (CoatOfArmsDesignerTypes): Complex CoA editor. `vbox_coa_designer` = main layout (center preview + right panel with tabs). Tab pattern: button_tab with VariableSystem state management. Uses `scrollbox` with `fixedgridbox` (addcolumn 92, addrow 92, datamodel_wrap 5). Color selection via palette grid (50x50 cells, wrap 7) + expandable colorpicker. Detail editing via scrollbar sliders (`hbox_scrollbar_coa_label` with `scrollbar_value_slider`). Pattern: VariableSystem.Set/Toggle/HasValue for page/tab state.

## 7. coat_of_arms.gui
Types (CoATypes): Realm shield widgets at various sizes (tiny/small/medium/big/huge). `coat_of_arms_icon` = icon with coatofarmsgfx shader. Pattern: Each size has crown strip (framesize-based tier), government-specific frame textures, hover glow. Uses `@overlay_alpha = 0.4`. Dynasty/house/title/realm variants with different frame textures.

## 8. color_picker.gui
Types (ColorPickerTypes): `colorpicker_simple` = 400x350, vbox with preview (298x50) + hue slider (32x256) + saturation/value area (256x256). Uses pdxgui_colorpicker.shader. `colorpicker_simple_popup` = 32x32 preview that opens a 318x276 window. `hbox_colorpicker_simple_components` = the actual picker grid.
`dummy_color_picker_buttons` template hides required-but-unused engine controls.

## 9. colors.gui
Color templates: `Color_Green` (0.5/0.65/0.2), `Color_Bright_Yellow` (0.9/0.9/0.6), `Color_Red` (1/0.4/0.35), `Color_Purple` (0.55/0.5/0.6), `Color_White` (1/1/1), `Color_Blue` (0.4/0.5/0.6), `Color_Grey` (0.45/0.5/0.55), `Color_Orange` (0.9/0.7/0.5/0.7), `Color_Black` (0/0/0/0.7), `Color_Button_Background` (0.1/0.1/0.13). Transparent variants at 0.2 alpha. Semantic aliases: Color_Holding_Leased = Blue, Color_County_OutsideRealm = Red.

## 10. cooltip.gui (LARGE)
Tooltip widget definitions. `DefaultTooltipWidget`, `GameConceptTooltipDefault` (with icon 52x52, description max_width 400), `GlossaryTooltip`. `character_opinion_tooltip` template (Background_Area heading, opinion value with solid_black_label bg, margin 3/1). Pattern: `GeneralTooltipSetup` + `DefaultTooltipBackground`, `set_parent_size_to_minimum = yes`, margin 8, text max_width 400. Object tooltips use `object_tooltip_pop_out` with blockoverrides for title/description/icon.

## 11. court_positions.gui
Empty file.

## 12. dialogs.gui
Types (Dialogs): `base_dialog` = window, parentanchor center, layer confirmation, 100%x100%, filter_mouse all. Uses Background_Fade + Window_Background_Popup. Content: vbox with `set_parent_size_to_minimum`, header_standard + description text_multi (max_width 430) + buttons.
`confirmation_popup` = base_dialog with cancel (button_standard) + accept (button_primary), hbox layout with 15px spacer. `rename_popup` = editbox_standard + optional colorpicker.
Pattern: dismiss via `[GameDialog.Decline]`, shortcut = close_window for cancel, confirm for accept.

## 13. edit_boxes.gui
Types (Editboxes): `editbox_standard` = margin_widget (margin 5L/5R/5T), size 72x30, tile_editbox.dds Corneredtiled spriteborder 16, inner editbox with focuspolicy=all, default_format="#high". `editbox_standard_with_label` = vbox with text_single label + hbox with editbox. `editbox_search_field` = hbox with search icon (30x30) + editbox.

## 14. event_windows.gui
Types (Events): `event_window_dimmer_widget` = radial dim (same as Background_Fade). `button_eventoption` = button_event_standard 500x36, with faded horizontal middle mask + overlay. Portrait status icon containers for event positioning. Pattern: event options use EventOption.Select onclick, with highlight_icon mouseover.

## 15. icons.gui
Types (IconTypes): `icon_flat_standard` = base flat icon using `Icon_Flat_Standard` template. Color variants: `icon_flat_standard_red` (frame 9), `_green` (8), `_gold` (1), `_black` (10), `_ash` (4) -- all use colors_textured.dds with `add` blend, framesize 96x96. Sized aliases: `icon_doctrine` (60x60), `icon_building` (75x65), `icon_culture_pillar` (44x44). Frontend icons at 30x30.

## 16. lines.gui
Types (Lines): `line` = line with linegfx. Templates: `Line_DynastyTree` (width 5, tiling_noise texture, screenspace effect), `Line_Lifestyles_Base` (width 13, uv_scale 0.01/1.0), `Line_Lifestyles_Unlocked/CanUnlock/Unavailable` (animated/static variants), `Line_Domicile_*` (same pattern). `ArrowLine` (width 10, animated uv).

## 17. lists.gui
Templates/types for character list items. `widget_character_list_item` = 10x110, portrait_head_small + button_standard with Corneredtiled spriteborder 10/30, character_list_arrow highlight_icon. `character_list_arrow` = 14x14, framesize 14x28. `character_age_health` = hbox with age text + health icon (23x23, 5-frame). `widget_skill_item_no_icon` = 32x25 with skill bg framesize 70x26. `dropdown_menu_standard` referenced for filter dropdowns.

## 18. mapmodes.gui
Types (MapModes): `map_modes_debug` = flowcontainer of button_round widgets for debug map modes. `icon_button_mapmode` in buttons.gui is the real map mode button (40x40 with glow circle + button_round).

## 19. misc_components.gui
Types (Miscelaneous): `skill_icon_label` = flowcontainer with skill bg + icon + text. `skill_icon_grid` = dynamicgridbox, flipdirection, wrap 6. `strength_balance` = hbox with balance icon (60x60, 3-frame) + text bg. `button_expandable_toggle_field` = button_tertiary with expand arrow + text for fold-out sections. `hbox_tab_buttons` = datamodel-driven tab row. `error_horse` = debug error display.
Template: `Create_Resetting_Fold_Out` (oncreate binds + sets unfolded).

## 20. popups.gui
Types (Popups): `activity_pulse_action_popup_right` = 750x124 toast notification. Pattern: edge texture (toast_blue_bg_edge) + torn-edge body (toast_blue_bg with mask_seamless_torn_edge_vertical alphamultiply). Auto-hides after 5s delay. Content: portrait + icon + title (with Toast_Header background + Color_Blue) + effects text_multi (420px, light_background format).

## 21. portraits.gui
Types (PortraitTypes): `portrait_head_small` = 85x90 widget with Background_Area_Solid bg + vertical fade. Inner portrait_button 80x100 with mask. Prison bars overlay. Portrait rank frames from portrait_rank.dds (framesize 196x194, tier-based frame). Variants at different sizes. Pattern: background glows, CoA overlay, opinion box, status icons all via blockoverride slots.

## 22. progressbars.gui
Types (ProgressBars):
- `progressbar_standard` = 50x20, min 0 max 100, Corneredtiled spriteborder 6, progress_standard.dds (fill) + progress_red.dds (empty), with progress_overlay.dds icon on top (Corneredstretched spriteborder 2).
- Variants: `progressbar_standard_transparent`, `progressbar_red` (red fill/black empty), `progressbar_green`, `progressbar_frozen`.
- `progressbar_royal_court` = grandeur bar, same structure.
- `progressbar_segmented_chance` = flowcontainer of frame-based icons (45x45 framesize, 25x25 display).
- `hbox_complex_bar_progress` = stretch-factor-based multi-section bar (left empty + filled + right empty via layoutstretchfactor_horizontal). `hbox_complex_bar_progress_next` adds increase(green)/decrease(red) prediction sections.
- `hbox_complex_bar_levels` = datamodel-driven level markers.
- `widget_level_marker` = 0x40 with glow animation (rotating mask), active/inactive bg icons, centered text.
- `arrow_progressbar_icon` = animated arrow texture (corneredtiled, translate_uv loop).
Templates: `Progressbar_Arrow_Animation` (repeat_texture shader + arrow texture + fade mask), `Progressbar_Changed_Animation` (0.5s curve).

## 23. sounds.gui
Sound templates: `Sound_WindowShow/Hide_Standard/Small/Sidebar/Suggestion`, `Sound_Window_Ambience_*`, snapshots (`Sound_Window_AmbienceMute_Snapshot`, `Sound_Panel_Popup_Snapshot`).
Types (ButtonSounds): `button_normal` = button (no sound), `button_toggle` = checkbox click sound, `button_arrow`, `button_list_new`, `button_increment/decrement`.
Types (SoundTypes): `widget_gamespeed_sounds` (play/pause sfx), `widget_stress_sounds` (stress level parameterized).
Pattern: sound via `state { name = _show; start_sound { soundeffect = "event:/..." } }`.

## 24. texticons_religion.gui
Texticon definitions for religion icons (catholic, orthodox, etc.). Pattern: `texticon { icon = name; iconsize { texture size={25,25} offset={0,6} fontsize=16 } }`. Pure data.

## 25. texticons_trigger.gui
Texticon definitions for trigger/condition icons (trigger_pass, trigger_fail, trigger_pass_inactive, trigger_fail_inactive). Size 18x18, offset {0,2}.

## 26. texticons_ui.gui
More texticons for UI elements. Same pattern as trigger icons (18x18 or 25x25).

## 27. value_breakdown.gui
Types (Breakdowns): `widget_value_breakdown_list` = vbox with datamodel ValueBreakdown.GetSubValues, margin 20/10. Each row: name text (min 180, max 275) + value text (right-aligned), with recursive tooltip support. Background_Area_Border_Solid. `widget_value_breakdown_tooltip` wraps list with tooltip background and header.

## 28. windows.gui
**KEY FILE.** Window infrastructure.
Layers (priority): debug(50) > confirmation(11) > frontend(10) > tutorial(9) > top(8) > events(7) > middle(6) > royal_court(5) > hud_layer(4) > windows_layer(3) > bottom(2) > bottom_bottom(1).
Window sizes: `Window_Size_Sidebar` = 610x100%, `Window_Size_MainTab` = 655x100%, `Window_Size_CharacterList` = 745x88%.
Window margins: `Window_Margins` = L40 R40 T18 B20. `Window_Margins_Sidebar` = R18. `Window_Margins_MainTab` = T50 R50 L40 B45. `Scrollbox_Margins` = T15 B15 L15 R20.
Window backgrounds: `Window_Background` (tile_window_background, Corneredtiled spriteborder 18/0, texture_density 2, overlay_effect). `Window_Background_Popup` (no_edge + popup frame, spriteborder 80, margin 2). `Window_Background_Sidebar` (spriteborder_right 23). `Window_Background_Subwindow` (no_edge + tooltip_frame with color 1.77/1.77/1.80). `Window_Background_NoDecoration`, `Window_Background_No_Edge`.
Window decorations: `Window_Decoration` = top/bottom frame strips (Corneredtiled spriteborder 100/0, texture_density 2, 100%x22) + center ornament (142x60, positioned -38 above). Variants: `_Spike`, `_Flat`, `_Warfare`, `_Frontend*`.
Types (WindowTypes):
- `spacer` = empty widget. `expand` = hbox with growing policy (flexbox spacer).
- `header_standard` = 0x50 widget with tiled bg + gradient + pattern + buttons_window_control (top|right) + header_text (top|hcenter, Font_Type_Flavor + Font_Size_Big, max 400px).
- `header_pattern` = Background_Header + Background_Header_Pattern + window controls + text.
- `header_with_divider` = vbox with dividers + multiline text.
- `widget_header_with_picture` = 0x120, illustration (centercrop + mask) + header overlay.
- `divider` = icon 3x3, white at color 0.1/0.1/0.1/0.8, edge fade + scratches masks.
- `divider_light` = same at 0.3/0.3/0.35/0.8.
Template: `Window_Movable` = `movable = yes; min_dist_from_screen_edge = 200`.

## Key Reusable Patterns

**Background sandwich**: bg texture (Corneredtiled) + overlay (Corneredstretched, blend overlay) + mask (alphamultiply). Standard rough edges on everything interactive.

**Button hierarchy**: button_normal (sound base) -> button_standard (the workhorse) -> button_primary/button_standard_big/etc. All buttons use `togglepushbuttongfx` with 7+ frame spritesheets. Mouseover via separate `highlight_icon` layer. Disabled state via diagonal stripes overlay.

**Color system**: Named color templates (Color_Red, etc.) applied via `using =`. Button tinting via `Master_Button_Modify_Texture` (colors_textured.dds, `add` blend, frame-selected).

**Layout idiom**: `vbox { layoutpolicy_horizontal = expanding; ... expand = {} ... }` for fill-remaining-space. `hbox { ... expand = {} ... text ... expand = {} }` for centering.

**Tooltip positioning**: `using = tooltip_se/ne/ws/es/nw/above/below` templates (not defined here but widely referenced).

**Scrollbox customization**: `scrollbox { blockoverride "scrollbox_background" {} blockoverride "scrollbox_margins" { margin = ... } blockoverride "scrollbox_content" { ... } }`.

**Tab state**: `VariableSystem.Set/Toggle/HasValue` for page switching. Tabs use `button_tab { down = "[VariableSystem.HasValue(...)]" onclick = "[VariableSystem.Set(...)]" }`.

**Standard spacing/margins**: button margin 3-5px, header margin 8-12px, scrollbox margin 15-20px, window margin 18-40px. Standard text margins 10px horizontal.
