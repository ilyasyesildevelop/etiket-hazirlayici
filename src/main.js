const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const APP_INFO = {
  version: 'v26.05.1.3',
  developer: 'İlyas Yeşil',
  copyright: '© 2026',
  license: 'Proprietary software. All rights reserved.',
};

const S = {
  rows: [], labels: [], sel: new Set(), curIdx: 0, page: 0, pageSize: 50, zoom: 80, search: '', headerFontSize: 6,
  settings: null, editingRowIdx: null, deletedIndices: new Set()
};

document.addEventListener('DOMContentLoaded', async () => {
  S.settings = await invoke('get_default_settings');
  initUI(); refreshAll(); loadPrinters(); loadRecentFiles();
});

function initUI() {
  $('btnOpenFile').onclick = openFile;
  $('btnLoad').onclick = loadSheet;
  if ($('btnClearList')) $('btnClearList').onclick = clearList;
  $('selectAll').onchange = e => { const f = getFiltered(); f.forEach(r => e.target.checked ? S.sel.add(r.idx) : S.sel.delete(r.idx)); renderTable(); };
  $('searchInput').oninput = e => { S.search = e.target.value; S.page = 0; renderTable(); };
  $('btnPrevPage').onclick = () => { S.page--; renderTable(); };
  $('btnNextPage').onclick = () => { S.page++; renderTable(); };
  $('pageSizeSelect').onchange = e => { S.pageSize = +e.target.value; S.page = 0; renderTable(); };
  $('zoomSlider').oninput = e => { S.zoom = +e.target.value; $('zoomValue').textContent = S.zoom + '%'; renderPreview(); };
  $('btnPrevLabel').onclick = () => { if (S.curIdx > 0) { S.curIdx--; renderPreview(); highlightRow(); } };
  $('btnNextLabel').onclick = () => { if (S.curIdx < getSelLabels().length - 1) { S.curIdx++; renderPreview(); highlightRow(); } };
  
  let currentRotation = 0;
  $('btnRotate').onclick = () => {
    currentRotation = currentRotation === 90 ? 0 : 90;
    $('labelCanvas').style.transform = `rotate(${currentRotation}deg)`;
  };

  $('labelWidth').onchange = e => { S.settings.width_mm = +e.target.value; updateSize(); };
  $('labelHeight').onchange = e => { S.settings.height_mm = +e.target.value; updateSize(); };
  $('globalFont').onchange = e => { S.settings.global_font_family = e.target.value; renderPreview(); };
  $('globalColor').onchange = e => { S.settings.global_color = e.target.value; renderPreview(); };
  $('headerText').onchange = e => { S.settings.header_text = e.target.value; renderPreview(); };
  $('headerFontSize').onchange = e => { S.headerFontSize = +e.target.value; renderPreview(); };
  $('cariMaxWords').onchange = e => { S.settings.cari_max_words = +e.target.value; reparse(); };
  $('chkShowDate').onchange = e => { S.settings.show_date = e.target.checked; renderPreview(); };
  $('chkShowPageNo').onchange = e => { S.settings.show_page_number = e.target.checked; renderPreview(); };
  $('labelMargin').onchange = e => { S.settings.label_margin = +e.target.value; renderPreview(); };
  $('sequenceFontSize').onchange = e => { S.settings.sequence_font_size = +e.target.value; renderPreview(); };
  $('btnAutoDistribute').onclick = autoDistribute;
  $('chkSplitChar').onchange = () => reparse();
  $('chkMoveLong').onchange = () => reparse();
  $('splitCharSelect').onchange = () => reparse();
  $('maxChars').onchange = () => reparse();
  $('btnPrint').onclick = printLabels;
  $('btnPDF').onclick = generatePDF;
  $('btnSaveSettings').onclick = saveSettings;
  $('btnLoadSettings').onclick = loadSettingsUI;
  $('btnResetSettings').onclick = async () => { S.settings = await invoke('get_default_settings'); refreshAll(); };
  $('btnRecent').onclick = () => $('recentDropdown').classList.toggle('hidden');
  $('btnManualLabel').onclick = () => { $('manualModal').classList.remove('hidden'); };
  $('btnCloseModal').onclick = () => { $('manualModal').classList.add('hidden'); };
  $('btnSaveManual').onclick = saveManualLabel;
  $('copies').onchange = e => S.settings.copies = +e.target.value;
  initAbout();
  document.querySelectorAll('.tab-btn').forEach(b => b.onclick = () => {
    document.querySelectorAll('.tab-btn').forEach(x => x.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(x => x.classList.remove('active'));
    b.classList.add('active'); $(b.dataset.tab).classList.add('active');
  });
  document.querySelectorAll('.align-btn').forEach(b => b.onclick = () => {
    document.querySelectorAll('.align-btn').forEach(x => x.classList.remove('active'));
    b.classList.add('active'); S.settings.alignment = b.dataset.align; renderPreview();
  });
  document.addEventListener('click', e => { if (!e.target.closest('#btnRecent,#recentDropdown')) $('recentDropdown').classList.add('hidden'); });
  updateSize();
  initColumnResize();
  initDragAndDrop();
}

// ===== DRAG AND DROP =====
function initDragAndDrop() {
  // 1. Tauri Native Events (Tauri v2)
  if (listen) {
    listen('tauri://drag-enter', () => document.body.classList.add('drag-over'));
    listen('tauri://drag-leave', () => document.body.classList.remove('drag-over'));
    listen('tauri://drag-drop', async (event) => {
      document.body.classList.remove('drag-over');
      const paths = event.payload?.paths || event.payload; 
      if (paths && Array.isArray(paths) && paths.length > 0) {
        const path = paths[0];
        if (path.toLowerCase().endsWith('.xlsx') || path.toLowerCase().endsWith('.xls')) {
          await handleFileSelect(path);
        } else {
          setStatus('error', 'Lütfen geçerli bir Excel (.xlsx, .xls) dosyası sürükleyin.');
        }
      }
    });
  }

  // 2. Fallback / HTML5 Drag and Drop Events
  document.addEventListener('dragover', (e) => {
    e.preventDefault();
    document.body.classList.add('drag-over');
  });
  document.addEventListener('dragleave', (e) => {
    e.preventDefault();
    if (!e.clientX && !e.clientY) {
      document.body.classList.remove('drag-over');
    }
  });
  document.addEventListener('drop', async (e) => {
    e.preventDefault();
    document.body.classList.remove('drag-over');
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      if (file.path && (file.path.toLowerCase().endsWith('.xlsx') || file.path.toLowerCase().endsWith('.xls'))) {
        await handleFileSelect(file.path);
      }
    }
  });
}

// ===== COLUMN RESIZE =====
function initColumnResize() {
  const table = $('dataTable');
  table.style.tableLayout = 'fixed';
  document.querySelectorAll('#dataTable th .resize-handle').forEach(handle => {
    let startX, startW, th;
    handle.addEventListener('mousedown', e => {
      e.preventDefault();
      th = handle.parentElement;
      startX = e.pageX;
      startW = th.offsetWidth;
      handle.classList.add('active');
      const onMove = ev => { th.style.width = Math.max(40, startW + ev.pageX - startX) + 'px'; };
      const onUp = () => { handle.classList.remove('active'); document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp); };
      document.addEventListener('mousemove', onMove);
      document.addEventListener('mouseup', onUp);
    });
  });
}

function initAbout() {
  const title = document.querySelector('.app-title');
  if (title && !title.querySelector('.app-version')) {
    title.insertAdjacentHTML('beforeend', ` <span class="app-version">${APP_INFO.version}</span>`);
  }

  const actionPanel = document.querySelector('.action-panel');
  if (actionPanel && !$('btnAbout')) {
    actionPanel.insertAdjacentHTML('beforeend',
      '<button id="btnAbout" class="btn btn-ghost"><span class="material-icons-round">info</span> Hakkında</button>');
  }

  const statusRight = document.querySelector('.status-right');
  if (statusRight && !$('statusAbout')) {
    statusRight.insertAdjacentHTML('beforeend',
      `<span class="status-sep">|</span><button id="statusAbout" class="status-about" type="button">Developed by ${APP_INFO.developer} · ${APP_INFO.copyright}</button>`);
  }

  if (!$('aboutModal')) {
    document.body.insertAdjacentHTML('beforeend', `
      <div id="aboutModal" class="modal-backdrop hidden" role="dialog" aria-modal="true" aria-labelledby="aboutTitle">
        <div class="about-dialog">
          <div class="about-header">
            <div class="about-title-group">
              <span class="material-icons-round">label</span>
              <div>
                <h2 id="aboutTitle">Etiket Hazırlayıcı</h2>
                <p>${APP_INFO.version}</p>
              </div>
            </div>
            <button id="btnCloseAbout" class="icon-btn" type="button" aria-label="Kapat">
              <span class="material-icons-round">close</span>
            </button>
          </div>
          <div class="about-body">
            <div class="about-signature">
              <strong>Developed by ${APP_INFO.developer}</strong>
              <span>${APP_INFO.copyright}</span>
            </div>
            <dl class="about-details">
              <div><dt>Lisans</dt><dd>${APP_INFO.license}</dd></div>
              <div><dt>Telif</dt><dd>Copyright ${APP_INFO.copyright} ${APP_INFO.developer}.</dd></div>
            </dl>
          </div>
        </div>
      </div>`);
  }

  const openAbout = () => $('aboutModal').classList.remove('hidden');
  const closeAbout = () => $('aboutModal').classList.add('hidden');
  $('btnAbout').onclick = openAbout;
  $('statusAbout').onclick = openAbout;
  $('btnCloseAbout').onclick = closeAbout;
  $('aboutModal').onclick = e => { if (e.target.id === 'aboutModal') closeAbout(); };
  document.addEventListener('keydown', e => { if (e.key === 'Escape' && !$('aboutModal').classList.contains('hidden')) closeAbout(); });
}

async function handleFileSelect(path) {
  $('filePath').value = path;
  setStatus('loading', 'Dosya açılıyor...');
  try {
    const sheets = await invoke('get_sheets', { filePath: path });
    $('sheetSelect').innerHTML = sheets.map(s => `<option value="${s.name}">${s.name} (${s.row_count})</option>`).join('');
    setStatus('success', 'Dosya açıldı: ' + path.split('\\').pop());
    
    // Otomatik yükleme
    if (sheets.length > 0) {
      $('sheetSelect').selectedIndex = 0;
      await loadSheet();
    }
  } catch (e) { 
    setStatus('error', '' + e); 
  }
}

async function openFile() {
  setStatus('loading', 'Dosya seçiliyor...');
  try {
    const path = await invoke('open_file_dialog');
    if (path) {
      await handleFileSelect(path);
    } else {
      setStatus('info', 'Dosya seçimi iptal edildi.');
    }
  } catch (e) { setStatus('error', '' + e); }
}

async function loadSheet() {
  const path = $('filePath').value, sheet = $('sheetSelect').value;
  if (!path || !sheet) { setStatus('error', 'Dosya ve sayfa seçin.'); return; }
  setStatus('loading', 'Yükleniyor...');
  try {
    const data = await invoke('load_excel', { filePath: path, sheetName: sheet });
    S.rows = data.rows; 
    S.manualRows = []; 
    S.sel = new Set(data.rows.map((_, i) => i)); 
    S.deletedIndices = new Set();
    S.page = 0;
    await reparse();
    renderTable(); S.curIdx = 0; renderPreview();
    setStatus('success', `${data.total} kayıt yüklendi.`);
    $('statusRowCount').textContent = `Kayıt: ${data.total}`;
    
    // Excel verilerinden eşsiz cari ve malzeme isimlerini Firebase'e gönder
    if (typeof fbSaveCari === 'function') {
      const uCari = new Set(S.rows.map(r => (r.cari_unvan||'').trim()).filter(Boolean));
      const uMalz = new Set(S.rows.map(r => (r.malz_aciklama||'').trim()).filter(Boolean));
      uCari.forEach(c => fbSaveCari(c));
      uMalz.forEach(m => fbSaveMalz(m));
    }
  } catch (e) { setStatus('error', '' + e); }
}

async function clearList() {
  if (!confirm("Tüm listeyi temizlemek istediğinize emin misiniz?")) return;
  try { await invoke('clear_all_data'); } catch(e) { console.error("Backend temizlenemedi", e); }
  S.rows = [];
  S.manualRows = [];
  S.labels = [];
  S.sel = new Set();
  S.deletedIndices = new Set();
  S.curIdx = 0;
  S.page = 0;
  S.search = '';
  if ($('searchInput')) $('searchInput').value = '';
  if ($('filePath')) $('filePath').value = '';
  if ($('sheetSelect')) $('sheetSelect').innerHTML = '<option value="">Sayfa seçin</option>';
  renderTable();
  renderPreview();
  setStatus('info', 'Liste temizlendi.');
  $('statusRowCount').textContent = `Kayıt: 0`;
}

function getRules() {
  return { split_char: $('splitCharSelect').value, move_long_text: $('chkMoveLong').checked, max_chars: +$('maxChars').value };
}

async function reparse() {
  S.settings.satir_rules = getRules();
  S.labels = await invoke('parse_all_labels', { rules: S.settings.satir_rules, cariMaxWords: S.settings.cari_max_words });
  renderPreview(); updateExample();
}

// Manuel kayıtları frontend state'inde tutuyoruz (önizleme/tablo için)
S.manualRows = S.manualRows || [];

function openModalWithData(idx, isEdit) {
  const L = S.labels[idx];
  if (!L) return;
  $('mCari').value = L.cari_unvan || '';
  $('mMalz').value = L.malz_aciklama || '';
  $('mEbat').value = L.ebat || '';
  $('mIslem').value = L.islem || '';
  $('mMetrekare').value = L.metrekare || '';
  $('mAdet').value = L.print_count || 1;
  $('mMusteri').value = L.musteri_adi || '';
  $('mDiger').value = L.diger_aciklamalar || '';
  
  S.editingRowIdx = isEdit ? idx : null;
  $('manualModal').classList.remove('hidden');
}

function editRow(idx) { openModalWithData(idx, true); }
function copyRow(idx) { openModalWithData(idx, false); }
async function deleteRow(idx) {
  if (!confirm("Bu satırı silmek istediğinize emin misiniz?")) return;
  S.deletedIndices.add(idx);
  S.sel.delete(idx);
  renderTable();
  renderPreview();
  updSel();
}

async function saveManualLabel() {
  const cari   = ($('mCari').value      || '').trim();
  const malz   = ($('mMalz').value      || '').trim();
  const ebat   = ($('mEbat').value      || '').trim();
  const islem  = ($('mIslem').value     || '').trim();
  const miktar = ($('mMetrekare').value || '').trim();
  const kopya  = Math.max(1, parseInt($('mAdet').value) || 1);
  const musteri= ($('mMusteri').value   || '').trim();
  const diger  = ($('mDiger').value     || '').trim();

  const isMetre = miktar.toLowerCase().includes('m²') || miktar.toLowerCase().includes('m2');

  const label = {
    cari_unvan:         cari,
    malz_aciklama:      malz,
    ebat:               ebat,
    islem:              islem,
    adet:               isMetre ? '' : (miktar || '1 ADET'),
    metrekare:          isMetre ? miktar : '',
    musteri_adi:        musteri,
    diger_aciklamalar:  diger,
    bekleyen_siparis:   '',
    print_count:        kopya,
  };

  try {
    await invoke('add_manual_label', { label });

    // Düzenleme moduysa eskiyi gizle
    if (S.editingRowIdx !== null) {
      S.deletedIndices.add(S.editingRowIdx);
      S.sel.delete(S.editingRowIdx);
      S.editingRowIdx = null;
    }

    // Frontend listesini güncelle
    const manualIdx = S.rows.length + S.manualRows.length;
    S.manualRows.push({
      cari_unvan:       cari,
      malz_aciklama:    malz,
      satir_aciklama:   islem,
      bekleyen_siparis: miktar,
      isManual:         true,
      idx:              manualIdx,
    });
    S.sel.add(manualIdx);

    // Backend'den temiz label listesini çek
    S.labels = await invoke('parse_all_labels', { rules: S.settings.satir_rules, cariMaxWords: S.settings.cari_max_words });
    renderTable();
    renderPreview();
    updSel();

    // Formu temizle ve kapat
    ['mCari','mMalz','mEbat','mIslem','mMetrekare','mMusteri','mDiger'].forEach(id => { $(id).value = ''; });
    $('mAdet').value = '1';
    $('manualModal').classList.add('hidden');
    setStatus('success', `Manuel etiket eklendi: ${cari || malz || '(isimsiz)'}`);
    if (cari) fbSaveCari(cari);
    if (malz) fbSaveMalz(malz);
    if (islem && typeof fbSaveIslem === 'function') fbSaveIslem(islem);
  } catch(e) { setStatus('error', '' + e); }
}

function getFiltered() {
  const baseRows = S.rows.map((r, i) => ({ ...r, idx: i }));
  const manualRows = (S.manualRows || []).map(r => ({ ...r }));
  const allRows = [...baseRows, ...manualRows].filter(r => !S.deletedIndices.has(r.idx));
  if (!S.search) return allRows;
  const t = S.search.toLowerCase();
  return allRows.filter(r => (r.cari_unvan + r.malz_aciklama + r.satir_aciklama).toLowerCase().includes(t));
}


function renderTable() {
  const f = getFiltered(), tp = Math.max(1, Math.ceil(f.length / S.pageSize));
  S.page = Math.min(S.page, tp - 1);
  const start = S.page * S.pageSize, pg = f.slice(start, start + S.pageSize);
  const tb = $('tableBody');
  if (!pg.length) { tb.innerHTML = '<tr class="empty-row"><td colspan="7"><div class="empty-state"><span class="material-icons-round">search_off</span><p>Kayıt yok</p></div></td></tr>'; }
  else {
    tb.innerHTML = pg.map(r => {
      const numBadge = r.isManual ? `<span style="background:var(--secondary);color:white;padding:2px 4px;border-radius:4px;font-size:10px;">Elle</span>` : `${r.idx+1}`;
      return `<tr class="${S.sel.has(r.idx)?'selected':''} ${S.curIdx===r.idx?'active':''}" data-idx="${r.idx}">
      <td class="col-check"><input type="checkbox" ${S.sel.has(r.idx)?'checked':''}/></td>
      <td class="col-num">${numBadge}</td><td class="col-cari" title="${esc(r.cari_unvan)}">${esc(r.cari_unvan)}</td>
      <td class="col-malz" title="${esc(r.malz_aciklama)}">${esc(r.malz_aciklama)}</td>
      <td class="col-satir" title="${esc(r.satir_aciklama)}">${esc(r.satir_aciklama)}</td>
      <td class="col-bekleyen">${esc(r.bekleyen_siparis)}</td>
      <td class="col-actions">
        <button class="btn-icon" onclick="event.stopPropagation();editRow(${r.idx})" title="Düzenle"><span class="material-icons-round" style="font-size:16px;">edit</span></button><button class="btn-icon" onclick="event.stopPropagation();copyRow(${r.idx})" title="Kopyala"><span class="material-icons-round" style="font-size:16px;">content_copy</span></button><button class="btn-icon" onclick="event.stopPropagation();deleteRow(${r.idx})" style="color:#d32f2f;" title="Sil"><span class="material-icons-round" style="font-size:16px;">delete</span></button>
      </td>
      </tr>`;
    }).join('');
    tb.querySelectorAll('tr[data-idx]').forEach(tr => {
      const idx = +tr.dataset.idx;
      tr.querySelector('input').onchange = e => { e.target.checked ? S.sel.add(idx) : S.sel.delete(idx); tr.classList.toggle('selected', e.target.checked); updSel(); };
      tr.onclick = e => { if (e.target.type === 'checkbox' || e.target.closest('.btn-icon')) return; S.curIdx = idx; renderPreview(); highlightRow(); };
    });
  }
  $('btnPrevPage').disabled = S.page === 0; $('btnNextPage').disabled = S.page >= tp - 1;
  $('pageInfo').textContent = `${S.page+1} / ${tp}`; $('totalInfo').textContent = `Toplam: ${f.length} kayıt`; updSel();
}

function highlightRow() {
  $('tableBody').querySelectorAll('tr').forEach(t => t.classList.remove('active'));
  const tr = $('tableBody').querySelector(`tr[data-idx="${S.curIdx}"]`);
  if (tr) tr.classList.add('active');
}

function updSel() {
  $('selectionInfo').textContent = `Seçili: ${S.sel.size} / ${S.rows.length}`;
  $('statusSelected').textContent = `Seçili: ${S.sel.size}`;
  const sl = getSelLabels();
  $('labelNavInfo').textContent = sl.length ? `${Math.min(S.curIdx+1, sl.length)} / ${sl.length}` : '0 / 0';
}

function getSelLabels() { return S.labels.filter((_, i) => S.sel.has(i) && !S.deletedIndices.has(i)); }

function getPrintLabels() {
  const selRows = getSelLabels();
  let printList = [];
  selRows.forEach(L => {
    let count = L.print_count || 1;
    if (count < 1) count = 1;
    for (let j = 1; j <= count; j++) {
      let newL = Object.assign({}, L);
      newL.print_idx = j;
      newL.print_total = count;
      printList.push(newL);
    }
  });
  return printList;
}

// ===== CANVAS PREVIEW =====
function renderPreview() {
  const canvas = $('labelCanvas'), ctx = canvas.getContext('2d');
  const sc = S.zoom / 100, DPI = 8;
  const W = S.settings.width_mm * DPI, H = S.settings.height_mm * DPI;
  const M = (S.settings.label_margin || 1.5) * DPI; // mm to px
  canvas.width = W; canvas.height = H;
  canvas.style.width = (W * sc * 0.55) + 'px'; canvas.style.height = (H * sc * 0.55) + 'px';
  ctx.fillStyle = '#FFF'; ctx.fillRect(0, 0, W, H);
  ctx.strokeStyle = '#000'; ctx.lineWidth = 2; ctx.strokeRect(1, 1, W - 2, H - 2);

  const labels = getSelLabels();
  if (!labels.length) { ctx.fillStyle = '#999'; ctx.font = '14px Inter'; ctx.textAlign = 'center'; ctx.fillText('Etiket seçin', W/2, H/2); updSel(); return; }
  const idx = Math.min(S.curIdx, labels.length - 1);
  const L = labels[idx]; if (!L) return;
  const font = S.settings.global_font_family, color = S.settings.global_color;
  const fs = S.settings.field_font_sizes;
  const hdrFs = S.headerFontSize || 6;
  const HEADER_H = hdrFs + 2;

  // Header: 3 fixed-width sections (left/center/right)
  ctx.fillStyle = '#333'; ctx.font = `${hdrFs}px ${font}`; ctx.textBaseline = 'top';
  const hdrW = W - M * 2;
  const hdrThird = hdrW / 3;
  if (S.settings.show_page_number) { ctx.textAlign = 'center'; ctx.fillText(`— ${idx+1} —`, M + hdrThird / 2, M + 1); }
  ctx.textAlign = 'center'; ctx.fillText(S.settings.header_text, M + hdrThird + hdrThird / 2, M + 1);
  if (S.settings.show_date) { ctx.textAlign = 'center'; ctx.fillText(`— ${todayStr()} —`, M + hdrThird * 2 + hdrThird / 2, M + 1); }
  // Header separator
  ctx.strokeStyle = '#666'; ctx.lineWidth = 0.5;
  ctx.beginPath(); ctx.moveTo(M, M + HEADER_H); ctx.lineTo(W - M, M + HEADER_H); ctx.stroke();

  // Quantity text logic (use only metrekare, or fallback)
  let miktarText = L.metrekare ? L.metrekare : "1 ADET";

  // Field columns
  const fw = S.settings.field_widths;
  const fields = [
    { text: L.cari_unvan,        pct: fw.cari_unvan,        sz: fs.cari_unvan,        bold: true,  wrap: true, wrapOnly: true },
    { text: L.malz_aciklama,     pct: fw.malz_aciklama,     sz: fs.malz_aciklama,     bold: true,  wrap: true, wrapOnly: true },
    { text: L.ebat,              pct: fw.ebat,              sz: fs.ebat,              bold: true,  wrap: false },
    { text: miktarText,          pct: fw.adet_metrekare,    sz: fs.adet_metrekare,    bold: false, wrap: true },
    { text: L.islem,             pct: fw.islem,             sz: fs.islem,             bold: true,  wrap: false },
    { text: L.musteri_adi,       pct: fw.musteri_adi,       sz: fs.musteri_adi,       bold: false, wrap: true },
    { text: L.diger_aciklamalar, pct: fw.diger_aciklamalar, sz: fs.diger_aciklamalar, bold: false, wrap: true, steppedMin: 20 },
  ];
  const total = fields.reduce((s, f) => s + f.pct, 0);
  const bodyTop = M + HEADER_H + 2;
  const bodyH = H - bodyTop - M;
  let xPos = M;
  const innerW = W - M * 2;

  fields.forEach((f, fi) => {
    const colW = (f.pct / total) * innerW;
    if (fi > 0) { ctx.strokeStyle = '#666'; ctx.lineWidth = 0.5; ctx.beginPath(); ctx.moveTo(xPos, bodyTop); ctx.lineTo(xPos, H - M); ctx.stroke(); }
    if (f.text && f.text.trim()) {
      ctx.save();
      const centerX = xPos + colW / 2;
      ctx.translate(centerX, H - M - 4);
      ctx.rotate(-Math.PI / 2);
      ctx.fillStyle = color;
      let fontSize = f.sz * 1.1;
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
      const maxTextLen = bodyH - 8;
      const minFs = (f.steppedMin || 0) * 1.1;

      if (f.wrapOnly) {
        // Cari / Malz: sadece kelime kaydırma, küçültme YOK
        ctx.font = `${f.bold ? 'bold ' : ''}${fontSize}px ${font}`;
        const lines = wordWrap(ctx, f.text, maxTextLen, 3);
        const lineH = fontSize * 1.2;
        const totalH = lines.length * lineH;
        const startY = -totalH / 2 + lineH / 2;
        lines.forEach((line, li) => ctx.fillText(line, maxTextLen / 2, startY + li * lineH));
      } else if (minFs > 0 && fontSize > minFs) {
        // Diğer Açıklamalar: sadece minFs'e kadar adımsal küçültme
        ctx.font = `${f.bold ? 'bold ' : ''}${fontSize}px ${font}`;
        const lines = wordWrap(ctx, f.text, maxTextLen, 3);
        const totalH = lines.length * fontSize * 1.2;
        if (totalH > colW - 4 || ctx.measureText(f.text).width > maxTextLen) {
          fontSize = minFs;
        }
        ctx.font = `${f.bold ? 'bold ' : ''}${fontSize}px ${font}`;
        const rewrapped = wordWrap(ctx, f.text, maxTextLen, 3);
        const finalLineH = fontSize * 1.2;
        const finalTotalH = rewrapped.length * finalLineH;
        const startY = -finalTotalH / 2 + finalLineH / 2;
        rewrapped.forEach((line, li) => ctx.fillText(line, maxTextLen / 2, startY + li * finalLineH));
      } else if (f.wrap && ctx.measureText(f.text).width > maxTextLen) {
        // Diğer wrap alanlar: kaydır, gerekirse küçült
        ctx.font = `${f.bold ? 'bold ' : ''}${fontSize}px ${font}`;
        const lines = wordWrap(ctx, f.text, maxTextLen, 3);
        const lineH = fontSize * 1.2;
        const totalH = lines.length * lineH;
        if (totalH > colW - 4) {
          fontSize = Math.max(6, fontSize * (colW - 4) / totalH);
          ctx.font = `${f.bold ? 'bold ' : ''}${fontSize}px ${font}`;
        }
        const finalLineH = fontSize * 1.2;
        const finalTotalH = lines.length * finalLineH;
        const startY = -finalTotalH / 2 + finalLineH / 2;
        lines.forEach((line, li) => ctx.fillText(line, maxTextLen / 2, startY + li * finalLineH));
      } else {
        ctx.font = `${f.bold ? 'bold ' : ''}${fontSize}px ${font}`;
        ctx.fillText(f.text, maxTextLen / 2, 0, maxTextLen);
      }
      ctx.restore();
    }
    xPos += colW;
  });

  // Draw Sequence Number (e.g. 1/10)
  let count = L.print_count || 1;
  const seqFs = S.settings.sequence_font_size || 8;
  ctx.save();
  ctx.fillStyle = '#000'; ctx.font = `bold ${seqFs}px Inter`; ctx.textAlign = 'right'; ctx.textBaseline = 'bottom';
  ctx.fillText(`1/${count}`, W - M - 8, H - M - 8);
  ctx.restore();

  updSel();
}

function wordWrap(ctx, text, maxW, maxLines) {
  const words = text.split(' '); let lines = [], curr = '';
  for (let w of words) {
    if (ctx.measureText(curr + ' ' + w).width < maxW) { curr += (curr ? ' ' : '') + w; }
    else { if (curr) lines.push(curr); curr = w; }
  }
  if (curr) lines.push(curr);
  return lines.slice(0, maxLines);
}

function todayStr() { const d = new Date(); return `${d.getDate().toString().padStart(2,'0')}.${(d.getMonth()+1).toString().padStart(2,'0')}.${d.getFullYear()}`; }

// ===== SETTINGS UI =====
function renderSliders() {
  const fw = S.settings.field_widths, fs = S.settings.field_font_sizes;
  const fields = [
    ['cari_unvan', 'CARİ ÜNVAN'], ['malz_aciklama', 'MALZ. AÇIKLAMA'], ['ebat', 'EBAT'],
    ['adet_metrekare', 'ADET / m²'], ['islem', 'İŞLEM'], ['musteri_adi', 'MÜŞTERİ ADI'], ['diger_aciklamalar', 'DİĞER AÇIKL.'],
  ];
  $('fieldWidthSliders').innerHTML = fields.map(([k, lbl]) => `
    <div class="width-slider-row">
      <label>${lbl}</label>
      <input type="range" min="3" max="40" value="${fw[k]}" data-key="${k}" class="slider-width" />
      <span class="width-value">${fw[k]}%</span>
      <input type="number" min="6" max="36" value="${fs[k]}" data-key="${k}" class="font-size-input" title="Font (pt)" />
    </div>`).join('');
  $('fieldWidthSliders').querySelectorAll('.slider-width').forEach(inp => {
    inp.oninput = () => { S.settings.field_widths[inp.dataset.key] = +inp.value; inp.nextElementSibling.textContent = inp.value + '%'; renderPreview(); };
  });
  $('fieldWidthSliders').querySelectorAll('.font-size-input').forEach(inp => {
    inp.onchange = () => { S.settings.field_font_sizes[inp.dataset.key] = +inp.value; renderPreview(); };
  });
}

function autoDistribute() {
  const keys = Object.keys(S.settings.field_widths);
  const v = Math.round(100 / keys.length);
  keys.forEach(k => S.settings.field_widths[k] = v);
  renderSliders(); renderPreview();
}

function updateSize() {
  const w = S.settings.width_mm, h = S.settings.height_mm;
  $('labelSizeInfo').textContent = `${w} × ${h} mm`;
  $('statusLabelSize').textContent = `Etiket: ${w} × ${h} mm`;
  renderPreview();
}

function updateExample() {
  if (!S.labels.length) return;
  const L = S.labels[Math.min(S.curIdx, S.labels.length - 1)];
  $('exampleOutput').innerHTML = [
    ['EBAT', L.ebat], ['ADET', L.adet], ['m²', L.metrekare], ['İŞLEM', L.islem],
    ['MÜŞTERİ', L.musteri_adi], ['DİĞER', L.diger_aciklamalar],
  ].filter(([,v]) => v).map(([k,v]) => `<div class="example-field"><span class="field-name">${k}:</span><span class="field-value">${esc(v)}</span></div>`).join('');
}

function refreshAll() {
  $('labelWidth').value = S.settings.width_mm; $('labelHeight').value = S.settings.height_mm;
  $('copies').value = S.settings.copies; $('maxChars').value = S.settings.satir_rules.max_chars;
  $('splitCharSelect').value = S.settings.satir_rules.split_char;
  $('chkMoveLong').checked = S.settings.satir_rules.move_long_text;
  $('globalFont').value = S.settings.global_font_family; $('globalColor').value = S.settings.global_color;
  $('headerText').value = S.settings.header_text; $('cariMaxWords').value = S.settings.cari_max_words;
  $('chkShowDate').checked = S.settings.show_date; $('chkShowPageNo').checked = S.settings.show_page_number;
  if ($('labelMargin')) $('labelMargin').value = S.settings.label_margin || 1.5;
  if ($('sequenceFontSize')) $('sequenceFontSize').value = S.settings.sequence_font_size || 8;
  renderSliders(); updateSize(); reparse();
}

// ===== ACTIONS =====
async function generatePDF() {
  const labels = getSelLabels();
  if (!labels.length) { setStatus('error', 'Etiket seçin.'); return; }
  setStatus('loading', 'PDF oluşturuluyor...');
  const w = S.settings.width_mm, h = S.settings.height_mm;
  const mg = S.settings.label_margin || 1.5;
  const seqFs = S.settings.sequence_font_size || 8;
  let html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>Etiketler</title><style>@page{size:${w}mm ${h}mm;margin:0}body{margin:0;font-family:${S.settings.global_font_family}}
  * { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  .label{width:${w}mm;height:${h}mm;page-break-after:always;position:relative;box-sizing:border-box;overflow:hidden;padding:${mg}mm;background:#FFF;}
  .header{height:3mm;font-size:7pt;display:flex;justify-content:space-between;align-items:center;border-bottom:0.2mm solid #666;margin-bottom:0.5mm}
  .body{display:flex;height:calc(100% - 4.5mm)}  .col{border-right:0.2mm solid #666;display:flex;align-items:center;justify-content:center;overflow:hidden;padding:0.5mm}
  .col:last-child{border-right:none}
  .col span{writing-mode:vertical-rl;text-orientation:mixed;transform:rotate(180deg);text-align:center;word-break:break-word;line-height:1.2}
  </style></head><body>`;
  const fw = S.settings.field_widths, fs = S.settings.field_font_sizes;
  const total = Object.values(fw).reduce((a,b)=>a+b,0);
  
  const printLabels = getPrintLabels();
  printLabels.forEach((L, i) => {
    let miktarText = L.metrekare ? L.metrekare : "1 ADET";
    // Diğer açıklamalar için adımsal font boyutu: uzunsa min 20'ye düşür, daha küçük yapma
    const digerText = L.diger_aciklamalar || '';
    const digerDefaultSz = fs.diger_aciklamalar;
    const digerMinSz = 20;
    const digerSz = (digerText.length > 50 && digerDefaultSz > digerMinSz) ? digerMinSz : digerDefaultSz;
    const flds = [
      [L.cari_unvan, fw.cari_unvan, fs.cari_unvan, true],
      [L.malz_aciklama, fw.malz_aciklama, fs.malz_aciklama, true],
      [L.ebat, fw.ebat, fs.ebat, true],
      [miktarText, fw.adet_metrekare, fs.adet_metrekare, false],
      [L.islem, fw.islem, fs.islem, true],
      [L.musteri_adi, fw.musteri_adi, fs.musteri_adi, false],
      [digerText, fw.diger_aciklamalar, digerSz, false],
    ];
    html += `<div class="label"><div class="header"><span>${S.settings.show_page_number?'— '+(i+1)+' —':''}</span><span>${S.settings.header_text}</span><span>${S.settings.show_date?'— '+todayStr()+' —':''}</span></div><div class="body">`;
    flds.forEach(([txt,pct,sz,bold]) => {
      const wPct = (pct/total*100).toFixed(1);
      html += `<div class="col" style="width:${wPct}%"><span style="font-size:${sz*0.4}pt;${bold?'font-weight:bold':''}">${esc(txt||'')}</span></div>`;
    });
    // Add sequence number at bottom right
    html += `<span style="position:absolute; bottom:1.5mm; right:3mm; font-size:${seqFs}pt; font-weight:bold; color:#000;">${L.print_idx}/${L.print_total}</span>`;
    html += '</div></div>';
  });
  html += '</body></html>';
  try {
    const sheetName = $('sheetSelect').value || 'etiketler';
    const path = await invoke('open_html_in_browser', { htmlContent: html, sheetName: sheetName });
    setStatus('success', 'PDF oluşturuldu: ' + path);
  } catch (e) { setStatus('error', '' + e); }
}

async function printLabels() {
  const labels = getPrintLabels();
  if (!labels.length) { setStatus('error', 'Etiket seçin.'); return; }
  const printer = $('printerSelect').value;
  if (!printer) { setStatus('error', 'Yazıcı seçin.'); return; }
  setStatus('loading', 'Yazdırılıyor...');
  try {
    const pplb = await invoke('generate_pplb', { labels, settingsData: S.settings });
    const result = await invoke('send_to_printer', { printerName: printer, pplbData: pplb });
    setStatus('success', result);
  } catch (e) { setStatus('error', '' + e); }
}

async function saveSettings() {
  try {
    const path = await invoke('save_settings_to_file', { settingsData: S.settings });
    setStatus('success', 'Ayarlar kaydedildi: ' + path.split('\\').pop());
  } catch (e) {
    if (e !== "İptal edildi") setStatus('error', '' + e);
  }
}

async function loadSettingsUI() {
  try {
    const data = await invoke('load_settings_from_file');
    S.settings = data;
    refreshAll();
    setStatus('success', 'Ayarlar yüklendi.');
  } catch (e) {
    if (e !== "İptal edildi") setStatus('error', '' + e);
  }
}

async function loadPrinters() {
  try {
    const p = await invoke('list_printers');
    $('printerSelect').innerHTML = '<option value="">Yazıcı seçin...</option>' + p.map(n => `<option value="${n}">${n}</option>`).join('');
  } catch (e) { console.error(e); }
}

async function loadRecentFiles() {
  try {
    const files = await invoke('get_recent_files');
    const dd = $('recentDropdown');
    dd.innerHTML = files.length ? files.map(f => `<div class="dropdown-item" data-path="${esc(f)}"><span class="material-icons-round">description</span>${esc(f.split('\\').pop())}</div>`).join('')
      : '<div class="dropdown-item"><span class="material-icons-round">info</span>Son dosya yok</div>';
    dd.querySelectorAll('[data-path]').forEach(item => {
      item.onclick = () => { $('filePath').value = item.dataset.path; dd.classList.add('hidden');
        invoke('get_sheets', { filePath: item.dataset.path }).then(s => { $('sheetSelect').innerHTML = s.map(x => `<option value="${x.name}">${x.name}</option>`).join(''); }); };
    });
  } catch (e) { console.error(e); }
}

function $(id) { return document.getElementById(id); }
function esc(s) { if (!s) return ''; const d = document.createElement('div'); d.textContent = s; return d.innerHTML.replace(/\n/g, '<br>'); }
function setStatus(t, m) { const i = $('statusIcon'); i.className = 'material-icons-round status-icon ' + t; i.textContent = t==='success'?'check_circle':t==='error'?'error':'sync'; $('statusText').textContent = m; }

// ===== FİREBASE =====
const _fbConfig = {
  apiKey: "AIzaSyBOHE4GoBfPXA6wYLVzXtr0Oc-uo2pSlMg",
  authDomain: "etiket-360.firebaseapp.com",
  projectId: "etiket-360",
  storageBucket: "etiket-360.firebasestorage.app",
  messagingSenderId: "602084004823",
  appId: "1:602084004823:web:fe093762bab8af062ef192",
  measurementId: "G-SKLNE0V7FL"
};

let _db = null;

function fbInit() {
  try {
    if (typeof firebase === 'undefined') return;
    if (!firebase.apps.length) firebase.initializeApp(_fbConfig);
    _db = firebase.firestore();
    // Offline persistence (IndexedDB cache)
    _db.enablePersistence({ synchronizeTabs: true }).catch(() => {});
    fbEnsureCollections().then(() => fbLoadSuggestions());
    console.log('[Firebase] Bağlandı:', _fbConfig.projectId);
  } catch (e) {
    console.warn('[Firebase] Başlatılamadı:', e);
  }
}

// Koleksiyonlar yoksa sentinel belge ile oluştur
async function fbEnsureCollections() {
  if (!_db) return;
  try {
    const collections = [
      { name: 'CariList',    sentinel: { name: '_init', _sentinel: true, createdAt: firebase.firestore.FieldValue.serverTimestamp() } },
      { name: 'MalzemeList', sentinel: { name: '_init', _sentinel: true, createdAt: firebase.firestore.FieldValue.serverTimestamp() } },
      { name: 'IslemList',   sentinel: { name: '_init', _sentinel: true, createdAt: firebase.firestore.FieldValue.serverTimestamp() } },
    ];
    for (const col of collections) {
      const ref = _db.collection(col.name).doc('_init');
      const snap = await ref.get();
      if (!snap.exists) {
        await ref.set(col.sentinel);
        console.log(`[Firebase] '${col.name}' koleksiyonu oluşturuldu.`);
        
        // IslemList için statik verileri ekle
        if (col.name === 'IslemList') {
          const staticIslemler = [
            'OVERLOK', 'SAÇAK', 'SPOR SAÇAK', 'KATLAMA', 'OVAL', 'OVAL OVERLOK', 
            'OVAL SAÇAK', 'BRD', 'BORDÜR', 'KARE', 'KARE OVERLOK', 'KARE SAÇAK'
          ];
          for (const islem of staticIslemler) {
            await fbSaveIslem(islem);
          }
        }
      }
    }
  } catch (e) {
    console.warn('[Firebase] Koleksiyon oluşturma hatası:', e);
  }
}

async function fbSaveCari(name) {
  if (!_db || !name) return;
  const key = name.trim().toUpperCase();
  try {
    await _db.collection('CariList').doc(key).set({ name: key, updatedAt: firebase.firestore.FieldValue.serverTimestamp() }, { merge: true });
  } catch (e) { console.warn('[Firebase] CariList kayıt hatası:', e); }
}

async function fbSaveMalz(name) {
  if (!_db || !name) return;
  const key = name.trim().toUpperCase();
  try {
    await _db.collection('MalzemeList').doc(key).set({ name: key, updatedAt: firebase.firestore.FieldValue.serverTimestamp() }, { merge: true });
  } catch (e) { console.warn('[Firebase] MalzemeList kayıt hatası:', e); }
}

async function fbSaveIslem(name) {
  if (!_db || !name) return;
  const key = name.trim().toUpperCase();
  try {
    await _db.collection('IslemList').doc(key).set({ name: key, updatedAt: firebase.firestore.FieldValue.serverTimestamp() }, { merge: true });
  } catch (e) { console.warn('[Firebase] IslemList kayıt hatası:', e); }
}

async function fbLoadSuggestions() {
  if (!_db) return;
  try {
    const cariSnap = await _db.collection('CariList').orderBy('name').get();
    const cariDL = $('cariDatalist');
    if (cariDL) cariDL.innerHTML = cariSnap.docs.map(d => `<option value="${d.data().name}"></option>`).join('');

    const malzSnap = await _db.collection('MalzemeList').orderBy('name').get();
    const malzDL = $('malzDatalist');
    if (malzDL) malzDL.innerHTML = malzSnap.docs.map(d => `<option value="${d.data().name}"></option>`).join('');

    const islemSnap = await _db.collection('IslemList').orderBy('name').get();
    const islemDL = $('islemDatalist');
    if (islemDL) islemDL.innerHTML = islemSnap.docs.map(d => `<option value="${d.data().name}"></option>`).join('');

    console.log(`[Firebase] ${cariSnap.size} cari, ${malzSnap.size} malzeme, ${islemSnap.size} işlem önerisi yüklendi.`);
  } catch (e) { console.warn('[Firebase] Öneri yükleme hatası:', e); }
}

// Firebase'i DOM yüklendiğinde başlat
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', fbInit);
} else {
  fbInit();
}
