/** Admin: Firebase tablo CRUD (CariList, MalzemeList, IslemList) */
const AdminData = {
  collectionId: 'CariList',
  rows: [],
  selected: new Set(),
  filter: '',
  editingId: null,
  loading: false,
};

function $(id) { return document.getElementById(id); }

function escHtml(s) {
  if (!s) return '';
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

function fbApi() { return window.EtiketFirebase; }

function currentCollectionMeta() {
  const api = fbApi();
  if (!api) return null;
  return api.collections.find((c) => c.id === AdminData.collectionId) || api.collections[0];
}

function notifyData(msg, isError) {
  if (typeof showPanelError === 'function' && typeof showPanelSuccess === 'function') {
    if (isError) showPanelError(msg);
    else showPanelSuccess(msg);
  }
}

function updateSelectionUI() {
  const n = AdminData.selected.size;
  const info = $('adminDataSelectionInfo');
  const delBtn = $('btnAdminDataDeleteSelected');
  if (info) info.textContent = `${n} seçili`;
  if (delBtn) delBtn.disabled = n === 0;

  const visible = getFilteredRows();
  const selectAll = $('adminDataSelectAll');
  if (selectAll && visible.length > 0) {
    selectAll.checked = visible.every((r) => AdminData.selected.has(r.id));
    selectAll.indeterminate = !selectAll.checked && visible.some((r) => AdminData.selected.has(r.id));
  } else if (selectAll) {
    selectAll.checked = false;
    selectAll.indeterminate = false;
  }
}

function getFilteredRows() {
  const q = AdminData.filter.trim().toLowerCase();
  if (!q) return AdminData.rows;
  return AdminData.rows.filter((r) => r.name.toLowerCase().includes(q));
}

function renderDataTable() {
  const tbody = $('adminDataTableBody');
  const countEl = $('adminDataCount');
  if (!tbody) return;

  const filtered = getFilteredRows();
  if (countEl) {
    const meta = currentCollectionMeta();
    countEl.textContent = filtered.length === AdminData.rows.length
      ? `${AdminData.rows.length} kayıt`
      : `${filtered.length} / ${AdminData.rows.length} kayıt`;
  }

  if (AdminData.loading) {
    tbody.innerHTML = '<tr><td colspan="3" class="admin-data-empty">Yükleniyor...</td></tr>';
    return;
  }

  if (!fbApi()?.isReady()) {
    tbody.innerHTML = '<tr><td colspan="3" class="admin-data-empty">Firebase bağlantısı yok.</td></tr>';
    return;
  }

  if (!filtered.length) {
    tbody.innerHTML = '<tr><td colspan="3" class="admin-data-empty">Kayıt bulunamadı.</td></tr>';
    updateSelectionUI();
    return;
  }

  tbody.innerHTML = filtered.map((row) => {
    const checked = AdminData.selected.has(row.id) ? 'checked' : '';
    const isEdit = AdminData.editingId === row.id;
    if (isEdit) {
      return `<tr class="admin-data-row editing" data-id="${escHtml(row.id)}">
        <td class="col-check"><input type="checkbox" disabled /></td>
        <td><input type="text" class="admin-data-edit-input" data-edit-id="${escHtml(row.id)}" value="${escHtml(row.name)}" /></td>
        <td class="col-actions">
          <button type="button" class="btn-icon admin-data-save-edit" data-id="${escHtml(row.id)}" title="Kaydet"><span class="material-icons-round">check</span></button>
          <button type="button" class="btn-icon admin-data-cancel-edit" title="İptal"><span class="material-icons-round">close</span></button>
        </td>
      </tr>`;
    }
    return `<tr class="admin-data-row" data-id="${escHtml(row.id)}">
      <td class="col-check"><input type="checkbox" class="admin-data-check" data-id="${escHtml(row.id)}" ${checked} /></td>
      <td class="col-name" title="${escHtml(row.name)}">${escHtml(row.name)}</td>
      <td class="col-actions">
        <button type="button" class="btn-icon admin-data-edit" data-id="${escHtml(row.id)}" title="Düzenle"><span class="material-icons-round">edit</span></button>
        <button type="button" class="btn-icon admin-data-delete" data-id="${escHtml(row.id)}" title="Sil"><span class="material-icons-round">delete</span></button>
      </td>
    </tr>`;
  }).join('');

  updateSelectionUI();
}

async function loadDataTable() {
  const api = fbApi();
  if (!api?.isReady()) {
    AdminData.rows = [];
    AdminData.loading = false;
    renderDataTable();
    return;
  }

  AdminData.loading = true;
  AdminData.selected.clear();
  AdminData.editingId = null;
  renderDataTable();

  try {
    const snap = await api.db.collection(AdminData.collectionId).orderBy('name').get();
    AdminData.rows = snap.docs
      .filter((d) => d.id !== '_init' && !d.data()?._sentinel)
      .map((d) => ({ id: d.id, name: d.data().name || d.id }));
  } catch (e) {
    AdminData.rows = [];
    notifyData(`Tablo yüklenemedi: ${e}`, true);
  } finally {
    AdminData.loading = false;
    renderDataTable();
  }
}

async function addRecord() {
  const api = fbApi();
  const input = $('adminDataNewName');
  const name = input?.value?.trim();
  if (!api?.isReady()) {
    notifyData('Firebase bağlantısı yok.', true);
    return;
  }
  if (!name) {
    notifyData('Kayıt adı girin.', true);
    return;
  }

  const key = api.docKey(name);
  if (AdminData.rows.some((r) => r.id === key)) {
    notifyData('Bu kayıt zaten var.', true);
    return;
  }

  try {
    await api.db.collection(AdminData.collectionId).doc(key).set({
      name: key,
      updatedAt: firebase.firestore.FieldValue.serverTimestamp(),
    });
    input.value = '';
    notifyData('Kayıt eklendi.');
    await loadDataTable();
    await api.reloadSuggestions();
  } catch (e) {
    notifyData(`Eklenemedi: ${e}`, true);
  }
}

async function saveEdit(oldId) {
  const api = fbApi();
  const input = document.querySelector(`.admin-data-edit-input[data-edit-id="${CSS.escape(oldId)}"]`);
  const newName = input?.value?.trim();
  if (!api?.isReady() || !newName) {
    notifyData('Geçerli bir ad girin.', true);
    return;
  }

  const newKey = api.docKey(newName);
  try {
    if (oldId === newKey) {
      await api.db.collection(AdminData.collectionId).doc(newKey).set({
        name: newKey,
        updatedAt: firebase.firestore.FieldValue.serverTimestamp(),
      }, { merge: true });
    } else {
      if (AdminData.rows.some((r) => r.id === newKey)) {
        notifyData('Bu isimde kayıt zaten var.', true);
        return;
      }
      const batch = api.db.batch();
      const col = api.db.collection(AdminData.collectionId);
      batch.set(col.doc(newKey), {
        name: newKey,
        updatedAt: firebase.firestore.FieldValue.serverTimestamp(),
      });
      batch.delete(col.doc(oldId));
      await batch.commit();
      if (AdminData.selected.has(oldId)) {
        AdminData.selected.delete(oldId);
        AdminData.selected.add(newKey);
      }
    }
    AdminData.editingId = null;
    notifyData('Kayıt güncellendi.');
    await loadDataTable();
    await api.reloadSuggestions();
  } catch (e) {
    notifyData(`Güncellenemedi: ${e}`, true);
  }
}

async function deleteRecords(ids) {
  const api = fbApi();
  if (!api?.isReady() || !ids.length) return;

  const meta = currentCollectionMeta();
  const label = meta?.singular || 'kayıt';
  if (!confirm(`${ids.length} ${label} silinsin mi? Bu işlem geri alınamaz.`)) return;

  try {
    const col = api.db.collection(AdminData.collectionId);
    const chunks = [];
    for (let i = 0; i < ids.length; i += 400) {
      chunks.push(ids.slice(i, i + 400));
    }
    for (const chunk of chunks) {
      const batch = api.db.batch();
      chunk.forEach((id) => batch.delete(col.doc(id)));
      await batch.commit();
    }
    ids.forEach((id) => AdminData.selected.delete(id));
    notifyData(`${ids.length} kayıt silindi.`);
    await loadDataTable();
    await api.reloadSuggestions();
  } catch (e) {
    notifyData(`Silinemedi: ${e}`, true);
  }
}

function switchAdminPanel(panel) {
  const card = document.querySelector('#licenseOverlay .license-card');
  const isData = panel === 'data';
  $('adminPanelSettings')?.classList.toggle('hidden', isData);
  $('adminPanelData')?.classList.toggle('hidden', !isData);
  document.querySelectorAll('.admin-nav-tab').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.adminPanel === panel);
  });
  card?.classList.toggle('license-card-wide', isData);
  card?.classList.toggle('license-card-admin', true);
  if (isData) loadDataTable();
}

function populateCollectionSelect() {
  const sel = $('adminDataCollection');
  const api = fbApi();
  if (!sel || !api) return;
  sel.innerHTML = api.collections.map((c) =>
    `<option value="${escHtml(c.id)}">${escHtml(c.label)}</option>`
  ).join('');
  sel.value = AdminData.collectionId;
}

function onTableClick(e) {
  const editBtn = e.target.closest('.admin-data-edit');
  const delBtn = e.target.closest('.admin-data-delete');
  const saveBtn = e.target.closest('.admin-data-save-edit');
  const cancelBtn = e.target.closest('.admin-data-cancel-edit');
  const check = e.target.closest('.admin-data-check');

  if (saveBtn) {
    saveEdit(saveBtn.dataset.id);
    return;
  }
  if (cancelBtn) {
    AdminData.editingId = null;
    renderDataTable();
    return;
  }
  if (editBtn) {
    AdminData.editingId = editBtn.dataset.id;
    renderDataTable();
    const inp = document.querySelector('.admin-data-edit-input');
    inp?.focus();
    inp?.select();
    return;
  }
  if (delBtn) {
    deleteRecords([delBtn.dataset.id]);
    return;
  }
  if (check) {
    const id = check.dataset.id;
    if (check.checked) AdminData.selected.add(id);
    else AdminData.selected.delete(id);
    updateSelectionUI();
  }
}

function initAdminDataUI() {
  populateCollectionSelect();

  $('adminDataCollection')?.addEventListener('change', (e) => {
    AdminData.collectionId = e.target.value;
    AdminData.filter = '';
    if ($('adminDataSearch')) $('adminDataSearch').value = '';
    loadDataTable();
  });

  $('adminDataSearch')?.addEventListener('input', (e) => {
    AdminData.filter = e.target.value;
    renderDataTable();
  });

  $('btnAdminDataAdd')?.addEventListener('click', addRecord);
  $('adminDataNewName')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') { e.preventDefault(); addRecord(); }
  });

  $('adminDataSelectAll')?.addEventListener('change', (e) => {
    const filtered = getFilteredRows();
    if (e.target.checked) filtered.forEach((r) => AdminData.selected.add(r.id));
    else filtered.forEach((r) => AdminData.selected.delete(r.id));
    renderDataTable();
  });

  $('btnAdminDataDeleteSelected')?.addEventListener('click', () => {
    deleteRecords([...AdminData.selected]);
  });

  $('btnAdminDataRefresh')?.addEventListener('click', () => loadDataTable());

  $('adminDataTableBody')?.addEventListener('click', onTableClick);
  $('adminDataTableBody')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && e.target.classList.contains('admin-data-edit-input')) {
      e.preventDefault();
      saveEdit(e.target.dataset.editId);
    }
    if (e.key === 'Escape' && AdminData.editingId) {
      AdminData.editingId = null;
      renderDataTable();
    }
  });

  document.querySelectorAll('.admin-nav-tab').forEach((btn) => {
    btn.addEventListener('click', () => switchAdminPanel(btn.dataset.adminPanel));
  });

}

window.AdminData = AdminData;
window.switchAdminPanel = switchAdminPanel;
window.initAdminDataUI = initAdminDataUI;
window.loadAdminDataTable = loadDataTable;

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initAdminDataUI);
} else {
  initAdminDataUI();
}
