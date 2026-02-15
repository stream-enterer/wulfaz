# 07 — Palette System

**Scope:** How mapgen palettes map ASCII characters to terrain, furniture, traps, and other placements.
**Purpose:** The consuming LLM uses this to implement palette loading and symbol resolution so mapgen "rows" strings produce actual tile content.

---

## WHAT A PALETTE IS

A palette is a reusable mapping from single characters to placement operations. When mapgen processes a "rows" grid like `"#....T.."`, it looks up each character in the active palette(s) to determine what terrain, furniture, traps, etc. to place.

JSON type: `"palette"` (registered as `mapgen_palette`).

---

## PALETTE JSON STRUCTURE

```json
{
    "type": "palette",
    "id": "house_palette",

    "terrain": {
        "#": "t_wall",
        ".": "t_floor",
        "+": "t_door_c",
        "w": "t_window_domestic"
    },

    "furniture": {
        "T": "f_table",
        "c": "f_chair",
        "B": "f_bed",
        "K": "f_sink"
    },

    "traps": {
        "^": "tr_beartrap"
    },

    "items": {
        "K": { "item": "SUS_kitchen_sink", "chance": 30, "repeat": [1, 3] }
    },

    "nested": {
        "R": { "chunks": [["house_bedroom", 50], ["house_study", 30], ["null", 20]] }
    },

    "toilets": { "t": {} },
    "vendingmachines": { "v": {} },
    "gaspumps": { "g": {} },

    "mapping": {
        "D": {
            "terrain": "t_door_locked",
            "furniture": "f_null",
            "traps": [["tr_alarm", 1]],
            "item": [{ "item": "office_supplies", "chance": 20 }]
        }
    }
}
```

---

## SYMBOL MAPPING CATEGORIES

Each character can map to multiple layers simultaneously:

| Category | JSON Key | Maps To | Phase |
|---|---|---|---|
| Terrain | `"terrain"` | `ter_str_id` | terrain |
| Furniture | `"furniture"` | `furn_str_id` | furniture |
| Traps | `"traps"` | `trap_str_id` | default |
| Items | `"items"` | `item_group_id` + chance + repeat | default |
| Toilets | `"toilets"` | Fixed furniture placement | default |
| Vending machines | `"vendingmachines"` | Vending machine + items | default |
| Gas pumps | `"gaspumps"` | Gas pump furniture | default |
| Nested | `"nested"` | Weighted nested mapgen chunks | nested_mapgen |
| Mapping | `"mapping"` | Multi-layer combo (terrain+furn+trap+items) | varies |
| Remove all | `"remove_all"` | Clear all layers | removal |

### Value Formats

Values can be:
- **Simple string:** `"#": "t_wall"` — single ID
- **Weighted list:** `"#": [["t_wall_w", 50], ["t_wall_b", 50]]` — random choice by weight
- **Parameter reference:** `"#": { "param": "wall_style", "fallback": "t_wall" }`
- **Switch/case:** `"#": { "switch": { "param": "era" }, "cases": { "modern": "t_wall", "old": "t_wall_w" } }`

> **SCOPE BOUNDARY:** The `"items"` category in palettes references `item_group_id` which is EXCLUDED from the Wulfaz port. Parse and store the mapping but resolve to no-op during generation.

---

## PALETTE COMPOSITION

A mapgen entry can reference multiple palettes:

```json
{
    "type": "mapgen",
    "om_terrain": "house",
    "palettes": ["base_residential", "gruvbox_colors", "modern_furniture"],
    "terrain": { ".": "t_floor" },
    "rows": [ ... ]
}
```

### Override Order

1. Palettes apply left to right — later palettes override earlier ones
2. Inline `"terrain"`/`"furniture"` in the mapgen entry override all palettes
3. Characters not defined in ANY source are mapped to `fill_ter`

Example: if `base_residential` defines `"#": "t_wall_w"` and `modern_furniture` redefines `"#": "t_wall_b"`, the result is `t_wall_b`.

> **IMPLEMENT FULLY:** Palette composition is how CDDA achieves variety without duplicating entire building layouts. A single "house" layout can use different palettes for different eras/styles.

---

## PALETTE INHERITANCE

Palettes can inherit from other palettes:

```json
{
    "type": "palette",
    "id": "fancy_house",
    "palettes": ["base_residential"],
    "furniture": {
        "c": "f_armchair"
    }
}
```

This creates a palette that has everything from `base_residential` plus an override for `"c"` → `f_armchair`.

### Parameter-Driven Inheritance

```json
{
    "type": "palette",
    "id": "house_with_variant",
    "parameters": {
        "damage_palette": {
            "type": "palette_id",
            "scope": "omt",
            "default": { "distribution": [["house_pristine", 30], ["house_damaged", 70]] }
        }
    },
    "palettes": [
        "base_residential",
        { "param": "damage_palette" }
    ]
}
```

The `damage_palette` parameter randomly selects between pristine and damaged variants, each of which overrides different symbols.

> **STUB OK — TODO:** Parameter-driven palette selection adds significant variety. Implement basic palette inheritance first, add parameter resolution later.

---

## THE PLACING_MAP SYSTEM

Internally, `mapgen_palette` stores a `placing_map`:

```cpp
using placing_map = unordered_map<map_key, vector<shared_ptr<jmapgen_piece>>>;
```

Each character maps to a list of `jmapgen_piece` operations. When a character is processed:

1. Look up character in `placing_map`
2. For each piece in the list, apply it (with phase ordering)
3. Terrain pieces apply in `terrain` phase, furniture in `furniture` phase, etc.

This means a single character can trigger terrain placement AND furniture placement AND trap placement AND item spawning simultaneously.

> **PORTING TRAP:** The character→piece list is NOT just terrain+furniture. A single `"D"` in the rows grid might place a locked door (terrain), a doormat (furniture), an alarm trap, AND scattered keys (items). The porting LLM must process all layers per character, not just terrain.

---

## PALETTE DIRECTORY

Key palette files in `data/json/mapgen_palettes/` (112 files total):

### Common/Shared Palettes
| File | Domain |
|---|---|
| `common_parameters.json` | Reusable parameter definitions (walls, roofs, carpets, fences) |
| `null.json` | Empty palette (marker/placeholder) |
| `building.json` | Generic building walls/floors/furniture |

### Residential
| File | Domain |
|---|---|
| `house_general_palette.json` | Standard residential layouts |
| `house_*.json` (multiple) | Variant palettes (survivor, hoarder, abandoned) |
| `apartment*.json` | Urban apartment buildings |

### Commercial / Urban
| File | Domain |
|---|---|
| `office*.json` | Office buildings |
| `bank.json` | Bank interiors |
| `gunstore.json` | Gun shop |
| `grocery_palette.json` | Grocery/convenience store |
| `mall_palette.json` | Shopping mall |
| `hotel_tower_palette.json` | Hotels |

### Institutional
| File | Domain |
|---|---|
| `hospital.json` | Hospital |
| `church.json`, `cathedral_palette.json` | Religious buildings |
| `prison.json` | Prison facilities |

### Infrastructure
| File | Domain |
|---|---|
| `road.json`, `highway.json` | Road/highway terrain |
| `sewers_palette.json` | Sewer tunnels |
| `subway.json` | Subway system |
| `nuclear_plant_palette.json` | Power plant |
| `radio_tower_palette.json` | Radio infrastructure |

### Military / Labs
| File | Domain |
|---|---|
| `lab/` (directory) | Laboratory complex variants |
| `military/` (directory) | Military facility variants |
| `militia/` (directory) | Militia camp variants |

### Rural / Outdoor
| File | Domain |
|---|---|
| `farm*.json` | Farm variants (horse, supply, general) |
| `trail.json`, `stream.json` | Natural terrain paths |
| `meadow_palettes.json`, `highlands.json` | Natural biome palettes |
| `orchard_apple.json` | Orchard terrain |

### Special / Themed
| File | Domain |
|---|---|
| `triffid.json` | Fungal/triffid structures |
| `mi-go_palette.json` | Alien structures |
| `netherum/` (directory) | Interdimensional structures |
| `underwater_structures_palette.json` | Underwater facilities |

> **PORTING TRAP:** Wulfaz should NOT port CDDA's palette files directly — they reference CDDA-specific terrain/furniture IDs. Port the SYSTEM (palette loading, symbol resolution, composition) and create Wulfaz-specific palettes in KDL format.
