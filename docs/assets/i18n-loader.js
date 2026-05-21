/**
 * i18n-loader.js — Unwrap documentation multi-language loader
 * Vanilla JS, no dependencies, < 150 lines.
 *
 * Attributes consumed:
 *   data-i18n="key.path"          → el.textContent
 *   data-i18n-html="key.path"     → el.innerHTML
 *   data-i18n-alt="key.path"      → el.setAttribute('alt', …)
 *   data-i18n-list="key.path"     → clone <template>, fill [data-key] children per array item
 *
 * Special data-key values:
 *   "_text"  → item itself is a string (used for string arrays)
 *   "id"     → also drives screenshot src mapping
 */

const STORAGE_KEY = 'unwrap-docs-lang';
const DEFAULT_LANG = 'en';
const SUPPORTED = ['en', 'vi', 'zh-CN'];

/** Resolve a dot-path against an object. Returns '' on miss. */
function get(obj, path) {
  return path.split('.').reduce((o, k) => (o != null ? o[k] : undefined), obj) ?? '';
}

/** Screenshot src map: screen id → path */
const SCREENSHOT_SRCS = {
  'welcome':       'docs/assets/screenshots/welcome.png',
  'asset-preview': 'docs/assets/screenshots/asset-preview.png',
  'code-decompile':'docs/assets/screenshots/code-decompile.png',
  'translate':     'docs/assets/screenshots/translate.png',
};

/** Fill a cloned template's [data-key] descendants from an item. */
function fillClone(clone, item) {
  clone.querySelectorAll('[data-key]').forEach(el => {
    const key = el.dataset.key;

    if (key === '_text') {
      // item itself is a plain string
      el.textContent = typeof item === 'string' ? item : '';
      return;
    }

    const val = typeof item === 'object' && item !== null ? item[key] : '';

    // Screenshot src derived from id field — MUST run before screenshotAlt
    // (an <img> may have both data-key="screenshotAlt" and data-key-src-map="id";
    //  the src-map handler also sets alt, so this is a complete handler).
    if (el.tagName === 'IMG' && el.dataset.keySrcMap === 'id' && item.id) {
      el.src = SCREENSHOT_SRCS[item.id] ?? '';
      el.setAttribute('alt', item.screenshotAlt ?? '');
      return;
    }

    // alt attribute on <img> (used by templates that don't drive src)
    if (el.tagName === 'IMG' && key === 'screenshotAlt') {
      el.setAttribute('alt', val ?? '');
      return;
    }

    el.textContent = val ?? '';
  });
}

/** Render all data-i18n-list containers. */
function renderLists(data) {
  document.querySelectorAll('[data-i18n-list]').forEach(container => {
    const keyPath = container.dataset.i18nList;
    const items = get(data, keyPath);
    if (!Array.isArray(items)) return;

    const tmpl = container.querySelector('template');
    if (!tmpl) return;

    // Remove any previously rendered clones (keep the template)
    [...container.children].forEach(c => {
      if (c.tagName !== 'TEMPLATE') c.remove();
    });

    items.forEach(item => {
      const clone = tmpl.content.cloneNode(true);
      fillClone(clone, item);
      container.appendChild(clone);
    });
  });
}

/** Apply all i18n attributes from loaded data. */
function applyData(data, lang) {
  document.documentElement.lang = lang;
  document.title = get(data, 'meta.title') || document.title;

  document.querySelectorAll('[data-i18n]').forEach(el => {
    el.textContent = get(data, el.dataset.i18n);
  });

  document.querySelectorAll('[data-i18n-html]').forEach(el => {
    el.innerHTML = get(data, el.dataset.i18nHtml);
  });

  document.querySelectorAll('[data-i18n-alt]').forEach(el => {
    el.setAttribute('alt', get(data, el.dataset.i18nAlt));
  });

  renderLists(data);

  document.querySelectorAll('.lang-switcher [data-lang]').forEach(btn => {
    btn.setAttribute('aria-pressed', String(btn.dataset.lang === lang));
  });

  localStorage.setItem(STORAGE_KEY, lang);
}

/** Show dismissible error banner. */
function showErrorBanner() {
  if (document.querySelector('.banner-error')) return; // already shown
  const banner = document.createElement('div');
  banner.className = 'banner-error';
  banner.innerHTML =
    '<span>' +
    '<strong>Could not load language file.</strong> ' +
    'If you opened this via <code>file://</code>, run ' +
    '<code>pnpm docs:serve</code> first, then open ' +
    '<code>http://localhost:5173/docs.html</code>.' +
    '</span>' +
    '<button class="banner-dismiss" aria-label="Dismiss">' +
    '<i class="ph ph-x"></i>' +
    '</button>';
  banner.querySelector('.banner-dismiss').addEventListener('click', () => banner.remove());
  document.body.insertAdjacentElement('afterbegin', banner);
}

/** Fetch and apply a language JSON. */
async function loadLang(lang) {
  if (!SUPPORTED.includes(lang)) lang = DEFAULT_LANG;
  try {
    const res = await fetch(`docs/assets/i18n/${lang}.json`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    // Remove error banner on successful load
    document.querySelector('.banner-error')?.remove();
    applyData(data, lang);
  } catch (err) {
    console.error('[docs] i18n load failed:', err);
    showErrorBanner();
  }
}

/** Detect preferred language. Priority: ?lang= → localStorage → navigator.language → 'en'. */
function detectDefault() {
  const params = new URLSearchParams(window.location.search);
  const fromUrl = params.get('lang');
  if (fromUrl && SUPPORTED.includes(fromUrl)) return fromUrl;
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && SUPPORTED.includes(stored)) return stored;
  const nav = (navigator.language || 'en').toLowerCase();
  if (nav.startsWith('vi')) return 'vi';
  if (nav.startsWith('zh')) return 'zh-CN';
  return DEFAULT_LANG;
}

/** Highlight active TOC item via IntersectionObserver. */
function initTocObserver() {
  const sectionIds = ['intro','features','architecture','workflow','how-to-use',
                      'tech-stack','shortcuts','bundled','troubleshoot','develop','license'];
  const tocLinks = sectionIds.reduce((map, id) => {
    const a = document.querySelector(`.toc a[href="#${id}"]`);
    if (a) map[id] = a;
    return map;
  }, {});

  // Track currently-intersecting sections; pick topmost-in-DOM as the
  // single active TOC item. Prevents the "two tabs highlighted" bug when
  // adjacent sections share viewport space.
  const intersecting = new Set();
  const observer = new IntersectionObserver(entries => {
    entries.forEach(entry => {
      if (entry.isIntersecting) intersecting.add(entry.target.id);
      else intersecting.delete(entry.target.id);
    });
    const activeId = sectionIds.find(id => intersecting.has(id)) ?? null;
    Object.entries(tocLinks).forEach(([id, link]) => {
      link.classList.toggle('toc-active', id === activeId);
    });
  }, { rootMargin: `-${getComputedStyle(document.documentElement).getPropertyValue('--topbar-h') || '56px'} 0px -60% 0px` });

  sectionIds.forEach(id => {
    const el = document.getElementById(id);
    if (el) observer.observe(el);
  });
}

/** Mobile TOC toggle. */
function initTocToggle() {
  const toggle = document.querySelector('.toc-toggle');
  const toc = document.querySelector('.toc');
  if (!toggle || !toc) return;
  toggle.addEventListener('click', () => {
    const open = toggle.getAttribute('aria-expanded') === 'true';
    toggle.setAttribute('aria-expanded', String(!open));
    toc.classList.toggle('toc-open', !open);
  });
  // Collapse on link click (mobile UX)
  toc.querySelectorAll('a').forEach(a => {
    a.addEventListener('click', () => {
      toggle.setAttribute('aria-expanded', 'false');
      toc.classList.remove('toc-open');
    });
  });
}

// ── Boot ────────────────────────────────────────────────────────────────────

document.querySelectorAll('.lang-switcher [data-lang]').forEach(btn => {
  btn.addEventListener('click', () => loadLang(btn.dataset.lang));
});

initTocObserver();
initTocToggle();
loadLang(detectDefault());
