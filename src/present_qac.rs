//! Self-contained .qac presentation card and paint-free inspection backed by `QualiaPortal.new_qac_reader()`.
//! Recognises `.qac` v1 (`qac\0\0\0\0\0`) vs legacy `Q4AU`/`UA4Q` without silent transcoding or alias inflation.
//! No `.q4au` fallback, no WebGPU canvas boot, and no implicit audio normalization/playback.
use crate::present::{base64_encode, escape_html};
use qualia_core_db::{
    audio::qac::{
        profiles::{AdmittedEnvelope, ProfileLimits},
        signal::{self, Sample},
    },
    q42::{
        field_envelope::{Family, StructuralView},
        field_semantics::{field, NodeState, Scratch},
    },
    render::portal_qac::{provenance_role_str, Limits as QacLimits, RetainedQac},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

pub const QAC_VIEWER_JS: &str = include_str!("present_qac.js");
pub const QAC_V1_MAGIC: &[u8; 8] = b"qac\0\0\0\0\0";
const MAX_INLINE_QAC_BYTES: usize = 8 * 1024 * 1024;
const DEFAULT_PREVIEW_RECORDS: u64 = 64;

#[derive(Clone, Debug, PartialEq)]
pub struct QacEntryInspection {
    pub index: u32,
    pub mode: u16,
    pub admitted: bool,
    pub profile_only: bool,
    pub supported_candidate: bool,
    pub profile_kind: String,
    pub scalar: String,
    pub channels: u32,
    pub bins: u32,
    pub records: u64,
    pub stored_bytes: u64,
    pub decoded_bytes: u64,
    pub start_tick: i64,
    pub rate_num: u64,
    pub rate_den: u64,
    pub origin_basis: String,
    pub window_kind: String,
    pub quantity: String,
    pub units: String,
    pub relation: String,
    pub purpose: String,
    pub epistemic: String,
    pub blake3_hex: String,
    pub derivative_label: String,
    pub derivative_values: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QacInspectionReport {
    pub is_v1_magic: bool,
    pub is_legacy_q4au: bool,
    pub admitted: bool,
    pub entry_count: u32,
    pub block_count: usize,
    pub index_digest_hex: String,
    pub sha256_hex: String,
    pub diagnostic: String,
    pub entries: Vec<QacEntryInspection>,
}

fn hex_lower(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}

fn entry_epistemic_role(view: &StructuralView<'_>, descriptor_id: u32) -> &'static str {
    let Some(desc) = view.blocks().find(|b| b.id() == descriptor_id) else { return "unknown"; };
    let Some(prov_id) = field(desc, 3).and_then(|f| f.value().try_into().ok()).map(u32::from_le_bytes) else { return "unknown"; };
    let Some(prov) = view.blocks().find(|b| b.id() == prov_id) else { return "unknown"; };
    field(prov, 1).and_then(|f| f.value().try_into().ok()).map(u32::from_le_bytes).and_then(provenance_role_str).unwrap_or("unknown")
}

fn sample_to_f64(sample: Sample) -> Option<f64> {
    match sample {
        Sample::U8(v) => Some(f64::from(v)),
        Sample::I16(v) => Some(f64::from(v)),
        Sample::I24(v) | Sample::I32(v) => Some(f64::from(v)),
        Sample::F32(v) => Some(f64::from(v)),
        Sample::F64(v) => Some(v),
        Sample::ComplexF32 { real, imag } => Some(f64::from(real).hypot(f64::from(imag))),
    }
}

fn extract_entry_derivative(bytes: &[u8], index: u32, limits: QacLimits) -> (String, Vec<f64>) {
    let Ok(retained) = RetainedQac::open(bytes.to_vec().into_boxed_slice(), index, limits) else { return (String::new(), Vec::new()); };
    let meta = retained.metadata();
    let count = meta.records.min(DEFAULT_PREVIEW_RECORDS);
    if meta.mode == 0x0100 {
        let Ok(win) = retained.pcm_window(0, count, 0) else { return (String::new(), Vec::new()); };
        let vals: Vec<f64> = (0..count).filter_map(|r| win.sample(r).ok().and_then(sample_to_f64)).collect();
        let label = format!("Visual Derivative · Stored PCM Samples (mode 0x0100 · ch=0 · scalar={:?} · units={} · unnormalized)", meta.scalar, meta.units);
        (label, vals)
    } else if meta.mode == 0x0101 {
        let frames = meta.records.min(1);
        let Ok(win) = retained.stft_window(0, frames, 0) else { return (String::new(), Vec::new()); };
        let mags: Vec<f64> = if frames > 0 { (0..win.bins()).filter_map(|b| win.sample(0, b).ok().and_then(sample_to_f64)).collect() } else { Vec::new() };
        let (wk, ob) = meta.transform.map(|t| (t.window.as_str(), t.origin_basis.as_str())).unwrap_or(("none", "epoch-sample-grid"));
        let label = format!("Visual Derivative · STFT Bin Magnitude |X[k]| = hypot(re, im) (mode 0x0101 · ch=0 · window_kind={wk} · origin_basis={ob})");
        (label, mags)
    } else {
        (String::new(), Vec::new())
    }
}

/// Paint-free structural and typed inspection of a `.qac` v1 container or refusal of legacy `Q4AU`/`UA4Q`.
pub fn inspect_qac_bytes(bytes: &[u8]) -> QacInspectionReport {
    let sha256_hex = hex_lower(&Sha256::digest(bytes));
    if bytes.starts_with(b"Q4AU") || bytes.starts_with(b"UA4Q") {
        return QacInspectionReport {
            is_v1_magic: false, is_legacy_q4au: true, admitted: false, entry_count: 0, block_count: 0,
            index_digest_hex: String::new(), sha256_hex,
            diagnostic: "Legacy Q4AU/UA4Q container refused: current .qac v1 magic qac\\0\\0\\0\\0\\0 required (no silent transcoding or alias inflation).".into(),
            entries: Vec::new(),
        };
    }
    let is_v1_magic = bytes.starts_with(QAC_V1_MAGIC);
    let limits = QacLimits::default();
    let view = match StructuralView::open(bytes, Family::QacV1Candidate, limits.envelope) {
        Ok(v) => v,
        Err(err) => {
            return QacInspectionReport {
                is_v1_magic, is_legacy_q4au: false, admitted: false, entry_count: 0, block_count: 0,
                index_digest_hex: String::new(), sha256_hex,
                diagnostic: format!("QAC structural inspection failed closed: {err:?}"),
                entries: Vec::new(),
            };
        }
    };
    let blocks = view.blocks().count();
    let index_digest_hex = hex_lower(view.index_digest());
    let mut metadata_scratch = vec![NodeState::default(); blocks];
    let mut entry_scratch = vec![NodeState::default(); view.entry_count() as usize];
    let mut entries = Vec::with_capacity(view.entry_count() as usize);
    for index in 1..=view.entry_count() {
        let Ok(e) = view.entry(index) else { continue };
        let epistemic = entry_epistemic_role(&view, e.descriptor_id()).to_string();
        let supported_candidate = matches!(e.mode(), 0x0100 | 0x0101);
        let mut item = QacEntryInspection {
            index, mode: e.mode(), admitted: false, profile_only: false, supported_candidate,
            profile_kind: "unclassified".into(), scalar: "Opaque".into(), channels: 0, bins: 0,
            records: e.record_count(), stored_bytes: e.stored_bytes(), decoded_bytes: e.decoded_bytes(),
            start_tick: e.start_tick(), rate_num: 0, rate_den: 1, origin_basis: "none".into(), window_kind: "none".into(),
            quantity: String::new(), units: String::new(), relation: String::new(), purpose: String::new(),
            epistemic, blake3_hex: hex_lower(e.digest()), derivative_label: String::new(), derivative_values: Vec::new(),
        };
        if matches!(e.mode(), 0x0100 | 0x0101 | 0x0104) {
            metadata_scratch.fill(NodeState::default());
            entry_scratch.fill(NodeState::default());
            if let Ok(sig) = signal::admit(
                &view, index,
                &mut Scratch { metadata: &mut metadata_scratch, entries: &mut entry_scratch },
                limits.shared, limits.signal,
            ) {
                item.admitted = true;
                item.profile_kind = match e.mode() { 0x0100 => "pcm-preservation".into(), 0x0101 => "stft-preservation".into(), _ => "opaque-source".into() };
                item.scalar = format!("{:?}", sig.scalar());
                item.channels = sig.channels();
                item.bins = sig.bins();
                item.quantity = sig.quantity().into();
                item.units = sig.units().into();
                item.relation = sig.relation().into();
                item.purpose = sig.purpose().into();
                if let Some(tm) = sig.timing() {
                    item.rate_num = tm.samples.numerator;
                    item.rate_den = tm.samples.denominator;
                }
                if let Some(t) = sig.transform() {
                    item.origin_basis = t.origin_basis.as_str().into();
                    item.window_kind = t.window.as_str().into();
                } else if e.mode() == 0x0100 {
                    item.origin_basis = "epoch-sample-grid".into();
                    item.window_kind = "none".into();
                }
                if supported_candidate {
                    let (label, vals) = extract_entry_derivative(bytes, index, limits);
                    item.derivative_label = label;
                    item.derivative_values = vals;
                } else {
                    item.derivative_label = "Opaque source entry (mode 0x0104): exact bytes preserved without synthetic waveform".into();
                }
            }
        }
        if !item.admitted {
            let prof_limits = ProfileLimits { envelope: limits.envelope, semantics: limits.shared, ..ProfileLimits::default() };
            if let Ok(env) = AdmittedEnvelope::open_alloc(bytes, prof_limits) {
                if let Ok(summary) = env.inspect_entry_summary(index) {
                    item.admitted = true;
                    item.profile_only = true;
                    item.profile_kind = summary.kind.as_str().into();
                    item.quantity = summary.quantity.into();
                    item.units = summary.units.into();
                    item.purpose = summary.purpose.into();
                    item.channels = summary.channels.unwrap_or(0);
                    item.origin_basis = "domain-profile".into();
                    item.window_kind = "none".into();
                    item.derivative_label = "Domain profile entry: non-waveform inspection only (no implicit audio synthesis)".into();
                }
            }
        }
        entries.push(item);
    }
    let admitted = entries.iter().any(|e| e.admitted);
    let diagnostic = format!("Admitted .qac v1 container ({} entries, {} metadata blocks, paint-free inspection)", view.entry_count(), blocks);
    QacInspectionReport {
        is_v1_magic, is_legacy_q4au: false, admitted, entry_count: view.entry_count(), block_count: blocks,
        index_digest_hex, sha256_hex, diagnostic, entries,
    }
}

pub fn qac_summary_json(bytes: &[u8]) -> Value {
    let r = inspect_qac_bytes(bytes);
    let entries: Vec<Value> = r.entries.iter().map(|e| json!({
        "index": e.index, "mode": e.mode, "admitted": e.admitted, "profile_kind": e.profile_kind,
        "scalar": e.scalar, "channels": e.channels, "bins": e.bins, "records": e.records,
        "rate_num": e.rate_num, "rate_den": e.rate_den, "origin_basis": e.origin_basis,
        "window_kind": e.window_kind, "quantity": e.quantity, "units": e.units,
        "epistemic": e.epistemic, "blake3_hex": e.blake3_hex,
    })).collect();
    json!({
        "is_v1_magic": r.is_v1_magic, "is_legacy_q4au": r.is_legacy_q4au, "admitted": r.admitted,
        "entry_count": r.entry_count, "block_count": r.block_count, "diagnostic": r.diagnostic, "entries": entries,
    })
}

fn render_static_svg(entry: &QacEntryInspection) -> String {
    if entry.derivative_values.is_empty() {
        return format!(r#"<div class="text-[11px] font-mono text-amber-300">{}</div>"#, escape_html(&entry.derivative_label));
    }
    let vals = &entry.derivative_values;
    let mut min_v = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let max_v = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if entry.mode == 0x0101 { min_v = 0.0; }
    let span = if (max_v - min_v).abs() > 1e-12 { max_v - min_v } else { 1.0 };
    let mut pts = String::new();
    for (idx, &v) in vals.iter().enumerate() {
        let x = 28.0 + if vals.len() > 1 { (idx as f64 / (vals.len() - 1) as f64) * 504.0 } else { 252.0 };
        let y = 18.0 + 84.0 - ((v - min_v) / span) * 84.0;
        let _ = write!(&mut pts, "{x:.1},{y:.1} ");
    }
    format!(
        r##"<div class="space-y-1"><div class="flex flex-wrap items-center justify-between text-[11px] font-mono text-emerald-300"><span>{label}</span><span class="text-[var(--muted-foreground)]">range=[{min_v:.4}, {max_v:.4}] · n={n}</span></div><svg viewBox="0 0 560 120" class="w-full h-[120px] rounded bg-[#070d0c] border border-[var(--border)]"><line x1="28" y1="60" x2="532" y2="60" stroke="rgba(148,163,184,0.22)" stroke-dasharray="3,3" /><polyline fill="none" stroke="#34d399" stroke-width="1.75" points="{pts}" /></svg></div>"##,
        label = escape_html(&entry.derivative_label), n = vals.len()
    )
}

pub fn render_session_qac_card(index: usize, safe_key: &str, bytes: &[u8]) -> String {
    let b64 = if bytes.len() <= MAX_INLINE_QAC_BYTES { base64_encode(bytes) } else { String::new() };
    let report = inspect_qac_bytes(bytes);
    let dl = if b64.is_empty() { String::new() } else {
        format!(r#"<a class="inline-block text-xs font-mono text-emerald-300 underline" href="data:application/octet-stream;base64,{b64}" download="{safe_key}">Download exact .qac ({len} B)</a>"#, len = bytes.len())
    };
    render_qac_host_html(&format!("qac-host-{index}"), safe_key, "", &b64, bytes.len() as u64, Some(&report), &dl)
}

pub fn render_project_qac_card(lane: &str, rel: &str, href: &str, bytes_len: u64) -> String {
    let host_id = format!("qac-proj-{}", escape_html(&format!("{lane}-{rel}")));
    let safe_href = escape_html(href);
    let dl = format!(r#"<a class="inline-block text-xs font-mono text-emerald-300 underline" href="{safe_href}" download>Download .qac ({bytes_len} B)</a>"#);
    render_qac_host_html(&host_id, &escape_html(rel), &safe_href, "", bytes_len, None, &dl)
}

fn render_qac_host_html(
    host_id: &str, escaped_key: &str, escaped_url: &str, b64: &str, bytes_len: u64,
    report: Option<&QacInspectionReport>, footer_html: &str,
) -> String {
    let url_attr = if escaped_url.is_empty() { String::new() } else { format!(r#" data-qac-url="{escaped_url}""#) };
    let b64_attr = if b64.is_empty() { String::new() } else { format!(r#" data-qac-b64="{b64}""#) };
    let mut entry_opts = String::new();
    let mut ch_opts = String::from(r#"<option value="0">ch 0</option>"#);
    let mut status_text = String::from("Waiting for QualiaPortal WASM reader on http origin…");
    let mut static_attr = "";
    let mut meta_text = String::from("No admitted metadata yet.");
    let mut win_text = String::from("No bounded sample/spectrum window yet.");
    let mut chart_html = String::new();
    if let Some(rep) = report {
        status_text = escape_html(&rep.diagnostic);
        if rep.admitted || rep.is_legacy_q4au { static_attr = r#" data-static-admitted="1""#; }
        for e in &rep.entries {
            let _ = write!(
                &mut entry_opts,
                r#"<option value="{idx}">Entry {idx} (0x{mode:04x} [{kind}] · {ep}) · {qty} ({units}) · {rec} rec</option>"#,
                idx = e.index, mode = e.mode, kind = escape_html(&e.profile_kind),
                ep = escape_html(&e.epistemic), qty = escape_html(&e.quantity), units = escape_html(&e.units), rec = e.records
            );
        }
        if let Some(f) = rep.entries.iter().find(|e| e.supported_candidate).or_else(|| rep.entries.first()) {
            if f.channels > 1 {
                ch_opts.clear();
                for c in 0..f.channels { let _ = write!(&mut ch_opts, r#"<option value="{c}">ch {c}</option>"#); }
            }
            let mut m = format!("container: magic=\"qac\\0\\0\\0\\0\\0\" (v1) entries={} blocks={} index_blake3={}\n", rep.entry_count, rep.block_count, rep.index_digest_hex);
            for e in &rep.entries {
                let _ = writeln!(
                    &mut m,
                    "entry={} mode=0x{:04x} kind={} scalar={} epistemic={} rate={}/{} origin_basis={} window_kind={} quantity={} units={} relation={} purpose={} channels={} bins={} records={}",
                    e.index, e.mode, e.profile_kind, e.scalar, e.epistemic, e.rate_num, e.rate_den, e.origin_basis, e.window_kind, e.quantity, e.units, e.relation, e.purpose, e.channels, e.bins, e.records
                );
            }
            meta_text = escape_html(m.trim_end());
            let preview_vals: Vec<String> = f.derivative_values.iter().take(12).map(|v| format!("{v:.6}")).collect();
            win_text = escape_html(&format!(
                "entry={} mode=0x{:04x} scalar={} start=0 records={} ch=0/{} bins={} values=[{}]",
                f.index, f.mode, f.scalar, f.records.min(DEFAULT_PREVIEW_RECORDS), f.channels, f.bins, preview_vals.join(", ")
            ));
            chart_html = render_static_svg(f);
        } else {
            meta_text = escape_html(&rep.diagnostic);
        }
    }
    format!(
        r#"<div class="synth-qac-host p-4 rounded-lg bg-[var(--background)] border border-emerald-500/40 space-y-2.5" id="{host_id}" data-synth-qac="1" data-qac-key="{escaped_key}"{url_attr}{b64_attr}><div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono"><span class="text-emerald-300 font-semibold">Qualia Acoustic Container (.qac) · {bytes_len} B</span><span class="text-[var(--muted-foreground)]">QualiaPortal.new_qac_reader() · paint-free · no implicit playback</span></div><div class="synth-qac-status text-[11px] font-mono text-[var(--muted-foreground)]"{static_attr}>{status_text}</div><div class="flex flex-wrap items-center gap-2 text-xs font-mono"><label class="text-[var(--muted-foreground)]">Entry:</label><select class="synth-qac-entry-select px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-xs font-mono">{entry_opts}</select><label class="text-[var(--muted-foreground)]">Channel:</label><select class="synth-qac-channel-select px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-xs font-mono">{ch_opts}</select><label class="text-[var(--muted-foreground)]">Start:</label><input type="text" value="0" class="synth-qac-start-input w-16 px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-xs font-mono" /><label class="text-[var(--muted-foreground)]">Records:</label><input type="number" value="32" min="1" max="256" class="synth-qac-records-input w-16 px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-xs font-mono" /><button type="button" class="synth-qac-apply-btn px-2.5 py-1 rounded bg-emerald-950/60 border border-emerald-500/50 text-emerald-200 text-xs font-mono">Inspect Window</button></div><div class="synth-qac-chart p-2.5 rounded bg-[var(--card)] border border-[var(--border)]">{chart_html}</div><pre class="synth-qac-meta p-2.5 rounded bg-[var(--card)] border border-[var(--border)] text-[11px] font-mono overflow-x-auto">{meta_text}</pre><pre class="synth-qac-window p-2.5 rounded bg-[var(--card)] border border-[var(--border)] text-[11px] font-mono overflow-x-auto">{win_text}</pre>{footer_html}</div>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{file_kind, ListedFile};
    use qualia_core_db::{
        audio::qac::signal::{
            stft::{StftDocument, StftQuantity, StftWriteBuffers, StftWriteOptions},
            writer::{PcmSamples, Provenance, ProvenanceRole, SignalRelation},
            Rate, Timing,
        },
        q42::field_semantics as sem,
    };

    #[test]
    fn qac_cards_embed_paint_free_reader_and_reject_q4au_alias() {
        assert_eq!(file_kind("signal.qac"), "qac");
        assert_eq!(file_kind("legacy.q4au"), "file");
        assert!(QAC_VIEWER_JS.contains("new_qac_reader") && QAC_VIEWER_JS.contains("inspect_qac"));
        assert!(QAC_VIEWER_JS.contains("Math.hypot") && !QAC_VIEWER_JS.contains("paint_frame"));
        for legacy in [b"Q4AU\x01\x00\x00\x00".as_slice(), b"UA4Q\x01\x00\x00\x00".as_slice()] {
            let rep = inspect_qac_bytes(legacy);
            assert!(rep.is_legacy_q4au && !rep.admitted);
            assert!(rep.diagnostic.contains("Legacy Q4AU/UA4Q container refused"));
            assert!(render_session_qac_card(0, "legacy.qac", legacy).contains("Legacy Q4AU/UA4Q container refused"));
        }
        let project_html = render_project_qac_card("working", "a/b.qac", "/project/file/working/a/b.qac", 42);
        assert!(project_html.contains(r#"data-qac-url="/project/file/working/a/b.qac""#) && project_html.contains("Download .qac (42 B)"));
        let lf = ListedFile { lane: "working".into(), rel: "tone.qac".into(), bytes: 16, kind: file_kind("tone.qac").into(), previewable: false };
        assert_eq!(lf.kind, "qac");
    }

    #[test]
    fn inspect_qac_bytes_surfaces_pcm_hann_stft_and_opaque_entries_deterministically() {
        let pcm = [8.0f32, -4.0, 2.0, -1.0, 0.5, -0.25];
        let doc = StftDocument {
            samples: PcmSamples::F32(&pcm),
            timing: Timing {
                scale: 2, epoch_seconds: 10, epoch_nanoseconds: 0,
                ticks: Rate::new(96_000, 1).unwrap(), samples: Rate::new(48_000, 1).unwrap(), start_tick: 0,
            },
            stream_id: 7, channels: 1, channel_order: "hydrophone-0", fft_size: 4, hop: 2,
            pcm_purpose: "calibrated-pressure-acquisition", stft_purpose: "hann-complex-stft",
            quantity: StftQuantity::Pressure, relation: SignalRelation::PhysicalBaseUnits,
            pcm_provenance: Provenance {
                role: ProvenanceRole::Measured, algorithm: None, algorithm_version: None,
                instrument_uri: Some("urn:instrument:synth-qac-test"), lineage: None,
            },
            stft_provenance: Provenance {
                role: ProvenanceRole::Derived, algorithm: Some("urn:etalon:signal:hann-stft-f32"),
                algorithm_version: Some("1"), instrument_uri: None, lineage: Some("symmetric-hann-radix2-no-padding"),
            },
            original: Some(b"RAW-HYDROPHONE-CAPTURE-BYTES"),
        };
        let (mut mono, mut fft, mut win) = ([0.0f32; 16], [0.0f32; 32], [0.0f32; 16]);
        let (mut spectra, mut metadata, mut payload, mut file) = ([0u8; 4096], [0u8; 8192], [0u8; 4096], [0u8; 16384]);
        let mut out = vec![0u8; 16384];
        let mut m = [sem::NodeState::default(); 32];
        let mut e = m;
        let n = doc.encode_into(
            StftWriteBuffers { mono: &mut mono, fft: &mut fft, window: &mut win, spectra: &mut spectra, metadata: &mut metadata, payload: &mut payload, file: &mut file },
            &mut sem::Scratch { metadata: &mut m, entries: &mut e },
            &mut out,
            StftWriteOptions::default(),
        ).unwrap();
        let qac_bytes = &out[..n];
        let rep = inspect_qac_bytes(qac_bytes);
        assert!(rep.is_v1_magic && !rep.is_legacy_q4au && rep.admitted && rep.entry_count == 3);
        assert_eq!((rep.entries[0].mode, rep.entries[0].epistemic.as_str()), (0x0104, "opaque-preserved"));
        assert_eq!((rep.entries[1].mode, rep.entries[1].epistemic.as_str(), rep.entries[1].units.as_str()), (0x0100, "measured", "Pa"));
        assert_eq!((rep.entries[1].rate_num, rep.entries[1].rate_den, rep.entries[1].derivative_values[0]), (48_000, 1, 8.0));
        assert_eq!((rep.entries[2].mode, rep.entries[2].epistemic.as_str(), rep.entries[2].scalar.as_str()), (0x0101, "derived", "ComplexF32"));
        assert_eq!(rep.entries[2].window_kind, "hann-symmetric-authored-f32");
        assert_eq!(rep.entries[2].origin_basis, "source-entry-first-sample");
        assert_eq!(rep.entries[2].derivative_values.len(), 4);
        let html_a = render_session_qac_card(1, "hydro.qac", qac_bytes);
        let html_b = render_session_qac_card(1, "hydro.qac", qac_bytes);
        assert_eq!(html_a, html_b);
        assert!(html_a.contains("hann-symmetric-authored-f32") && html_a.contains("source-entry-first-sample"));
        assert!(!html_a.contains("window_kind=rectangular") && html_a.contains(&base64_encode(qac_bytes)));
    }
}
