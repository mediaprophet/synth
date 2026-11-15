(function () {
  function qacBase64ToBytes(b64) {
    const bin = atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
  }

  function isLegacyQ4auMagic(bytes) {
    if (!bytes || bytes.length < 4) return false;
    const m = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
    return m === 'Q4AU' || m === 'UA4Q';
  }

  async function ensureQacWasmModule() {
    if (window.__synthQacWasmMod) return window.__synthQacWasmMod;
    const shellPath = window.SYNTH_PORTAL_SHELL || '/qualia/js/qualia-shell.js';
    const pkgUrl = new URL('../pkg/qualia/qualia.js', new URL(shellPath, location.href)).href;
    const mod = await import(pkgUrl);
    if (typeof mod.default === 'function') {
      await mod.default();
    }
    if (typeof mod.init_panic_hook === 'function') {
      try { mod.init_panic_hook(); } catch (_) {}
    }
    window.__synthQacWasmMod = mod;
    return mod;
  }

  function readPcmSample(view, byteOffset, scalar) {
    if (scalar === 'F32') return view.getFloat32(byteOffset, true);
    if (scalar === 'F64') return view.getFloat64(byteOffset, true);
    if (scalar === 'I16') return view.getInt16(byteOffset, true);
    if (scalar === 'I32') return view.getInt32(byteOffset, true);
    if (scalar === 'U8') return view.getUint8(byteOffset);
    if (scalar === 'I24') {
      const b0 = view.getUint8(byteOffset);
      const b1 = view.getUint8(byteOffset + 1);
      const b2 = view.getUint8(byteOffset + 2);
      const raw = b0 | (b1 << 8) | (b2 << 16);
      return (raw & 0x800000) ? (raw | ~0xffffff) : raw;
    }
    return 0;
  }

  function scalarByteWidth(scalar) {
    if (scalar === 'U8') return 1;
    if (scalar === 'I16') return 2;
    if (scalar === 'I24') return 3;
    if (scalar === 'I32' || scalar === 'F32') return 4;
    if (scalar === 'F64' || scalar === 'ComplexF32') return 8;
    if (scalar === 'ComplexF64') return 16;
    return 4;
  }

  function extractWindowSeries(win, mode, selectedChannel) {
    const raw = win && win.original_interleaved_bytes;
    if (!raw || typeof raw.length !== 'number') return { kind: 'empty', values: [], text: 'No window bytes returned.' };
    const bytes = raw instanceof Uint8Array ? raw : new Uint8Array(raw);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const scalar = win.scalar || '';
    const channels = Math.max(1, Number(win.channels) || 1);
    const bins = Math.max(1, Number(win.bins) || 1);
    const ch = Math.min(Math.max(0, Number(selectedChannel) || 0), channels - 1);
    const width = scalarByteWidth(scalar);
    const records = Math.max(0, Number(win.records) || 0);
    const header = [
      `layout=${win.layout} scalar=${scalar} start=${win.start_record_decimal || '0'} records=${records} ch=${ch}/${channels} bins=${bins} bytes=${bytes.length}`,
      `byte_order=${win.byte_order}`
    ];
    if (win.first_time_seconds_numerator !== undefined) {
      header.push(`first_time_s=${win.first_time_seconds_numerator}/${win.first_time_seconds_denominator}`);
    }
    if (Number(mode) === 0x100) {
      const vals = [];
      const stride = channels * bins * width;
      for (let r = 0; r < records; r++) {
        const off = r * stride + ch * bins * width;
        if (off + width <= bytes.byteLength) vals.push(readPcmSample(view, off, scalar));
      }
      const preview = vals.slice(0, 12).map(v => Number(v).toPrecision(6)).join(', ');
      header.push(`pcm_ch${ch}[0..${ Math.min(12, vals.length) }]: ${preview}`);
      return { kind: 'pcm', values: vals, text: header.join('\n') };
    }
    if (Number(mode) === 0x101) {
      const mags = [];
      const pairs = [];
      const frameStride = channels * bins * width;
      const chBase = ch * bins * width;
      for (let b = 0; b < bins; b++) {
        const off = chBase + b * width;
        if (off + width <= bytes.byteLength && off < frameStride + width) {
          const re = scalar === 'ComplexF64' ? view.getFloat64(off, true) : view.getFloat32(off, true);
          const im = scalar === 'ComplexF64' ? view.getFloat64(off + 8, true) : view.getFloat32(off + 4, true);
          const mag = Math.hypot(re, im);
          mags.push(mag);
          if (b < 8) {
            pairs.push(`[${b}]=${re.toPrecision(4)}${im >= 0 ? '+' : ''}${im.toPrecision(4)}i (|X|=${mag.toPrecision(4)})`);
          }
        }
      }
      header.push(`stft_ch${ch}_frame0_bins: ${pairs.join(' ')}`);
      return { kind: 'stft', values: mags, text: header.join('\n') };
    }
    return { kind: 'opaque', values: [], text: header.join('\n') };
  }

  function renderDerivativeSvg(series, meta, channel, decimalStart) {
    if (!series || !series.values || !series.values.length) {
      return '<div class="text-[11px] font-mono text-[var(--muted-foreground)]">No chartable PCM/STFT samples in current window.</div>';
    }
    const vals = series.values;
    const w = 560;
    const h = 120;
    const padX = 28;
    const padY = 18;
    const plotW = w - padX * 2;
    const plotH = h - padY * 2;
    let minV = Infinity;
    let maxV = -Infinity;
    for (let i = 0; i < vals.length; i++) {
      const v = Number(vals[i]);
      if (Number.isFinite(v)) {
        if (v < minV) minV = v;
        if (v > maxV) maxV = v;
      }
    }
    if (!Number.isFinite(minV) || !Number.isFinite(maxV)) {
      minV = 0;
      maxV = 1;
    }
    if (series.kind === 'stft') minV = 0;
    const span = (maxV - minV) > 1e-12 ? (maxV - minV) : 1.0;
    const pts = vals.map((v, idx) => {
      const x = padX + (vals.length > 1 ? (idx / (vals.length - 1)) * plotW : plotW / 2);
      const y = padY + plotH - ((v - minV) / span) * plotH;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
    const winKind = meta.window_kind || (meta.transform && (meta.transform.window_kind || meta.transform.window)) || 'none';
    const origBasis = meta.origin_basis || (meta.transform && meta.transform.origin_basis) || 'epoch-sample-grid';
    const title = series.kind === 'stft'
      ? `Visual Derivative · STFT Magnitude |X[k]| = hypot(re, im) · frame=${decimalStart} ch=${channel} window=${winKind} basis=${origBasis}`
      : `Visual Derivative · Stored PCM Samples (mode 0x0100) · start=${decimalStart} ch=${channel} scalar=${meta.scalar} units=${meta.units} (unnormalized)`;
    return `<div class="space-y-1">
      <div class="flex flex-wrap items-center justify-between text-[11px] font-mono text-emerald-300">
        <span>${title}</span>
        <span class="text-[var(--muted-foreground)]">range=[${minV.toPrecision(4)}, ${maxV.toPrecision(4)}] · n=${vals.length}</span>
      </div>
      <svg viewBox="0 0 ${w} ${h}" class="w-full h-[120px] rounded bg-[#070d0c] border border-[var(--border)]">
        <line x1="${padX}" y1="${padY + plotH / 2}" x2="${w - padX}" y2="${padY + plotH / 2}" stroke="rgba(148,163,184,0.22)" stroke-dasharray="3,3" />
        <polyline fill="none" stroke="#34d399" stroke-width="1.75" points="${pts}" />
      </svg>
    </div>`;
  }

  function formatMetadata(summary, meta, entrySummary) {
    const rateNum = meta.rate_num || meta.sample_rate_numerator || (entrySummary && entrySummary.rate_num) || '?';
    const rateDen = meta.rate_den || meta.sample_rate_denominator || (entrySummary && entrySummary.rate_den) || '?';
    const winKind = meta.window_kind || (meta.transform && (meta.transform.window_kind || meta.transform.window)) || 'none';
    const origBasis = meta.origin_basis || (meta.transform && meta.transform.origin_basis) || 'epoch-sample-grid';
    const epistemic = meta.epistemic || meta.provenance_role || (entrySummary && entrySummary.epistemic) || 'unknown';
    const lines = [
      `container: magic="qac\\0\\0\\0\\0\\0" (v1) entries=${summary.entry_count} blocks=${summary.block_count} blake3=${summary.source_blake3}`,
      `entry=${meta.selected_entry} mode=0x${Number(meta.mode).toString(16)} scalar=${meta.scalar} epistemic=${epistemic} records=${meta.records_decimal}`,
      `clock: rate_num=${rateNum} rate_den=${rateDen} (${rateNum}/${rateDen} Hz) ticks=${meta.ticks_per_second_numerator}/${meta.ticks_per_second_denominator} start_tick=${meta.start_tick_decimal}`,
      `basis: origin_basis=${origBasis} window_kind=${winKind}`,
      `semantics: quantity=${meta.quantity} units=${meta.units} relation=${meta.relation} purpose=${meta.purpose}`,
      `topology: channels=${meta.channels} bins=${meta.bins} order=${meta.channel_order}`,
      `calibration: scale=${meta.units_per_stored_sample} offset=${meta.stored_offset} (${meta.playback})`
    ];
    if (meta.transform) {
      const t = meta.transform;
      const winBytes = t.window_coefficients_le ? `${t.window_coefficients_le.length} B authored f32` : 'none';
      lines.push(
        `transform: kind=${t.kind} fft=${t.fft_size} hop=${t.hop} window_kind=${t.window_kind || t.window} (${winBytes})`,
        `transform_origin: origin_basis=${t.origin_basis} origin_samples=${t.origin_samples_decimal} in_rate=${t.input_rate_numerator}/${t.input_rate_denominator}`,
        `fft_spec: ${t.bin_order}; ${t.convention}`
      );
    }
    return lines.join('\n');
  }

  async function renderQacHost(host) {
    const statusEl = host.querySelector('.synth-qac-status');
    const selectEl = host.querySelector('.synth-qac-entry-select');
    const channelEl = host.querySelector('.synth-qac-channel-select');
    const startEl = host.querySelector('.synth-qac-start-input');
    const recordsEl = host.querySelector('.synth-qac-records-input');
    const applyBtn = host.querySelector('.synth-qac-apply-btn');
    const metaEl = host.querySelector('.synth-qac-meta');
    const winEl = host.querySelector('.synth-qac-window');
    const chartEl = host.querySelector('.synth-qac-chart');
    try {
      let bytes = null;
      if (host.dataset.qacB64 && host.dataset.qacB64.length > 0) {
        bytes = qacBase64ToBytes(host.dataset.qacB64);
      } else if (host.dataset.qacUrl) {
        const resp = await fetch(host.dataset.qacUrl);
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        bytes = new Uint8Array(await resp.arrayBuffer());
      } else {
        throw new Error('No .qac bytes or URL supplied');
      }

      if (isLegacyQ4auMagic(bytes)) {
        const msg = 'Legacy Q4AU/UA4Q container refused: current .qac v1 magic qac\\0\\0\\0\\0\\0 required (no silent transcoding or alias inflation).';
        if (statusEl) statusEl.textContent = msg;
        if (metaEl) metaEl.textContent = msg;
        if (winEl) winEl.textContent = 'Refused legacy Q4AU/UA4Q payload.';
        return;
      }

      const mod = await ensureQacWasmModule();
      if (!mod.QualiaPortal || typeof mod.QualiaPortal.new_qac_reader !== 'function') {
        throw new Error('QualiaPortal.new_qac_reader is absent in WASM package');
      }
      const reader = mod.QualiaPortal.new_qac_reader();
      const summary = reader.inspect_qac(bytes);
      const entries = Array.isArray(summary.entries) ? summary.entries : [];
      if (selectEl) {
        selectEl.innerHTML = '';
        for (const e of entries) {
          const opt = document.createElement('option');
          opt.value = String(e.index);
          const hexMode = '0x' + Number(e.mode).toString(16);
          const kindLabel = e.profile_kind ? ` [${e.profile_kind}]` : '';
          const unitLabel = e.units ? ` (${e.units})` : '';
          const epLabel = e.epistemic ? ` · ${e.epistemic}` : '';
          opt.textContent = `Entry ${e.index} (${hexMode}${kindLabel}${epLabel}) · ${e.quantity || e.descriptor_id}${unitLabel} · ${e.record_count_decimal} rec`;
          selectEl.appendChild(opt);
        }
      }

      let currentMeta = null;
      let currentEntrySummary = null;

      const updateWindow = () => {
        if (!currentMeta) return;
        const ch = channelEl ? Math.max(0, Number(channelEl.value) || 0) : 0;
        const startStr = (startEl && startEl.value ? startEl.value.trim() : '0') || '0';
        const totalRec = Math.max(0, Number(currentMeta.records_decimal) || 0);
        const startNum = Math.max(0, Number(startStr) || 0);
        const avail = Math.max(0, totalRec - startNum);
        const reqCount = recordsEl ? Math.max(1, Number(recordsEl.value) || 32) : 32;
        const count = Math.min(Math.max(0, avail), Math.min(reqCount, 256));
        const win = reader.qac_window(String(startNum), count, ch);
        const series = extractWindowSeries(win, currentMeta.mode, ch);
        if (winEl) winEl.textContent = series.text;
        if (chartEl) chartEl.innerHTML = renderDerivativeSvg(series, currentMeta, ch, String(startNum));
      };

      const loadEntry = (entryIndex) => {
        currentEntrySummary = entries.find(e => Number(e.index) === Number(entryIndex)) || null;
        if (!currentEntrySummary || !(currentEntrySummary.mode === 0x100 || currentEntrySummary.mode === 0x101)) {
          currentMeta = null;
          if (metaEl) metaEl.textContent = JSON.stringify(currentEntrySummary || summary, null, 2);
          if (winEl) winEl.textContent = 'Non-waveform / opaque entry (mode 0x0104 or domain profile): exact bytes preserved without synthetic waveform preview.';
          if (chartEl) chartEl.innerHTML = '<div class="text-[11px] font-mono text-amber-300">Opaque / domain profile entry · no implicit waveform synthesis.</div>';
          if (statusEl) statusEl.textContent = `Admitted Entry ${entryIndex} (non-waveform / opaque preservation · paint-free)`;
          return;
        }
        currentMeta = reader.load_qac(bytes, entryIndex);
        if (channelEl) {
          channelEl.innerHTML = '';
          const chCount = Math.max(1, Number(currentMeta.channels) || 1);
          for (let c = 0; c < chCount; c++) {
            const opt = document.createElement('option');
            opt.value = String(c);
            opt.textContent = `ch ${c}`;
            channelEl.appendChild(opt);
          }
        }
        if (metaEl) metaEl.textContent = formatMetadata(summary, currentMeta, currentEntrySummary);
        updateWindow();
        if (statusEl) statusEl.textContent = `Admitted via QualiaPortal.new_qac_reader · Entry ${entryIndex} · paint-free inspection`;
      };

      const inspectable = entries.filter(e => e.mode === 0x100 || e.mode === 0x101);
      const initial = inspectable.length ? inspectable[0].index : (entries[0] ? entries[0].index : 1);
      if (selectEl) {
        selectEl.value = String(initial);
        selectEl.onchange = () => loadEntry(Number(selectEl.value));
      }
      if (channelEl) channelEl.onchange = updateWindow;
      if (startEl) startEl.onchange = updateWindow;
      if (recordsEl) recordsEl.onchange = updateWindow;
      if (applyBtn) applyBtn.onclick = updateWindow;
      if (entries.length) loadEntry(initial);
    } catch (err) {
      if (statusEl) statusEl.textContent = 'QAC inspection failed closed: ' + (err && err.message ? err.message : err);
    }
  }

  window.bootSynthQacCards = async function bootSynthQacCards() {
    const hosts = document.querySelectorAll('[data-synth-qac="1"]');
    if (!hosts.length) return;
    const http = location.protocol === 'http:' || location.protocol === 'https:';
    if (!http) {
      hosts.forEach(host => {
        const s = host.querySelector('.synth-qac-status');
        if (s && s.dataset.staticAdmitted !== '1') {
          s.textContent = 'Static Rust paint-free inspection active (serve over http(s) via synth serve for live WASM window scrubbing).';
        }
      });
      return;
    }
    for (const host of hosts) {
      if (host.dataset.qacBooted === '1') continue;
      host.dataset.qacBooted = '1';
      await renderQacHost(host);
    }
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      if (typeof window.bootSynthQacCards === 'function') window.bootSynthQacCards();
    });
  } else {
    window.bootSynthQacCards();
  }
})();
