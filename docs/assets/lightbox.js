/**
 * lightbox.js — click-to-enlarge images inside <main>.
 *
 * Vanilla shell + `panzoom` CDN library for wheel/drag/pinch zoom-pan.
 * Dependencies: window.panzoom (loaded via CDN before this script).
 *
 * Behaviour:
 *   - Click any <main img> → modal opens with image, alt text as caption.
 *   - Inside modal: wheel zooms, drag pans, pinch on touch.
 *   - Dismiss: ESC key, click backdrop, click close button.
 *   - Focus restored to triggering image on close.
 *   - Body scroll locked while open.
 */

(function () {
  'use strict';

  // ── Build modal DOM once ─────────────────────────────────────────────────
  const modal = document.createElement('div');
  modal.className = 'lightbox';
  modal.setAttribute('role', 'dialog');
  modal.setAttribute('aria-modal', 'true');
  modal.setAttribute('aria-hidden', 'true');
  modal.innerHTML = `
    <button class="lightbox-close" aria-label="Close (Esc)" type="button">
      <i class="ph ph-x" aria-hidden="true"></i>
    </button>
    <div class="lightbox-stage">
      <div class="lightbox-pan">
        <img class="lightbox-img" alt="">
      </div>
    </div>
    <div class="lightbox-caption" aria-live="polite"></div>
  `;
  document.body.appendChild(modal);

  const stage    = modal.querySelector('.lightbox-stage');
  const panEl    = modal.querySelector('.lightbox-pan');
  const imgEl    = modal.querySelector('.lightbox-img');
  const captionEl = modal.querySelector('.lightbox-caption');
  const closeBtn = modal.querySelector('.lightbox-close');

  let panzoomInstance = null;
  let lastTrigger = null;

  // ── Open / Close ─────────────────────────────────────────────────────────
  function open(src, alt, triggerEl) {
    imgEl.src = src;
    imgEl.alt = alt || '';
    captionEl.textContent = alt || '';
    captionEl.style.display = alt ? '' : 'none';

    lastTrigger = triggerEl;
    modal.classList.add('lightbox-open');
    modal.setAttribute('aria-hidden', 'false');
    document.body.style.overflow = 'hidden';

    // Wait one frame for layout, then attach panzoom
    requestAnimationFrame(() => {
      if (window.panzoom) {
        panzoomInstance = window.panzoom(panEl, {
          maxZoom: 6,
          minZoom: 0.5,
          bounds: false,
          smoothScroll: false,
          zoomDoubleClickSpeed: 1, // disable default double-click (we handle below)
        });
      }
      closeBtn.focus();
    });
  }

  function close() {
    modal.classList.remove('lightbox-open');
    modal.setAttribute('aria-hidden', 'true');
    document.body.style.overflow = '';
    if (panzoomInstance) {
      panzoomInstance.dispose();
      panzoomInstance = null;
    }
    imgEl.src = '';
    if (lastTrigger && typeof lastTrigger.focus === 'function') {
      lastTrigger.focus();
    }
    lastTrigger = null;
  }

  // ── Wire triggers ────────────────────────────────────────────────────────
  // Event delegation on <main> — covers images added later (i18n template clones).
  document.addEventListener('click', (e) => {
    const target = e.target;
    if (!(target instanceof HTMLImageElement)) return;
    if (!target.closest('main')) return;
    if (target.classList.contains('no-zoom')) return;
    if (!target.src) return;
    e.preventDefault();
    open(target.currentSrc || target.src, target.alt, target);
  });

  // ── Dismiss handlers ─────────────────────────────────────────────────────
  closeBtn.addEventListener('click', close);
  modal.addEventListener('click', (e) => {
    // Only close on backdrop click — not on image, caption, or close button
    if (e.target === modal || e.target === stage) close();
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && modal.classList.contains('lightbox-open')) close();
  });

  // ── Double-click toggles zoom (manual; panzoom's built-in disabled above) ─
  panEl.addEventListener('dblclick', (e) => {
    if (!panzoomInstance) return;
    const transform = panzoomInstance.getTransform();
    if (transform.scale > 1.5) {
      // Already zoomed: reset to 1x at center
      panzoomInstance.zoomAbs(stage.clientWidth / 2, stage.clientHeight / 2, 1);
      panzoomInstance.moveTo(0, 0);
    } else {
      // Zoom to 2x at click point
      const rect = panEl.getBoundingClientRect();
      panzoomInstance.smoothZoom(e.clientX - rect.left, e.clientY - rect.top, 2);
    }
  });
})();
