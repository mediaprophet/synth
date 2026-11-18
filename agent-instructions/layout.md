# layout (10)

Flex/grid, theme, UI sheets, diegetic HUD, InputMap, companion frustum, paged typeset, nodegraph. Find: `synth_find q=layout` (or `input`, `hud`, `flexbox`).

**Crates:** `layout-box-autolayout`, `layout-paged-typesetter`, `layout-nodegraph-compiler`, `layout-presentation-deck`.

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.layout.solve_box_tree` | viewport, flex/grid `boxes` |
| `hypermedia.layout.bind_theme_scope` | `scope`, `mode`, accent/surface tokens |
| `hypermedia.layout.render_ui_sheet` | `layout`, `theme` → RGBA8 |
| `hypermedia.layout.project_ui_svg` | `layout`, `theme` → SVG |
| `hypermedia.layout.project_diegetic_surface` | UI RGBA + 3D quad pose + pick ray |
| `hypermedia.layout.route_input_action` | hardware event + HUD rects + InputMap |
| `hypermedia.layout.synchronize_companion_frustum` | dome `rows`×`cols`, FOV, Lamport frame |
| `hypermedia.layout.solve_flexbox_grid_constraints` | flex/grid container + Hug/Fill children |
| `hypermedia.layout.typeset_paged_foliation` | multi-column, LaTeX math, CSL bibliography |
| `hypermedia.layout.compile_dataflow_nodegraph` | typed NodeNetwork + morph frame |

`route_input_action` is the Qualia InputMap (keyboard/gamepad/pointer, deadzone, HUD rects). `project_diegetic_surface` places a UI RGBA on a 3D quad with a pick ray. Camera look-at for albedo is `compile_10d_pbr` in [manifold.md](manifold.md), not a Godot Camera3D node.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
