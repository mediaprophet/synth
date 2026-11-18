# kinetics (16)

XPBD, DirectSpace, NavMesh funnel+ORCA, NLA/LensTween, two-bone IK, FEM/LBM, cloth, CPM. Find: `synth_find q=xpbd` (or `nla`, `ik`, `navmesh`, `kinetics`).

**Crates:** `kinetics-continuum-multiphysics`, `kinetics-garment-draping`, `kinetics-block-simulation`, `kinetics-motion-compositor`, `kinetics-schedule-cpm`. Runtime providers are Etalon (`etalon-sim`). These are Qualia capacities, not a third engine.

## Morphisms

| ID | Args |
|---|---|
| `hypermedia.kinetics.sample_frames` | `numerator`, `denominator`, `count` |
| `hypermedia.kinetics.solve_two_bone_ik` | `root`, `target`, `pole`, bone lengths |
| `hypermedia.kinetics.interpolate_keyframes` | `keyframes`, `fps`, `easing` |
| `hypermedia.kinetics.export_lottie` | vector layers → Bodymovin JSON |
| `hypermedia.kinetics.interpolate_motor_fti` | prev/curr PGA motors, `alpha`, `teleport` |
| `hypermedia.kinetics.step_island_xpbd` | rigid `bodies`, Area3D overrides, substeps |
| `hypermedia.kinetics.query_direct_space` | colliders, ray/sweep, collision mask |
| `hypermedia.kinetics.navigate_funnel_orca` | NavMesh portals, agent, ORCA neighbors |
| `hypermedia.kinetics.evaluate_nla_channelbag` | ActionChannelbag strips, LensTween |
| `hypermedia.kinetics.step_simulation_zone` | point cloud, curl-noise/attractor |
| `hypermedia.kinetics.rig_biomechanical_archetype` | fauna archetype, speed, turn rate |
| `hypermedia.kinetics.solve_continuum_fem_lbm` | elastic params, D3Q19 LBM |
| `hypermedia.kinetics.drape_orthotropic_garment` | textile + pinned vertices |
| `hypermedia.kinetics.simulate_causal_block_diagram` | causal blocks, RK4 |
| `hypermedia.kinetics.schedule_cpm_evm_montecarlo` | task network, trials, Gantt SVG |
| `hypermedia.kinetics.evaluate_mobility_fracture_risk` | clinical mobility / fracture-risk screen |

## Familiar names

| Ask for | Call |
|---|---|
| XPBD / rigid island | `step_island_xpbd` |
| Ray / sweep / hit | `query_direct_space` |
| NavMesh / crowd | `navigate_funnel_orca` |
| NLA / tween | `evaluate_nla_channelbag` |
| Two-bone IK | `solve_two_bone_ik` |
| Terrain contact XPBD | `hypermedia.geodesy.couple_continuum_xpbd` ([geodesy.md](geodesy.md)) |

Invoke inside `synth_produce_present`. The visitor runtime (QualiaPortal + world host) plays the produced HMC; Synth does not simulate the live world.

Back to [README.md](README.md) · contract [../AGENTS.md](../AGENTS.md)
