//! QualiaPortal WASM embed for synth origin (`/qualia/js/qualia-shell.js`).
//!
//! Official `.10d` painter is QualiaPortal (`load_10d` + sibling `load_emf`).
//! PNG contact sheets are the always-works review. 2D canvas is degraded-only.
//! `file://` cannot load WASM; do not claim GPU when the adapter is absent.

use std::fmt::Write as _;

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub const QUALIA_SHELL_SRC: &str = "/qualia/js/qualia-shell.js";

pub fn sibling_with_ext(path: &str, ext: &str) -> String {
    let suffix = if ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{ext}")
    };
    let lower = path.to_ascii_lowercase();
    if let Some(stem) = lower
        .rsplit_once('.')
        .filter(|(_, e)| *e == "10d" || *e == "emf" || *e == "png")
        .map(|(s, _)| path[..s.len()].to_string())
    {
        format!("{stem}{suffix}")
    } else {
        format!("{path}{suffix}")
    }
}

pub fn dir_of(rel: &str) -> &str {
    rel.rsplit_once('/').map(|(d, _)| d).unwrap_or("")
}

pub fn related_png(ten_d_rel: &str, png_rel: &str) -> bool {
    let png = png_rel.to_ascii_lowercase();
    if !png.ends_with(".png") {
        return false;
    }
    let stem = ten_d_rel
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(ten_d_rel);
    if png_rel.starts_with(stem) {
        return true;
    }
    let dir = dir_of(ten_d_rel);
    if dir.is_empty() {
        return png_rel
            .rsplit_once('/')
            .map(|(_, n)| n)
            .unwrap_or(png_rel)
            .starts_with(stem);
    }
    if !png_rel.starts_with(dir) {
        return false;
    }
    png.contains("front-view")
        || png.contains("side-view")
        || png.contains("three-quarter")
        || png.contains("turnaround")
        || png.contains("_albedo")
        || png.contains("_normal")
        || png.contains("_orm")
        || png.contains("_bsdf")
        || png.contains("preview")
}

pub struct PortalPng<'a> {
    pub rel: &'a str,
    pub src: &'a str,
}

pub fn render_contact_sheet(pngs: &[PortalPng<'_>]) -> String {
    if pngs.is_empty() {
        return String::new();
    }
    let mut imgs = String::new();
    for png in pngs {
        let _ = write!(
            imgs,
            r#"<figure class="synth-contact-item"><img src="{src}" alt="{alt}" /><figcaption>{alt}</figcaption></figure>"#,
            src = escape_html(png.src),
            alt = escape_html(png.rel),
        );
    }
    format!(
        r#"<div class="synth-contact-sheet" data-synth-contact-sheet="png">
  <div class="text-[11px] font-mono text-[var(--muted-foreground)]">PNG contact sheet (always-works review · not the QualiaPortal painter)</div>
  <div class="synth-contact-row">{imgs}</div>
</div>"#
    )
}

/// Portal canvas plus PNG contact sheet. `mesh_url` is fetched only on an http(s) origin.
pub fn render_portal_host(
    host_id: &str,
    mesh_url: &str,
    emf_url: &str,
    pngs: &[PortalPng<'_>],
) -> String {
    let sheet = render_contact_sheet(pngs);
    let id = escape_html(host_id);
    let mesh = escape_html(mesh_url);
    let emf = escape_html(emf_url);
    format!(
        r#"<div class="synth-portal-host space-y-2" id="{id}" data-synth-portal="1" data-mesh-url="{mesh}" data-emf-url="{emf}">
  <div class="flex flex-wrap items-center justify-between gap-2 text-xs font-mono">
    <span class="text-emerald-300 font-semibold">QualiaPortal WASM</span>
    <span class="text-[var(--muted-foreground)]">load_10d · sibling load_emf · {shell}</span>
  </div>
  <div class="synth-portal-status text-[11px] font-mono text-[var(--muted-foreground)]">QualiaPortal: waiting for http origin + WASM</div>
  <canvas class="synth-portal-canvas" width="680" height="360"></canvas>
  <p class="text-[11px] font-mono text-[var(--muted-foreground)]">file:// will not load WASM. PNG contact sheet is the review when GPU or WASM is absent. 2D canvas below is degraded-only.</p>
  {sheet}
</div>"#,
        shell = QUALIA_SHELL_SRC,
    )
}

pub fn webgpu_snapshot_html() -> String {
    r#"<pre id="webgpu-snapshot" class="mt-2 p-2.5 rounded-lg bg-[var(--background)] border border-[var(--border)] text-[11px] font-mono text-[var(--muted-foreground)] overflow-x-auto">WebGPU adapter: probing… (will not claim GPU if navigator.gpu / requestAdapter is absent)</pre>"#.to_string()
}

/// Browser WebGPU adapter snapshot + QualiaPortal boot.
/// Field names match Qualia `browser-capability` adapter info (vendor/architecture/device/description/features).
/// Does not copy `hypermedia.discourse.inspect_engine_telemetry`'s 256-frame ring-buffer table.
pub const PORTAL_BOOT_JS: &str = r#"
window.SYNTH_PORTAL_SHELL = '/qualia/js/qualia-shell.js';
async function snapshotWebGpuAdapter() {
  const el = document.getElementById('webgpu-snapshot');
  if (!el) return;
  const lines = [
    'WebGPU capability snapshot (browser adapter.info)',
    'hypermedia.discourse.inspect_engine_telemetry names the 256-frame ring buffer / VRAM ledger — not forked here'
  ];
  const gpu = navigator.gpu;
  if (!gpu || typeof gpu.requestAdapter !== 'function') {
    lines.push('navigator.gpu: absent');
    lines.push('gpu: unavailable');
    el.textContent = lines.join('\n');
    return;
  }
  try {
    const adapter = await gpu.requestAdapter();
    if (!adapter) {
      lines.push('requestAdapter: null');
      lines.push('gpu: unavailable');
      el.textContent = lines.join('\n');
      return;
    }
    const info = adapter.info || {};
    lines.push('vendor: ' + (info.vendor || ''));
    lines.push('architecture: ' + (info.architecture || ''));
    lines.push('device: ' + (info.device || ''));
    lines.push('description: ' + (info.description || ''));
    const feats = adapter.features && typeof adapter.features.forEach === 'function'
      ? Array.from(adapter.features).sort().join(', ')
      : '';
    lines.push('features: ' + (feats || '(none advertised)'));
    if (info.isFallbackAdapter || adapter.isFallbackAdapter) lines.push('isFallbackAdapter: true');
    el.textContent = lines.join('\n');
  } catch (err) {
    lines.push('requestAdapter failed: ' + (err && err.message ? err.message : err));
    lines.push('gpu: unavailable');
    el.textContent = lines.join('\n');
  }
}
function siblingUrl(meshUrl, ext) {
  if (typeof meshUrl !== 'string') return '';
  const lower = meshUrl.toLowerCase();
  if (!lower.endsWith('.10d')) return '';
  const suffix = ext.charAt(0) === '.' ? ext : '.' + ext;
  return meshUrl.slice(0, meshUrl.length - 4) + suffix;
}
function setPortalStatus(host, text) {
  const el = host.querySelector('.synth-portal-status');
  if (el) el.textContent = text;
}
function markPortalGpu(host, live) {
  host.classList.toggle('portal-gpu-live', !!live);
  const card = host.closest('.resource-card');
  if (card) card.classList.toggle('portal-gpu-live', !!live);
}
async function loadPortalMesh(portal, host, meshUrl) {
  if (!meshUrl) {
    setPortalStatus(host, 'QualiaPortal ready · no fetchable .10d URL (PNG contact sheet is the review)');
    return;
  }
  setPortalStatus(host, 'Fetching ' + meshUrl.split('/').pop());
  const response = await fetch(meshUrl);
  if (!response.ok) throw new Error(response.status + ' ' + meshUrl);
  const bytes = new Uint8Array(await response.arrayBuffer());
  const receipt = portal.load_10d(bytes);
  const tris = receipt && (receipt.triangle_count ?? receipt.total_triangles);
  const verts = receipt && receipt.vertex_count;
  const renderer = window.__qualiaAnatomyInit && window.__qualiaAnatomyInit.renderer || 'unknown';
  let meshStatus = (tris != null ? tris : '?') + ' triangles · ' + (verts != null ? verts : '?') + ' verts · QualiaPortal/' + renderer;
  setPortalStatus(host, meshStatus);
  const emfUrl = host.dataset.emfUrl || siblingUrl(meshUrl, '.emf');
  if (!emfUrl) {
    setPortalStatus(host, meshStatus + ' · mesh-only (no field)');
    return;
  }
  try {
    const emfRes = await fetch(emfUrl);
    if (emfRes.status === 404) {
      setPortalStatus(host, meshStatus + ' · mesh-only (no field)');
      return;
    }
    if (!emfRes.ok) {
      setPortalStatus(host, meshStatus + ' · .emf ' + emfRes.status + ' (mesh kept)');
      return;
    }
    const emfBytes = new Uint8Array(await emfRes.arrayBuffer());
    const emfReceipt = portal.load_emf(emfBytes);
    const nx = emfReceipt && emfReceipt.nx;
    const cells = emfReceipt && emfReceipt.cell_count;
    meshStatus += ' · .emf' + (nx != null ? ' ' + nx + '×' + (emfReceipt.ny ?? '?') + '×' + (emfReceipt.nz ?? '?') : '') + (typeof cells === 'number' ? ' (' + cells + ' cells)' : '');
    setPortalStatus(host, meshStatus);
  } catch (err) {
    setPortalStatus(host, meshStatus + ' · .emf skipped (' + (err && err.message ? err.message : err) + ')');
  }
}
async function bootSynthPortals() {
  const hosts = document.querySelectorAll('[data-synth-portal]');
  const http = location.protocol === 'http:' || location.protocol === 'https:';
  if (!http) {
    hosts.forEach(function (host) {
      setPortalStatus(host, 'file:// cannot load QualiaPortal WASM. PNG contact sheet is the always-works review.');
      markPortalGpu(host, false);
    });
    return;
  }
  if (!hosts.length) return;
  let loadQualiaPortal, startPortalLoop;
  try {
    const mod = await import(window.SYNTH_PORTAL_SHELL);
    loadQualiaPortal = mod.loadQualiaPortal;
    startPortalLoop = mod.startPortalLoop;
  } catch (err) {
    hosts.forEach(function (host) {
      setPortalStatus(host, 'QualiaPortal WASM unavailable (' + (err && err.message ? err.message : err) + '). PNG / degraded 2D only. Serve ' + window.SYNTH_PORTAL_SHELL);
      markPortalGpu(host, false);
    });
    return;
  }
  for (const host of hosts) {
    if (host.dataset.portalBooted === '1') continue;
    host.dataset.portalBooted = '1';
    const canvas = host.querySelector('canvas.synth-portal-canvas');
    const meshUrl = host.dataset.meshUrl || '';
    let yaw = 0.6, pitch = 0.3, zoom = 3.5, spinning = true, drag = null;
    let loaded = null;
    try {
      loaded = await loadQualiaPortal(canvas, { allowWebGl2: true });
    } catch (err) {
      setPortalStatus(host, 'Degraded 2D / PNG — QualiaPortal GPU unavailable (' + (err && err.message ? err.message : err) + ')');
      markPortalGpu(host, false);
      continue;
    }
    const portal = loaded && loaded.portal;
    const renderer = (loaded && loaded.renderer) || (window.__qualiaAnatomyInit && window.__qualiaAnatomyInit.renderer) || null;
    const gpuOk = !!portal && (renderer === 'webgpu' || renderer === 'webgl2');
    if (!gpuOk) {
      const detail = (loaded && loaded.portalError && (loaded.portalError.message || loaded.portalError)) || '';
      setPortalStatus(host, 'Degraded 2D / PNG — QualiaPortal GPU unavailable' + (detail ? ' · ' + detail : ''));
      markPortalGpu(host, false);
      continue;
    }
    markPortalGpu(host, true);
    const liveCanvas = loaded.canvas || canvas;
    liveCanvas.classList.add('synth-portal-canvas');
    if (typeof portal.set_camera === 'function') portal.set_camera(yaw, pitch, zoom);
    startPortalLoop(liveCanvas, function () {
      if (spinning && portal && typeof portal.set_camera === 'function') {
        yaw += 0.008;
        portal.set_camera(yaw, pitch, zoom);
      }
    }, function (next) {
      next.classList.add('synth-portal-canvas');
    });
    liveCanvas.onpointerdown = function (event) {
      drag = { x: event.clientX, y: event.clientY, yaw: yaw, pitch: pitch };
      spinning = false;
      liveCanvas.setPointerCapture(event.pointerId);
    };
    liveCanvas.onpointermove = function (event) {
      if (!drag) return;
      yaw = drag.yaw + (event.clientX - drag.x) * 0.01;
      pitch = drag.pitch + (event.clientY - drag.y) * 0.01;
      if (portal && typeof portal.set_camera === 'function') portal.set_camera(yaw, pitch, zoom);
    };
    liveCanvas.onpointerup = function () { drag = null; };
    try {
      await loadPortalMesh(portal, host, meshUrl);
    } catch (err) {
      setPortalStatus(host, 'QualiaPortal load_10d failed: ' + (err && err.message ? err.message : err) + ' · PNG contact sheet remains');
    }
  }
}
window.bootSynthPortals = bootSynthPortals;
window.snapshotWebGpuAdapter = snapshotWebGpuAdapter;
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', function () {
    snapshotWebGpuAdapter();
    bootSynthPortals();
  });
} else {
  snapshotWebGpuAdapter();
  bootSynthPortals();
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sibling_ext_replaces_trailing_10d_only() {
        assert_eq!(
            sibling_with_ext("working/heart/heart.10d", "emf"),
            "working/heart/heart.emf"
        );
        assert_eq!(
            sibling_with_ext("/project/file/working/a.10d", "png"),
            "/project/file/working/a.png"
        );
    }

    #[test]
    fn related_png_matches_stem_and_view_names() {
        assert!(related_png("heart/heart.10d", "heart/heart_turnaround.png"));
        assert!(related_png("heart/heart.10d", "heart/front-view-assumed.png"));
        assert!(!related_png("heart/heart.10d", "liver/liver.png"));
    }
}
