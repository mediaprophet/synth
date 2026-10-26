//! Programmatic interaction of the real native authoring component.
//! This exercises Aether's mounted event path, not native layout/human acceptance.
use super::*;
use dioxus::core::{AttributeValue, ElementId, Mutation, Mutations};
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

// A retained mutation sink only: it knows no texture form state or recipe logic.
// Controls are located in the actual component's emitted tree after each diff.
#[derive(Clone, Default)]
struct RenderedNode {
    tag: String,
    text: String,
    attrs: HashMap<String, AttributeValue>,
    children: Vec<usize>,
    parent: Option<usize>,
}

struct RenderedControls {
    nodes: Vec<RenderedNode>,
    ids: HashMap<ElementId, usize>,
    stack: Vec<usize>,
}

impl RenderedControls {
    fn new() -> Self {
        Self {
            nodes: vec![RenderedNode::default()],
            ids: HashMap::from([(ElementId::from_raw(0), 0)]),
            stack: vec![],
        }
    }

    fn alloc(&mut self, node: RenderedNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }

    fn clone_tree(&mut self, index: usize) -> usize {
        let mut node = self.nodes[index].clone();
        let children = std::mem::take(&mut node.children);
        node.parent = None;
        let copy = self.alloc(node);
        for child in children {
            let child = self.clone_tree(child);
            self.nodes[child].parent = Some(copy);
            self.nodes[copy].children.push(child);
        }
        copy
    }

    fn top(&self) -> usize {
        *self.stack.last().expect("mutation needs stack node")
    }

    fn detach(&mut self, index: usize) {
        if let Some(parent) = self.nodes[index].parent.take() {
            self.nodes[parent].children.retain(|child| *child != index);
        }
    }

    fn apply(&mut self, mutations: Mutations) {
        for edit in mutations.edits {
            match edit {
                Mutation::PushId { id } => self.stack.push(self.ids[&id]),
                Mutation::SetId { id } => {
                    self.ids.insert(id, self.top());
                }
                Mutation::Child { index } => {
                    let child = self.nodes[self.top()].children[index];
                    *self.stack.last_mut().unwrap() = child;
                }
                Mutation::Pop => {
                    self.stack.pop().expect("balanced mutation stack");
                }
                Mutation::CreateElement { tag, .. } => {
                    let index = self.alloc(RenderedNode {
                        tag,
                        ..Default::default()
                    });
                    self.stack.push(index);
                }
                Mutation::CreateText { value } => {
                    let index = self.alloc(RenderedNode {
                        text: value,
                        ..Default::default()
                    });
                    self.stack.push(index);
                }
                Mutation::Clone => {
                    let copy = self.clone_tree(self.top());
                    *self.stack.last_mut().unwrap() = copy;
                }
                Mutation::AppendChildren { m } => {
                    let children = self.stack.split_off(self.stack.len() - m);
                    let parent = self.top();
                    for child in children {
                        self.detach(child);
                        self.nodes[child].parent = Some(parent);
                        self.nodes[parent].children.push(child);
                    }
                }
                Mutation::ReplaceWith { m }
                | Mutation::InsertBefore { m }
                | Mutation::InsertAfter { m } => {
                    let children = self.stack.split_off(self.stack.len() - m);
                    let anchor = self.top();
                    let parent = self.nodes[anchor].parent.expect("attached mutation anchor");
                    let position = self.nodes[parent]
                        .children
                        .iter()
                        .position(|i| *i == anchor)
                        .unwrap();
                    let replace = matches!(edit, Mutation::ReplaceWith { .. });
                    let after = matches!(edit, Mutation::InsertAfter { .. });
                    for child in &children {
                        self.detach(*child);
                    }
                    let position = position + usize::from(after);
                    self.nodes[parent]
                        .children
                        .splice(position..position, children.iter().copied());
                    for child in children {
                        self.nodes[child].parent = Some(parent);
                    }
                    if replace {
                        self.detach(anchor);
                        self.stack.pop();
                    }
                }
                Mutation::SetAttribute { name, value, .. } => {
                    let top = self.top();
                    self.nodes[top].attrs.insert(name, value);
                }
                Mutation::SetText { value } => {
                    let top = self.top();
                    self.nodes[top].text = value;
                }
                Mutation::Remove => {
                    let node = self.stack.pop().unwrap();
                    self.detach(node);
                }
                Mutation::NewEventListener { .. } | Mutation::RemoveEventListener { .. } => {}
            }
        }
    }

    fn text(&self, index: usize) -> String {
        let node = &self.nodes[index];
        let mut text = node.text.clone();
        for child in &node.children {
            text.push_str(&self.text(*child));
        }
        text
    }

    fn matching(&self, predicate: impl Fn(&RenderedNode, &str) -> bool) -> Vec<usize> {
        fn visit(
            tree: &RenderedControls,
            index: usize,
            predicate: &impl Fn(&RenderedNode, &str) -> bool,
            found: &mut Vec<usize>,
        ) {
            if predicate(&tree.nodes[index], &tree.text(index)) {
                found.push(index);
            }
            for child in &tree.nodes[index].children {
                visit(tree, *child, predicate, found);
            }
        }
        let mut found = vec![];
        visit(self, 0, &predicate, &mut found);
        found
    }

    fn button(&self, label: &str, occurrence: usize) -> usize {
        *self
            .matching(|n, text| n.tag == "button" && text == label)
            .get(occurrence)
            .unwrap_or_else(|| panic!("missing button {label} #{occurrence}"))
    }

    fn field(&self, label: &str) -> usize {
        let found = self.matching(|n, _| {
            matches!(n.attrs.get("aria-label"), Some(AttributeValue::Text(value)) if value == label)
        });
        assert_eq!(found.len(), 1, "expected one rendered field {label}");
        found[0]
    }

    fn disabled(&self, index: usize) -> bool {
        matches!(
            self.nodes[index].attrs.get("disabled"),
            Some(AttributeValue::Bool(true))
        )
    }

    fn value(&self, label: &str) -> String {
        let index = self.field(label);
        match self.nodes[index]
            .attrs
            .get("value")
            .expect("rendered value attribute")
        {
            AttributeValue::Text(value) => value.clone(),
            value => panic!("unexpected value attribute {value:?}"),
        }
    }

    fn id(&self, index: usize) -> ElementId {
        self.ids
            .iter()
            .find_map(|(id, node)| (*node == index).then_some(*id))
            .expect("interactive rendered control has mounted element id")
    }
}

#[derive(Clone)]
struct AcceptedComponentState {
    detail: Option<DetailRecipe>,
    context: String,
}

#[derive(Clone)]
struct ComponentHarnessProps {
    accepted: Rc<RefCell<AcceptedComponentState>>,
    emitted: Rc<RefCell<Vec<Option<DetailRecipe>>>>,
}

fn texture_effect_component_app(props: ComponentHarnessProps) -> Element {
    let accepted = props.accepted.borrow().clone();
    rsx! {
        DetailControls {
            detail: accepted.detail,
            context_token: accepted.context,
            fresh: true,
            preview_uri: None,
            onapply: move |recipe| props.emitted.borrow_mut().push(recipe),
            oninspect: move |_| {},
        }
    }
}

fn texture_effect_component_accepted() -> PaintRecipe {
    PaintRecipe {
        version: 1,
        side: 64,
        base_sha256: "a".repeat(64),
        layers: vec![PaintLayer {
            name: "Retained imported paint".into(),
            visible: false,
            opacity: 0.4,
            strokes: vec![PaintStroke {
                size: 8.0,
                colour: [12, 34, 56, 255],
                opacity: 0.6,
                hardness: 0.8,
                points: vec![[3.0, 7.0], [8.0, 9.0]],
            }],
        
        ..crate::graphical::surfaces::detail::PaintLayer::default()
    }],
        effects: vec![],
    }
}

#[derive(Clone)]
struct ComponentFormValue(String);

impl dioxus::html::HasFileData for ComponentFormValue {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        vec![]
    }
}

impl dioxus::html::HasFormData for ComponentFormValue {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        vec![]
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Click handlers do not inspect coordinates, but the native event still carries
// the concrete mouse contract expected by Aether's HTML event converter.
struct ComponentMouse;

impl dioxus::html::InteractionLocation for ComponentMouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        dioxus::html::geometry::ClientPoint::new(0.0, 0.0)
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        dioxus::html::geometry::PagePoint::new(0.0, 0.0)
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        dioxus::html::geometry::ScreenPoint::new(0.0, 0.0)
    }
}

impl dioxus::html::InteractionElementOffset for ComponentMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        dioxus::html::geometry::ElementPoint::new(0.0, 0.0)
    }
}

impl dioxus::html::ModifiersInteraction for ComponentMouse {
    fn modifiers(&self) -> dioxus::html::Modifiers {
        dioxus::html::Modifiers::empty()
    }
}

impl dioxus::html::PointerInteraction for ComponentMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        Some(dioxus::html::input_data::MouseButton::Primary)
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        dioxus::html::input_data::MouseButtonSet::empty()
    }
}

impl dioxus::html::HasMouseData for ComponentMouse {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Unlike the framework's serialization-enabled test converter, this needs no
// feature changes. Each native event carries its concrete platform-independent
// data once; unexpected event conversion fails explicitly.
struct ComponentEventConverter;
macro_rules! component_event_converter {
    (enum Event {
        $(#[convert = $convert:ident]
          #[events = [$($event:tt)*]]
          $(#[raw = [$($raw:tt)*]])?
          $group:ident($data:ident),)*
    }) => {
        impl dioxus::html::HtmlEventConverter for ComponentEventConverter {
            $(fn $convert(&self, event: &PlatformEventData) -> dioxus::html::$data {
                event.downcast::<RefCell<Option<dioxus::html::$data>>>()
                    .expect("matching concrete native event payload")
                    .borrow_mut().take().expect("single targeted handler")
            })*
        }
    };
}
dioxus::html::with_html_event_groups!(component_event_converter);

struct ComponentHarness {
    dom: VirtualDom,
    rendered: RenderedControls,
    accepted: Rc<RefCell<AcceptedComponentState>>,
    emitted: Rc<RefCell<Vec<Option<DetailRecipe>>>>,
}

impl ComponentHarness {
    fn new() -> Self {
        set_event_converter(Box::new(ComponentEventConverter));
        let accepted = Rc::new(RefCell::new(AcceptedComponentState {
            detail: Some(DetailRecipe::Paint(texture_effect_component_accepted())),
            context: "session-1:part-A".into(),
        }));
        let emitted = Rc::new(RefCell::new(vec![]));
        let mut dom = VirtualDom::new_with_props(
            texture_effect_component_app,
            ComponentHarnessProps {
                accepted: accepted.clone(),
                emitted: emitted.clone(),
            },
        );
        let mut rendered = RenderedControls::new();
        rendered.apply(dom.rebuild_to_vec());
        Self {
            dom,
            rendered,
            accepted,
            emitted,
        }
    }

    fn render(&mut self) {
        self.rendered.apply(self.dom.render_immediate_to_vec());
    }

    fn dispatch<T: 'static>(&mut self, event: &str, target: usize, data: T) {
        let payload = PlatformEventData::new(Box::new(RefCell::new(Some(data))));
        self.dom.runtime().handle_event(
            event,
            Event::new(Rc::new(payload) as Rc<dyn Any>, true),
            self.rendered.id(target),
        );
        self.render();
    }

    fn change(&mut self, label: &str, value: &str) {
        let target = self.rendered.field(label);
        assert!(!self.rendered.disabled(target), "field is enabled: {label}");
        self.dispatch(
            "change",
            target,
            dioxus::html::FormData::new(ComponentFormValue(value.into())),
        );
    }

    fn input(&mut self, label: &str, value: &str) {
        let target = self.rendered.field(label);
        assert!(!self.rendered.disabled(target), "field is enabled: {label}");
        self.dispatch(
            "input",
            target,
            dioxus::html::FormData::new(ComponentFormValue(value.into())),
        );
    }

    fn click(&mut self, label: &str, occurrence: usize) {
        let target = self.rendered.button(label, occurrence);
        assert!(
            !self.rendered.disabled(target),
            "button is enabled: {label} #{occurrence}"
        );
        self.force_click(label, occurrence);
    }

    fn force_click(&mut self, label: &str, occurrence: usize) {
        let target = self.rendered.button(label, occurrence);
        self.dispatch(
            "click",
            target,
            dioxus::html::MouseData::new(ComponentMouse),
        );
    }

    fn set_accepted(&mut self, detail: DetailRecipe, context: &str) {
        *self.accepted.borrow_mut() = AcceptedComponentState {
            detail: Some(detail),
            context: context.into(),
        };
        self.dom.mark_dirty(ScopeId::APP);
        self.render();
    }

    fn emitted_paint(&self) -> PaintRecipe {
        let emitted = self.emitted.borrow();
        assert_eq!(
            emitted.len(),
            1,
            "exactly one actual component Apply emission"
        );
        match &emitted[0] {
            Some(DetailRecipe::Paint(p)) => p.clone(),
            other => panic!("expected emitted paint recipe, got {other:?}"),
        }
    }
}

const LOAD: &str = "Load accepted detail recipe";
const ADD: &str = "Add effect (paint version 2)";
const APPLY: &str = "Apply shared texture/detail recipe";

#[test]
fn texture_effect_component_load_edit_order_disable_apply_retains_imported_source() {
    let original = texture_effect_component_accepted();
    let mut ui = ComponentHarness::new();
    assert!(ui.rendered.disabled(ui.rendered.button(APPLY, 0)));
    ui.click(LOAD, 0);
    ui.click(ADD, 0);
    ui.change("Texture effect 0 operation", "sepia");
    ui.input("Texture effect 0 sepia intensity", "0.25");
    ui.change("Texture effect 0 enabled", "false");
    ui.change("Texture effect to add", "invert");
    ui.click(ADD, 0);
    ui.click("Move effect up", 1);
    assert_eq!(ui.rendered.value("Texture effect 0 operation"), "invert");
    assert_eq!(ui.rendered.value("Texture effect 1 operation"), "sepia");
    assert_eq!(
        ui.rendered.value("Texture effect 1 sepia intensity"),
        "0.25"
    );
    let enabled = ui.rendered.field("Texture effect 1 enabled");
    assert!(matches!(
        ui.rendered.nodes[enabled].attrs.get("checked"),
        Some(AttributeValue::Bool(false))
    ));
    assert!(
        ui.emitted.borrow().is_empty(),
        "local edits do not emit prematurely"
    );
    ui.click(APPLY, 0);
    let applied = ui.emitted_paint();
    assert_eq!(applied.version, 2);
    assert_eq!(applied.side, original.side);
    assert_eq!(applied.base_sha256, original.base_sha256);
    assert_eq!(applied.layers, original.layers);
    assert_eq!(
        applied.effects,
        vec![
            TextureEffectStep {
                enabled: true,
                effect: TextureEffect::Invert
            },
            TextureEffectStep {
                enabled: false,
                effect: TextureEffect::Sepia { intensity: 0.25 }
            },
        ]
    );
}

#[test]
fn texture_effect_component_invalid_numeric_text_is_retained_and_blocks_old_value_emission() {
    let mut ui = ComponentHarness::new();
    ui.click(LOAD, 0);
    ui.click(ADD, 0);
    ui.change("Texture effect 0 operation", "sepia");
    ui.input("Texture effect 0 sepia intensity", "0.25");
    for invalid in ["-", "NaN", "1.1"] {
        ui.input("Texture effect 0 sepia intensity", invalid);
        assert_eq!(
            ui.rendered.value("Texture effect 0 sepia intensity"),
            invalid
        );
        ui.click(APPLY, 0);
        assert!(
            ui.emitted.borrow().is_empty(),
            "invalid visible text must not emit the prior 0.25"
        );
    }
    ui.input("Texture effect 0 sepia intensity", "0.75");
    ui.input("Layer 0 opacity", "-");
    ui.click(APPLY, 0);
    assert_eq!(ui.rendered.value("Layer 0 opacity"), "-");
    assert!(ui.emitted.borrow().is_empty());
    ui.input("Layer 0 opacity", "0.4");
    ui.input("Diameter in pixels", "NaN");
    ui.click(APPLY, 0);
    assert_eq!(ui.rendered.value("Diameter in pixels"), "NaN");
    assert!(ui.emitted.borrow().is_empty());
    ui.input("Diameter in pixels", "8");
    ui.click(APPLY, 0);
    assert_eq!(
        ui.emitted_paint().effects[0].effect,
        TextureEffect::Sepia { intensity: 0.75 }
    );
}

#[test]
fn texture_effect_component_changed_accepted_source_or_context_preserves_draft_until_reload() {
    for source_changed in [true, false] {
        let mut ui = ComponentHarness::new();
        ui.click(LOAD, 0);
        ui.click(ADD, 0);
        ui.change("Texture effect 0 operation", "sepia");
        ui.input("Texture effect 0 sepia intensity", "0.37");
        let mut next = texture_effect_component_accepted();
        let context = if source_changed {
            next.layers[0].name = "New accepted source layer".into();
            "session-1:part-A"
        } else {
            "session-2:part-B"
        };
        ui.set_accepted(DetailRecipe::Paint(next.clone()), context);
        assert_eq!(
            ui.rendered.value("Texture effect 0 sepia intensity"),
            "0.37"
        );
        assert!(ui
            .rendered
            .disabled(ui.rendered.field("Texture effect 0 sepia intensity")));
        assert!(ui.rendered.disabled(ui.rendered.button(APPLY, 0)));
        assert!(ui
            .rendered
            .text(0)
            .contains("Your local draft is retained; reload"));
        // Runtime dispatch also exercises the component's explicit !ready guard.
        ui.force_click(APPLY, 0);
        assert!(ui.emitted.borrow().is_empty());
        ui.click(LOAD, 0);
        assert!(ui.rendered.matching(|node, _| {
            matches!(node.attrs.get("aria-label"), Some(AttributeValue::Text(value)) if value == "Texture effect 0 sepia intensity")
        }).is_empty(), "explicit reload replaces the retained effect draft");
        assert_eq!(ui.rendered.value("Layer 0 name"), next.layers[0].name);
        assert!(!ui.rendered.disabled(ui.rendered.button(APPLY, 0)));
        ui.click(APPLY, 0);
        assert_eq!(ui.emitted_paint(), next);
    }
}

#[path = "image_completed_component_tests.rs"]
mod image_completed_component_tests;
