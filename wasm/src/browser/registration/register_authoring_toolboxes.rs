//! Registration of the 11 native authoring toolboxes and tool-chains into Poet's Tool Chest.
//!
//! Aligned with POET Implementation Plan and Playbook §18.3.

use super::*;

fn studio_tool(
    id: &'static str,
    label: &'static str,
    icon: &'static str,
    kind: ToolKind,
    description: &'static str,
) -> Box<dyn crate::tool_chest::core::tool::Tool> {
    let capability_scope = if kind == ToolKind::PlaceContainer {
        Some(SCOPE_PLACE.into())
    } else {
        None
    };
    Box::new(SimpleTool::new(
        ToolMetadata {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            kind,
            capability_scope,
            ontology_prefix: "authoring".into(),
            description: description.into(),
        },
        ActionType::Invoke,
    ))
}

pub(super) fn register_all_authoring_toolboxes(reg: &mut Registry) {
    register_grid_studio(reg);
    register_word_studio(reg);
    register_deck_studio(reg);
    register_pdf_studio(reg);
    register_vector_studio(reg);
    register_photo_studio(reg);
    register_light_studio(reg);
    register_cad_studio(reg);
    register_sound_studio(reg);
    register_film_studio(reg);
    register_art_studio(reg);
}

fn register_grid_studio(reg: &mut Registry) {
    let cells_chain = ToolChain::new(
        ToolChainMetadata {
            id: "grid_studio:cells".into(),
            label: "Cells & Ranges".into(),
            icon: "cells".into(),
            description: "Edit cell values, insert and delete rows and columns.".into(),
        },
        vec![
            studio_tool("grid_studio:set_cell", "Set Cell", "edit", ToolKind::RunAction, "Set cell value or formula"),
            studio_tool("grid_studio:clear_range", "Clear Range", "clear", ToolKind::RunAction, "Clear cell values in selection"),
            studio_tool("grid_studio:insert_row", "Insert Row", "plus", ToolKind::RunAction, "Insert row above active cell"),
            studio_tool("grid_studio:insert_col", "Insert Column", "plus", ToolKind::RunAction, "Insert column to left of active cell"),
            studio_tool("grid_studio:delete_row", "Delete Row", "minus", ToolKind::RunAction, "Delete selected rows"),
            studio_tool("grid_studio:delete_col", "Delete Column", "minus", ToolKind::RunAction, "Delete selected columns"),
        ],
    );

    let formulas_chain = ToolChain::new(
        ToolChainMetadata {
            id: "grid_studio:formulas".into(),
            label: "Formulas & Calculation".into(),
            icon: "formula".into(),
            description: "Formula evaluator, dependency tracing, and cycle detection.".into(),
        },
        vec![
            studio_tool("grid_studio:recalculate", "Recalculate", "calc", ToolKind::RunAction, "Trigger incremental sheet recalculation"),
            studio_tool("grid_studio:trace_precedents", "Trace Precedents", "arrow", ToolKind::Query, "Highlight cell formula dependencies"),
            studio_tool("grid_studio:trace_dependents", "Trace Dependents", "arrow", ToolKind::Query, "Highlight dependent formula cells"),
            studio_tool("grid_studio:detect_cycles", "Detect Cycles", "alert", ToolKind::Query, "Audit workbook for circular reference errors"),
        ],
    );

    let data_chain = ToolChain::new(
        ToolChainMetadata {
            id: "grid_studio:data".into(),
            label: "Data & Interop".into(),
            icon: "data".into(),
            description: "Sort, filter, and import/export OpenXML XLSX spreadsheets.".into(),
        },
        vec![
            studio_tool("grid_studio:sort_asc", "Sort A-Z", "sort", ToolKind::RunAction, "Sort selection ascending"),
            studio_tool("grid_studio:sort_desc", "Sort Z-A", "sort", ToolKind::RunAction, "Sort selection descending"),
            studio_tool("grid_studio:filter_range", "Filter", "filter", ToolKind::Toggle, "Toggle auto-filter on active table"),
            studio_tool("grid_studio:import_xlsx", "Import XLSX", "import", ToolKind::RunAction, "Import Microsoft Excel workbook"),
            studio_tool("grid_studio:export_xlsx", "Export XLSX", "export", ToolKind::RunAction, "Export workbook to OpenXML XLSX"),
        ],
    );

    let styles_chain = ToolChain::new(
        ToolChainMetadata {
            id: "grid_studio:styles".into(),
            label: "Cell Formats".into(),
            icon: "style".into(),
            description: "Format currency, percentage, dates, and typography.".into(),
        },
        vec![
            studio_tool("grid_studio:fmt_currency", "Currency", "currency", ToolKind::RunAction, "Format selection as currency"),
            studio_tool("grid_studio:fmt_percent", "Percentage", "percent", ToolKind::RunAction, "Format selection as percentage"),
            studio_tool("grid_studio:fmt_date", "Date", "calendar", ToolKind::RunAction, "Format selection with 1900/1904 date format"),
            studio_tool("grid_studio:bold", "Bold", "bold", ToolKind::RunAction, "Toggle bold text in selected cells"),
            studio_tool("grid_studio:align_center", "Center", "align", ToolKind::RunAction, "Center-align text in selected cells"),
        ],
    );

    let charts_chain = ToolChain::new(
        ToolChainMetadata {
            id: "grid_studio:charts".into(),
            label: "Charts & Pivot".into(),
            icon: "chart".into(),
            description: "Visualise data series with charts and interactive pivots.".into(),
        },
        vec![
            studio_tool("grid_studio:chart_bar", "Bar Chart", "chart", ToolKind::PlaceContainer, "Insert bar chart from selection"),
            studio_tool("grid_studio:chart_line", "Line Chart", "chart", ToolKind::PlaceContainer, "Insert line chart from selection"),
            studio_tool("grid_studio:chart_pie", "Pie Chart", "chart", ToolKind::PlaceContainer, "Insert pie chart from selection"),
            studio_tool("grid_studio:pivot_table", "Pivot Table", "table", ToolKind::PlaceContainer, "Create interactive pivot summary"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "grid_studio".into(),
            label: "Grid · Spreadsheet".into(),
            icon: "sheet".into(),
            ontology_prefix: "authoring".into(),
            description: "High-performance spreadsheet calculation engine with XLSX interop.".into(),
            enabled_by_default: true,
            family: "sheet".into(),
        },
        vec![cells_chain, formulas_chain, data_chain, styles_chain, charts_chain],
    ));
}

fn register_word_studio(reg: &mut Registry) {
    let typography_chain = ToolChain::new(
        ToolChainMetadata {
            id: "word_studio:typography".into(),
            label: "Typography".into(),
            icon: "font".into(),
            description: "Typefaces, weights, inline decorations, and unicode shaping.".into(),
        },
        vec![
            studio_tool("word_studio:bold", "Bold", "bold", ToolKind::RunAction, "Apply bold formatting"),
            studio_tool("word_studio:italic", "Italic", "italic", ToolKind::RunAction, "Apply italic formatting"),
            studio_tool("word_studio:underline", "Underline", "underline", ToolKind::RunAction, "Apply underline"),
            studio_tool("word_studio:strikethrough", "Strikethrough", "strike", ToolKind::RunAction, "Apply strikethrough"),
            studio_tool("word_studio:code", "Inline Code", "code", ToolKind::RunAction, "Format as monospace code"),
        ],
    );

    let paragraphs_chain = ToolChain::new(
        ToolChainMetadata {
            id: "word_studio:paragraphs".into(),
            label: "Paragraphs & Headings".into(),
            icon: "paragraph".into(),
            description: "Hierarchical headings, lists, quotes, and alignments.".into(),
        },
        vec![
            studio_tool("word_studio:h1", "Heading 1", "h1", ToolKind::RunAction, "Format as Heading 1"),
            studio_tool("word_studio:h2", "Heading 2", "h2", ToolKind::RunAction, "Format as Heading 2"),
            studio_tool("word_studio:bullet_list", "Bullet List", "list", ToolKind::RunAction, "Format as bulleted list"),
            studio_tool("word_studio:num_list", "Numbered List", "list", ToolKind::RunAction, "Format as numbered list"),
            studio_tool("word_studio:blockquote", "Blockquote", "quote", ToolKind::RunAction, "Format as callout blockquote"),
        ],
    );

    let styles_chain = ToolChain::new(
        ToolChainMetadata {
            id: "word_studio:styles".into(),
            label: "Page Layout & Flow".into(),
            icon: "layout".into(),
            description: "Margins, columns, page breaks, and document themes.".into(),
        },
        vec![
            studio_tool("word_studio:page_break", "Page Break", "break", ToolKind::RunAction, "Insert page break"),
            studio_tool("word_studio:columns", "Columns", "columns", ToolKind::RunAction, "Configure multi-column flow"),
            studio_tool("word_studio:margins", "Margins", "margins", ToolKind::RunAction, "Adjust paginated margins"),
        ],
    );

    let tables_chain = ToolChain::new(
        ToolChainMetadata {
            id: "word_studio:tables".into(),
            label: "Tables".into(),
            icon: "table".into(),
            description: "Insert paginated tables with automatic headers.".into(),
        },
        vec![
            studio_tool("word_studio:insert_table", "Insert Table", "table", ToolKind::RunAction, "Insert table"),
            studio_tool("word_studio:add_row", "Add Row", "plus", ToolKind::RunAction, "Add row below"),
            studio_tool("word_studio:add_col", "Add Column", "plus", ToolKind::RunAction, "Add column right"),
        ],
    );

    let references_chain = ToolChain::new(
        ToolChainMetadata {
            id: "word_studio:references".into(),
            label: "Citations & References".into(),
            icon: "book".into(),
            description: "Footnotes, bibliography, and Zotero interop.".into(),
        },
        vec![
            studio_tool("word_studio:citation", "Insert Citation", "quote", ToolKind::RunAction, "Insert bibliographic citation"),
            studio_tool("word_studio:footnote", "Footnote", "note", ToolKind::RunAction, "Insert numbered footnote"),
            studio_tool("word_studio:zotero", "Zotero Sync", "sync", ToolKind::Query, "Sync library with Zotero collections"),
        ],
    );

    let review_chain = ToolChain::new(
        ToolChainMetadata {
            id: "word_studio:review".into(),
            label: "Review & Track Changes".into(),
            icon: "check".into(),
            description: "Track edits, inline comments, and CML anchors.".into(),
        },
        vec![
            studio_tool("word_studio:track_changes", "Track Changes", "track", ToolKind::Toggle, "Toggle tracking of document revisions"),
            studio_tool("word_studio:comment", "Add Comment", "comment", ToolKind::RunAction, "Add inline comment at selection"),
            studio_tool("word_studio:cml_anchor", "CML Semantic Anchor", "anchor", ToolKind::RunAction, "Attach stable CML semantic anchor"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "word_studio".into(),
            label: "Document Studio · Flow Document".into(),
            icon: "doc".into(),
            ontology_prefix: "authoring".into(),
            description: "Structured paginated document processor with CML anchors and DOCX interop.".into(),
            enabled_by_default: true,
            family: "authoring".into(),
        },
        vec![typography_chain, paragraphs_chain, styles_chain, tables_chain, references_chain, review_chain],
    ));
}

fn register_deck_studio(reg: &mut Registry) {
    let layout_chain = ToolChain::new(
        ToolChainMetadata {
            id: "deck_studio:layout".into(),
            label: "Slides & Layout".into(),
            icon: "slide".into(),
            description: "Slide masters, templates, and slide sequences.".into(),
        },
        vec![
            studio_tool("deck_studio:new_slide", "New Slide", "plus", ToolKind::RunAction, "Create slide"),
            studio_tool("deck_studio:duplicate", "Duplicate", "copy", ToolKind::RunAction, "Duplicate slide"),
            studio_tool("deck_studio:slide_master", "Master Slide", "master", ToolKind::Toggle, "Edit theme master"),
        ],
    );

    let present_chain = ToolChain::new(
        ToolChainMetadata {
            id: "deck_studio:present".into(),
            label: "Presentation".into(),
            icon: "play".into(),
            description: "Presenter view and PPTX export.".into(),
        },
        vec![
            studio_tool("deck_studio:start_show", "Start Slideshow", "play", ToolKind::Navigate, "Launch presentation"),
            studio_tool("deck_studio:export_pptx", "Export PPTX", "export", ToolKind::RunAction, "Export slides to PPTX"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "deck_studio".into(),
            label: "Presentation Studio · Slides".into(),
            icon: "slide".into(),
            ontology_prefix: "authoring".into(),
            description: "Presentation deck authoring with PPTX round-trip and presenter views.".into(),
            enabled_by_default: true,
            family: "authoring".into(),
        },
        vec![layout_chain, present_chain],
    ));
}

fn register_pdf_studio(reg: &mut Registry) {
    let forms_chain = ToolChain::new(
        ToolChainMetadata {
            id: "pdf_studio:forms".into(),
            label: "PDF Forms & Redact".into(),
            icon: "form".into(),
            description: "AcroForms, redactions, and electronic signatures.".into(),
        },
        vec![
            studio_tool("pdf_studio:rotate", "Rotate Page", "rotate", ToolKind::RunAction, "Rotate page 90 degrees"),
            studio_tool("pdf_studio:redact", "Redact Region", "redact", ToolKind::RunAction, "Irreversibly redact content"),
            studio_tool("pdf_studio:sign", "Digital Signature", "sign", ToolKind::RunAction, "Apply cryptographic PDF signature"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "pdf_studio".into(),
            label: "PDF Studio · Forms & Intelligence".into(),
            icon: "pdf".into(),
            ontology_prefix: "authoring".into(),
            description: "PDF document manipulation, forms, redactions, and digital signatures.".into(),
            enabled_by_default: true,
            family: "authoring".into(),
        },
        vec![forms_chain],
    ));
}

fn register_vector_studio(reg: &mut Registry) {
    let paths_chain = ToolChain::new(
        ToolChainMetadata {
            id: "vector_studio:paths".into(),
            label: "Paths & Booleans".into(),
            icon: "pen".into(),
            description: "Vector paths, Bezier curves, and boolean operations.".into(),
        },
        vec![
            studio_tool("vector_studio:pen", "Pen Tool", "pen", ToolKind::RunAction, "Draw Bezier paths"),
            studio_tool("vector_studio:union", "Path Union", "union", ToolKind::RunAction, "Boolean union shapes"),
            studio_tool("vector_studio:subtract", "Path Difference", "subtract", ToolKind::RunAction, "Boolean subtract shapes"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "vector_studio".into(),
            label: "Vector Studio · Illustration".into(),
            icon: "vector".into(),
            ontology_prefix: "authoring".into(),
            description: "Precision 2D vector art and layout engine with SVG and PDF support.".into(),
            enabled_by_default: true,
            family: "graphics".into(),
        },
        vec![paths_chain],
    ));
}

fn register_photo_studio(reg: &mut Registry) {
    let raster_chain = ToolChain::new(
        ToolChainMetadata {
            id: "photo_studio:layers".into(),
            label: "Layers & Adjustments".into(),
            icon: "layer".into(),
            description: "Tiled raster image layers, masks, and adjustment filters.".into(),
        },
        vec![
            studio_tool("photo_studio:brush", "Paint Brush", "brush", ToolKind::RunAction, "Pressure-sensitive painting"),
            studio_tool("photo_studio:marquee", "Rect Selection", "select", ToolKind::RunAction, "Select rectangular region"),
            studio_tool("photo_studio:levels", "Color Levels", "levels", ToolKind::RunAction, "Adjust histogram levels"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "photo_studio".into(),
            label: "Photo Studio · Raster Imaging".into(),
            icon: "photo".into(),
            ontology_prefix: "authoring".into(),
            description: "Tiled multi-channel raster image editing with PSD/PSB preservation.".into(),
            enabled_by_default: true,
            family: "graphics".into(),
        },
        vec![raster_chain],
    ));
}

fn register_light_studio(reg: &mut Registry) {
    let raw_chain = ToolChain::new(
        ToolChainMetadata {
            id: "light_studio:raw".into(),
            label: "RAW Development".into(),
            icon: "raw".into(),
            description: "Non-destructive RAW camera processing and EXIF metadata.".into(),
        },
        vec![
            studio_tool("light_studio:exposure", "Exposure", "sun", ToolKind::RunAction, "Adjust RAW exposure"),
            studio_tool("light_studio:white_balance", "White Balance", "wb", ToolKind::RunAction, "Calibrate temperature and tint"),
            studio_tool("light_studio:export", "Export JPEG", "export", ToolKind::RunAction, "Export developed image"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "light_studio".into(),
            label: "Light Studio · Photography".into(),
            icon: "camera".into(),
            ontology_prefix: "authoring".into(),
            description: "Digital asset management and non-destructive RAW photo development.".into(),
            enabled_by_default: true,
            family: "graphics".into(),
        },
        vec![raw_chain],
    ));
}

fn register_cad_studio(reg: &mut Registry) {
    let cad_chain = ToolChain::new(
        ToolChainMetadata {
            id: "cad_studio:draw".into(),
            label: "Precision CAD".into(),
            icon: "ruler".into(),
            description: "f64 geometric entities, object snapping, and DXF interop.".into(),
        },
        vec![
            studio_tool("cad_studio:line", "Line (Exact)", "line", ToolKind::RunAction, "Draw precision coordinate line"),
            studio_tool("cad_studio:circle", "Circle", "circle", ToolKind::RunAction, "Draw circle by center and radius"),
            studio_tool("cad_studio:snap", "Toggle Snapping", "magnet", ToolKind::Toggle, "Toggle endpoint and midpoint snap"),
            studio_tool("cad_studio:dimension", "Dimension", "dimension", ToolKind::RunAction, "Add associative measurement dimension"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "cad_studio".into(),
            label: "CAD Studio · 2D Engineering".into(),
            icon: "cad".into(),
            ontology_prefix: "authoring".into(),
            description: "High-precision CAD drafting engine with geometric constraints and DXF support.".into(),
            enabled_by_default: true,
            family: "spatial".into(),
        },
        vec![cad_chain],
    ));
}

fn register_sound_studio(reg: &mut Registry) {
    let dsp_chain = ToolChain::new(
        ToolChainMetadata {
            id: "sound_studio:dsp".into(),
            label: "Audio DSP & Mix".into(),
            icon: "mixer".into(),
            description: "Sample-accurate timeline, routing, and real-time DSP effects.".into(),
        },
        vec![
            studio_tool("sound_studio:split", "Split Clip", "cut", ToolKind::RunAction, "Split audio clip at playhead"),
            studio_tool("sound_studio:eq", "Parametric EQ", "eq", ToolKind::RunAction, "Open 4-band parametric equalizer"),
            studio_tool("sound_studio:arm", "Arm Track", "record", ToolKind::Toggle, "Arm track for audio recording"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "sound_studio".into(),
            label: "Sound Studio · Multitrack DAW".into(),
            icon: "audio".into(),
            ontology_prefix: "authoring".into(),
            description: "Sample-exact multitrack audio recording, routing, and DSP processing.".into(),
            enabled_by_default: true,
            family: "audio".into(),
        },
        vec![dsp_chain],
    ));
}

fn register_film_studio(reg: &mut Registry) {
    let video_chain = ToolChain::new(
        ToolChainMetadata {
            id: "film_studio:timeline".into(),
            label: "Video Timeline & Grade".into(),
            icon: "film".into(),
            description: "Non-linear video editing, ripple trimming, and color grading.".into(),
        },
        vec![
            studio_tool("film_studio:ripple", "Ripple Trim", "trim", ToolKind::RunAction, "Ripple trim timeline edit"),
            studio_tool("film_studio:grade", "Color Grade", "color", ToolKind::RunAction, "Open primary color grade wheels"),
            studio_tool("film_studio:export", "Export Video", "export", ToolKind::RunAction, "Render project export"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "film_studio".into(),
            label: "Film Studio · Video Editor".into(),
            icon: "film".into(),
            ontology_prefix: "authoring".into(),
            description: "Non-linear multi-track video editing, grading, and WebCodecs export.".into(),
            enabled_by_default: true,
            family: "audio".into(),
        },
        vec![video_chain],
    ));
}

fn register_art_studio(reg: &mut Registry) {
    let scene_chain = ToolChain::new(
        ToolChainMetadata {
            id: "art_studio:scene".into(),
            label: "Scene & Posing".into(),
            icon: "actor".into(),
            description: "3D character poses, camera framing, and inpainting masks.".into(),
        },
        vec![
            studio_tool("art_studio:pose", "Pose Character", "pose", ToolKind::RunAction, "Adjust 3D character pose rig"),
            studio_tool("art_studio:camera", "Framing Camera", "camera", ToolKind::RunAction, "Adjust camera focal length and angle"),
            studio_tool("art_studio:mask", "Inpaint Mask", "brush", ToolKind::RunAction, "Paint spatial generative inpaint mask"),
            studio_tool("art_studio:generate", "Generate Draft", "sparkles", ToolKind::RunAction, "Dispatch diffusion generation task"),
        ],
    );

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "art_studio".into(),
            label: "Art Studio · Spatial Authoring".into(),
            icon: "sparkles".into(),
            ontology_prefix: "authoring".into(),
            description: "Spatial scene authoring, character posing, and generative image synthesis.".into(),
            enabled_by_default: true,
            family: "ai".into(),
        },
        vec![scene_chain],
    ));
}
