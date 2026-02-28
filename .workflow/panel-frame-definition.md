# The Panel Frame

A formal definition and design reference for the panel frame interface pattern
in golden-age strategy games.

---

## 0. Scope and Method

### Subject

The **panel frame**: a display-surface partitioning pattern used in strategy
games of the golden age (~1989--2003).

**Strategy** encompasses: real-time strategy (RTS), city builders,
tycoon/management simulations, 4X games, grand strategy, colony simulations,
and wargames. The common thread is that the player operates as an external agent
managing a system, not as an embodied character.

### Platform Scope

This definition describes the **Western PC mouse-input panel frame**. It does
not claim universality across platforms or regional traditions.

Four input assumptions underpin the pattern:

| # | Assumption | What it provides |
|---|-----------|-----------------|
| 1 | High-precision random-access pointing | Any screen pixel reachable in one motion |
| 2 | Zero-cost context switching between viewport and panel | No mode change to transition between regions |
| 3 | Screen-edge scrolling | The pointing device also navigates the viewport |
| 4 | Fitts's Law dynamics | Screen edges and corners function as infinite targets |

Platforms that violate these assumptions degrade the pattern. Console controllers
satisfy none (StarCraft 64, C&C on PS1). Touch screens satisfy assumptions 1
and 2 partially but lack 3 and 4. The definition's factors, archetypes, and
guidance all presuppose mouse input. Where console or touch adaptations are
discussed, they appear as boundary cases, not as primary examples.

### Unit of Analysis

The unit of analysis is the **view** or **mode**, not the game. A single game
may contain multiple views, some of which are panel frames and some of which
are not. Master of Orion II contains a galaxy view (panel frame), a colony
screen (different panel frame), a ship design screen (modal), and a tactical
combat screen (yet another panel frame). Civilization II contains a map view
(panel frame) and a city screen (modal). When this document classifies a game,
it specifies which view is being classified.

### Method

The definition follows the Berlin Interpretation's structure (IRDC 2008):
canonical examples, then weighted factors that discriminate members from
non-members. It modifies the Berlin Interpretation in three ways:

1. **One necessary condition.** Viewport partitioning is a hard gate. The
   Berlin Interpretation defines no necessary conditions, which leads to the
   notorious problem of calling Diablo II a roguelike. By admitting exactly one
   necessary condition, the panel frame definition avoids this failure mode
   while keeping the boundary minimal.

2. **Three-tier weighting.** The Berlin Interpretation's binary high/low split
   is replaced with three tiers: necessary, characteristic, and incidental. This
   provides more discriminatory resolution without the calibration burden of
   continuous weights.

3. **Quantitative supplement.** The viewport budget provides a measurable,
   reproducible (given stated methodology) metric that the Berlin
   Interpretation lacks entirely.

Possessing some factors does not make a UI a panel frame. Lacking some factors
does not disqualify one. The factors identify a cluster; membership is a matter
of degree, subject to the single necessary condition.

---

## 1. Preliminary Definitions

**Display surface.** The full rectangular pixel area available to the game
(window or fullscreen).

**Viewport.** The primary, scrollable, spatially-continuous region of the
display surface into which the game world is rendered. Three functional
qualifiers distinguish the viewport from other world-rendering elements (such
as minimaps or unit portraits): (a) it is the **primary** world rendering, not
a compressed or abstracted summary; (b) it is **scrollable**, responding to
player navigation input to reveal off-screen portions of the game world; (c) it
is **spatially continuous**, rendering a contiguous region of game space without
gaps, jumps, or abstraction layers.

**Panel.** A region of the display surface that is not part of the viewport and
is used to present interface elements. Panels are distinguished from viewport
overlays by occupying dedicated screen area rather than sharing coordinates with
the viewport.

**Overlay element.** An interface element rendered on top of the viewport,
occupying the same pixel coordinates as some portion of the game world. Health
bars above units, selection circles, and coverage-zone indicators are overlays.
Their presence does not disqualify a panel frame.

**View / Mode.** A distinct interface state within a game, characterized by a
stable layout of viewport(s) and/or panels that persists across multiple player
actions. A game may contain multiple views. Each view is classified
independently.

**Panel frame.** An interface layout within a classified view in which:

> The display surface is partitioned into a viewport and one or more panels,
> such that the viewport is a proper subset of the display surface and the
> panels are opaque, persistent, and non-overlapping with the viewport. This
> partitioning persists across normal gameplay states within the classified
> view.

The temporal qualifier ("persists across normal gameplay states") excludes
snapshot panel states. Diablo II's inventory-open state technically satisfies
the spatial predicate but is not the game's normal gameplay state; the
inventory is a transient overlay that the player opens and closes. The panel
frame is a property of the persistent layout within a view, not of every
possible interface state.

---

## 2. The Necessary Condition

**Viewport partitioning**: the viewport is a proper subset of the display
surface, persistent across normal gameplay states within the classified view.

Formally: `viewport_area < display_surface_area`, where the inequality holds
continuously during normal play in the classified view.

This is the sole necessary condition. Without it, no combination of other
factors can establish a panel frame. With it alone, the interface is not
necessarily a panel frame --- the characteristic factors provide the additional
discriminatory power.

### Why Only One Necessary Condition

Every additional necessary condition risks excluding a legitimate edge case.
Opacity, permanence, fixed geometry, and spatial exclusivity are strong
indicators, but none is truly constitutive in the way that viewport partitioning
is. A semi-transparent panel that permanently occupies a fixed region and never
overlaps the viewport is *nearly* a panel frame; the factors should handle it by
scoring it lower, not by rejecting it at the gate.

The deeper reason for one necessary condition: viewport partitioning is not
merely a high-value factor. It is the constitutive act of the pattern. An
interface where the game world fills the entire screen and UI elements float
over it is doing something categorically different from an interface where the
game world has been architecturally confined. The former treats the world as
primary and UI as overlay. The latter treats the world and the UI as co-equal
spatial inhabitants of the display surface, each with guaranteed territory.
This is a difference of kind, not degree.

---

## 3. Factors

### Note on Factor Independence

The previous version of this document listed six high-value factors, five of
which were entailed by the three-word phrase "opaque, persistent,
non-overlapping" already present in the boxed definition. Those five factors
(viewport partitioning, opacity, permanence, fixed geometry, spatial
exclusivity) did not provide five independent pieces of classificatory evidence;
they provided one criterion stated five times, creating false confidence in
borderline judgments.

The restructured factors below are genuinely independent. Each adds new
discriminatory power that is not derivable from the others. The test for
independence: can two factors be independently violated in at least one
historical example?

### Tier 2 --- Characteristic Factors

Strong indicators of pattern membership. Each is independently testable and
adds discriminatory power beyond the necessary condition.

---

#### 3.1 Opacity

**Definition.** Panels are visually opaque. No game-world content is visible
through a panel. The boundary between viewport and panel is a hard visual edge.

**Diagnostic test.** Take a screenshot during normal gameplay. Can you see any
viewport content (terrain, units, effects) through any panel region? If yes, the
factor is violated.

**Gradient treatment.** Opacity is treated as a high-weight property on a
continuous gradient, not as a binary. A panel that is 95% opaque with a subtle
edge fade (Warcraft III) scores nearly full satisfaction. A panel that is 50%
transparent (a glass-effect HUD) scores low. The gradient exists, but the
golden-age canon clusters overwhelmingly at the opaque end.

**Example satisfying.** StarCraft (1998, main gameplay view): fully opaque
bottom panel with race-specific textures. Zero viewport content visible through
the panel.

**Anti-pattern.** Tiberian Sun (1999): semi-transparent sidebar edges, intended
to recover viewport real estate. The transparency made the sidebar harder to
read against busy terrain and created visual noise at the boundary. The hard
edge between panel and viewport is perceptually load-bearing; it creates two
distinct visual fields that the eye can process independently.

---

#### 3.2 Permanence

**Definition.** Panels persist across selection states and game phases within
the classified view. They do not appear on hover, fade in contextually, or
auto-dismiss. The default state of the view includes the panels, and the game
is designed around that assumption.

**Diagnostic test.** Play the game for five minutes without opening any menus
or inventories. Are all panel regions continuously visible throughout? If a panel
appears or disappears based on what is selected or what game phase is active,
permanence is violated for that region.

**Example satisfying.** Age of Empires II (1999, main gameplay view): the bottom
panel and top resource bar are visible at all times. The content of the bottom
panel changes with selection, but the panel region itself never disappears.

**Anti-pattern.** Paradox grand strategy contextual panels (Europa Universalis,
Crusader Kings): the top bar is permanent, but the right-side panel appears and
disappears based on what the player clicks. When present, it occupies a fixed
position; when absent, the viewport expands. This partial satisfaction
illustrates why permanence is a factor, not a necessary condition --- Paradox
games are weak panel frames at best, and the contextual side panel is a primary
reason.

---

#### 3.3 Fixed Geometry

**Definition.** Panel dimensions and positions do not change during normal
gameplay. The viewport-to-panel boundary is architecturally stable within a
given display resolution.

**Diagnostic test.** Can the player resize, drag, collapse, or reposition any
panel region during normal gameplay? If yes, fixed geometry is violated.

**Example satisfying.** Command & Conquer (1995, main gameplay view): the right
sidebar is exactly the same width and position from the first second of a match
to the last.

**Anti-pattern.** Civilization III advisor screens that slide in from panel
regions, altering the viewport boundary. The panel region is not stable; it
expands and contracts.

---

#### 3.4 Spatial Exclusivity

**Definition.** No z-ordering exists between viewport content and panel content.
Every pixel of the display surface belongs to exactly one of: the viewport, or a
panel. The game world is never occluded by panel UI; the player sees 100% of
the viewport at all times.

**Diagnostic test.** During normal gameplay, does any panel-region content
(buttons, text, decorations) render on top of viewport pixels, or vice versa?
Note: viewport *overlays* (health bars, selection indicators) do not violate
spatial exclusivity --- they are a different layer. The test concerns panel
regions specifically.

**Independence from opacity.** Total Annihilation's minimap overlays the
viewport in certain configurations (violates spatial exclusivity) but is opaque
(satisfies opacity). These factors can be violated independently.

**Example satisfying.** Pharaoh (1999, main gameplay view): the right sidebar
and viewport occupy non-overlapping regions with no z-ordering interaction.

**Anti-pattern.** RollerCoaster Tycoon's floating information windows (ride
details, guest thoughts, financial reports) hover over the viewport. The
persistent toolbar satisfies spatial exclusivity; the floating windows violate
it. This is a principled design choice for comparison-heavy management
simulations, not an error --- but it moves the interface away from the panel
frame pattern for those elements.

---

#### 3.5 Information Density

**Definition.** Panels carry sufficient information to justify their area cost.
A single icon in a screen corner does not constitute a panel frame. The panels
must present multiple pieces of state simultaneously, all visible without
interaction.

**Disambiguation.** "Information density" is ambiguous without qualification.
Three sub-types, each measuring a different property:

| Sub-type | Measures | Example of high density |
|----------|---------|----------------------|
| **Display density** | Pixels of distinct visual information per unit area | AoE2's resource bar: four resource icons + numbers + population count + age indicator in ~20 vertical pixels |
| **Control density** | Interactive elements per unit area | StarCraft's 3x3 command card: 9 clickable ability buttons in a compact grid |
| **Navigation density** | Spatial navigation affordances per unit area | A minimap with real-time unit positions, fog of war, and camera-box indicator |

A panel frame typically requires high density on **at least one** of these three
sub-types across its panel regions. Many canonical examples achieve high density
on all three: the command card has high control density, the minimap has high
navigation density, and the information panel has high display density.

**Diagnostic test.** For each panel region, identify which density sub-type it
primarily serves. If no region achieves notably high density on any sub-type,
the factor is not satisfied.

**Example satisfying.** Age of Empires II (1999, main gameplay view): the top
resource bar achieves high display density (six data points in minimal vertical
space), the minimap achieves high navigation density, and the bottom panel
achieves moderate control density and high display density (unit stats, garrison
count, production queue).

**Anti-pattern.** A hypothetical interface with a wide opaque border around the
viewport containing only a single resource counter. The border satisfies
viewport partitioning but fails information density: the area cost is not
justified by the information delivered.

---

#### 3.6 Functional Role Heterogeneity

**Definition.** The panel contains elements serving distinct cognitive
functions. At minimum, two of the following four roles must be present across
the panel regions:

| Role | Cognitive function | Canonical example |
|------|-------------------|------------------|
| Telemetry display | Communicates quantitative game state | Resource counters, HP bars, population count |
| Control surface | Receives player commands | Command card buttons, build queue, tool palette |
| Navigation tool | Provides spatial orientation and camera control | Minimap with click-to-navigate |
| Context display | Shows properties of the currently selected entity or tool | Unit portrait, building stats, tool options |

**Diagnostic test.** List every element in the panel regions. Assign each to one
of the four roles. If the panel serves only one role (e.g., only telemetry),
this factor is not satisfied.

**Why this factor matters.** It distinguishes a panel frame from a simple status
bar. A narrow resource bar at the top of the screen serves only telemetry; it
does not make the interface a panel frame on its own. A panel frame's panels
are heterogeneous: they combine reading, commanding, and navigating into a
unified spatial structure. This heterogeneity is what justifies the viewport
area cost.

**Example satisfying.** StarCraft (1998, main gameplay view): minimap
(navigation), unit portrait and stats (context display), resource counters
(telemetry), command card (control surface). All four roles are present.

**Anti-pattern.** A game with only a thin resource bar and nothing else: one
role (telemetry), no heterogeneity.

---

### Tier 3 --- Incidental Factors

These distinguish sub-types within the panel frame pattern. Their absence does
not meaningfully weaken pattern membership; their presence helps classify which
archetype a given panel frame belongs to.

---

#### 3.7 Minimap

A persistent minimap occupies one panel region, providing a compressed
representation of the full game map with camera-position indicator and
optionally with unit positions and fog of war. Nearly universal in RTS. Less
common in city builders and tycoon games, which may substitute a zoomable world
or a modal overview map.

---

#### 3.8 Contextual Sub-Panels

Part of a panel changes content based on game state (selected unit, active
building, current tool) while the region itself remains spatially fixed. The
container is static; the content is dynamic. StarCraft's command card,
Age of Empires' unit info area, Caesar III's building info panel.

---

#### 3.9 Decorative Surround

Panel artwork reinforces the game's aesthetic. StarCraft's race-specific panel
textures (Zerg organic, Protoss crystalline, Terran metallic), Warcraft II's
gold-trimmed stone borders, Age of Empires II's parchment-and-stone frame.

Decoration can serve functional purposes. In competitive StarCraft, race-
specific textures instantly communicate which race the player is controlling ---
useful in spectator mode and during mirror matches. The visual weight of panel
decoration also affects perceptual boundary strength: heavily-decorated borders
create a stronger attention-switching boundary between viewport and panel.
Decoration is classified as incidental for pattern membership but may be
functionally significant for interaction design.

---

#### 3.10 Resource Bar

A narrow horizontal bar (usually at the top edge) displays global resource
counts, population, or score. Often present in addition to a larger primary
panel at the bottom or side. Age of Empires' top resource bar, Warcraft III's
top resource bar, Command & Conquer's top credit display.

---

#### 3.11 Layout Convention

Two dominant conventions in the canon:

- **Bottom panel.** Primary panel spans the bottom edge. Minimap and command
  interface embedded within. Age of Empires, StarCraft, Warcraft III.
- **Right sidebar.** Primary panel spans the right edge. Build queue and
  minimap stacked vertically. Dune II, Command & Conquer, Caesar III.
- **Top toolbar.** Primary panel spans the top edge, often paired with a
  side toolbox. SimCity, RollerCoaster Tycoon.

Layout convention distinguishes sub-types but does not determine pattern
membership.

---

#### 3.12 Panel-Embedded Controls

The panel contains clickable buttons for issuing commands: build orders, unit
abilities, tool selections. The panel is not merely informational; it is an
input surface. The mouse moves between the viewport (to select, place, scroll)
and the panel (to command). Present in nearly all canonical examples but not
definitionally required --- a hypothetical read-only panel frame (minimap +
stats, no buttons) would still satisfy the pattern if other factors hold.

---

## 4. The Viewport Budget

### Definition

    viewport_budget = viewport_area / display_surface_area

In an overlay HUD, `viewport_budget = 1.0`. In a panel frame,
`viewport_budget < 1.0`.

### Measurement Methodology

To ensure reproducibility:

1. Measure from the **inner edge** of decorative borders. If a panel has a
   3-pixel ornamental bevel, the panel boundary is at the inner edge of that
   bevel.
2. Include **all permanently visible regions** in the panel area measurement.
   A top resource bar counts as panel area.
3. **Exclude** contextual panels that appear and disappear (e.g., Paradox
   side panels). Measure only what is visible in the default, unselected state.
4. For games with non-rectangular panel regions (e.g., an L-shaped panel),
   compute actual pixel areas, not bounding-rectangle approximations.

### Shape Descriptors

The viewport budget is a scalar that collapses two-dimensional layout
information. Two games with identical budgets of 0.70 can have opposite
functional layouts: a tall narrow right sidebar versus a short wide bottom
bar. To recover this information, always report the viewport budget alongside:

- **Panel placement type**: right sidebar, bottom panel, top bar, L-shape,
  or combination.
- **Viewport aspect ratio** after panel subtraction: the width-to-height ratio
  of the viewport region specifically.

### Observed Range

The viewport budget is **not** a quality metric. A game with budget 0.55 is not
"more of a panel frame" than one with budget 0.80. The budget measures how the
pattern is instantiated, not how well.

| Game | Year | Sub-genre | Panel placement | Viewport budget | Viewport AR |
|------|------|-----------|----------------|----------------|-------------|
| Dune II | 1992 | RTS | Right sidebar + top bar | ~0.55 | ~1.50:1 |
| Warcraft: Orcs & Humans | 1994 | RTS | Left sidebar | ~0.60 | ~0.75:1 |
| Command & Conquer | 1995 | RTS | Right sidebar | ~0.75 | ~1.50:1 |
| Warcraft II | 1995 | RTS | Left sidebar | ~0.65 | ~0.80:1 |
| Master of Orion II | 1996 | 4X | Right panel + bottom bar | ~0.65 | ~1.20:1 |
| Age of Empires | 1997 | RTS | Bottom panel + top bar | ~0.68 | ~1.90:1 |
| Dark Reign | 1997 | RTS | Bottom panel | ~0.70 | ~1.85:1 |
| StarCraft | 1998 | RTS | Bottom panel | ~0.60 | ~1.94:1 |
| Caesar III | 1998 | City builder | Right sidebar | ~0.70 | ~1.10:1 |
| Age of Empires II | 1999 | RTS | Bottom panel + top bar | ~0.70 | ~1.90:1 |
| Pharaoh | 1999 | City builder | Right sidebar | ~0.68 | ~1.10:1 |
| RollerCoaster Tycoon | 1999 | Tycoon | Top toolbar + side toolbox | ~0.80 | ~1.45:1 |
| SimCity 3000 | 1999 | City builder | Bottom bar + side toolbox | ~0.78 | ~1.50:1 |
| Civilization II | 1996 | 4X | Right sidebar + top bar | ~0.65 | ~1.15:1 |
| Alpha Centauri | 1999 | 4X | Bottom panel + right info | ~0.68 | ~1.50:1 |
| Zeus: Master of Olympus | 2000 | City builder | Right sidebar | ~0.68 | ~1.10:1 |
| Red Alert 2 | 2000 | RTS | Right sidebar + top bar | ~0.72 | ~1.50:1 |
| Cossacks | 2001 | RTS | Right sidebar | ~0.75 | ~1.35:1 |
| Europa Universalis | 2000 | Grand strategy | Top bar + contextual side | ~0.78 | ~1.50:1 |
| Warcraft III | 2002 | RTS | Bottom panel + top bar | ~0.65 | ~1.78:1 |
| Age of Mythology | 2002 | RTS | Bottom panel + top bar | ~0.68 | ~1.85:1 |
| Panzer General | 1994 | Wargame | Right sidebar + bottom bar | ~0.65 | ~1.20:1 |
| Heroes of Might and Magic III | 1999 | TBS-RPG | Right sidebar + bottom bar | ~0.60 | ~1.10:1 |
| Total Annihilation | 1997 | RTS | Bottom bar (thin) | ~0.85 | ~1.90:1 |
| SimCity 2000 | 1993 | City builder | Top toolbar + side toolbox | ~0.78 | ~1.40:1 |
| Transport Tycoon Deluxe | 1995 | Tycoon | Top toolbar + bottom bar | ~0.82 | ~1.50:1 |

Observed range: approximately **0.55 to 0.85**. Strategy panel frames typically
allocate 15--45% of the display surface to panels.

---

## 5. Canon

### Primary Canon

The definition is calibrated against these exemplars. Each entry specifies the
classified view.

#### RTS

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| Dune II | 1992 | Main gameplay | RTS Sidebar | ~0.55 |
| Warcraft: Orcs & Humans | 1994 | Main gameplay | RTS Sidebar | ~0.60 |
| Command & Conquer | 1995 | Main gameplay | RTS Sidebar | ~0.75 |
| Warcraft II | 1995 | Main gameplay | RTS Sidebar | ~0.65 |
| Dark Reign | 1997 | Main gameplay | RTS Bottom Triptych | ~0.70 |
| Age of Empires | 1997 | Main gameplay | RTS Bottom Triptych | ~0.68 |
| Total Annihilation | 1997 | Main gameplay | RTS Bottom Triptych | ~0.85 |
| StarCraft | 1998 | Main gameplay | RTS Bottom Triptych | ~0.60 |
| Age of Empires II | 1999 | Main gameplay | RTS Bottom Triptych | ~0.70 |
| Red Alert 2 | 2000 | Main gameplay | RTS Sidebar | ~0.72 |
| Cossacks | 2001 | Main gameplay | RTS Sidebar | ~0.75 |
| Warcraft III | 2002 | Main gameplay | RTS Bottom Triptych | ~0.65 |
| Age of Mythology | 2002 | Main gameplay | RTS Bottom Triptych | ~0.68 |

#### City Builder

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| SimCity | 1989 | Main gameplay | Maxis Toolbar | ~0.78 |
| SimCity 2000 | 1993 | Main gameplay | Maxis Toolbar | ~0.78 |
| SimCity 3000 | 1999 | Main gameplay | Maxis Toolbar | ~0.78 |
| Caesar III | 1998 | Main gameplay | City Builder Sidebar | ~0.70 |
| Pharaoh | 1999 | Main gameplay | City Builder Sidebar | ~0.68 |
| Zeus: Master of Olympus | 2000 | Main gameplay | City Builder Sidebar | ~0.68 |
| Emperor: Rise of the Middle Kingdom | 2002 | Main gameplay | City Builder Sidebar | ~0.68 |

#### Tycoon / Management

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| Transport Tycoon Deluxe | 1995 | Main gameplay | Maxis Toolbar | ~0.82 |
| RollerCoaster Tycoon | 1999 | Main gameplay | Maxis Toolbar | ~0.80 |
| RollerCoaster Tycoon 2 | 2002 | Main gameplay | Maxis Toolbar | ~0.80 |

#### 4X

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| Master of Magic | 1994 | Overworld map | 4X Map Panel | ~0.65 |
| Civilization II | 1996 | Map view | 4X Map Panel | ~0.65 |
| Master of Orion II | 1996 | Galaxy view | 4X Map Panel | ~0.65 |
| Alpha Centauri | 1999 | Map view | 4X Map Panel | ~0.68 |

#### Grand Strategy

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| Europa Universalis | 2000 | Map view | Paradox partial frame | ~0.78 |

#### Wargame

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| Panzer General | 1994 | Main gameplay | Wargame Frame | ~0.65 |

#### TBS-RPG

| Game | Year | View classified | Layout archetype | Viewport budget |
|------|------|----------------|-----------------|----------------|
| Heroes of Might and Magic III | 1999 | Adventure map | Wargame Frame | ~0.60 |

---

### Near-Miss Set

Games that satisfy some but not all panel frame criteria. Each tests a
different boundary condition of the definition.

| Game | Year | Factors satisfied | Factors violated | Why excluded or borderline |
|------|------|------------------|-----------------|--------------------------|
| **Dungeon Keeper** | 1997 | Viewport partitioning, opacity, permanence, fixed geometry | Functional role heterogeneity (panel contains a first-person sub-viewport, blurring viewport/panel distinction) | The first-person sub-viewport embedded in the panel creates a secondary viewport within a panel region, violating the assumption that panels present interface elements rather than world renderings |
| **Homeworld** | 1999 | Viewport partitioning (partially) | Fixed geometry, spatial exclusivity (contextual 3D overlays, no fixed "up" direction) | Fully 3D viewport with rotatable camera; panel elements are contextual overlays rather than persistent opaque regions. Abandoned the panel frame for a spatial-overlay approach |
| **Sacrifice** | 2001 | None meaningful | All characteristic factors | Third-person camera, spell wheel overlay, no persistent panels. Complete pattern rejection in favor of action-game interface |
| **Advance Wars** | 2001 | Viewport partitioning, contextual panels | Permanence (panels appear contextually), all four input assumptions (GBA: no mouse, d-pad navigation) | Console platform lacks mouse input; panels are contextual rather than permanent; the interface pattern is fundamentally different despite surface resemblance |
| **Diablo II** | 2000 | Viewport partitioning (in inventory-open state), opacity (inventory panel) | Permanence (inventory is transient), temporal persistence (not the normal gameplay state) | The inventory-open state is a snapshot, not a persistent layout. Normal gameplay uses an overlay HUD with `viewport_budget = 1.0` |
| **Romance of the Three Kingdoms** (series) | 1985-- | Viewport partitioning, opacity, permanence, fixed geometry | Information density inverted (panel is primary interaction surface with small viewport as geographic context) | Koei's design tradition inverts the Western assumption that the viewport is primary. The panel-primary layout is structurally distinct from the viewport-primary pattern this definition describes |

---

## 6. Layout Archetypes

Each archetype is a named region in the morphological space defined by panel
placement, functional decomposition, and interaction model. A game may combine
elements of multiple archetypes.

---

### 6.1 The RTS Sidebar

**Panel placement.** Right sidebar (occasionally left). Optional top resource
bar.

**Viewport budget range.** 0.55--0.75.

**Functional decomposition.** Single panel with minimap, build queue, and unit
information stacked vertically. The sidebar is the sole command surface for base
management operations.

**Interaction model.** Click-to-command. The player selects a unit or building
in the viewport, then clicks a sidebar button to issue a command or start
production, then returns to the viewport. Sequential construction from a
catalog: the sidebar presents available buildings/units, the player selects one,
then places it.

**Sub-genre affinity.** RTS games with significant base-building components.
City builders with hierarchical building menus (Impressions lineage).

**Exemplars.** Dune II (1992), Command & Conquer (1995), Red Alert (1996),
Red Alert 2 (2000), Warcraft: Orcs & Humans (1994), Warcraft II (1995).

**Why the sidebar suits this interaction.** Deep hierarchical building
categories (water infrastructure, entertainment, religion, military in
Impressions titles; structures vs. units tabs in Westwood titles) map naturally
to a vertical panel with expandable categories. The sidebar is a toolbox.
Toolboxes are conventionally vertical, following the paint-program and CAD
tradition.

---

### 6.2 The RTS Bottom Triptych

**Panel placement.** Bottom panel spanning the full screen width. Optional top
resource bar.

**Viewport budget range.** 0.60--0.75 (StarCraft at the dense end, Total
Annihilation's thin bar at 0.85).

**Functional decomposition.** Three regions arranged horizontally:
- Left: minimap (navigation)
- Center: information panel (context display + telemetry)
- Right: command card (control surface)

This decomposition answers three questions: Where am I? What am I looking at?
What can I do? It maps to the orient-assess-act decision loop.

**Interaction model.** Click-to-command, but with the command card as the
primary control surface rather than the build queue. Unit abilities,
formations, and actions are presented as a grid of icons.

**Sub-genre affinity.** RTS games emphasizing unit micro-management and spatial
combat over base building.

**Exemplars.** StarCraft (1998), Age of Empires (1997), Age of Empires II
(1999), Warcraft III (2002), Age of Mythology (2002), Dark Reign (1997).

**Fitts's Law advantage.** The minimap occupies the bottom-left screen corner:
an infinite target in two dimensions. The command card occupies the
bottom-right, near another corner. Screen corners are the fastest targets for
imprecise mouse movements. In a right-sidebar layout, the minimap sits at the
top of the sidebar --- not a corner, requiring precision in both axes.

---

### 6.3 The City Builder Sidebar

**Panel placement.** Right sidebar.

**Viewport budget range.** 0.60--0.75.

**Functional decomposition.** Minimap at top, hierarchical building menus
below, overlay/advisor toggle buttons. The sidebar is a tool palette: the player
selects a building type or data overlay, then interacts with the viewport.

**Interaction model.** Tool-palette. The player selects a tool (build road,
place temple, toggle water overlay), and the tool remains active for multiple
viewport interactions. This is a persistent-mode interaction, unlike the
RTS click-to-command where each action is atomic.

**Sub-genre affinity.** Historical city builders with building placement as the
primary mechanic.

**Exemplars.** Caesar III (1998), Pharaoh (1999), Zeus: Master of Olympus
(2000), Emperor: Rise of the Middle Kingdom (2002).

**Viewport augmentation.** Caesar III's overlay system (toggled by sidebar
buttons or hotkeys: F for fire, W for water, D for damage, C for crime)
transforms the viewport into a data visualization surface. This reduces panel
information density requirements by offloading spatial data to the viewport.

---

### 6.4 The Maxis Toolbar

**Panel placement.** Top toolbar with side or bottom toolbox. Sometimes
floating sub-windows for detailed information.

**Viewport budget range.** 0.75--0.85.

**Functional decomposition.** Toolbar contains small icons that open cascading
menus or activate tools. Toolbox (when present) shows options for the currently
selected tool category. Information delivery is sparse in the panel; the
viewport itself carries most game-state information through visual encoding
(zone coloring, traffic density, building state).

**Interaction model.** Paint-program metaphor. The player selects a tool (zone,
road, bulldoze, landscape) and "paints" the viewport. Direct manipulation of a
canvas, not command-and-control of agents.

**Sub-genre affinity.** Zone-based city builders, tycoon games with
canvas-style interaction.

**Exemplars.** SimCity (1989), SimCity 2000 (1993), SimCity 3000 (1999),
RollerCoaster Tycoon (1999), RollerCoaster Tycoon 2 (2002), Transport Tycoon
Deluxe (1995).

---

### 6.5 The 4X Map Panel

**Panel placement.** Variable; often right sidebar or combination of top/bottom
bars with side info panel.

**Viewport budget range.** 0.65--0.80.

**Functional decomposition.** Navigation minimap, abstracted information about
the selected tile/city/unit, and high-level management controls (end turn,
diplomacy button, tech tree button). The panel is an entry point to modal
screens rather than a complete command surface.

**Interaction model.** Inspect-and-manage. The player clicks a map entity to
inspect it in the panel, then optionally enters a modal screen for detailed
management. The panel provides just enough information to decide whether to
drill deeper.

**Sub-genre affinity.** Turn-based 4X and grand strategy games with strategic
map views.

**Exemplars.** Civilization II (1996, map view), Master of Orion II (1996,
galaxy view), Alpha Centauri (1999, map view), Master of Magic (1994, overworld
map).

---

### 6.6 The Wargame Frame

**Panel placement.** Variable; often right sidebar, or top + side, or full
border.

**Viewport budget range.** 0.60--0.80.

**Functional decomposition.** Hex/counter or unit-stack display in the viewport.
Panel shows terrain information, unit stats (attack/defense/movement values),
and turn-phase controls.

**Interaction model.** Inspect-and-manage with strong emphasis on reading
terrain and unit data from the panel before committing to orders. Information
density is high in the display-density sub-type (numerical stats).

**Sub-genre affinity.** Hex-based wargames, tactical combat screens in
strategy games.

**Exemplars.** Panzer General (1994), Heroes of Might and Magic III (1999,
adventure map view).

---

## 7. The Three-Region Convention

StarCraft's minimap / info panel / command card triptych codified a functional
decomposition within the bottom-panel archetype. The three regions answer three
questions:

| Region | Question | Cognitive function |
|--------|---------|-------------------|
| Left (minimap) | Where am I? | Spatial orientation, camera navigation |
| Center (info) | What am I looking at? | Selected entity properties, production state |
| Right (command card) | What can I do? | Available actions, abilities, build options |

This decomposition maps to the decision loop: **orient** (minimap) -->
**assess** (info panel) --> **act** (command card).

### Genre Specificity

The three-region convention is specific to real-time games with spatial
awareness and unit-command requirements. It is not universal to all panel frames.

- **City builders** do not need a persistent command card because building
  placement is modal (select tool, then place repeatedly). The "What can I do?"
  question is answered by the sidebar toolbox, not a bottom-right grid.
- **4X games** operate on turn-based cycles where the full screen can be
  dedicated to a single decision context at a time; they use modal dialogs
  rather than persistent panels for detailed management.
- **Management sims** need comparison windows, not command cards. The "What can
  I do?" question is less relevant than "How do these values compare?"

The three-region convention is a pattern within a pattern: the panel frame
defines the spatial structure; the triptych defines a functional allocation
within the bottom-panel archetype specifically.

---

## 8. Information Channels

A panel frame does not operate in isolation. It is one component of a
multi-channel information architecture. The golden-age strategy games that best
implemented the pattern designed three channels together.

| Channel | Content type | Delivery mode | Update frequency | Exemplar |
|---------|-------------|---------------|-----------------|----------|
| **Panel** | Persistent quantitative data | Player-initiated read (glance at panel) | Continuous or per-selection | Resource counts, HP, production queue, unit stats |
| **Audio** | Transient event notifications | System-pushed (plays automatically) | Event-driven | "Base under attack," production complete, unit acknowledgment |
| **Viewport overlay** | Spatial data | Context-attached to world entities | Continuous | Health bars, selection circles, rally-point flags, coverage zones |

### Channel Overload Principle

No single channel should carry all three content types. Overloading the panel
with event notifications creates clutter (flashing alert icons compete with
persistent data). Overloading audio with quantitative data creates confusion
(you cannot "hear" that you have 347 gold). Overloading viewport overlays with
persistent data creates visual noise (permanent number displays on every
building obscure the game world).

Design the three channels together, not sequentially. For every piece of
information the player needs, ask: is it persistent or transient? Is it spatial
or abstract? Is it player-initiated or system-pushed? The answers determine
which channel carries it.

### Viewport Augmentation

Caesar III's data overlay system (water coverage, fire risk, desirability,
crime) exemplifies a fourth mode: **viewport augmentation**, where the viewport
temporarily transforms into a data visualization surface. The overlays replace
normal rendering with schematic displays showing coverage radii and risk levels.
This reduces panel information density requirements by offloading spatial data to
the viewport itself. A panel that would otherwise need to display "water
coverage: 73% of city blocks" can instead let the player toggle an overlay and
read the answer directly from the viewport.

### Expert/Novice Channel Divergence

Novice players rely heavily on the panel channel (reading button labels,
checking resource numbers). Expert players shift to the audio channel for event
awareness and the viewport overlay channel for spatial awareness, reducing the
panel to a peripheral status display. The same panel frame is the primary
interface for one player and a background reference for another. This divergence
is not a defect; it is the pattern working as intended across skill levels.

---

## 9. Input and Interaction

### The Four Input Assumptions (Restated)

The panel frame assumes:

1. **High-precision random-access pointing.** The player can target any screen
   pixel in a single motion.
2. **Zero-cost context switching.** The cursor transitions between viewport and
   panel without mode changes, button holds, or menu traversals.
3. **Screen-edge scrolling.** Moving the cursor to a screen edge scrolls the
   viewport. The same device that points also navigates.
4. **Fitts's Law dynamics.** Screen edges are infinite targets in one dimension;
   screen corners in two. Interface elements placed at edges and corners are
   faster to reach than mid-screen elements.

Any platform that cannot satisfy all four should not use the panel frame
pattern. Alternative patterns for non-mouse platforms: radial menus for
controllers, gesture-based navigation for touch.

### Fitts's Law Analysis by Layout

| Layout type | Minimap position | Primary interaction target | Average travel from viewport center |
|------------|-----------------|--------------------------|-----------------------------------|
| Bottom triptych | Bottom-left corner (infinite target in 2D) | Command card, bottom-right (near corner) | Primarily vertical (downward) |
| Right sidebar | Mid-right edge (infinite in 1D) | Build queue, mid-sidebar | Primarily horizontal (rightward) |
| Top toolbar | Top edge (infinite in 1D) | Tool icons, top bar | Primarily vertical (upward) |

Bottom-triptych layouts place both the minimap and the command card near screen
corners, providing the strongest Fitts's Law advantages for the two
highest-frequency interaction targets. This contributes to the dominance of
the bottom-triptych layout in competitive RTS, where action speed matters.

### Expert vs. Novice Divergence

This is the single most consequential interaction-design property of the panel
frame.

**Novice use.** The panel is the primary input surface. Novices click buttons to
discover and issue commands. The command card teaches what each unit can do.
The build queue teaches what is available. The panel is a learning interface.

**Expert use.** The panel is a status display. Experts issue all commands via
keyboard shortcuts. They never click the command card. They glance at it to
check cooldown timers, ability availability, and production progress. The
minimap is the only panel element that retains high interaction value across all
skill levels.

**Design requirement.** Every interactive panel element must serve both
populations simultaneously. Concrete techniques from the golden-age canon:

| Technique | How it serves novices | How it serves experts | Example |
|-----------|---------------------|---------------------|---------|
| Hotkey letters on icons | Visual label for learning | Reminder of binding | StarCraft's command card: "S" on Stop, "A" on Attack |
| Consistent grid positions | Discoverable by exploration | Predictable by muscle memory | Warcraft III: "Build" always in same cell for every building |
| Audio acknowledgment | Confirms the click worked | Confirms the hotkey worked without panel glance | C&C: "Unit ready," "Building" |
| Read-only status utility | Shows what is available | Shows cooldowns, queue state, availability at a glance | StarCraft: greyed-out abilities indicate insufficient resources |

---

## 10. Resolution and Scaling

### Historical Context

The golden age spans roughly three resolution eras:

| Period | Dominant resolution | Total pixels | Implications |
|--------|-------------------|-------------|-------------|
| 1989--1994 | 320x200 | 64,000 | Every panel pixel extremely costly. Dune II's 0.55 budget yields a viewport of ~192x200 |
| 1995--2000 | 640x480 | 307,200 | The core period. Most canonical examples designed here |
| 2000--2003 | 800x600 to 1024x768 | 480,000--786,432 | Higher resolutions emerge. Panel pixel dimensions hold steady; viewport grows |

### Three Scaling Strategies

1. **Fixed resolution.** StarCraft locked to 640x480 for competitive fairness:
   higher resolutions would provide tactical advantage through wider viewport.
   Panel dimensions are absolute; pixel-tuned for a single layout.

2. **Fixed-pixel panels, expanding viewport.** Age of Empires II: panel
   elements maintain fixed pixel dimensions at all resolutions. At 1024x768,
   only the viewport grows. This creates competitive asymmetry (more terrain
   visible at higher resolutions) but is simpler to implement and avoids
   scaling artifacts.

3. **Proportional scaling.** Rare in the golden age. Modern strategy games
   increasingly use this approach, scaling all UI elements by a DPI factor.

### Modern Guidance

Golden-age viewport budgets (0.55--0.85) translate to very large panel areas at
modern resolutions. A 0.70 budget at 1920x1080 yields panels consuming 622,080
pixels --- more total pixels than the entire 640x480 display surface. Modern
implementations should:

1. Design the panel at a **reference resolution** of 1920x1080.
2. Scale panel elements by **integer or near-integer DPI factor**, not by
   resolution ratio. A 4K display at 2x scaling shows the same logical layout
   as 1080p with crisper rendering.
3. All additional pixels go to the viewport. **Never let the viewport shrink
   below the reference size.**
4. Fix the panel's **logical dimensions** (in points or equivalent units), not
   its pixel dimensions.
5. Adjust the viewport budget upward toward **0.80--0.90** at modern
   resolutions. The golden-age budgets reflected golden-age constraints; at
   1920x1080, a 0.70 budget creates panels so large they are difficult to fill
   with useful information.

---

## 11. Anti-Patterns and Boundary Cases

Per-factor anti-patterns are documented inline in Section 3. This section covers
systemic anti-patterns that violate the pattern as a whole or affect multiple
factors simultaneously.

### Complete Pattern Rejection

**Sacrifice** (2001). Third-person camera, no persistent panels, spell wheel
overlay. The game is a strategy title that rejected the panel frame entirely in
favor of an action-game interface. Every factor is violated. Sacrifice
demonstrates that strategy games can function without the pattern, though at the
cost of reduced persistent information display.

**Homeworld** (1999). Fully 3D space RTS with rotatable camera, contextual
overlays, no fixed orientation. The 3D viewport with no stable "up" direction
made a fixed panel frame impractical. Information is delivered through
contextual overlays and modal screens. Homeworld abandoned the pattern and
influenced later 3D RTS designs (e.g., Sins of a Solar Empire).

### Console Degradation

When PC panel frames are ported to consoles, specific failures reveal the
pattern's input assumptions.

**StarCraft 64** (2000, N64). The command card was navigated by D-pad,
requiring up to 4 directional inputs to reach the opposite corner (vs. one
click with a mouse). The minimap became nearly useless: an analog stick cannot
provide the sub-pixel precision needed to click on a specific minimap pixel
representing a distant base. Viewport scrolling and panel interaction both
required the same analog stick, creating a mode-switching cost that violates
assumption 2 (zero-cost context switching).

**Command & Conquer on PlayStation** (1995). The sidebar was reorganized for
controller navigation with category sub-sorting (structure, infantry, vehicle,
aircraft). Cursor emulation via D-pad was too slow for tactical play and too
imprecise for small sidebar buttons.

### The Sawyer Hybrid

RollerCoaster Tycoon's toolbar and toolbox satisfy the panel frame. But the
game also opens floating windows (ride information, guest thoughts, financial
reports) that hover over the viewport, violating spatial exclusivity for
information delivery. This is a principled design choice: management simulations
require comparison of multiple data sources, and floating windows provide the
most powerful comparison architecture of the golden age. The trade-off is window
management overhead, viewport occlusion, and loss of canonical layout (every
player's screen looks different).

The correct classification: the persistent toolbar forms a panel frame; the
floating windows are a separate pattern layered on top. The panel frame is a
property of the persistent layout, not of every UI element.

### The Paradox Partial Frame

Europa Universalis and Crusader Kings have a permanent top bar (satisfies
permanence) and a right-side panel that appears/disappears based on game state
(violates permanence for that region). When the side panel is present, it
satisfies opacity, fixed geometry, and spatial exclusivity. When absent, the
viewport expands.

This partial satisfaction places Paradox grand strategy at the boundary of the
pattern: the permanent top bar contributes to panel frame membership, but the
contextual side panel points toward the post-golden-age dissolution of the
pattern into contextual, dismissible interfaces.

---

## 12. Lineages

Lineages trace paths through the morphological space over time. **Within-studio
lineages** (the same development team iterating on its own prior work) are
verifiable from release sequences and documented developer statements.
**Cross-studio lineages** (one studio's design influencing another's) are
speculative without access to internal design documents. The convergent
appearance of bottom-panel layouts in Age of Empires (Ensemble, 1997),
StarCraft (Blizzard, 1998), and Dark Reign (Auran, 1997) within a 12-month
window suggests shared constraints (higher resolutions, wider monitors) drove
convergent evolution at least as much as direct inheritance.

Lineages are presented as historical context, not as causal claims.

---

### Westwood

Dune II (1992) --> Command & Conquer (1995) --> Red Alert (1996) --> Tiberian
Sun (1999) --> Red Alert 2 (2000) --> Generals (2003).

Right sidebar throughout, with progressive refinement: single-column to
dual-column build menu, explicit command buttons removed in favor of
context-sensitive cursor, sidebar width narrowed from ~40% to ~25% as
resolution increased. Generals (2003) broke from the sidebar to adopt a bottom
panel, marking the end of the Westwood sidebar lineage.

The sidebar narrowed because its absolute pixel width barely changed while
resolutions doubled. Panel dimensions are driven by content legibility (the
widest content element), not proportional ratios. At 320x200, 128 pixels was
the minimum for legible icons. At 640x480, 160 pixels achieved the same
legibility with more layout room.

---

### Blizzard

Warcraft: Orcs & Humans (1994) --> Warcraft II (1995) --> StarCraft (1998) -->
Warcraft III (2002).

Sidebar (left-side) in Warcraft I and II; bottom triptych from StarCraft
onward. The shift from sidebar to bottom panel occurred between Warcraft II and
StarCraft. The bottom triptych in StarCraft codified the three-region convention
that influenced subsequent RTS design.

---

### Ensemble

Age of Empires (1997) --> Age of Empires II (1999) --> Age of Mythology (2002).

Bottom panel from the start. Ensemble invested in information density: AoE2's
top resource bar achieves extreme display density (six data points in ~20
vertical pixels), and the bottom panel packs unit stats, production queue,
garrison count, and derived-state indicators (idle villager button, town bell)
into a compact region.

---

### Maxis

SimCity (1989) --> SimCity 2000 (1993) --> SimCity 3000 (1999).

Toolbar-and-toolbox layout throughout, borrowed from the desktop-application and
paint-program tradition. The interaction model (select tool, paint on canvas)
persisted across the entire lineage. Highest viewport budgets in the canon
(0.75--0.85), reflecting the viewport-as-data philosophy: the city rendering
itself carries most game state visually.

---

### Impressions

Caesar III (1998) --> Pharaoh (1999) --> Zeus: Master of Olympus (2000) -->
Emperor: Rise of the Middle Kingdom (2002).

Right sidebar throughout. The most consistent within-studio lineage: four games
over four years with nearly identical panel layout. Deep hierarchical building
menus, minimap at top, overlay toggle buttons. The sidebar persisted because the
interaction pattern (select building category, drill into sub-category, place
building) maps naturally to vertical hierarchical menus.

---

### Sawyer

Transport Tycoon (1994) --> RollerCoaster Tycoon (1999) --> RollerCoaster
Tycoon 2 (2002).

Toolbar-and-toolbox for persistent controls, floating windows for information
delivery. The hybrid approach persisted across all three titles. The floating
windows provide the most powerful comparison architecture of the golden age
(simultaneous multi-variable inspection) at the cost of window management
overhead and viewport occlusion.

---

## 13. Weaknesses

Fundamental limitations of the panel frame pattern.

**Viewport budget cost.** Every panel pixel is lost from the game world. At
low resolutions (640x480), a 0.65 viewport budget means the player sees only
~200,000 pixels of the game world. The panel's information value must justify
this cost at every resolution.

**Mouse travel cost.** Fitts's Law imposes a time cost on every viewport-to-
panel transition. A player who must check the panel 100 times per minute (common
in competitive RTS) loses seconds per minute to mouse travel that could be spent
on gameplay actions.

**Resolution dependency.** The pattern was designed for 640x480-era constraints
where viewport pixels were scarce and persistent information display required
dedicated screen area. As resolutions increased, the cost/benefit ratio shifted:
more viewport pixels available, more room for overlays and contextual displays,
less justification for large permanent panels.

**Single-user-model compromise.** The panel cannot optimally serve both novice
and expert simultaneously. It is too button-dense for novices (overwhelming) and
too large for experts (wasted space). The pattern scaffolds the transition from
novice to expert, but at any given moment it is suboptimal for one population.

**Static spatial allocation.** The panel cannot adapt to changing information
needs within a session. Early game (base building) and late game (army
management) have different information priorities, but the panel layout is fixed.
Contextual sub-panels (Section 3.8) partially address this, but the spatial
allocation of the panel regions themselves does not change.

---

## 14. The Transition

### Why the Pattern Gave Way

The panel frame's dominance faded after approximately 2003 due to converging
factors:

1. **Higher resolutions** reduced the viewport budget's relative cost. At
   1920x1080, even a 0.85 budget yields a viewport larger than the entire
   640x480 display surface. The pressure to minimize panel area decreased.

2. **3D cameras and strategic zoom** absorbed minimap functionality. Supreme
   Commander (2007) allowed continuous zoom from ground level to full-map
   overview, making the minimap a compression artifact that could be eliminated.

3. **Viewport overlays** absorbed status information. Company of Heroes (2006)
   attached health bars, capture indicators, and damage indicators directly to
   viewport entities. Information migrated from panel to viewport.

4. **The general trend**: information migrates from dedicated panel space into
   the viewport through overlays, contextual tooltips, and spatial encoding.
   The panel shrinks.

### What Persists

The three-question decomposition (Where am I? What am I looking at? What can I
do?) remains cognitively valid even when the answers migrate from panel to
viewport. Post-golden-age strategy games still answer these questions; they
increasingly do so through viewport augmentation rather than dedicated panel
regions.

### Post-Golden-Age Markers

- **Supreme Commander** (2007): strategic zoom eliminates minimap; information
  density moves into the viewport at different zoom levels.
- **Company of Heroes** (2006): contextual overlays deliver status information
  in the viewport; panel shrinks to minimal command interface.
- **StarCraft II** (2010): retains the bottom triptych but reduces panel area
  relative to its predecessor and increases viewport overlay usage.
- **Age of Empires IV** (2021): retains triptych structure with reduced panel
  prominence and expanded viewport overlays.

---

## 15. Design Guidance

Organized by developer decision points. Each recommendation cites the factor,
analysis, or historical precedent it derives from.

### Interaction-Pattern Lookup Table

| Primary interaction pattern | Recommended archetype | Rationale |
|----------------------------|----------------------|-----------|
| Commanding autonomous agents (RTS) | Bottom triptych (Section 6.2) | Corner minimap for Fitts's Law; triptych maps to orient-assess-act loop |
| Sequential construction from catalog | Right sidebar (Sections 6.1, 6.3) | Vertical hierarchy suits building menus; persistent tool state suits sidebar |
| Canvas/zone painting | Top toolbar + toolbox (Section 6.4) | Paint-program metaphor; highest viewport budgets for viewport-as-data |
| Multi-variable comparison | Toolbar + floating windows (Section 11, Sawyer hybrid) | Floating windows enable simultaneous comparison at cost of viewport occlusion |

### Concrete Guidance Points

1. **Panel dimensions: driven by content, not ratios.** A sidebar needs enough
   width for its widest content element (a two-column build grid, a minimap, a
   stat block) and no more. Do not target "25% of screen width" --- target
   "160 logical pixels for a two-column icon grid at 32px per icon."
   *(Derived from a2-02.)*

2. **Panel allocation mirrors decision structure.** If the game's core
   decisions are "what to do with this unit," the command card dominates panel
   area. If decisions are "which units to build and where," stats and
   production dominate. *(Derived from a2-06.)*

3. **Minimap in a screen corner.** Screen corners are infinite Fitts's Law
   targets in two dimensions. Bottom-left is the convention for right-handed
   users. Do not place the minimap mid-panel or mid-edge.
   *(Derived from a2-04.)*

4. **Viewport overlays as panel complement.** Implement toggleable data
   overlays (Caesar III model) to reduce panel information density
   requirements. Spatial data belongs in the viewport; abstract data belongs
   in the panel. *(Derived from a2-08.)*

5. **Three channels designed together.** For every piece of information the
   player needs: is it persistent or transient? Spatial or abstract?
   Player-initiated or system-pushed? Assign to panel, audio, or viewport
   overlay accordingly. Do not overload any single channel.
   *(Derived from a2-16, a2-23.)*

6. **Dual-use elements.** Every interactive panel element must serve novice
   (click target, learning interface) and expert (status display, hotkey
   reference) simultaneously. Print hotkey letters on icons. Maintain
   consistent grid positions across all selection contexts. Make the panel
   useful as a read-only display. *(Derived from a2-09, the highest-scoring
   proposition in the dialectic.)*

7. **Audio as first-class channel.** Design the audio notification system
   alongside the panel, not after it. Assign every player-relevant event to
   either panel (persistent, player-initiated) or audio (transient,
   system-pushed) delivery. Distinctive audio cues (StarCraft, C&C, AoE2)
   allow experienced players to monitor game state without looking at the
   panel. *(Derived from a2-10, a3-20.)*

8. **Resolution scaling: fixed logical dimensions.** Design at 1920x1080
   reference. Scale by integer DPI factor for higher resolutions. Viewport
   absorbs all extra pixels. Never let the viewport shrink below reference
   size. *(Derived from a2-14.)*

9. **Verify input assumptions.** Before committing to a panel frame, confirm
   that the target platform provides all four input assumptions (Section 0). If
   any assumption is violated, choose a different interaction pattern.
   *(Derived from a2-11, a2-24.)*

10. **Adjust viewport budget for modern resolutions.** Golden-age budgets
    (0.55--0.85) at modern resolutions produce enormous panels. Target
    0.80--0.90 at 1920x1080 and above. Fill panel space with genuinely useful
    information or shrink the panel. *(Derived from a1-22.)*

---

## 16. Terminology

All terms defined in this document, collected for reference.

| Term | Section | Definition |
|------|---------|-----------|
| Display surface | 1 | The full rectangular pixel area available to the game |
| Viewport | 1 | The primary, scrollable, spatially-continuous region rendering the game world |
| Panel | 1 | A display-surface region outside the viewport, used for interface elements |
| Overlay element | 1 | An interface element rendered on top of the viewport, sharing its pixel coordinates |
| View / Mode | 1 | A distinct interface state with a stable layout persisting across multiple player actions |
| Panel frame | 1 | An interface layout where the display surface is partitioned into a viewport and one or more opaque, persistent, non-overlapping panels, persisting across normal gameplay states within the classified view |
| Viewport partitioning | 2 | The necessary condition: viewport_area < display_surface_area, persistent within the classified view |
| Viewport budget | 4 | viewport_area / display_surface_area; a scalar metric of panel cost |
| Display density | 3.5 | Pixels of distinct visual information per unit area |
| Control density | 3.5 | Interactive elements per unit area |
| Navigation density | 3.5 | Spatial navigation affordances per unit area |
| Functional role heterogeneity | 3.6 | The property of panels serving multiple distinct cognitive functions (telemetry, control, navigation, context) |
| Triptych | 7 | The three-region bottom-panel convention: minimap / info / command card |
| Channel overload | 8 | The anti-pattern of a single information channel carrying all cognitive-function types |
| Viewport augmentation | 8 | Temporarily transforming the viewport into a data visualization surface (e.g., Caesar III's overlay system) |
| Archetype | 6 | A named cluster in morphological space defined by panel placement, functional decomposition, and interaction model |
| Lineage | 12 | A path through morphological space traced by successive releases from a studio or design tradition |
| Near-miss | 5 | A game satisfying some but not all panel-frame factors, used to test the definition's boundary discrimination |
