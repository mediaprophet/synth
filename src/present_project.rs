//! Project workbench HTML: working vs delivered files, and user annotations.
use crate::{
    present::escape_html,
    present_portal::{self, PortalPng, PORTAL_BOOT_JS, QUALIA_SHELL_SRC},
    present_style::SYNTH_STUDIO_CSS,
    project::{current, Annotation, ListedFile, Project},
};
use std::fmt::Write as _;

pub fn render_project_html() -> Result<String, String> {
    let Some(project) = current() else {
        return Ok(render_unattached());
    };
    let working = project.list_lane("working")?;
    let delivered = project.list_lane("delivered")?;
    let annotations = project.load_annotations()?;
    let sources = project.list_sources();
    Ok(page(
        project,
        &working,
        &delivered,
        &annotations,
        &sources,
    ))
}

fn render_unattached() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Synth project</title>
  <style>{css}</style>
</head>
<body>
  <main class="max-w-4xl mx-auto space-y-5 p-6">
    <header class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-5 space-y-3">
      <div class="text-xs font-mono uppercase tracking-wider text-amber-300">synth.project/v1</div>
      <h1 class="text-xl font-semibold">No project is attached</h1>
      <p class="text-sm text-[var(--muted-foreground)]">Start Synth with a project directory so this page shows the work in progress, the delivered files, and your notes.</p>
      <pre class="p-3 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs font-mono">synth serve --project C:\github\anatomy --open</pre>
      <p class="text-[11px] font-mono text-[var(--muted-foreground)]">QualiaPortal WASM: {shell}</p>
      {webgpu_snapshot}
      <a class="inline-block text-xs font-mono text-emerald-300" href="/">QualiaDB demo studio</a>
    </header>
  </main>
  <script>{portal_boot_js}</script>
</body>
</html>"#,
        css = SYNTH_STUDIO_CSS,
        shell = QUALIA_SHELL_SRC,
        webgpu_snapshot = present_portal::webgpu_snapshot_html(),
        portal_boot_js = PORTAL_BOOT_JS,
    )
}

fn page(
    project: &Project,
    working: &[ListedFile],
    delivered: &[ListedFile],
    annotations: &[Annotation],
    sources: &[std::path::PathBuf],
) -> String {
    let working_cards = render_file_cards(working);
    let delivered_cards = render_file_cards(delivered);
    let notes = render_annotations(annotations);
    let file_options = render_file_options(working, delivered);
    let source_list = render_sources(sources);
    let brief = if project.brief.trim().is_empty() {
        "Working files are being made. Delivered files are what you have already accepted. Tag a part of a model, rate it, comment, and attach a reference link."
            .to_string()
    } else {
        project.brief.clone()
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{name} · Synth project</title>
  <style>{css}</style>
</head>
<body>
  <main class="max-w-7xl mx-auto space-y-5 p-4">
    <header class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-5 shadow-sm space-y-4">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <div class="text-xs font-mono uppercase tracking-wider text-emerald-400">synth.project/v1 · current work</div>
          <h1 class="text-xl font-semibold mt-0.5">{name}</h1>
          <p class="text-sm text-[var(--muted-foreground)] mt-1 max-w-3xl">{brief}</p>
        </div>
        <div class="flex flex-wrap items-center gap-2 text-xs font-mono">
          <a class="px-2.5 py-1 rounded-md bg-[var(--background)] border border-[var(--border)]" href="/">QualiaDB demo studio</a>
          <span class="px-2.5 py-1 rounded-md bg-amber-950/50 border border-amber-500/40 text-amber-200">Working {working_n}</span>
          <span class="px-2.5 py-1 rounded-md bg-emerald-950/50 border border-emerald-500/40 text-emerald-200">Delivered {delivered_n}</span>
          <span class="px-2.5 py-1 rounded-md bg-[var(--background)] border border-[var(--border)]">Notes {notes_n}</span>
        </div>
      </div>
      <div class="text-[11px] font-mono text-[var(--muted-foreground)]">{root}</div>
      <div class="text-[11px] font-mono text-[var(--muted-foreground)]">QualiaPortal WASM: <span class="text-emerald-300">{shell}</span> · COOP/COEP on this origin. PNG thumbs stay. 2D is degraded-only.</div>
      {webgpu_snapshot}
      <div class="flex flex-wrap items-center gap-2 pt-2 border-t border-[var(--border)]" role="tablist">
        <button type="button" onclick="switchProjectTab('tab-working', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono font-semibold border border-amber-500/50 bg-amber-950/60 text-amber-300 filter-pill-active">Working</button>
        <button type="button" onclick="switchProjectTab('tab-delivered', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)] text-[var(--foreground)]">Delivered</button>
        <button type="button" onclick="switchProjectTab('tab-notes', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)] text-[var(--foreground)]">Tags, comments, ratings</button>
        <button type="button" onclick="switchProjectTab('tab-source', this)" class="studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)] text-[var(--foreground)]">Source vibes</button>
      </div>
    </header>

    <section id="tab-working" class="studio-tab space-y-4">
      <p class="text-sm text-[var(--muted-foreground)]">Files being created or updated. Comment on a file from the notes tab.</p>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">{working_cards}</div>
    </section>

    <section id="tab-delivered" class="studio-tab hidden space-y-4">
      <p class="text-sm text-[var(--muted-foreground)]">Accepted or published output. This is the product so far, separate from the work still in motion.</p>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">{delivered_cards}</div>
    </section>

    <section id="tab-notes" class="studio-tab hidden space-y-4">
      <div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-5 space-y-3">
        <h2 class="text-sm font-semibold">Annotate a model</h2>
        <p class="text-xs text-[var(--muted-foreground)]">Pick the file, name the aspect (apex, chamber, surface), add tags, a 1–5 rating, a comment, and optional reference URLs the agent should look at.</p>
        <form id="note-form" class="space-y-3" onsubmit="return submitNote(event)">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <label class="text-xs font-mono space-y-1">
              <span class="text-[var(--muted-foreground)]">File</span>
              <select name="target" id="note-target" class="w-full p-2 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs">{file_options}</select>
            </label>
            <label class="text-xs font-mono space-y-1">
              <span class="text-[var(--muted-foreground)]">Aspect of the model</span>
              <input name="aspect" id="note-aspect" placeholder="apex, left ventricle, colour" class="w-full p-2 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs" />
            </label>
          </div>
          <label class="text-xs font-mono space-y-1 block">
            <span class="text-[var(--muted-foreground)]">Tags (comma separated)</span>
            <input name="tags" id="note-tags" placeholder="shape, heart, wrong-silhouette" class="w-full p-2 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs" />
          </label>
          <div class="text-xs font-mono space-y-1">
            <span class="text-[var(--muted-foreground)]">Rating</span>
            <div class="flex gap-2 pt-1">
              <label><input type="radio" name="rating" value="1" /> 1</label>
              <label><input type="radio" name="rating" value="2" /> 2</label>
              <label><input type="radio" name="rating" value="3" /> 3</label>
              <label><input type="radio" name="rating" value="4" /> 4</label>
              <label><input type="radio" name="rating" value="5" /> 5</label>
            </div>
          </div>
          <label class="text-xs font-mono space-y-1 block">
            <span class="text-[var(--muted-foreground)]">Comment</span>
            <textarea name="comment" id="note-comment" rows="4" placeholder="What is wrong, missing, or right about this part." class="w-full p-2 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs"></textarea>
          </label>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <label class="text-xs font-mono space-y-1">
              <span class="text-[var(--muted-foreground)]">Reference URL</span>
              <input name="link_url" id="note-link-url" placeholder="https://…" class="w-full p-2 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs" />
            </label>
            <label class="text-xs font-mono space-y-1">
              <span class="text-[var(--muted-foreground)]">What the link is</span>
              <input name="link_note" id="note-link-note" placeholder="anatomical photo, diagram, measurement table" class="w-full p-2 rounded-lg bg-[var(--background)] border border-[var(--border)] text-xs" />
            </label>
          </div>
          <div class="flex items-center gap-3">
            <button type="submit" class="px-4 py-1.5 rounded-lg text-xs font-mono font-semibold bg-emerald-600/30 border border-emerald-500 text-emerald-200">Save note</button>
            <span id="note-status" class="text-xs font-mono text-[var(--muted-foreground)]"></span>
          </div>
        </form>
      </div>
      <div class="space-y-3">{notes}</div>
    </section>

    <section id="tab-source" class="studio-tab hidden space-y-4">
      <p class="text-sm text-[var(--muted-foreground)]">VibeScript files that name the work. Synth produces from these.</p>
      <div class="space-y-2">{source_list}</div>
    </section>
  </main>
  <script>
    function switchProjectTab(tabId, btn) {{
      document.querySelectorAll('.studio-tab').forEach(el => el.classList.add('hidden'));
      const target = document.getElementById(tabId);
      if (target) target.classList.remove('hidden');
      document.querySelectorAll('.studio-nav-btn').forEach(b => {{
        b.className = 'studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono border border-[var(--border)] bg-[var(--background)] text-[var(--foreground)]';
      }});
      if (btn) {{
        const active = tabId === 'tab-working'
          ? 'studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono font-semibold border border-amber-500/50 bg-amber-950/60 text-amber-300 filter-pill-active'
          : 'studio-nav-btn px-3.5 py-1.5 rounded-lg text-xs font-mono font-semibold border border-emerald-500/70 bg-emerald-950/60 text-emerald-200 filter-pill-active';
        btn.className = active;
      }}
    }}
    function annotateFile(lane, file) {{
      const select = document.getElementById('note-target');
      if (select) select.value = lane + '::' + file;
      switchProjectTab('tab-notes', document.querySelectorAll('.studio-nav-btn')[2]);
      const aspect = document.getElementById('note-aspect');
      if (aspect) aspect.focus();
    }}
    async function submitNote(event) {{
      event.preventDefault();
      const status = document.getElementById('note-status');
      const target = document.getElementById('note-target')?.value || '';
      const parts = target.split('::');
      const ratingEl = document.querySelector('input[name="rating"]:checked');
      const payload = {{
        lane: parts[0] || 'working',
        file: parts.slice(1).join('::'),
        aspect: document.getElementById('note-aspect')?.value || '',
        tags: (document.getElementById('note-tags')?.value || '').split(',').map(s => s.trim()).filter(Boolean),
        rating: ratingEl ? Number(ratingEl.value) : 0,
        comment: document.getElementById('note-comment')?.value || '',
        links: []
      }};
      const url = document.getElementById('note-link-url')?.value || '';
      if (url) {{
        payload.links.push({{ url, note: document.getElementById('note-link-note')?.value || '' }});
      }}
      if (status) status.textContent = 'Saving…';
      try {{
        const res = await fetch('/api/project/annotations', {{
          method: 'POST',
          headers: {{ 'Content-Type': 'application/json' }},
          body: JSON.stringify(payload)
        }});
        const data = await res.json();
        if (!data.ok) {{
          if (status) status.textContent = data.error || 'Save failed';
          return false;
        }}
        window.location.reload();
      }} catch (err) {{
        if (status) status.textContent = String(err);
      }}
      return false;
    }}
    {portal_boot_js}
  </script>
</body>
</html>"#,
        name = escape_html(&project.name),
        css = SYNTH_STUDIO_CSS,
        shell = QUALIA_SHELL_SRC,
        webgpu_snapshot = present_portal::webgpu_snapshot_html(),
        portal_boot_js = PORTAL_BOOT_JS,
        brief = escape_html(&brief),
        working_n = working.len(),
        delivered_n = delivered.len(),
        notes_n = annotations.len(),
        root = escape_html(&project.root.display().to_string().trim_start_matches(r"\\?\").to_string()),
        working_cards = if working_cards.is_empty() {
            empty_lane("Nothing in working yet. Produce into this project's working folder.")
        } else {
            working_cards
        },
        delivered_cards = if delivered_cards.is_empty() {
            empty_lane("Nothing marked delivered yet.")
        } else {
            delivered_cards
        },
        file_options = file_options,
        notes = if notes.is_empty() {
            empty_lane("No tags, comments, or ratings yet.")
        } else {
            notes
        },
        source_list = if source_list.is_empty() {
            empty_lane("No .vibe sources listed in synth.project.json.")
        } else {
            source_list
        },
    )
}

fn empty_lane(msg: &str) -> String {
    format!(
        r#"<div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 text-sm text-[var(--muted-foreground)]">{}</div>"#,
        escape_html(msg)
    )
}

fn portal_file_preview(file: &ListedFile, files: &[ListedFile]) -> String {
    let href = format!(
        "/project/file/{}/{}",
        file.lane,
        url_encode_path(&file.rel)
    );
    let emf_rel = present_portal::sibling_with_ext(&file.rel, "emf");
    let emf_url = format!(
        "/project/file/{}/{}",
        file.lane,
        url_encode_path(&emf_rel)
    );
    let mut png_owned: Vec<(String, String)> = Vec::new();
    for other in files {
        if other.kind == "png"
            && present_portal::related_png(&file.rel, &other.rel)
            && other.bytes <= 8 * 1024 * 1024
        {
            png_owned.push((
                other.rel.clone(),
                format!(
                    "/project/file/{}/{}",
                    other.lane,
                    url_encode_path(&other.rel)
                ),
            ));
        }
    }
    let pngs: Vec<PortalPng<'_>> = png_owned
        .iter()
        .map(|(rel, src)| PortalPng { rel, src })
        .collect();
    let id = format!(
        "portal-{}-{}",
        file.lane,
        file.rel.replace(['/', '\\', '.', ' '], "-")
    );
    present_portal::render_portal_host(&id, &href, &emf_url, "", "", &pngs)
}

fn render_file_cards(files: &[ListedFile]) -> String {
    let mut out = String::new();
    for file in files {
        let href = format!(
            "/project/file/{}/{}",
            file.lane,
            url_encode_path(&file.rel)
        );
        let preview = if file.kind == "10d" {
            portal_file_preview(file, files)
        } else if file.previewable && file.bytes <= 8 * 1024 * 1024 {
            if file.kind == "html" {
                format!(
                    r#"<a class="text-xs font-mono text-emerald-300" href="{href}" target="_blank" rel="noreferrer">Open HTML</a>"#
                )
            } else {
                format!(
                    r#"<a href="{href}" target="_blank" rel="noreferrer"><img src="{href}" alt="{alt}" class="w-full max-h-64 object-contain rounded-md bg-[var(--background)] border border-[var(--border)]" /></a>"#,
                    alt = escape_html(&file.rel)
                )
            }
        } else {
            format!(
                r#"<a class="text-xs font-mono text-emerald-300" href="{href}" target="_blank" rel="noreferrer">Open {kind}</a>"#,
                kind = escape_html(&file.kind)
            )
        };
        let _ = write!(
            out,
            r#"<article class="resource-card bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-2">
  <div class="flex items-start justify-between gap-2">
    <div>
      <div class="text-xs font-mono text-[var(--muted-foreground)]">{lane} · {kind} · {bytes} B</div>
      <h3 class="text-sm font-semibold break-all">{rel}</h3>
    </div>
    <button type="button" class="px-2 py-1 rounded-md text-[11px] font-mono border border-emerald-500/50 bg-emerald-950/60 text-emerald-300" onclick="annotateFile('{lane}','{rel_js}')">Tag / comment</button>
  </div>
  {preview}
</article>"#,
            lane = escape_html(&file.lane),
            kind = escape_html(&file.kind),
            bytes = file.bytes,
            rel = escape_html(&file.rel),
            rel_js = escape_js(&file.rel),
            preview = preview,
        );
    }
    out
}

fn render_file_options(working: &[ListedFile], delivered: &[ListedFile]) -> String {
    let mut out = String::from(r#"<option value="">Select a file</option>"#);
    for file in working.iter().chain(delivered) {
        let _ = write!(
            out,
            r#"<option value="{lane}::{rel}">{lane} / {rel}</option>"#,
            lane = escape_html(&file.lane),
            rel = escape_html(&file.rel)
        );
    }
    out
}

fn render_annotations(items: &[Annotation]) -> String {
    let mut out = String::new();
    for item in items {
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            item.tags
                .iter()
                .map(|t| {
                    format!(
                        r#"<span class="px-2 py-0.5 rounded-md bg-[var(--background)] border border-[var(--border)] text-[11px] font-mono">{}</span>"#,
                        escape_html(t)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        let links = item
            .links
            .iter()
            .map(|l| {
                format!(
                    r#"<a class="text-xs text-emerald-300 break-all" href="{url}" target="_blank" rel="noreferrer">{url}</a><span class="text-xs text-[var(--muted-foreground)]"> {note}</span>"#,
                    url = escape_html(&l.url),
                    note = escape_html(&l.note)
                )
            })
            .collect::<Vec<_>>()
            .join("<br/>");
        let rating = if item.rating == 0 {
            "unrated".to_string()
        } else {
            format!("{}/5", item.rating)
        };
        let aspect = if item.aspect.trim().is_empty() {
            String::new()
        } else {
            format!(" · aspect {}", escape_html(&item.aspect))
        };
        let _ = write!(
            out,
            r#"<article class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-4 space-y-2">
  <div class="text-xs font-mono text-[var(--muted-foreground)]">{lane} / {file}{aspect} · {rating} · {when}</div>
  <div class="flex flex-wrap gap-1">{tags}</div>
  <p class="text-sm whitespace-pre-wrap">{comment}</p>
  <div>{links}</div>
</article>"#,
            lane = escape_html(&item.lane),
            file = escape_html(&item.file),
            aspect = aspect,
            rating = rating,
            when = escape_html(&item.created_at),
            tags = tags,
            comment = escape_html(&item.comment),
            links = links,
        );
    }
    out
}

fn render_sources(paths: &[std::path::PathBuf]) -> String {
    let mut out = String::new();
    for path in paths {
        let _ = write!(
            out,
            r#"<div class="bg-[var(--card)] border border-[var(--border)] rounded-xl p-3 font-mono text-xs break-all">{}</div>"#,
            escape_html(&path.display().to_string())
        );
    }
    out
}

fn url_encode_path(rel: &str) -> String {
    rel.split('/')
        .map(|seg| {
            let mut out = String::new();
            for b in seg.bytes() {
                match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                        out.push(b as char)
                    }
                    _ => out.push_str(&format!("%{b:02X}")),
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn escape_js(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
