/**
 * Accessibility, Persona Modality & Remote Gateway Manager
 * Controls:
 * - Audience Personas: Layperson / Public, Subject Matter Expert (SME), Ontologist / Developer
 * - Sensory Accessibility: OLED High Contrast Dark, Clinical Paper Light, Text Scaling, Dyslexic Typography, Reduced Motion
 * - Internationalization (i18n): Multi-language & Phonological Script Flags
 * - Companion Gateway: Remote Tablet / Phone WebRTC pairing status
 */

export class AccessibilityManager {
  constructor(triggerBtn, modalContainer) {
    this.triggerBtn = triggerBtn;
    this.modal = modalContainer;

    this.state = {
      persona: 'sme',               // 'layperson' | 'sme' | 'dev'
      contrastTheme: 'default',     // 'default' | 'high-contrast-dark' | 'high-contrast-light'
      textScale: 100,               // 100 | 125 | 150 | 200
      dyslexicFont: false,          // boolean
      reducedMotion: false,         // boolean
      language: 'en',               // 'en' | 'fr' | 'zh' | 'ar' | 'phonetic'
      companionConnected: true      // simulated tablet remote gateway status
    };

    this.loadPreferences();
    this.applyAll();
    this.initEvents();
  }

  loadPreferences() {
    try {
      const saved = localStorage.getItem('hypercanvas_a11y_prefs');
      if (saved) {
        Object.assign(this.state, JSON.parse(saved));
      }
    } catch (e) {
      console.warn('Could not load accessibility preferences', e);
    }
  }

  savePreferences() {
    try {
      localStorage.setItem('hypercanvas_a11y_prefs', JSON.stringify(this.state));
    } catch (e) {
      console.warn('Could not save accessibility preferences', e);
    }
  }

  applyAll() {
    const body = document.body;

    // Remove old theme classes
    body.classList.remove(
      'a11y-high-contrast-dark',
      'a11y-high-contrast-light',
      'a11y-scale-125',
      'a11y-scale-150',
      'a11y-scale-200',
      'a11y-dyslexic',
      'a11y-reduced-motion',
      'persona-layperson',
      'persona-sme',
      'persona-dev'
    );

    // Contrast Theme
    if (this.state.contrastTheme === 'high-contrast-dark') {
      body.classList.add('a11y-high-contrast-dark');
    } else if (this.state.contrastTheme === 'high-contrast-light') {
      body.classList.add('a11y-high-contrast-light');
    }

    // Text Scale
    if (this.state.textScale === 125) body.classList.add('a11y-scale-125');
    if (this.state.textScale === 150) body.classList.add('a11y-scale-150');
    if (this.state.textScale === 200) body.classList.add('a11y-scale-200');

    // Dyslexic Font
    if (this.state.dyslexicFont) body.classList.add('a11y-dyslexic');

    // Reduced Motion
    if (this.state.reducedMotion) body.classList.add('a11y-reduced-motion');

    // Persona Modality class
    body.classList.add(`persona-${this.state.persona}`);

    // RTL for Arabic
    if (this.state.language === 'ar') {
      document.documentElement.dir = 'rtl';
    } else {
      document.documentElement.dir = 'ltr';
    }

    // Update trigger button badge
    if (this.triggerBtn) {
      const personaLabel = this.state.persona === 'layperson' ? '👤 Layperson' : (this.state.persona === 'sme' ? '🔬 SME' : '⚡ Dev');
      this.triggerBtn.innerHTML = `<span>♿</span> ${personaLabel} | a11y`;
    }
  }

  setTheme(themeKey) {
    this.state.contrastTheme = themeKey || 'default';
    this.savePreferences();
    this.applyAll();
    if (this.modal && !this.modal.classList.contains('hidden')) {
      this.renderModalContent();
    }
  }

  initEvents() {
    if (this.triggerBtn) {
      this.triggerBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.toggleModal();
      });
    }

    document.addEventListener('click', (e) => {
      if (this.modal && !this.modal.classList.contains('hidden') && !this.modal.contains(e.target) && e.target !== this.triggerBtn) {
        this.modal.classList.add('hidden');
      }
    });
  }

  toggleModal() {
    if (!this.modal) return;
    const isHidden = this.modal.classList.contains('hidden');
    if (!isHidden) {
      this.modal.classList.add('hidden');
      return;
    }

    this.renderModalContent();
    this.modal.classList.remove('hidden');
  }

  renderModalContent() {
    this.modal.innerHTML = `
      <div class="settings-modal-header">
        <span>♿ Accessibility, Persona Modalities & Remote</span>
        <button style="background:transparent; border:none; color:var(--text-secondary); cursor:pointer; font-size:14px;" id="btn-close-a11y-modal">✕</button>
      </div>

      <!-- 1. Audience Persona Modality -->
      <div class="settings-section-title">1. UI Audience Modality Persona</div>
      <div class="settings-options-group">
        <label class="settings-radio-label">
          <input type="radio" name="a11y-persona" value="layperson" ${this.state.persona === 'layperson' ? 'checked' : ''} />
          <span><strong>👤 Layperson / General Public</strong> (Visual narrative, simplified terms, hidden IRIs)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="a11y-persona" value="sme" ${this.state.persona === 'sme' ? 'checked' : ''} />
          <span><strong>🔬 Subject Matter Expert (SME)</strong> (Hydrology flow, pathology biomarkers, legal duty)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="a11y-persona" value="dev" ${this.state.persona === 'dev' ? 'checked' : ''} />
          <span><strong>⚡ Ontologist & Core Dev</strong> (Raw RDF quads, VibeScript AST, gas accounting)</span>
        </label>
      </div>

      <!-- 2. Visual Contrast & Lighting Theme -->
      <div class="settings-section-title">2. Visual Contrast & Lighting Theme</div>
      <div class="settings-options-group">
        <label class="settings-radio-label">
          <input type="radio" name="a11y-theme" value="default" ${this.state.contrastTheme === 'default' ? 'checked' : ''} />
          <span>🌌 <strong>Cyber-Semantic Slate Dark</strong> (Default Grid & Cyan/Blue Neon)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="a11y-theme" value="high-contrast-dark" ${this.state.contrastTheme === 'high-contrast-dark' ? 'checked' : ''} />
          <span>🖤 <strong>OLED Pure Black</strong> (High Contrast Dark, &gt; 7:1 Ratio)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="a11y-theme" value="high-contrast-light" ${this.state.contrastTheme === 'high-contrast-light' ? 'checked' : ''} />
          <span>☀️ <strong>Clinical Paper Light</strong> (Daylight High Contrast)</span>
        </label>
      </div>
      <div style="margin-top: 4px;">
        <button class="btn" id="btn-reset-default-theme" style="font-size:10px; padding:3px 8px; border:1px solid var(--accent-cyan); color:var(--accent-cyan); background:var(--surface-panel); border-radius:var(--radius-xs); cursor:pointer;">
          🔄 Reset to Default Cyber-Dark Theme
        </button>
      </div>

      <!-- 3. Scalable Typography & Dyslexia -->
      <div class="settings-section-title">3. Typography & Sensory Assistance</div>
      <div style="display:flex; gap:8px; align-items:center;">
        <span style="font-size:12px; color:var(--text-secondary);">Text Size:</span>
        <button class="btn a11y-scale-btn ${this.state.textScale === 100 ? 'active' : ''}" data-scale="100">100%</button>
        <button class="btn a11y-scale-btn ${this.state.textScale === 125 ? 'active' : ''}" data-scale="125">125%</button>
        <button class="btn a11y-scale-btn ${this.state.textScale === 150 ? 'active' : ''}" data-scale="150">150%</button>
        <button class="btn a11y-scale-btn ${this.state.textScale === 200 ? 'active' : ''}" data-scale="200">200%</button>
      </div>
      <div class="settings-options-group" style="margin-top:6px;">
        <label class="settings-check-label">
          <input type="checkbox" id="chk-a11y-dyslexic" ${this.state.dyslexicFont ? 'checked' : ''} />
          <span>Dyslexic-Friendly Typography (High Legibility & Spacing)</span>
        </label>
        <label class="settings-check-label">
          <input type="checkbox" id="chk-a11y-motion" ${this.state.reducedMotion ? 'checked' : ''} />
          <span>Reduced Motion Mode (Disable Animations & Orbit Momentum)</span>
        </label>
      </div>

      <!-- 4. Language & i18n -->
      <div class="settings-section-title">4. Internationalization & Scripts</div>
      <div style="display:flex; gap:6px; flex-wrap:wrap;">
        <button class="btn a11y-lang-btn ${this.state.language === 'en' ? 'active' : ''}" data-lang="en">🇬🇧 English</button>
        <button class="btn a11y-lang-btn ${this.state.language === 'fr' ? 'active' : ''}" data-lang="fr">🇫🇷 Français</button>
        <button class="btn a11y-lang-btn ${this.state.language === 'zh' ? 'active' : ''}" data-lang="zh">🇨🇳 中文</button>
        <button class="btn a11y-lang-btn ${this.state.language === 'ar' ? 'active' : ''}" data-lang="ar">🇸🇦 العربية (RTL)</button>
        <button class="btn a11y-lang-btn ${this.state.language === 'phonetic' ? 'active' : ''}" data-lang="phonetic">🔤 Phonetic / Q42</button>
      </div>

      <!-- 5. Remote Companion Gateway -->
      <div class="settings-section-title">5. Multi-Device Companion Remote Gateway</div>
      <div style="display:flex; justify-content:space-between; align-items:center; background:var(--surface-base); padding:8px 10px; border-radius:var(--radius-sm); border:1px solid var(--border-subtle);">
        <div style="display:flex; flex-direction:column; gap:2px;">
          <span style="font-size:12px; font-weight:600;">📱 Tablet / Phone Controller</span>
          <span style="font-size:10px; color:var(--accent-emerald);">● WebRTC Gateway Connected (Peer: tablet_remote_01)</span>
        </div>
        <button class="btn" id="btn-ping-companion" style="font-size:11px; padding:3px 8px;">Ping</button>
      </div>
    `;

    // Modal Close
    this.modal.querySelector('#btn-close-a11y-modal')?.addEventListener('click', () => {
      this.modal.classList.add('hidden');
    });

    // Reset Theme Button
    this.modal.querySelector('#btn-reset-default-theme')?.addEventListener('click', () => {
      this.setTheme('default');
    });

    // Persona Radio
    this.modal.querySelectorAll('input[name="a11y-persona"]').forEach(radio => {
      radio.addEventListener('change', (e) => {
        this.state.persona = e.target.value;
        this.savePreferences();
        this.applyAll();
      });
    });

    // Theme Radio
    this.modal.querySelectorAll('input[name="a11y-theme"]').forEach(radio => {
      radio.addEventListener('change', (e) => {
        this.setTheme(e.target.value);
      });
    });

    // Scale Buttons
    this.modal.querySelectorAll('.a11y-scale-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        this.state.textScale = parseInt(btn.getAttribute('data-scale'), 10);
        this.savePreferences();
        this.applyAll();
        this.renderModalContent();
      });
    });

    // Checkboxes
    this.modal.querySelector('#chk-a11y-dyslexic')?.addEventListener('change', (e) => {
      this.state.dyslexicFont = e.target.checked;
      this.savePreferences();
      this.applyAll();
    });

    this.modal.querySelector('#chk-a11y-motion')?.addEventListener('change', (e) => {
      this.state.reducedMotion = e.target.checked;
      this.savePreferences();
      this.applyAll();
    });

    // Language Buttons
    this.modal.querySelectorAll('.a11y-lang-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        this.state.language = btn.getAttribute('data-lang');
        this.savePreferences();
        this.applyAll();
        this.renderModalContent();
      });
    });

    // Companion Ping
    this.modal.querySelector('#btn-ping-companion')?.addEventListener('click', () => {
      alert('📱 Companion Remote Ping:\nLatency: 8ms\nCapabilities: 3D Gyro Trackball, Stylus Inking, Remote Tool Chest.');
    });
  }
}
