# The Panel Frame Bible: A Practitioner's Analysis of Golden-Age Strategy Game Interface Design

## Thesis

The panel frame -- the partitioning of the display surface into an opaque persistent viewport and one or more opaque persistent panels -- is the defining interface pattern of golden-age strategy games (c. 1989--2003). Its variations are not aesthetic choices. They are functional responses to genre-specific interaction patterns, pointing-device ergonomics, information-delivery requirements, and hardware constraints. This document traces the concrete lineage of every major variant, analyzes the mechanical reasons each design succeeded or failed, and distills actionable implementation principles for a developer building this pattern today.

---

## Part I: The Westwood Sidebar Lineage

### Dune II (1992): The Ur-Sidebar

Dune II ran at 320x200 in 256-color VGA. The right sidebar consumed roughly 128 of those 320 horizontal pixels -- a full 40% of the screen width. This was not a considered UI decision so much as a hardware necessity: at 320 pixels wide, you cannot build a useful control surface in less space. The sidebar contained, from top to bottom: a radar minimap, the player's spice credit counter, and a build menu showing one construction option at a time with scroll arrows.

The critical design fact about Dune II's sidebar is that it was **modal and sequential**. You could not queue buildings. You could not see all available options simultaneously. You selected a unit, then clicked a sidebar command icon (Move, Attack), then clicked a destination. This three-click pattern was slow and error-prone. Every interaction required a round-trip from viewport to sidebar and back.

The sidebar's 40% width was the original sin. It meant the viewport -- the actual game -- occupied only 192x200 pixels of usable space. Players spent more time looking at UI chrome than at the battlefield. But this established the right-side convention that would persist for a decade.

### Command & Conquer (1995): The Refinement

C&C moved to 640x480, immediately doubling the pixel budget. Westwood narrowed the sidebar to approximately 160 pixels out of 640 -- roughly 25% of screen width. This was not arbitrary. At 640 pixels wide, the viewport at 480 pixels was now wide enough to show meaningful tactical terrain. The sidebar could shrink because higher resolution meant smaller icons could remain legible.

The sidebar gained dual-column layout: structures on one tab, units on another, with tab-switching buttons. This was the first **information-architecture** innovation in the pattern. Rather than scrolling through a single list, players could switch context between "what to build" and "what to train." C&C also eliminated the explicit command buttons from the sidebar -- unit commands became context-sensitive cursor modes (right-click to move, left-click to attack). This removed an entire class of sidebar-to-viewport round-trips.

The sidebar still held: radar minimap (top), credit display, power bar (a brilliant innovation showing supply/demand at a glance), and the tabbed build queue.

### Red Alert (1996): Functional Accretion

Red Alert added repair, sell, and map-reveal buttons to the sidebar. These are **infrequent but critical** operations -- you do not repair constantly, but when you need it, the button must be immediately findable. Placing them as persistent sidebar buttons rather than burying them in menus was a correct information-architecture decision: the sidebar became a complete command surface for base management.

### Tiberian Sun (1999): The Transparency Experiment

Tiberian Sun experimented with making the sidebar edges semi-transparent, attempting to recover some of the viewport real estate lost to the sidebar. This was a principled attempt to soften the hard partition between panel and viewport. It largely failed: the transparency made the sidebar harder to read against busy terrain, and it created visual noise at the boundary. The lesson is that the panel frame's opacity is load-bearing. The hard edge between panel and viewport serves a perceptual function -- it creates two distinct visual fields that the eye can process independently. Softening that edge degrades both.

### Red Alert 2 (2000): Peak Sidebar

Red Alert 2 represents the apex of the Westwood sidebar. The minimap was refined, the build tabs were clean, the power bar was immediately readable. At this point the sidebar had been iterated for eight years and every pixel was justified. The sidebar width remained at roughly 25%, but the information density within it was high: minimap, credits, power bar, two-column build queue with progress indicators, and persistent utility buttons.

### Generals (2003): The Capitulation

When EA Pacific (formerly Westwood Pacific) built Generals, they abandoned the sidebar entirely in favor of a bottom bar. The stated rationale was modernization -- the bottom bar "was by then more prevalent in the genre." But the deeper reason is that Generals was designed around a different interaction model. The sidebar excels at **base building**, where the player repeatedly selects structures from a catalog. Generals introduced the general's-powers system, where abilities are unlocked through combat experience rather than construction. The sidebar's vertical build queue was no longer the primary interaction surface. The bottom bar provided space for both unit commands and the new powers system in a horizontal layout.

This is the key insight about the sidebar's decline: **it was optimized for one specific interaction pattern (sequential construction from a catalog), and when that pattern ceased to dominate, the layout ceased to serve.**

### Why the Sidebar Narrowed

The narrowing from 40% to 25% follows directly from resolution increases. At 320x200, 128 pixels was the minimum for legible icons and text. At 640x480, 160 pixels provided the same legibility with more room for layout. The sidebar's absolute pixel width barely changed; its proportional share fell because the viewport grew. This teaches a general principle: **panel dimensions should be driven by content legibility requirements, not by proportional ratios.** A sidebar needs enough width for its widest content element (typically a two-column build grid or a minimap), and no more.

---

## Part II: The Blizzard Migration from Sidebar to Bottom Panel

### Warcraft I and II (1994--1995): The Inherited Sidebar

Warcraft: Orcs & Humans and Warcraft II used left sidebars clearly derived from the Dune II pattern. The left placement was a minor differentiation from Westwood's right-side convention, but the functional architecture was identical: minimap at top, unit portrait and stats in the middle, command buttons at the bottom of the sidebar.

The left-side placement is slightly worse from a Fitts's Law perspective for right-handed mouse users. The natural resting position of the cursor trends toward screen center or slightly right-of-center. A right sidebar means the mouse travels less distance to reach panel controls. Blizzard presumably chose left to visually distinguish from C&C, not for ergonomic reasons.

### StarCraft (1998): The Revolutionary Shift

StarCraft moved the entire control surface to the bottom of the screen, creating the three-region triptych that would define RTS interfaces for the next decade:

- **Left region: Minimap** (~128x128 pixels in the bottom-left corner)
- **Center region: Information panel** (unit portrait, wireframe, stats, production queue)
- **Right region: Command card** (3x3 grid of ability/command buttons)

The bottom panel consumed approximately 150 pixels of vertical space out of the 480-pixel-tall screen -- roughly 31% of screen height. The game viewport occupied the remaining 330 pixels of height across the full 640-pixel width.

This was a dramatic improvement in viewport utilization. The Westwood sidebar consumed 25% of horizontal space across the full screen height. StarCraft's bottom panel consumed 31% of vertical space across the full screen width. In raw pixel area:

- Westwood sidebar: ~160 x 480 = 76,800 pixels of panel area. Viewport: ~480 x 480 = 230,400 pixels.
- StarCraft bottom panel: ~640 x 150 = 96,000 pixels of panel area. Viewport: ~640 x 330 = 211,200 pixels.

StarCraft's panel is actually *larger* in absolute pixel area, but the viewport is wider and the aspect ratio better matches the horizontal bias of terrain perception. A wider, shorter viewport shows more lateral terrain, which matters for flanking, army positioning, and base layout -- the core spatial reasoning tasks of RTS play.

### The Fitts's Law Argument

The bottom-panel layout provides a concrete Fitts's Law advantage that the sidebar cannot match. The minimap sits in the bottom-left corner of the screen. Screen corners are "infinite" targets in two dimensions -- the cursor physically cannot overshoot because two screen edges converge. This makes the minimap trivially easy to hit with a fast, imprecise mouse flick.

In a right-sidebar layout, the minimap sits at the top of the sidebar -- typically at coordinates roughly (560, 80) on a 640x480 screen. This position is not a corner (there is screen above it and to the right of it within the sidebar). Hitting the minimap requires precision in both axes. At high APM, this difference compounds. Professional StarCraft players perform 300--400 actions per minute; even a 50-millisecond improvement per minimap interaction adds up across a 15-minute game.

The command card in the bottom-right is also near a screen corner, providing a similar (though less dramatic) Fitts's Law benefit for right-handed players.

The center information panel is NOT at a screen edge and therefore does not benefit from Fitts's Law. But the center panel is primarily *read* (you glance at unit stats), not *clicked*. The information panel optimizes for visual scanning, not pointing. This functional decomposition -- edges for interaction targets, center for information display -- is a principled Fitts's Law optimization.

### Why Blizzard and Ensemble Chose Bottom Over Side

The hypothesis is that multiple factors converged:

1. **Wider viewport** better serves the lateral spatial reasoning of RTS combat.
2. **Corner minimap** is a Fitts's Law win for the most frequent panel interaction (checking map state).
3. **The triptych layout** (Where/What/Do) maps cleanly onto a horizontal strip but poorly onto a narrow vertical strip.
4. **Aspect ratio trends**: monitors were moving from 4:3 to wider ratios, making vertical space more scarce and horizontal space more abundant. A bottom panel trades the more plentiful dimension.

---

## Part III: The Ensemble Information-Density Philosophy

### Age of Empires II (1999): Peak Information Density

Age of Empires II is arguably the densest panel frame ever shipped. The interface has three distinct zones:

**Top resource bar**: Four resource counters (food, wood, gold, stone) plus population (current/maximum) plus current age indicator, all in a single horizontal strip consuming roughly 20 pixels of height. This is the most information-efficient element in golden-age strategy UI. Each resource uses an icon plus a number -- no labels, no progress bars, no decoration. The player learns the four icons once and thereafter reads pure numbers.

**Bottom panel (left)**: Minimap with terrain-type toggle buttons adjacent.

**Bottom panel (center/right)**: Unit portrait, stats (HP, attack, armor, range displayed numerically), and command buttons. When a building is selected: production queue, garrison count, rally point button. When idle villagers exist: a dedicated idle-villager button near the minimap.

The idle-villager button deserves special attention. It is a **derived-state indicator** -- it communicates not raw data but a judgment ("you have villagers doing nothing, which is bad"). This is a higher level of information abstraction than showing a number. It collapses an entire scan of the map into a single glanceable element. Ensemble also added the town bell button, which commands all villagers to garrison -- another abstraction that collapses dozens of individual commands into one.

### Ensemble vs. Blizzard: Contrasting Approaches

StarCraft's command card is a 3x3 grid consuming significant panel real estate. It prominently displays *commands the player can issue*. Age of Empires II's command area is smaller and less prominent; it shows *what the player is looking at* with more emphasis on stats.

The difference reflects genre-specific interaction patterns. In StarCraft, unit abilities are complex and varied: Stim Pack, Siege Mode, Cloaking, Psionic Storm. The command card must teach the player what each unit can do. In Age of Empires II, most units have only Attack and Patrol as active commands; the tactical depth comes from *which* units to build and *where* to position them. The panel therefore allocates more space to numeric stats (attack/armor/range comparisons drive composition decisions) and less to command buttons.

The principle: **panel real-estate allocation should mirror the decision structure of the game.** If decisions are primarily "what to do with this unit," the command card dominates. If decisions are primarily "which units to build and where," stats and production dominate.

---

## Part IV: The Impressions Sidebar Orthodoxy

### Caesar III (1998), Pharaoh (1999), Zeus (2000), Emperor (2002)

While the RTS world migrated to bottom panels, Impressions Games maintained right sidebars across four successive city builders. The sidebar contained: minimap at top, hierarchical building menus below, and advisor/overlay buttons.

The persistence of the sidebar in city builders is not conservatism -- it reflects a fundamentally different interaction pattern. City builders have:

1. **Deep hierarchical building categories**: Water infrastructure, entertainment, religion, government, military, housing. A vertical sidebar naturally accommodates a hierarchical menu with expandable categories far better than a horizontal bottom bar.
2. **No unit commanding**: There is no command card. The primary interaction is "select a building type, then place it." The sidebar IS the command surface.
3. **Deliberate pacing**: There is no APM pressure. The player can afford the slightly longer mouse travel to a sidebar because decisions are made over seconds, not milliseconds.
4. **Persistent tool state**: When you select "build road" in Caesar III, that tool stays active until you explicitly switch. The sidebar is a toolbox, not a command card. Toolboxes are conventionally vertical (paint programs, CAD applications, image editors).

The overlay system in Caesar III deserves dedicated analysis. The sidebar includes buttons that toggle the entire viewport into data-visualization mode: water coverage, fire risk, desirability, crime, damage, entertainment. Each overlay replaces the normal city view with a schematic showing coverage radii, risk levels, or influence values using color-coded columns on buildings.

This is a **viewport augmentation** system that works in partnership with the panel frame. The overlays make the viewport itself carry information that would otherwise need to live in the panel. A status panel showing "water coverage: 73%" is less useful than an overlay showing *which specific blocks* lack water. The overlay system reduces pressure on panel information density by making the viewport do informational work.

The hotkey bindings (F for fire, W for water, D for damage, C for crime, Space to toggle) further demonstrate this complementarity: rapid overlay toggling lets the player use the viewport as a multi-layer information display, with the panel controlling which layer is active.

---

## Part V: The Maxis and Sawyer Alternatives

### SimCity 2000 (1993): The Desktop-Application Convention

SimCity 2000 uses a toolbar-and-menu layout borrowed directly from paint programs and desktop productivity applications. A horizontal toolbar at the top of the screen contains small icons that open cascading menus for zoning, infrastructure, and services. The "Place Forest" tool behaves like a spray-paint tool. The bulldozer works like an eraser.

This works for SimCity because the interaction model IS a paint program. The player "paints" zones onto terrain, "draws" roads and power lines, "fills" areas with services. The metaphor is direct manipulation of a canvas, not command-and-control of agents. The toolbar-and-menu pattern is the correct UI for canvas-based interaction.

It fails for RTS because RTS requires commanding autonomous agents, not painting static terrain. The toolbar metaphor collapses when the "brush" can move on its own, refuse orders, die, or fight back.

### Transport Tycoon (1994) and RollerCoaster Tycoon (1999): The Sawyer Window System

Chris Sawyer's games use a distinctive hybrid: a persistent toolbar/toolbox for tool selection, combined with floating windows for information delivery. Vehicle profit reports, station details, company finances, town ratings -- all appear in independently positionable, resizable floating windows that the player can arrange over the viewport.

This is the most powerful information-comparison architecture of the golden age. A fixed panel can show one thing at a time: one unit's stats, one building's production. Sawyer's floating windows let the player simultaneously compare train route profitability on one window, station throughput on another, and company finances on a third, all while watching the viewport underneath.

The tradeoffs are:

1. **Window management overhead**: The player must manually arrange, open, close, and resize windows. This is cognitive load that a fixed panel eliminates.
2. **Viewport occlusion**: Floating windows cover the viewport. The player must choose between information access and viewport visibility. A fixed panel makes this tradeoff permanent and predictable.
3. **No canonical layout**: Every player's screen looks different. This makes it harder to write guides, produce tutorials, or share strategies. A fixed panel provides a shared visual vocabulary.

Sawyer's choice was correct for management sims where comparison is the core cognitive task. It would be wrong for RTS where rapid spatial action dominates. The lesson: **the interaction's cognitive demand structure determines whether fixed or floating information surfaces are appropriate.**

---

## Part VI: The Three-Region Convention as Functional Decomposition

StarCraft's bottom-panel triptych answers three questions:

| Region | Question | Content |
|--------|----------|---------|
| Left (minimap) | Where am I? | Spatial context, fog of war, camera position |
| Center (info) | What am I looking at? | Selected entity stats, production state, queue |
| Right (command card) | What can I do? | Available actions, abilities, build options |

This decomposition was adopted by Age of Empires II, Warcraft III, and nearly every subsequent RTS. It works because it mirrors the decision loop: orient (minimap) -> assess (info) -> act (commands).

**Is it universal?** No. City builders do not need a persistent command card because building placement is modal (select tool, then place repeatedly). The "What can I do?" question is answered by the sidebar toolbox, not a bottom-right grid. 4X games (Civilization, Master of Orion) operate on turn-based cycles where the full screen can be dedicated to a single decision context at a time -- they use modal dialogs rather than persistent panels.

The triptych is genre-specific to real-time games with both spatial awareness and unit-command requirements. It is unnecessary for turn-based games and insufficient for management sims (which need comparison windows).

---

## Part VII: Expert vs. Novice Path Separation

In StarCraft and Warcraft III, the command card serves two radically different user populations:

**Novices** click the command card buttons. They need large, labeled, discoverable targets. The 3x3 grid with icon+hotkey-letter overlays teaches them what each unit can do. The command card is an *input surface*.

**Experts** press hotkeys exclusively. They never click the command card. For them, the command card is an *information display*: they glance at it to check cooldown timers, ability availability, and production progress. The card becomes a read-only status panel.

The golden-age masters handled this divergence through:

1. **Hotkey letters printed on icons**: The command card simultaneously teaches the hotkey mapping and provides a click target. One element serves both populations.
2. **Consistent grid positions**: "Build" is always in the same grid cell for every building. "Attack" is always in the same cell for every unit. Experts internalize positional memory; novices use the icon art. Same layout, different cognitive pathways.
3. **Audio acknowledgment**: When a command is issued (by click or hotkey), the unit speaks. "Yes, sir," "Acknowledged," "I'm on it." This confirms the action succeeded without requiring the player to verify visually. Audio serves as a **complementary confirmation channel** that offloads verification from the panel.

The audio system extends beyond command confirmation. Production-complete sounds ("Construction complete," "Nuclear launch detected"), alert sounds ("Your base is under attack"), and idle-worker notifications are all examples of the panel frame offloading information delivery to the audio channel. These reduce the frequency of panel checks and thereby reduce mouse travel to the panel. The best implementations (StarCraft, C&C) use distinctive, immediately recognizable sound cues that convey specific information: you can identify *which* unit was built or *what type* of attack is occurring from the sound alone.

---

## Part VIII: Resolution Scaling Strategies and Their Consequences

### The StarCraft Lock: Fixed Resolution for Competitive Integrity

StarCraft locked to 640x480. Blizzard's rationale was explicitly competitive: higher resolutions provide a tactical advantage because the player can see more terrain, stage attacks from further away, and use abilities (nukes, lockdown, storm) with more foreknowledge. Forcing 640x480 ensured all players had identical information access.

The consequence for panel design: the panel's pixel dimensions are absolute. The bottom bar is exactly X pixels tall at every resolution because there is only one resolution. Every icon is exactly Y pixels. The designers could pixel-tune every element for one specific layout. This is a luxury unavailable at variable resolutions.

### The Age of Empires II Approach: Fixed Panels, Expanding Viewport

Age of Empires II supported 800x600 and higher. The panel elements (resource bar, bottom panel) maintained fixed pixel dimensions. At higher resolutions, only the viewport grew. A player at 1024x768 saw more terrain but the same panel.

This creates a **competitive asymmetry** -- higher-resolution players see more -- but Ensemble deemed this acceptable because AoE2's competitive scene was less resolution-sensitive than StarCraft's (larger units, longer engagement distances, less reliance on fog-of-war edge plays).

For panel design, the fixed-panel approach means: **design the panel at the minimum supported resolution, then let the viewport absorb all additional pixels.** The panel never scales. Icons, text, and layout are pixel-fixed. This is simpler to implement and avoids scaling artifacts, but it means the panel becomes proportionally smaller at higher resolutions, potentially becoming too small on very large displays.

### Guidance for Modern Implementation

A modern panel frame must handle resolutions from 1920x1080 to 3840x2160 and beyond. The correct approach is:

1. **Design the panel at a reference resolution** (1920x1080 is the modern baseline).
2. **Scale panel elements by an integer or near-integer DPI factor**, not by resolution ratio. A 4K display at 2x scaling should show the same logical layout as 1080p, with crisper rendering.
3. **Never let the viewport shrink below the reference size.** Additional pixels always go to the viewport.
4. **Fix the panel's logical dimensions** (in "points" or "em-equivalents"), not its pixel dimensions.

---

## Part IX: Console Adaptation and What It Reveals

### StarCraft 64 (2000)

The Nintendo 64 port of StarCraft reveals exactly what the panel frame assumes about input devices. The command card was reduced to a 3x3 grid navigated by D-pad, limiting units to 9 commands maximum. Michael Morhaime himself described it as "clearly a port" not designed for the interface.

The specific failures:

1. **Minimap interaction**: A minimap requires precise pointing to indicate a map location. An analog stick cannot provide the sub-pixel precision needed to click on a specific minimap pixel representing a distant base. The minimap's usefulness collapsed.
2. **Command card speed**: Cycling through a 3x3 grid with a D-pad takes at minimum 4 directional inputs to reach the opposite corner. With a mouse, any cell is one click. The command card's advantage as a random-access interface disappears with sequential-access input.
3. **Viewport scrolling vs. panel interaction**: On PC, the mouse handles both viewport scrolling (edge-of-screen) and panel interaction (clicking buttons). On console, the stick handles viewport scrolling, but panel interaction requires a mode switch. The panel frame assumes a *single pointing device* that fluidly transitions between viewport and panel.

### Command & Conquer on PlayStation

The PS1 port of C&C reorganized the sidebar for controller navigation, sub-sorting by category (structure, infantry, vehicle, aircraft, superweapon) to reduce list-scrolling. GamePro noted "inaccurate cursor movement" as a fundamental problem. The cursor emulation was too slow for tactical play and too imprecise for small sidebar buttons.

The pattern's implicit assumptions, revealed by console failure:

- **High-precision random-access pointing** (mouse click on any screen pixel)
- **Zero-cost context switching** between viewport and panel (the mouse traverses the boundary with no mode change)
- **Edge scrolling** for viewport navigation (the same device that points also scrolls)
- **Fitts's Law dynamics** (screen edges as infinite targets; meaningful only with a free-roaming cursor)

Any platform that violates these assumptions will degrade the panel frame. Touch screens partially satisfy them (precision pointing, but no edge scrolling). Controllers satisfy none of them.

---

## Part X: The Transition Markers

### Supreme Commander (2007): The Minimap Absorbed Into the Viewport

Supreme Commander's strategic zoom allowed continuous zoom from ground-level to a full-map overview where units became abstract icons. The game shipped without a default minimap (it was available as an optional overlay). The minimap -- one-third of the sacred triptych -- was absorbed into the viewport itself. The "Where am I?" question was answered by zooming out rather than by glancing at a panel element.

This innovation reveals that the minimap was always a compression artifact: a small rendering of the map crammed into the panel because the viewport could not show the full map. When the viewport can show the full map, the minimap becomes redundant.

### Company of Heroes (2006): Contextual Overlays

Company of Heroes attached status information directly to viewport elements -- health bars above units, capture-point indicators on strategic locations, directional damage indicators in the world. Information migrated from the panel to the viewport through overlays.

### The General Trend: Panel Dissolution

The post-golden-age trend is clear: information migrates from the panel into the viewport through overlays, contextual tooltips, and strategic zoom. The panel shrinks. Modern RTS games (StarCraft II, Age of Empires IV) retain the triptych but make it smaller and more transparent. The trajectory suggests the panel frame is slowly being replaced by viewport augmentation -- but it has not been fully replaced because the three-question decomposition (Where/What/Do) remains cognitively valid even when some answers can be embedded in the viewport.

---

## Part XI: Design Principles for Modern Implementation

Drawing from the entire analysis above, here are concrete, actionable principles:

### 1. Choose Layout Based on Interaction Pattern, Not Genre Convention

| Primary interaction | Recommended layout |
|--------------------|--------------------|
| Commanding autonomous agents (RTS) | Bottom panel, three-region triptych |
| Sequential construction from catalog (city builder) | Right sidebar with hierarchical menus |
| Canvas painting (zone-based builders) | Top toolbar with tool palette |
| Multi-variable comparison (management sim) | Toolbar + floating windows |

### 2. Viewport Budget: Defend 65--75% of Screen Area

The viewport should never consume less than 65% of total screen area. At the 1920x1080 reference resolution, this means the panel may consume at most ~350 pixels of height (bottom panel) or ~480 pixels of width (sidebar). In practice, aim for 70% viewport: 330 pixels of bottom panel height or 575 pixels of sidebar width.

### 3. Place High-Frequency Interaction Targets at Screen Edges and Corners

The minimap goes in a corner. Period. Bottom-left is the convention and it is correct for right-handed users (the longest diagonal from the cursor's natural resting position, but compensated by the infinite-target property of the corner). Command buttons go near an adjacent edge. Information-read panels go in the non-edge center of the panel strip, where they benefit from proximity to both interaction regions without needing Fitts's Law advantages themselves.

### 4. Separate Information Channels by Cognitive Function

- **Panel**: Persistent quantitative data (resource counts, HP, production queues). Read at the player's initiative.
- **Audio**: Event notifications (production complete, attack alerts, idle workers). Pushed to the player asynchronously.
- **Viewport overlays**: Spatial data (health bars, selection indicators, coverage zones). Contextually attached to world-space entities.

No single channel should try to carry all three types. Overloading the panel with notifications creates clutter. Overloading audio with quantitative data creates confusion. Overloading overlays with persistent data creates visual noise.

### 5. Design for Two Users Simultaneously

Every interactive panel element should serve both novice (click) and expert (hotkey) users. Concrete techniques:
- Print the hotkey letter on every button.
- Maintain consistent grid positions across all contexts (same position = same function category).
- Make the panel useful as a read-only status display when the player stops clicking it.
- Provide audio confirmation for all commands so experts can verify without looking at the panel.

### 6. Use Fixed Logical Dimensions, Scaled by DPI Factor

Design the panel at 1920x1080 as the reference. Scale by integer DPI factors for higher resolutions (2x at 4K). Never use proportional scaling (the panel should not grow to fill 25% of a 4K screen when 25% of 1080p was sufficient). All additional pixels go to the viewport.

### 7. Build Overlay Systems as Viewport Complements

Especially for city builders and management games: implement data overlays that transform the viewport into a visualization surface. Toggleable with hotkeys (Caesar III's F/W/D/C convention). The overlay system reduces the information load on the panel by letting the viewport carry spatial data that the panel would otherwise need to summarize as numbers.

### 8. Audio is a First-Class Information Channel, Not an Afterthought

Design the audio notification system alongside the panel, not after it. Identify every event that the player needs to know about and assign it to either panel (persistent, player-initiated) or audio (transient, system-initiated). The best golden-age games (StarCraft, C&C, Age of Empires II) have audio cues so distinctive that experienced players can play with their eyes partially off the screen.

### 9. Respect the Panel Frame's Input Assumptions

The pattern requires: a free-roaming cursor with sub-pixel precision, zero-cost transitions between viewport and panel, and screen-edge scrolling. If your target platform cannot provide all three, do not use the panel frame. Adapt to a different pattern (radial menus for controllers, gesture-based for touch).

### 10. Know When to Let the Pattern Die

If your game's information delivery is primarily spatial (unit positions, terrain features, coverage areas), invest in viewport augmentation (overlays, strategic zoom, contextual world-space UI) and shrink the panel. The panel frame was a compression necessity when viewports were small and rendering was expensive. Modern hardware can embed information directly into the viewport. The panel should persist only for information that is fundamentally non-spatial: resource counts, production queues, global timers, and the minimap (until you implement strategic zoom, at which point even that can migrate).

---

## Conclusion

The golden-age panel frame was not one design but a family of designs, each optimized for a specific interaction pattern. The Westwood sidebar served sequential construction. The Blizzard triptych served agent-command RTS. The Impressions sidebar served hierarchical city building. The Sawyer window system served multi-variable management comparison. The SimCity toolbar served canvas painting.

The practitioner's task is not to copy StarCraft's layout or Caesar III's sidebar. It is to identify the primary interaction pattern of the game being built, select the layout archetype that serves that pattern, then apply the concrete principles -- Fitts's Law corner placement, information channel separation, expert/novice path divergence, fixed-logical-dimension scaling, audio integration, overlay complementarity -- to execute that archetype at a level matching the best of the golden age.

The panel frame is not dead. It is evolving. The principles that made it work in 1998 are the same principles that make interface design work in any era: reduce the cost of frequent actions, place information where the eye naturally falls, separate channels by cognitive function, and respect the capabilities and limitations of the input device.
