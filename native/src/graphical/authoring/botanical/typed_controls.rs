//! Explicit native entry for a source-bound botanical graph.
//! This form supplies no specimen facts or automatic graph adoption.
use super::{BotanicalEdit, BotanicalLimits, BotanicalRecipe, Organ};
use crate::graphical::controller::EditorSnapshot;
use crate::graphical::preview::SourceRef;
use dioxus::prelude::*;

const MAX_KEY_BYTES: usize = 128;
const MAX_NUMBER_BYTES: usize = 64;

fn assign_bounded(target: &mut String, value: String, maximum: usize) -> bool {
    if value.len() > maximum {
        return false;
    }
    *target = value;
    true
}

#[derive(Clone, Default)]
struct OrganRow {
    id: u64,
    key: String,
    parent: String,
    position: [String; 3],
}

#[derive(Clone, Default)]
struct AttractorRow {
    id: u64,
    position: [String; 3],
}

#[derive(Clone)]
struct GraphDraft {
    source: SourceRef,
    part_id: String,
    generation: u64,
    version: u32,
    organs: Vec<OrganRow>,
    attractors: Vec<AttractorRow>,
    next_row_id: u64,
    // length, kill radius, perception radius, angle, iteration count,
    // tip radius, pipe exponent, ring slices.
    parameters: [String; 8],
}

fn same_generation(draft: &GraphDraft, generation: u64) -> bool {
    draft.generation == generation
}

fn point(values: &[String; 3]) -> Result<[f64; 3], String> {
    let mut point = [0.0_f64; 3];
    for (index, value) in values.iter().enumerate() {
        point[index] = value
            .parse()
            .map_err(|_| "Enter finite coordinates in metres")?;
        if !point[index].is_finite() {
            return Err("Enter finite coordinates in metres".into());
        }
    }
    Ok(point)
}

impl GraphDraft {
    fn new(source: SourceRef, part_id: String, generation: u64) -> Self {
        Self {
            source,
            part_id,
            generation,
            version: 1,
            organs: Vec::new(),
            attractors: Vec::new(),
            next_row_id: 0,
            parameters: std::array::from_fn(|_| String::new()),
        }
    }

    fn next_id(&mut self) -> Option<u64> {
        let id = self.next_row_id;
        self.next_row_id = self.next_row_id.checked_add(1)?;
        Some(id)
    }

    fn ready(&self, fresh: bool, source: &SourceRef, part_id: &str) -> bool {
        fresh && &self.source == source && self.part_id == part_id
    }

    fn may_apply(
        &self,
        fresh: bool,
        source: &SourceRef,
        part_id: &str,
        has_graph: bool,
        replace_confirmed: bool,
    ) -> bool {
        self.ready(fresh, source, part_id) && (!has_graph || replace_confirmed)
    }

    fn edit(&self) -> Result<BotanicalEdit, String> {
        let number = |index: usize| -> Result<f64, String> {
            let value: f64 = self.parameters[index]
                .parse()
                .map_err(|_| "Enter all graph and kernel values")?;
            if !value.is_finite() {
                return Err("Graph and kernel values must be finite".into());
            }
            Ok(value)
        };
        let integer = |index: usize| -> Result<usize, String> {
            self.parameters[index]
                .parse()
                .map_err(|_| "Enter whole iteration and ring counts".into())
        };
        let organs = self
            .organs
            .iter()
            .map(|row| {
                Ok(Organ {
                    key: row.key.clone(),
                    parent: if row.parent.is_empty() {
                        None
                    } else {
                        Some(row.parent.clone())
                    },
                    position_m: point(&row.position)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let attractors_m = self
            .attractors
            .iter()
            .map(|row| point(&row.position))
            .collect::<Result<Vec<_>, _>>()?;
        let recipe = BotanicalRecipe {
            version: self.version,
            part_id: self.part_id.clone(),
            organs,
            attractors_m,
            node_length_m: number(0)?,
            kill_radius_m: number(1)?,
            perception_radius_m: number(2)?,
            cone_angle_rad: number(3)?,
            max_iterations: integer(4)?,
            tip_radius_m: number(5)?,
            pipe_exponent: number(6)?,
            ring_slices: integer(7)?,
        };
        recipe.validate(BotanicalLimits::default())?;
        Ok(BotanicalEdit::SetRecipe { recipe })
    }
}

#[component]
pub(crate) fn TypedBotanicalControls(
    selected_part: String,
    source: SourceRef,
    fresh: bool,
    has_graph: bool,
    live_view: Signal<EditorSnapshot>,
    live_selected: Signal<String>,
    onedit: EventHandler<BotanicalEdit>,
) -> Element {
    let mut draft = use_signal(|| None::<GraphDraft>);
    let mut status = use_signal(String::new);
    let mut replace_confirmed = use_signal(|| false);
    let mut next_generation = use_signal(|| 0u64);
    let current = draft();
    let generation = current.as_ref().map(|d| d.generation).unwrap_or(0);
    let ready = current
        .as_ref()
        .is_some_and(|d| d.ready(fresh, &source, &selected_part));
    let selected_for_start = selected_part.clone();
    let source_for_start = source.clone();
    let max_organs = BotanicalLimits::default().max_organs;
    let max_attractors = BotanicalLimits::default().max_attractors;
    rsx! {
        fieldset { style: "margin-top:10px;border:1px solid #405264;",
            legend { "Enter botanical graph" }
            p { "Enter source-specific graph values. No specimen information is supplied; applying replaces the selected part's graph after normal validation." }
            button { disabled: !fresh || selected_part.is_empty() || current.is_some(),
                onclick: move |_| {
                    if !fresh || selected_for_start.is_empty() || draft().is_some() {
                        status.set("Existing graph entry retained; discard it explicitly before starting another.".into());
                        return;
                    }
                    let id = next_generation();
                    let Some(next) = id.checked_add(1) else { status.set("Graph entry generation exhausted".into()); return; };
                    next_generation.set(next);
                    draft.set(Some(GraphDraft::new(source_for_start.clone(), selected_for_start.clone(), id)));
                    replace_confirmed.set(false);
                    status.set("Enter organ and kernel values for the selected source.".into());
                }, "Start graph entry" }
            if current.is_some() && !ready {
                p { role: "status", "Source or selection changed. The local form is retained but cannot be applied to this source." }
            }
            if let Some(form) = current.clone() {
                p { "Selected part {form.part_id}. Enter the root first with its parent empty; later organs name an earlier parent." }
                label { "Branch geometry profile"
                    select { disabled: !ready, value: "{form.version}", onchange: move |e| {
                        if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) {
                            match e.value().as_str() { "1" => d.version = 1, "2" => d.version = 2, _ => status.set("Unsupported botanical profile".into()) }
                        }
                    },
                        option { value: "1", "Separated edge sweeps (legacy v1)" }
                        option { value: "2", "Continuous linear chains (v2; forks remain capped)" }
                    }
                }

                for row in form.organs.iter() {
                    { let row_id = row.id; rsx! { div { key: "organ-{row_id}",
                        label { "Organ key" input { disabled: !ready, maxlength: 128, value: "{row.key}",
                            oninput: move |e| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if let Some(row) = d.organs.iter_mut().find(|row| row.id == row_id) { if !assign_bounded(&mut row.key, e.value(), MAX_KEY_BYTES) { status.set("Organ key exceeds 128 bytes".into()); } } } } }
                        label { "Parent key (empty for root)" input { disabled: !ready, maxlength: 128, value: "{row.parent}",
                            oninput: move |e| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if let Some(row) = d.organs.iter_mut().find(|row| row.id == row_id) { if !assign_bounded(&mut row.parent, e.value(), MAX_KEY_BYTES) { status.set("Parent key exceeds 128 bytes".into()); } } } } }
                        for axis in 0..3 {
                            label { "Position {axis} (m)" input { disabled: !ready, maxlength: 64, value: "{row.position[axis]}",
                                oninput: move |e| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if let Some(row) = d.organs.iter_mut().find(|row| row.id == row_id) { if !assign_bounded(&mut row.position[axis], e.value(), MAX_NUMBER_BYTES) { status.set("Coordinate exceeds 64 bytes".into()); } } } } }
                        }
                        button { disabled: !ready, onclick: move |_| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if let Some(index) = d.organs.iter().position(|row| row.id == row_id) { d.organs.remove(index); } }, "Remove organ" }
                    } } }
                }
                button { disabled: !ready || form.organs.len() >= max_organs,
                    onclick: move |_| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if d.organs.len() < max_organs { if let Some(id) = d.next_id() { d.organs.push(OrganRow { id, ..OrganRow::default() }); } } }, "Add organ" }
                for row in form.attractors.iter() {
                    { let row_id = row.id; rsx! { div { key: "attractor-{row_id}",
                        for axis in 0..3 {
                            label { "Attractor {axis} (m)" input { disabled: !ready, maxlength: 64, value: "{row.position[axis]}",
                                oninput: move |e| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if let Some(row) = d.attractors.iter_mut().find(|row| row.id == row_id) { if !assign_bounded(&mut row.position[axis], e.value(), MAX_NUMBER_BYTES) { status.set("Coordinate exceeds 64 bytes".into()); } } } } }
                        }
                        button { disabled: !ready, onclick: move |_| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if let Some(index) = d.attractors.iter().position(|row| row.id == row_id) { d.attractors.remove(index); } }, "Remove attractor" }
                    } } }
                }
                button { disabled: !ready || form.attractors.len() >= max_attractors,
                    onclick: move |_| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if d.attractors.len() < max_attractors { if let Some(id) = d.next_id() { d.attractors.push(AttractorRow { id, ..AttractorRow::default() }); } } }, "Add attractor" }
                for (index, label) in ["Node length (m)", "Kill radius (m)", "Perception radius (m)", "Cone angle (radians)", "Maximum iterations", "Tip radius (m)", "Pipe exponent", "Ring slices"].iter().enumerate() {
                    label { "{label}" input { disabled: !ready, maxlength: 64, value: "{form.parameters[index]}",
                        oninput: move |e| if let Some(d) = draft.write().as_mut().filter(|d| same_generation(d, generation)) { if !assign_bounded(&mut d.parameters[index], e.value(), MAX_NUMBER_BYTES) { status.set("Graph value exceeds 64 bytes".into()); } } } }
                }
                if has_graph {
                    label { input { r#type: "checkbox", checked: replace_confirmed(), disabled: !ready,
                        onchange: move |e| if draft().as_ref().is_some_and(|d| same_generation(d, generation)) { replace_confirmed.set(e.checked()); } }
                        "I understand this will replace the accepted graph for this selected part."
                    }
                }
                button { disabled: !ready || (has_graph && !replace_confirmed()), onclick: move |_| {
                    if let Some(form) = draft().as_ref().filter(|d| same_generation(d, generation)) {
                        let snapshot = live_view();
                        let selected_now = live_selected();
                        if snapshot.reconciliation_required || !snapshot.publication_is_current() || !form.may_apply(fresh, &snapshot.source, &selected_now, has_graph, replace_confirmed()) {
                            status.set("Current source, selected part or replacement confirmation changed; graph not applied.".into());
                            return;
                        }
                        match form.edit() { Ok(edit) => onedit.call(edit), Err(error) => status.set(error) }
                    }
                }, "Apply entered graph" }
                button { onclick: move |_| { if draft().as_ref().is_some_and(|d| same_generation(d, generation)) { draft.set(None); status.set("Local graph entry discarded; accepted source was not changed.".into()); } }, "Discard local graph entry" }
            }
            p { role: "status", "{status}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qualia_core_db::poet_host::invoke::studio_store::{self, StudioArtifact};

    #[test]
    fn entered_graph_uses_selected_part_and_refuses_invalid_parent_or_number() {
        let mut bounded = String::from("kept");
        assert!(!assign_bounded(
            &mut bounded,
            "x".repeat(MAX_KEY_BYTES + 1),
            MAX_KEY_BYTES
        ));
        assert_eq!(bounded, "kept");
        assert!(assign_bounded(
            &mut bounded,
            "x".repeat(MAX_NUMBER_BYTES),
            MAX_NUMBER_BYTES
        ));
        assert!(!assign_bounded(&mut bounded, "é".repeat(65), MAX_KEY_BYTES));
        let id = format!("urn:poet:typed-botanical-test:{}", std::process::id());
        let (_, stamp) = studio_store::create_artifact_if_absent(StudioArtifact::new(
            &id,
            "Geometry",
            "typed botanical form test",
        ))
        .unwrap();
        let mut form = GraphDraft::new(
            SourceRef {
                artifact_id: id,
                stamp,
            },
            "branch".into(),
            0,
        );
        assert!(same_generation(&form, 0));
        assert!(!same_generation(&form, 1));
        let current = form.source.clone();
        assert!(form.ready(true, &current, "branch"));
        assert!(!form.ready(false, &current, "branch"));
        assert!(!form.ready(true, &current, "other"));
        let mut foreign = current.clone();
        foreign.artifact_id.push_str(":other");
        assert!(!form.ready(true, &foreign, "branch"));
        assert!(!form.may_apply(true, &current, "branch", true, false));
        assert!(form.may_apply(true, &current, "branch", true, true));
        assert!(!form.may_apply(false, &current, "branch", true, true));
        form.organs = vec![
            OrganRow {
                id: 0,
                key: "root".into(),
                parent: String::new(),
                position: ["0".into(), "0".into(), "0".into()],
            },
            OrganRow {
                id: 1,
                key: "tip".into(),
                parent: "root".into(),
                position: ["0".into(), "0".into(), "1".into()],
            },
        ];
        form.attractors.push(AttractorRow {
            id: 2,
            position: ["0".into(), "0".into(), "2".into()],
        });
        form.parameters = ["0.2", "0.05", "0.8", "3.0", "2", "0.02", "2", "8"].map(str::to_owned);
        match form.edit().unwrap() {
            BotanicalEdit::SetRecipe { recipe } => {
                assert_eq!(recipe.part_id, "branch");
                assert_eq!(recipe.organs[1].parent.as_deref(), Some("root"));
                assert_eq!(recipe.tip_radius_m, 0.02);
            }
            _ => panic!("typed form must create an explicit graph edit"),
        }
        form.version = 2;
        match form.edit().unwrap() {
            BotanicalEdit::SetRecipe { recipe } => assert_eq!(recipe.version, 2),
            _ => panic!("typed profile must remain a recipe edit"),
        }
        form.version = 1;
        form.organs[1].parent = "unknown".into();
        assert!(form.edit().is_err());
        form.organs[1].parent = "root".into();
        form.parameters[5] = "NaN".into();
        assert!(form.edit().is_err());
        // Queued handlers resolve row IDs, not shifted list indexes.
        form.organs.remove(0);
        assert!(form.organs.iter_mut().find(|row| row.id == 0).is_none());
        assert_eq!(
            form.organs.iter().find(|row| row.id == 1).unwrap().key,
            "tip"
        );
        form.next_row_id = 3;
        assert_eq!(form.next_id(), Some(3));
    }
}
