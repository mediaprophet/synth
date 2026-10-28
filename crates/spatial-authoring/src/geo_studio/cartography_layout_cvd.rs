//! Poet Cartography Studio: Publication Print Layout Composer, Thematic Choropleth
//! Classifiers (Fisher-Jenks Exact DP, Quantile, Equal-Interval, StdDev, Log, Bivariate 3x3),
//! and Brettel/Viénot Color-Vision-Deficiency (CVD) & WCAG 2.1 Contrast Auditor.
//!
//! Adapted from RustGIS (v0.4.0) under the MIT License.
//! Copyright (c) 2026 RustGIS Contributors & Poet Authors.

#![forbid(unsafe_code)]

use std::fmt::Write;

/// Errors arising during thematic classification and publication map composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartographyError {
    EmptyData,
    InvalidClassCount,
    ByteBudgetExceeded { requested: usize, budget: usize },
    InvalidData(&'static str),
}

impl std::fmt::Display for CartographyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyData => write!(f, "Classification dataset cannot be empty"),
            Self::InvalidClassCount => write!(f, "Class count k must be >= 1"),
            Self::ByteBudgetExceeded { requested, budget } => write!(f, "Budget exceeded: {requested} > {budget}"),
            Self::InvalidData(msg) => write!(f, "Invalid cartography data: {msg}"),
        }
    }
}
impl std::error::Error for CartographyError {}

/// Thematic classification algorithms for choropleth mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassificationMethod {
    EqualInterval,
    Quantile,
    StandardDeviation,
    Logarithmic,
    JenksNaturalBreaks,
}

/// Computes exact Fisher-Jenks natural breaks via 1D dynamic programming minimizing SSD.
/// Returns `k + 1` break boundaries `[min, b1, ..., bk-1, max]`.
pub fn classify_jenks_fisher_exact(
    values: &[f64],
    k: usize,
    byte_budget: usize,
) -> Result<Vec<f64>, CartographyError> {
    if values.is_empty() { return Err(CartographyError::EmptyData); }
    if k == 0 { return Err(CartographyError::InvalidClassCount); }
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.is_empty() { return Err(CartographyError::EmptyData); }
    let (n, lo, hi) = (sorted.len(), sorted[0], sorted[sorted.len() - 1]);
    if k == 1 || (hi - lo).abs() < 1e-12 { return Ok(vec![lo, hi]); }
    let k = k.min(n);
    let req_mem = (n + 1) * (k + 1) * 16 + n * 16;
    if req_mem > byte_budget {
        return Err(CartographyError::ByteBudgetExceeded { requested: req_mem, budget: byte_budget });
    }

    let (mut s1, mut s2) = (vec![0.0; n + 1], vec![0.0; n + 1]);
    for i in 1..=n {
        let v = sorted[i - 1];
        s1[i] = s1[i - 1] + v;
        s2[i] = s2[i - 1] + v * v;
    }
    let ssd = |m: usize, l: usize| -> f64 {
        let count = (l - m + 1) as f64;
        let sum = s1[l] - s1[m - 1];
        let sum_sq = s2[l] - s2[m - 1];
        (sum_sq - (sum * sum) / count).max(0.0)
    };

    let mut d = vec![vec![f64::INFINITY; k + 1]; n + 1];
    let mut p = vec![vec![0usize; k + 1]; n + 1];
    for l in 1..=n { d[l][1] = ssd(1, l); p[l][1] = 1; }
    for j in 2..=k {
        for l in j..=n {
            let (mut min_val, mut best_m) = (f64::INFINITY, j);
            for m in j..=l {
                let cost = d[m - 1][j - 1] + ssd(m, l);
                if cost < min_val { min_val = cost; best_m = m; }
            }
            d[l][j] = min_val;
            p[l][j] = best_m;
        }
    }

    let mut breaks = vec![0.0; k + 1];
    breaks[0] = lo;
    breaks[k] = hi;
    let mut curr_end = n;
    for j in (2..=k).rev() {
        let start_idx = p[curr_end][j];
        breaks[j - 1] = sorted[start_idx.saturating_sub(2)];
        curr_end = start_idx.saturating_sub(1).max(1);
    }
    Ok(breaks)
}

/// Dispatches thematic classification returning `k + 1` break boundaries `[min, b1, ..., bk-1, max]`.
pub fn classify_thematic_breaks(
    values: &[f64],
    k: usize,
    method: ClassificationMethod,
    byte_budget: usize,
) -> Result<Vec<f64>, CartographyError> {
    if values.is_empty() { return Err(CartographyError::EmptyData); }
    if k == 0 { return Err(CartographyError::InvalidClassCount); }
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.is_empty() { return Err(CartographyError::EmptyData); }
    let (n, lo, hi) = (sorted.len(), sorted[0], sorted[sorted.len() - 1]);
    let mem = (k + 1) * 8 + n * 8;
    if mem > byte_budget { return Err(CartographyError::ByteBudgetExceeded { requested: mem, budget: byte_budget }); }

    match method {
        ClassificationMethod::JenksNaturalBreaks => classify_jenks_fisher_exact(values, k, byte_budget),
        ClassificationMethod::EqualInterval => {
            let step = (hi - lo) / (k as f64);
            Ok((0..=k).map(|i| if i == k { hi } else { lo + (i as f64) * step }).collect())
        }
        ClassificationMethod::Quantile => Ok((0..=k).map(|i| {
            if i == 0 { lo } else if i == k { hi } else {
                sorted[(((n - 1) as f64 * (i as f64) / (k as f64)).round() as usize).min(n - 1)]
            }
        }).collect()),
        ClassificationMethod::StandardDeviation => {
            let mean = sorted.iter().sum::<f64>() / n as f64;
            let sd = (sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
            let mut breaks = vec![lo; k + 1];
            breaks[k] = hi;
            for i in 1..k {
                breaks[i] = (mean + ((i as f64) - (k as f64 / 2.0)) * sd).clamp(lo, hi);
                if breaks[i] < breaks[i - 1] { breaks[i] = breaks[i - 1]; }
            }
            Ok(breaks)
        }
        ClassificationMethod::Logarithmic => {
            let (log_lo, log_hi, off) = if lo > 0.0 { (lo.ln(), hi.ln(), 0.0) } else {
                let shift = 1.0 - lo;
                ((lo + shift).ln(), (hi + shift).ln(), shift)
            };
            Ok((0..=k).map(|i| {
                if i == 0 { lo } else if i == k { hi } else {
                    let frac = i as f64 / k as f64;
                    ((log_lo + (log_hi - log_lo) * frac).exp() - off).clamp(lo, hi)
                }
            }).collect())
        }
    }
}

/// Standard Stevens/Brewer 3x3 perceptual bivariate choropleth palette (9 classes).
const BIVARIATE_3X3_PALETTE: [[u8; 4]; 9] = [
    [0xe8, 0xe8, 0xe8, 0xff], [0xb0, 0xd5, 0xdf, 0xff], [0x64, 0xac, 0xbe, 0xff],
    [0xe4, 0xac, 0xb6, 0xff], [0xad, 0x9b, 0xaf, 0xff], [0x62, 0x7e, 0x96, 0xff],
    [0xc8, 0x5a, 0x5a, 0xff], [0x98, 0x51, 0x73, 0xff], [0x5a, 0x42, 0x64, 0xff],
];

/// Maps a bivariate `(val_x, val_y)` pair into class index `0..9` and perceptual RGBA8 color.
pub fn bivariate_choropleth_3x3_index(
    val_x: f64, breaks_x: &[f64; 4],
    val_y: f64, breaks_y: &[f64; 4],
) -> (u8, [u8; 4]) {
    let ix = if val_x <= breaks_x[1] { 0usize } else if val_x <= breaks_x[2] { 1usize } else { 2usize };
    let iy = if val_y <= breaks_y[1] { 0usize } else if val_y <= breaks_y[2] { 1usize } else { 2usize };
    let idx = (iy * 3 + ix) as u8;
    (idx, BIVARIATE_3X3_PALETTE[idx as usize])
}

/// Color-vision-deficiency impairment types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvdDeficiencyType {
    Protanopia,
    Deuteranopia,
    Tritanopia,
    Achromatopsia,
}

fn srgb_to_linear(c: u8) -> f64 {
    let v = c as f64 / 255.0;
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(v: f64) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Linearizes sRGB, applies Brettel/Machado projection, and maps back to sRGB.
pub fn simulate_cvd_rgba8(rgba: [u8; 4], kind: CvdDeficiencyType) -> [u8; 4] {
    let (r, g, b) = (srgb_to_linear(rgba[0]), srgb_to_linear(rgba[1]), srgb_to_linear(rgba[2]));
    let m: [[f64; 3]; 3] = match kind {
        CvdDeficiencyType::Protanopia => [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]],
        CvdDeficiencyType::Deuteranopia => [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.011820, 0.042940, 0.968881]],
        CvdDeficiencyType::Tritanopia => [[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.303900]],
        CvdDeficiencyType::Achromatopsia => [[0.2126, 0.7152, 0.0722]; 3],
    };
    let (r_s, g_s, b_s) = (m[0][0] * r + m[0][1] * g + m[0][2] * b, m[1][0] * r + m[1][1] * g + m[1][2] * b, m[2][0] * r + m[2][1] * g + m[2][2] * b);
    [linear_to_srgb(r_s), linear_to_srgb(g_s), linear_to_srgb(b_s), rgba[3]]
}

fn relative_luminance(rgba: [u8; 4]) -> f64 {
    0.2126 * srgb_to_linear(rgba[0]) + 0.7152 * srgb_to_linear(rgba[1]) + 0.0722 * srgb_to_linear(rgba[2])
}

/// Evaluates WCAG 2.1 relative luminance contrast ratio `(L_max + 0.05) / (L_min + 0.05)`.
pub fn wcag21_contrast_ratio(fg_rgba: [u8; 4], bg_rgba: [u8; 4]) -> f64 {
    let (l1, l2) = (relative_luminance(fg_rgba), relative_luminance(bg_rgba));
    let (max_l, min_l) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (max_l + 0.05) / (min_l + 0.05)
}

fn rgba_to_lab(rgba: [u8; 4]) -> [f64; 3] {
    let (r, g, b) = (srgb_to_linear(rgba[0]), srgb_to_linear(rgba[1]), srgb_to_linear(rgba[2]));
    let x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047;
    let y = (0.2126729 * r + 0.7151522 * g + 0.0721750 * b) / 1.00000;
    let z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883;
    let f = |t: f64| if t > 0.00885645 { t.cbrt() } else { 7.787037 * t + 16.0 / 116.0 };
    [(116.0 * f(y) - 16.0).max(0.0), 500.0 * (f(x) - f(y)), 200.0 * (f(y) - f(z))]
}

fn delta_e(c1: [u8; 4], c2: [u8; 4]) -> f64 {
    let ([l1, a1, b1], [l2, a2, b2]) = (rgba_to_lab(c1), rgba_to_lab(c2));
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

/// Palette accessibility and CVD safety audit findings.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteAccessibilityReport {
    pub min_pairwise_delta_e: f64,
    pub min_protan_delta_e: f64,
    pub min_deutan_delta_e: f64,
    pub min_tritan_delta_e: f64,
    pub wcag_aa_text_pass: bool,
    pub is_cvd_safe: bool,
}

/// Audits a cartographic palette for CVD distinguishability (Delta E >= 10.0) and WCAG AA contrast.
pub fn audit_cartographic_palette(palette: &[[u8; 4]], text_bg_rgba: [u8; 4]) -> PaletteAccessibilityReport {
    if palette.len() < 2 {
        let pass = palette.iter().all(|&c| wcag21_contrast_ratio(c, text_bg_rgba) >= 4.5);
        return PaletteAccessibilityReport {
            min_pairwise_delta_e: 100.0, min_protan_delta_e: 100.0, min_deutan_delta_e: 100.0,
            min_tritan_delta_e: 100.0, wcag_aa_text_pass: pass, is_cvd_safe: true,
        };
    }
    let (mut min_norm, mut min_prot, mut min_deut, mut min_trit) = (f64::INFINITY, f64::INFINITY, f64::INFINITY, f64::INFINITY);
    for i in 0..palette.len() {
        let (c1, p1, d1, t1) = (palette[i], simulate_cvd_rgba8(palette[i], CvdDeficiencyType::Protanopia), simulate_cvd_rgba8(palette[i], CvdDeficiencyType::Deuteranopia), simulate_cvd_rgba8(palette[i], CvdDeficiencyType::Tritanopia));
        for j in (i + 1)..palette.len() {
            let c2 = palette[j];
            min_norm = min_norm.min(delta_e(c1, c2));
            min_prot = min_prot.min(delta_e(p1, simulate_cvd_rgba8(c2, CvdDeficiencyType::Protanopia)));
            min_deut = min_deut.min(delta_e(d1, simulate_cvd_rgba8(c2, CvdDeficiencyType::Deuteranopia)));
            min_trit = min_trit.min(delta_e(t1, simulate_cvd_rgba8(c2, CvdDeficiencyType::Tritanopia)));
        }
    }
    let wcag_aa_text_pass = palette.iter().all(|&c| wcag21_contrast_ratio(c, text_bg_rgba) >= 4.5);
    let is_cvd_safe = min_prot >= 10.0 && min_deut >= 10.0 && min_trit >= 10.0;
    PaletteAccessibilityReport {
        min_pairwise_delta_e: min_norm, min_protan_delta_e: min_prot,
        min_deutan_delta_e: min_deut, min_tritan_delta_e: min_trit,
        wcag_aa_text_pass, is_cvd_safe,
    }
}

/// Print physical dimensions in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSizeMm {
    pub width_mm: f64,
    pub height_mm: f64,
    pub dpi: u32,
}

impl PageSizeMm {
    pub const A4_PORTRAIT: Self = Self { width_mm: 210.0, height_mm: 297.0, dpi: 300 };
    pub const A4_LANDSCAPE: Self = Self { width_mm: 297.0, height_mm: 210.0, dpi: 300 };
}

/// Alternating bar map scale specification.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleBarSpec {
    pub map_scale_denom: f64,
    pub divisions: usize,
    pub segment_ground_m: f64,
}

/// Publication print layout composition specifications.
#[derive(Debug, Clone, PartialEq)]
pub struct PrintLayoutSpec {
    pub page: PageSizeMm,
    pub title: String,
    pub subtitle_crs: String,
    pub author_attribution: String,
    pub map_bbox_proj: [f64; 4],
    pub graticule_step_m: f64,
    pub scale_bar: ScaleBarSpec,
    pub legend_items: Vec<(String, [u8; 4])>,
    pub include_north_arrow: bool,
    pub include_inset_map: bool,
}

/// Composes a publication-grade vector SVG map layout with neatline, graticule, scale, rose, and legend.
pub fn compose_publication_map_svg(spec: &PrintLayoutSpec, byte_budget: usize) -> Result<String, CartographyError> {
    if byte_budget < 256 {
        return Err(CartographyError::ByteBudgetExceeded { requested: 256, budget: byte_budget });
    }
    let mut svg = String::with_capacity(4096);
    let (w, h) = (spec.page.width_mm, spec.page.height_mm);
    write!(&mut svg, r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.2}mm" height="{:.2}mm"><rect width="100%" height="100%" fill="#ffffff"/>"##, w, h, w, h).map_err(|_| CartographyError::InvalidData("SVG header"))?;
    write!(&mut svg, r##"<g id="title-block"><text x="15" y="16" font-family="sans-serif" font-size="5" font-weight="bold" fill="#111827">{}</text><text x="15" y="21" font-family="sans-serif" font-size="2.8" fill="#4b5563">{}</text><text x="15" y="25" font-family="sans-serif" font-size="2.2" fill="#6b7280">Author: {}</text></g>"##, spec.title, spec.subtitle_crs, spec.author_attribution).map_err(|_| CartographyError::InvalidData("SVG title"))?;

    let leg_w = if spec.legend_items.is_empty() { 0.0 } else { 50.0 };
    let (map_x, map_y) = (15.0, 28.0);
    let (map_w, map_h) = ((w - 30.0 - leg_w).max(20.0), (h - 42.0).max(20.0));
    write!(&mut svg, r##"<g id="map-frame"><rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="#f8fafc" stroke="#111827" stroke-width="0.5"/>"##, map_x, map_y, map_w, map_h).map_err(|_| CartographyError::InvalidData("SVG frame"))?;

    let [x_min, y_min, x_max, y_max] = spec.map_bbox_proj;
    let (span_x, span_y) = ((x_max - x_min).max(1e-6), (y_max - y_min).max(1e-6));
    if spec.graticule_step_m > 0.0 {
        write!(&mut svg, r##"<g id="graticule" stroke="#cbd5e1" stroke-width="0.2" stroke-dasharray="1,1">"##).map_err(|_| CartographyError::InvalidData("SVG graticule"))?;
        let mut gx = (x_min / spec.graticule_step_m).ceil() * spec.graticule_step_m;
        while gx <= x_max {
            let px = map_x + ((gx - x_min) / span_x) * map_w;
            write!(&mut svg, r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="1.8" fill="#475569" text-anchor="middle">{:.0}m E</text>"##, px, map_y, px, map_y + map_h, px, map_y - 1.0, gx).map_err(|_| CartographyError::InvalidData("SVG tick"))?;
            gx += spec.graticule_step_m;
        }
        let mut gy = (y_min / spec.graticule_step_m).ceil() * spec.graticule_step_m;
        while gy <= y_max {
            let py = map_y + (1.0 - (gy - y_min) / span_y) * map_h;
            write!(&mut svg, r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="1.8" fill="#475569" text-anchor="end">{:.0}m N</text>"##, map_x, py, map_x + map_w, py, map_x - 1.0, py + 0.6, gy).map_err(|_| CartographyError::InvalidData("SVG tick"))?;
            gy += spec.graticule_step_m;
        }
        write!(&mut svg, r##"</g>"##).map_err(|_| CartographyError::InvalidData("SVG close"))?;
    }

    let div_k = spec.scale_bar.divisions.max(1);
    let seg_w = ((spec.scale_bar.segment_ground_m * 1000.0) / spec.scale_bar.map_scale_denom).clamp(4.0, map_w / (div_k as f64 + 1.0));
    let (sb_x, sb_y) = (map_x + 5.0, map_y + map_h - 10.0);
    write!(&mut svg, r##"<g id="scale-bar">"##).map_err(|_| CartographyError::InvalidData("SVG scale"))?;
    for d in 0..div_k {
        let fill = if d % 2 == 0 { "#111827" } else { "#ffffff" };
        let rx = sb_x + (d as f64) * seg_w;
        write!(&mut svg, r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="2.5" fill="{}" stroke="#111827" stroke-width="0.3"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="1.8" fill="#111827" text-anchor="middle">{:.0}m</text>"##, rx, sb_y, seg_w, fill, rx, sb_y - 0.8, (d as f64) * spec.scale_bar.segment_ground_m).map_err(|_| CartographyError::InvalidData("SVG bar"))?;
    }
    write!(&mut svg, r##"<text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="1.8" fill="#111827">1 : {:.0}</text></g>"##, sb_x, sb_y + 5.5, spec.scale_bar.map_scale_denom).map_err(|_| CartographyError::InvalidData("SVG ratio"))?;

    if spec.include_north_arrow {
        let (nx, ny) = (map_x + map_w - 12.0, map_y + 12.0);
        write!(&mut svg, r##"<g id="north-arrow"><polygon points="{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}" fill="#111827"/><polygon points="{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}" fill="#cbd5e1" stroke="#111827" stroke-width="0.2"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="3" font-weight="bold" fill="#111827" text-anchor="middle">N</text></g>"##, nx, ny - 6.0, nx, ny + 3.0, nx - 3.0, ny + 1.0, nx, ny - 6.0, nx + 3.0, ny + 1.0, nx, ny + 3.0, nx, ny - 7.0).map_err(|_| CartographyError::InvalidData("SVG rose"))?;
    }
    if spec.include_inset_map {
        let (ix, iy) = (map_x + map_w - 34.0, map_y + map_h - 24.0);
        write!(&mut svg, r##"<g id="inset-map"><rect x="{:.2}" y="{:.2}" width="30" height="20" fill="#f1f5f9" stroke="#334155" stroke-width="0.3"/><rect x="{:.2}" y="{:.2}" width="10" height="6" fill="none" stroke="#dc2626" stroke-width="0.35"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="1.8" fill="#475569">Overview</text></g>"##, ix, iy, ix + 10.0, iy + 7.0, ix + 2.0, iy + 3.0).map_err(|_| CartographyError::InvalidData("SVG inset"))?;
    }
    if !spec.legend_items.is_empty() {
        let (lx, ly) = (map_x + map_w + 4.0, map_y);
        let lh = ((spec.legend_items.len() as f64) * 5.0 + 8.0).min(map_h);
        write!(&mut svg, r##"<g id="legend"><rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="#ffffff" stroke="#94a3b8" stroke-width="0.3"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="2.6" font-weight="bold" fill="#111827">Legend</text>"##, lx, ly, leg_w - 6.0, lh, lx + 3.0, ly + 5.0).map_err(|_| CartographyError::InvalidData("SVG leg"))?;
        for (i, (lbl, c)) in spec.legend_items.iter().enumerate() {
            let sy = ly + 8.0 + (i as f64) * 4.5;
            if sy + 3.0 <= ly + lh {
                write!(&mut svg, r##"<rect x="{:.2}" y="{:.2}" width="4" height="3" fill="rgb({},{},{})" stroke="#000000" stroke-width="0.1"/><text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="2.0" fill="#374151">{}</text>"##, lx + 3.0, sy, c[0], c[1], c[2], lx + 8.5, sy + 2.2, lbl).map_err(|_| CartographyError::InvalidData("SVG item"))?;
            }
        }
        write!(&mut svg, r##"</g>"##).map_err(|_| CartographyError::InvalidData("SVG close"))?;
    }
    write!(&mut svg, r##"</g></svg>"##).map_err(|_| CartographyError::InvalidData("SVG end"))?;
    if svg.len() > byte_budget {
        return Err(CartographyError::ByteBudgetExceeded { requested: svg.len(), budget: byte_budget });
    }
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fisher_jenks_exact_and_thematic_classifiers() {
        let values = vec![1.0, 1.2, 1.5, 10.0, 10.5, 11.0, 50.0, 52.0, 55.0];
        let jenks_breaks = classify_jenks_fisher_exact(&values, 3, 65536).expect("Jenks DP breaks");
        assert_eq!(jenks_breaks.len(), 4);
        assert!((jenks_breaks[0] - 1.0).abs() < 1e-6 && (jenks_breaks[3] - 55.0).abs() < 1e-6);
        assert!(jenks_breaks[1] <= 1.5 && jenks_breaks[1] >= 1.0 && jenks_breaks[2] <= 11.0 && jenks_breaks[2] >= 10.0);

        let eq = classify_thematic_breaks(&values, 3, ClassificationMethod::EqualInterval, 65536).expect("eq");
        assert_eq!(eq.len(), 4);
        assert!((eq[0] - 1.0).abs() < 1e-6 && (eq[3] - 55.0).abs() < 1e-6);

        let quant = classify_thematic_breaks(&values, 3, ClassificationMethod::Quantile, 65536).expect("quant");
        assert_eq!(quant.len(), 4);

        let sd = classify_thematic_breaks(&values, 3, ClassificationMethod::StandardDeviation, 65536).expect("sd");
        assert_eq!(sd.len(), 4);

        let log = classify_thematic_breaks(&values, 3, ClassificationMethod::Logarithmic, 65536).expect("log");
        assert_eq!(log.len(), 4);

        let bx = [0.0, 10.0, 20.0, 30.0];
        let by = [0.0, 100.0, 200.0, 300.0];
        let (idx, color) = bivariate_choropleth_3x3_index(15.0, &bx, 250.0, &by);
        assert_eq!(idx, 7);
        assert_eq!(color, [0x98, 0x51, 0x73, 0xff]);
    }

    #[test]
    fn test_cvd_simulation_wcag_and_palette_audit() {
        let red = [255, 0, 0, 255];
        let (p_sim, d_sim, t_sim) = (simulate_cvd_rgba8(red, CvdDeficiencyType::Protanopia), simulate_cvd_rgba8(red, CvdDeficiencyType::Deuteranopia), simulate_cvd_rgba8(red, CvdDeficiencyType::Tritanopia));
        assert!(p_sim != red && d_sim != red && t_sim != red);

        let cr = wcag21_contrast_ratio([0, 0, 0, 255], [255, 255, 255, 255]);
        assert!((cr - 21.0).abs() < 1e-3, "Black on white contrast ratio must be 21.0");

        let okabe_ito = vec![[230, 159, 0, 255], [86, 180, 233, 255], [0, 158, 115, 255]];
        let report_safe = audit_cartographic_palette(&okabe_ito, [255, 255, 255, 255]);
        assert!(report_safe.is_cvd_safe, "Okabe-Ito must be detected as CVD safe");

        let unsafe_rg = vec![[230, 20, 20, 255], [20, 180, 20, 255]];
        let report_unsafe = audit_cartographic_palette(&unsafe_rg, [255, 255, 255, 255]);
        assert!(!report_unsafe.is_cvd_safe, "Unsafe Red-Green palette must fail CVD audit");
    }

    #[test]
    fn test_publication_svg_composition_and_budget_gate() {
        let spec = PrintLayoutSpec {
            page: PageSizeMm::A4_LANDSCAPE, title: "Biodiversity Corridor".to_string(),
            subtitle_crs: "GDA2020 / MGA Zone 56".to_string(), author_attribution: "Ecopunk Studio".to_string(),
            map_bbox_proj: [500000.0, 6900000.0, 520000.0, 6920000.0], graticule_step_m: 5000.0,
            scale_bar: ScaleBarSpec { map_scale_denom: 25000.0, divisions: 4, segment_ground_m: 500.0 },
            legend_items: vec![("Remnant Forest".to_string(), [34, 197, 94, 255])],
            include_north_arrow: true, include_inset_map: true,
        };

        let svg = compose_publication_map_svg(&spec, 65536).expect("svg compose");
        assert!(svg.contains(r#"<svg"#) && svg.contains(r#"id="map-frame""#));
        assert!(svg.contains(r#"id="graticule""#) && svg.contains(r#"id="scale-bar""#));
        assert!(svg.contains(r#"id="north-arrow""#) && svg.contains(r#"id="inset-map""#));
        assert!(svg.contains(r#"id="legend""#) && svg.contains("Biodiversity Corridor"));

        let err = compose_publication_map_svg(&spec, 50).expect_err("budget fail");
        match err {
            CartographyError::ByteBudgetExceeded { requested, budget } => {
                assert_eq!(budget, 50);
                assert!(requested > budget);
            }
            _ => panic!("Expected ByteBudgetExceeded error"),
        }
    }
}
