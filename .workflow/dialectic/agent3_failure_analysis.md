# Failure Analysis: Panel Frame Bible

A topology of definitional risks, blind spots, and compounding failure modes in the comprehensive panel-frame-bible project.

---

## 1. Definitional Circularity and the Naming Illusion

### 1.1 The predicate problem

The boxed definition states that panels are opaque, persistent, non-overlapping regions of the display surface that are not part of the viewport. The viewport is the proper subset of the display surface that renders the game world. This appears rigorous until you ask what counts as "rendering the game world." The minimap renders the game world. The unit portrait renders the game world. The build-preview overlay renders the game world. If rendering the game world is the viewport criterion, then every in-game visual element is part of the viewport, and the definition collapses to "panels are the parts that don't render the game world except when they do."

The implicit escape hatch is that "the viewport" means the primary, scrollable, spatially-continuous rendering of the game world. But this is a much more specific claim than the definition makes, and it smuggles in assumptions about continuity, scrollability, and primacy that do the actual discriminatory work. The boxed definition presents itself as a spatial predicate (area relationship) when it is actually a functional predicate (the "main" view). This mismatch means the definition will fail silently on any game where the primary interaction surface is not obviously the largest contiguous world-rendering region — for instance, Dwarf Fortress's fortress mode, where ASCII representations of game state fill regions that could be classified as either viewport or panel depending on whether you consider the z-level list a panel or an extension of the viewport.

### 1.2 High-value factors as definitional echoes

The six high-value factors are supposed to provide discriminatory power beyond the boxed definition. Examine them:

- **Viewport partitioning**: restates that the viewport is a proper subset of the display surface (the core definition).
- **Opacity**: restates "opaque" from the core definition.
- **Permanence**: restates "persistent" from the core definition.
- **Fixed geometry**: strengthens "persistent" by adding that panels do not move or resize, which is new information — but it is immediately contradicted by any game with resizable panels (Civilization III's advisor screens, which slide in from panel regions).
- **Spatial exclusivity**: restates "non-overlapping" from the core definition.
- **Information density**: this is the only factor that adds genuinely new discriminatory content. It says panels carry high information density. But it is also the vaguest — density compared to what? By what metric?

Five of six high-value factors are entailed by the three-word phrase "opaque, persistent, non-overlapping." They do not discriminate; they elaborate. A definition with five redundant factors and one vague one creates false confidence: the reader sees six criteria and assumes they are jointly constraining, when in practice the definition's discriminatory power rests entirely on the boxed predicate and the poorly-specified "information density" criterion.

The compounding risk: when someone applies the definition to a borderline case, they will check five factors that all pass or fail together (because they are the same criterion stated five ways) and one factor that is subjective. The definition will produce confident binary judgments (5/6 factors met!) on cases where it should produce uncertain continuous judgments.

### 1.3 The membership test that cannot fail

A formal definition's value lies in its ability to exclude non-members. Consider what the panel frame definition excludes. Any game with a full-screen viewport and no persistent opaque regions is excluded. Any game with no viewport (a pure menu game) is excluded. But these are trivially non-members that no one would confuse for panel frames. The interesting question is: can the definition exclude a game that a knowledgeable designer would intuitively say "that's not really a panel frame, even though it looks like one"?

Consider Diablo II (2000). It has a persistent bottom panel (the belt/skill bar). It has an opaque right panel (the inventory, when open). By the letter of the definition, Diablo II in the inventory-open state has a panel frame layout with a viewport budget less than 1.0. But no one classifies Diablo II as a panel-frame game. The definition lacks the ability to distinguish between "this game's primary mode of interaction uses a panel frame" and "this game temporarily enters a panel-frame-like state." The definition is a snapshot predicate applied to a temporal phenomenon.

---

## 2. Survivorship Bias and the Canon's Silent Majority

### 2.1 The canon as confirmation set

A canon of 15 games that all exemplify the pattern cannot test the definition's exclusion power. It can only test its inclusion power — and inclusion is easy when the canon was selected by applying the definition. The canon is a tautological confirmation set. To test the definition rigorously, you need a set of near-miss games: games that share most but not all of the panel-frame properties, where the definition must correctly draw the line.

Candidates for a near-miss set that the canon ignores: Dungeon Keeper (1997), which has a first-person sub-viewport embedded in a panel. Myth: The Fallen Lords (1997), which uses a bottom panel but rotates the viewport in 3D, breaking the 2D scrolling assumption. Homeworld (1999), which has persistent panels around a fully 3D viewport with no fixed "up." Syndicate (1993), which has a panel frame that is half minimap and half control surface, with the viewport occupying the upper portion. Each of these tests a different boundary condition of the definition. Without them, the canon is a display case, not a test suite.

### 2.2 Geographic and platform erasure

The canon is exclusively English-language PC games from Western studios. This is not merely an omission; it is a systematic bias that shapes the definition itself. Japanese strategy games from Koei — Romance of the Three Kingdoms, Nobunaga's Ambition, Bandit Kings of Ancient China — used panel frames extensively in the 1980s and 1990s, but with fundamentally different information hierarchies. Koei games often placed the panel as the primary interaction surface with a small viewport serving as geographic context, inverting the Western assumption that the viewport is primary. If the definition was built from a canon that included Koei, the "viewport as primary" assumption might not have survived.

Eastern European titles like Cossacks: European Wars (2001), which supported thousands of units on screen with minimal panel space, or Knights and Merchants (1998), which used a distinctive radial menu system alongside panels, represent alternative adaptations that the definition may misclassify. The definition was fitted to Western PC strategy games. Its generalization to "strategy games broadly" is an unearned extrapolation.

### 2.3 Console strategy as a falsification opportunity

Console strategy games operated under fundamentally different constraints: lower resolution (256x224 for SNES), no mouse cursor, d-pad navigation, limited button count. Advance Wars (2001) partitions its screen with panels, but the panels serve different functions than PC panels because cursor movement is expensive (d-pad) rather than cheap (mouse). Fire Emblem (2003) uses a split-screen layout during combat that could be classified as a temporary panel frame. Final Fantasy Tactics (1997) uses a 3D isometric viewport with menu-driven panel overlays that defy the opacity binary.

These games are not edge cases; they are a substantial portion of the strategy game corpus. A definition that cannot account for them is not a definition of "the strategy game panel frame" but of "the Western PC strategy game panel frame with mouse input." The hidden platform assumption interacts with the geographic bias to create a definition that appears universal but is actually narrow.

---

## 3. False Dichotomies and the Continuous Spectrum

### 3.1 The opacity binary

The definition requires panels to be opaque. This creates a clean binary: opaque panel frame vs. transparent overlay HUD. But real games occupy a spectrum. Total Annihilation (1997) used semi-transparent build menus. Supreme Commander (2007, just outside the golden age but architecturally descended from TA) used freely-resizable semi-transparent panels. Within the golden age itself, Warcraft III (2002) used panel textures with transparency effects at the edges, creating a soft boundary between panel and viewport.

The opacity binary forces a categorical judgment on what is a continuous gradient. The consequence is that the definition will confidently classify clear cases (StarCraft's fully opaque panel: yes; Quake's transparent HUD: no) and produce arbitrary results on intermediate cases. Worse, the user of the definition will not know when they are in the arbitrary zone because the definition presents the binary as exhaustive.

### 3.2 The view-level vs. game-level confusion

The "vs modal screen" distinction reveals a deeper confusion: the definition treats "panel frame" as a property of a game when it is actually a property of a view or mode within a game. Civilization II has a map view (panel frame) and a city screen (modal). Master of Orion II has a galaxy view (panel frame), a colony screen (different panel frame), a ship design screen (modal), and a combat screen (yet another panel frame). Is Master of Orion II a "panel frame game"? Which of its four major views is the canonical one?

The definition implicitly privileges the "main" view, but many golden-age strategy games spent significant play-time in secondary views. A Civilization II player might spend more time in city screens than on the main map. The definition's implicit view hierarchy does not match the player's actual time allocation, creating a mismatch between what the definition describes and what the player experiences.

This compounds with the survivorship bias: the canon was likely selected based on the main-map view, ignoring that many canonical games spend substantial time in non-panel-frame views. The definition describes one mode of a multi-modal system and calls it the game's UI pattern.

---

## 4. The Lineage Trap and False Teleology

### 4.1 Post-hoc narrative construction

Lineage-based organization (Westwood tree, Blizzard tree, Ensemble tree) implies that design decisions were made by conscious reference to predecessors. But game developers are influenced by dozens of sources simultaneously: competitor games, platform SDKs, publisher requirements, individual developer taste, engine constraints, team size. Attributing StarCraft's bottom panel to "Blizzard's evolution from Warcraft II's sidebar" is a narrative that may be entirely fictional. Without access to internal design documents or developer testimony, lineage claims are reverse-engineered stories that privilege visual similarity over actual causal history.

The danger is not merely inaccuracy but distortion: lineage framing causes the reader to see StarCraft's panel as a "descendant" of Warcraft II's panel, which highlights their similarities and obscures their differences. It creates a lens that pre-selects what the reader notices. A formal definition should be lens-neutral.

### 4.2 The innovation attribution problem

Lineage framing attributes innovations to studios ("Westwood pioneered the sidebar") rather than to the broader design ecosystem. But many innovations were independently discovered. The bottom-panel layout appeared in Age of Empires (Ensemble, 1997), StarCraft (Blizzard, 1998), and Dark Reign (Auran, 1997) within a 12-month window. Was this convergent evolution driven by shared constraints (higher resolutions making sidebars less efficient), or did one studio copy another? The lineage model cannot accommodate convergent evolution; it assumes a tree structure when the actual history may be a web or a field.

This interacts with the golden-age boundary problem: if the definition's lineage trees are rooted in the late 1980s, they miss the pre-history of screen partitioning in earlier games (the Atari ST and Amiga strategy games of 1985-1988 that established many of these conventions). The lineage appears to begin with Dune II (1992) because the canon begins there, but the design patterns predate the canon.

---

## 5. The Viewport Budget as a Misleading Metric

### 5.1 Area without shape is meaningless

Viewport budget is defined as viewport_area / display_surface_area. Two games can have identical viewport budgets of 0.70 but wildly different functional layouts: Game A has a right sidebar (tall, narrow panel), giving the viewport a wide horizontal view ideal for RTS base management. Game B has a bottom bar (short, wide panel), giving the viewport a square-ish view ideal for isometric exploration. The viewport budget number is identical. The functional implications are opposite.

This is not merely an imprecision; it is a systematic loss of design-relevant information. Anyone using the viewport budget to compare games will conflate layouts that create fundamentally different player experiences. The metric encourages quantitative comparison where qualitative comparison is needed.

### 5.2 Measurement methodology absence

The document apparently provides "approximate" viewport budget numbers. But without a measurement methodology, these numbers fail the basic scientific criterion of reproducibility. Were panels measured by their bounding rectangles (including dead space in corners)? Were decorative borders counted as panel area or viewport area? Were status bars included? At 640x480, a 1-pixel measurement error in panel width creates a 0.3% error in viewport budget — trivial for a single measurement, but systematic across 15 games if the measurement method is inconsistent.

The absence of methodology interacts with the area-without-shape problem: if you cannot reproduce the measurements, you cannot even verify whether two games with "the same" viewport budget actually have the same viewport budget or just approximately similar ones within an unknown error margin.

---

## 6. Genre Conflation and the Universality Illusion

### 6.1 Interaction pattern divergence

An RTS player issues 200+ actions per minute (APM) and needs the panel to provide instant visual feedback on unit selection, production queues, and ability cooldowns. A city builder player issues maybe 5-10 actions per minute and needs the panel to display long-term trend information. A 4X player shifts between rapid tactical decisions (combat) and slow strategic decisions (diplomacy, research) and needs panels that serve both tempos. These are not variations on a single pattern; they are fundamentally different information architectures that happen to share a visual similarity (rectangles on the edges of the screen).

A definition that groups them under "panel frame" because they look alike is committing the same error as grouping whales and fish because they both swim. The visual similarity is a convergent response to the shared constraint of "display surface has finite area" — not evidence of a shared design lineage or shared design intent.

### 6.2 The depth-of-modal-interaction spectrum

City builders and tycoon games (SimCity, Transport Tycoon, Caesar III) use the panel frame as a thin interface layer over a viewport that carries almost all game state. 4X games (Civilization, Master of Orion) use the panel frame as a navigation aid with most complexity in modal screens. Grand strategy games (EU, Victoria) use the panel frame as one of many layered information surfaces, with tooltips, nested panels, and map modes doing the heavy information-delivery work.

These three patterns — thin-panel/thick-viewport, navigation-panel/modal-complexity, and layered-panel/map-mode — are architecturally distinct. A definition that classifies all three as "panel frame" is operating at a level of abstraction where the classification provides no design guidance. You could know that a game "uses a panel frame" and still know nothing about how its information architecture works.

---

## 7. Temporal Blindness and the Static Snapshot Fallacy

### 7.1 The panel as dynamic system

The definition describes panel frames as static spatial layouts. But panels are temporal systems: the command card in StarCraft changes its contents up to several times per second as the player shifts selection. The resource bar ticks every game-second. The minimap updates continuously. Alert icons flash, production progress bars fill, idle-worker counts increment. The rate, rhythm, and responsiveness of these updates are design-critical properties that the spatial definition entirely ignores.

Two panel frames with identical spatial layouts but different update rates create different cognitive loads. A panel that updates 30 times per second demands peripheral attention; a panel that updates once per minute can be checked deliberately. The definition cannot distinguish between these because it has no temporal vocabulary.

### 7.2 The selection-state dependency

In most RTS games, the panel's content is a function of the player's current selection. No selection: the panel shows global controls. Single unit selected: the panel shows that unit's abilities. Multiple units selected: the panel shows a group interface. Building selected: the panel shows production options. The panel is not a fixed information display but a context-sensitive interface that changes state in response to player action.

This means the viewport budget is not constant within a single game session — the same screen region shows different amounts of information at different moments. A resource bar showing 4 numbers has a different effective information density than a command card showing 15 ability icons, even though they occupy the same screen area. The static definition measures the container but not the content, the vessel but not the cargo.

---

## 8. The Missing Interaction Model

### 8.1 Attention as the hidden currency

The definition models screen space as the scarce resource ("viewport budget"). But the actual scarce resource is player attention. A panel that requires frequent mouse travel from the viewport reduces the player's ability to monitor the game world. A panel that can be operated entirely by keyboard shortcuts imposes zero attention cost from the viewport. Two panel frames with identical viewport budgets and identical visual layouts but different input models create different attention economies.

StarCraft's bottom panel illustrates this: novice players click panel buttons (high attention cost, frequent viewport-to-panel gaze shifts), while expert players use keyboard hotkeys (zero panel attention cost during normal play, panel serves only as a passive status display). The "same" panel frame is two completely different interfaces for two different player populations. The definition describes the visual artifact but not the interaction system it participates in.

### 8.2 Mouse travel geometry

The physical layout of panels creates characteristic mouse-travel patterns. A right sidebar requires horizontal mouse movement from the viewport center. A bottom panel requires diagonal movement. An L-shaped panel wrapping two edges creates an ambiguous travel path. Fitts's Law predicts that panels at screen edges (infinite-width targets) are faster to reach than panels inset from the edge — but the definition says nothing about edge-anchoring as a structural property.

Mouse travel geometry compounds with the viewport shape problem: a right sidebar forces the viewport leftward, and the player's average mouse position shifts left, changing the travel distance to every other screen element. The spatial ripple effects of panel placement are a design-critical concern that the static area-based definition cannot capture.

---

## 9. The Decorative Surround Paradox

### 9.1 Atmosphere as function

The definition classifies decorative surround as a low-value factor — an aesthetic choice with minimal structural import. But consider StarCraft's race-specific panel textures. The Zerg panel uses organic textures; the Protoss panel uses crystalline textures; the Terran panel uses metallic textures. These decorations serve a functional purpose in competitive play: they instantly communicate which race the player is controlling, useful in spectator mode and in mental context-switching during mirror matches. The decoration is information.

More subtly, the visual weight of panel decoration affects the perceived boundary between viewport and panel. A heavily-decorated panel with ornate borders creates a strong perceptual boundary; a minimally-decorated panel with subtle borders creates a weak one. Perceptual boundary strength affects how easily the player shifts attention between viewport and panel — a strong boundary creates a higher attention-switching cost. The decoration is an interaction affordance disguised as aesthetics.

### 9.2 The affordance collapse

By dismissing decorative surround as low-value, the definition collapses a meaningful design dimension into a footnote. This interacts with the missing interaction model: if mouse travel and attention switching are unmodeled, and panel decoration (which affects attention switching) is also unmodeled, then the definition is triply blind to a phenomenon that skilled UI designers consider important.

---

## 10. Scale Conflation and the Panel-as-Container Error

### 10.1 The heterogeneity within "panel"

The definition uses "panel" to describe any opaque, persistent, non-viewport region. This subsumes wildly different UI elements under a single term: a 32-pixel-tall resource bar showing four numbers. A 200x400-pixel command card with icons, tooltips, hotkey labels, and contextual state. A 180x180-pixel minimap with real-time unit positions, fog of war, and camera-box indicators. A 120-pixel-wide unit portrait with health bar, status icons, and selection indicator.

These are not the same kind of thing. The resource bar is a passive telemetry display. The command card is an active control surface. The minimap is a navigation and awareness tool. The unit portrait is a context display. They have different update rates, different interaction models, different cognitive load profiles, and different design constraints. Calling them all "panels" is like calling a steering wheel, a speedometer, and a rearview mirror all "dashboard components" — technically correct but analytically useless when you need to design any one of them.

### 10.2 Display density vs. control density vs. navigation density

The information density factor partially acknowledges this problem but fails to resolve it. "Information density" could mean: pixels of distinct visual information per unit area (display density), interactive controls per unit area (control density), or spatial navigation affordances per unit area (navigation density). A minimap has high navigation density but low control density. A command card has high control density but moderate display density. A resource bar has low control density, low navigation density, and moderate display density. Collapsing these into a single "information density" metric obscures the distinctions that matter for design.

---

## 11. The Missing User Model and the Expert-Novice Inversion

### 11.1 The panel frame as training wheels

For novice players, the panel frame provides discoverability: all available commands are visible as buttons, all resources are displayed as numbers, the minimap provides spatial orientation. The panel is the primary learning interface. For expert players, the panel frame is largely irrelevant: commands are issued via hotkeys, resource counts are tracked mentally or glanced at peripherally, and the minimap is the only panel element that retains high value.

This means the panel frame's importance inverts with player skill. A definition that treats the panel frame as a fixed structural feature of the game misses the fact that it is a skill-dependent feature of the player-game system. The "same" panel frame is the primary interface for one player and a vestigial display for another. A panel frame bible that does not acknowledge this inversion will produce guidance that is either correct for novices and wrong for experts, or correct for experts and wrong for novices, but never correct for both.

### 11.2 Spectator mode as a third user

Competitive RTS games added spectator and replay modes in the late golden age. Spectators need different panel information than players: they need production tabs, resource comparisons, supply counts, army value estimates. Games like StarCraft: Brood War and Warcraft III eventually developed spectator UIs that modified the panel frame — adding overlays, changing information displays, replacing the command card with analytics. The panel frame is not just skill-dependent but role-dependent. The definition has no model of user roles.

---

## 12. The Audio Blind Spot and Multi-Modal Information Architecture

### 12.1 Audio as invisible panel space

Golden-age strategy games used audio as an information channel: unit acknowledgment sounds confirm command receipt, alert sounds signal events outside the viewport, ambient sounds provide location-awareness cues, production-complete jingles notify the player of state changes. Each of these audio channels carries information that would otherwise require panel space (a "command received" indicator, an "event outside viewport" alert panel, a "production complete" notification area).

The panel frame's viewport budget is therefore not the full information budget. A game with a high viewport budget (small panels) might compensate with rich audio information, while a game with a low viewport budget (large panels) might have minimal audio. Comparing viewport budgets across games without accounting for audio information substitution will systematically overestimate the information cost of high-viewport-budget designs and underestimate the information cost of low-viewport-budget designs.

### 12.2 The multi-modal compounding effect

The audio blind spot compounds with the temporal blindness and the missing interaction model. Audio information is inherently temporal (it exists only in the moment of playback), interaction-dependent (acknowledgment sounds play only when commands are issued), and user-model-dependent (expert players learn to extract more information from audio cues). A definition that is blind to all three of these dimensions is triply unable to account for audio's role in the panel frame's information architecture.

---

## Compounding Topology

The failure modes above are not independent. They form a connected topology where each blind spot amplifies others:

- **Definitional circularity** prevents the definition from excluding borderline cases, which means the **survivorship bias** in the canon goes untested, which means the **genre conflation** persists because no near-miss game forces the definition to distinguish RTS panels from 4X panels.
- **The viewport budget fallacy** (area without shape) combines with **temporal blindness** (static snapshot) and **the audio blind spot** (visual-only) to create a metric that captures perhaps 30% of the actual information architecture and presents it as a complete measurement.
- **The missing interaction model** combines with **the missing user model** to make the definition blind to the most consequential design dimension: how different players actually use the panel frame. This blindness is then locked in by the **lineage trap**, which attributes design evolution to studio inheritance rather than to changing player populations and skill levels.
- **Scale conflation** (treating all panels as equivalent) combines with **genre conflation** (treating all strategy games as equivalent) to produce a definition that is simultaneously too broad (it includes everything with rectangles on the screen edge) and too narrow (it excludes console strategy games, Japanese strategy games, and games with semi-transparent panels).
- **The decorative surround paradox** is a microcosm of the entire project's risk: a design dimension that appears unimportant in a static spatial analysis becomes important when you consider interaction, attention, and perception — but the definition lacks the vocabulary to express why.

The fundamental meta-failure is this: the panel frame bible models a visual layout pattern when the actual design object is a multi-modal, temporally-dynamic, user-dependent, interaction-driven information architecture that happens to have a visual layout as one of its manifestations. The definition mistakes the shadow on the cave wall for the thing that casts it.
