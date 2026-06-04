/** Lisans kontrolü ve yönetici paneli */
const Admin = {
  token: null,
  locked: false,
  savedUsername: '',
};

function $(id) { return document.getElementById(id); }

function showPanelError(msg) {
  const el = $('adminPanelError');
  const ok = $('adminPanelSuccess');
  if (ok) ok.classList.add('hidden');
  if (el) { el.textContent = msg; el.classList.remove('hidden'); }
}

function showPanelSuccess(msg) {
  const el = $('adminPanelSuccess');
  const err = $('adminPanelError');
  if (err) err.classList.add('hidden');
  if (el) { el.textContent = msg; el.classList.remove('hidden'); }
}

function hidePanelMessages() {
  $('adminPanelError')?.classList.add('hidden');
  $('adminPanelSuccess')?.classList.add('hidden');
  $('adminLoginError')?.classList.add('hidden');
}

function setAdminCardMode(enabled) {
  document.querySelector('#licenseOverlay .license-card')?.classList.toggle('license-card-admin', !!enabled);
}

function showLicenseOverlay(locked, message) {
  const overlay = $('licenseOverlay');
  if (!overlay) return;
  overlay.classList.remove('hidden');
  document.body.classList.toggle('license-locked', !!locked);
  Admin.locked = locked;
  setAdminCardMode(false);

  const msgEl = $('licenseLockMessage');
  if (msgEl) {
    msgEl.textContent = message || '';
    msgEl.style.display = locked ? 'block' : 'none';
  }

  $('licenseLoginView')?.classList.remove('hidden');
  $('licenseAdminView')?.classList.add('hidden');
}

function hideLicenseOverlay() {
  $('licenseOverlay')?.classList.add('hidden');
  document.body.classList.remove('license-locked');
  Admin.locked = false;
  setAdminCardMode(false);
  $('licenseLoginView')?.classList.remove('hidden');
  $('licenseAdminView')?.classList.add('hidden');
}

function showAdminPanel(data) {
  $('licenseLoginView')?.classList.add('hidden');
  $('licenseAdminView')?.classList.remove('hidden');
  setAdminCardMode(true);
  hidePanelMessages();
  if (typeof switchAdminPanel === 'function') switchAdminPanel('settings');

  const expiry = $('adminExpiryDate');
  if (expiry && data?.expiry_date) expiry.value = data.expiry_date;

  const user = $('adminNewUsername');
  if (data?.username) {
    Admin.savedUsername = data.username;
    if (user) user.value = data.username;
  }

  const cont = $('btnAdminContinue');
  if (cont) cont.classList.toggle('hidden', Admin.locked);
}

async function refreshLicenseAndMaybeUnlock() {
  const status = await window.__TAURI__.core.invoke('get_license_status');
  if (!status.is_locked) {
    if (Admin.token) {
      try {
        await window.__TAURI__.core.invoke('admin_logout', { token: Admin.token });
      } catch (_) { /* ignore */ }
      Admin.token = null;
    }
    hideLicenseOverlay();
    if (typeof window.startEtiketApp === 'function') {
      await window.startEtiketApp();
    }
    return status;
  }
  Admin.locked = true;
  const cont = $('btnAdminContinue');
  if (cont) cont.classList.add('hidden');
  return status;
}

async function onAdminLogin(e) {
  e.preventDefault();
  hidePanelMessages();
  const username = $('adminUsername')?.value?.trim() || '';
  const password = $('adminPassword')?.value || '';
  const errEl = $('adminLoginError');

  try {
    const res = await window.__TAURI__.core.invoke('admin_login', { username, password });
    Admin.token = res.token;
    showAdminPanel(res);
    $('btnAdminContinue')?.classList.toggle('hidden', Admin.locked);
  } catch (err) {
    if (errEl) {
      errEl.textContent = String(err);
      errEl.classList.remove('hidden');
    }
  }
}

async function onSaveExpiry() {
  hidePanelMessages();
  if (!Admin.token) return;
  const expiry_date = $('adminExpiryDate')?.value;
  if (!expiry_date) {
    showPanelError('Son kullanma tarihi seçin.');
    return;
  }
  try {
    await window.__TAURI__.core.invoke('admin_set_expiry', {
      token: Admin.token,
      expiryDate: expiry_date,
    });
    showPanelSuccess('Son kullanma tarihi kaydedildi.');
    const status = await refreshLicenseAndMaybeUnlock();
    if (status.is_locked) {
      $('btnAdminContinue')?.classList.remove('hidden');
      showPanelSuccess('Tarih kaydedildi. Gelecek bir tarih seçerek uygulamayı açabilirsiniz.');
    }
  } catch (err) {
    showPanelError(String(err));
  }
}

async function onSaveCredentials() {
  hidePanelMessages();
  if (!Admin.token) return;
  const new_username = $('adminNewUsername')?.value?.trim() || '';
  const new_password = $('adminNewPassword')?.value || '';
  const current_password = $('adminCurrentPassword')?.value || '';

  const wantsCredsChange = !!new_password
    || (!!new_username && new_username !== Admin.savedUsername);

  if (!wantsCredsChange) {
    showPanelError('Sadece tarih değiştirmek için tarih yanındaki «Kaydet» butonunu kullanın.');
    return;
  }
  if (!new_password) {
    showPanelError('Şifre değiştirmek için yeni şifre girin. Sadece tarih için «Kaydet» butonunu kullanın.');
    return;
  }
  if (!new_username) {
    showPanelError('Kullanıcı adı boş olamaz.');
    return;
  }
  if (!current_password) {
    showPanelError('Hesap değişikliği için mevcut şifrenizi girin.');
    return;
  }

  try {
    await window.__TAURI__.core.invoke('admin_change_credentials', {
      token: Admin.token,
      currentPassword: current_password,
      newUsername: new_username,
      newPassword: new_password,
    });
    $('adminCurrentPassword').value = '';
    $('adminNewPassword').value = '';
    $('adminPassword').value = '';
    Admin.savedUsername = new_username;
    showPanelSuccess('Yönetici hesabı güncellendi.');
  } catch (err) {
    showPanelError(String(err));
  }
}

async function onAdminLogout() {
  if (Admin.token) {
    try {
      await window.__TAURI__.core.invoke('admin_logout', { token: Admin.token });
    } catch (_) { /* ignore */ }
  }
  Admin.token = null;
  hidePanelMessages();
  if (Admin.locked) {
    $('licenseAdminView')?.classList.add('hidden');
    $('licenseLoginView')?.classList.remove('hidden');
  } else {
    hideLicenseOverlay();
  }
}

async function openAdminFromApp() {
  showLicenseOverlay(false, '');
  const msgEl = $('licenseLockMessage');
  if (msgEl) msgEl.style.display = 'none';

  if (Admin.token) {
    try {
      const info = await window.__TAURI__.core.invoke('admin_get_info', { token: Admin.token });
      showAdminPanel(info);
      $('btnAdminContinue')?.classList.remove('hidden');
      return;
    } catch (_) {
      Admin.token = null;
    }
  }

  $('licenseAdminView')?.classList.add('hidden');
  $('licenseLoginView')?.classList.remove('hidden');
}

function initAdminUI() {
  $('adminLoginForm')?.addEventListener('submit', onAdminLogin);
  $('btnAdminSaveExpiry')?.addEventListener('click', onSaveExpiry);
  $('btnAdminSaveCreds')?.addEventListener('click', onSaveCredentials);
  $('btnAdminLogout')?.addEventListener('click', onAdminLogout);
  $('btnAdminContinue')?.addEventListener('click', () => refreshLicenseAndMaybeUnlock());
  $('btnOpenAdmin')?.addEventListener('click', openAdminFromApp);
  $('adminExpiryDate')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') { e.preventDefault(); onSaveExpiry(); }
  });
}

async function checkLicenseOnStartup() {
  initAdminUI();
  const status = await window.__TAURI__.core.invoke('get_license_status');
  if (status.is_locked) {
    showLicenseOverlay(true, status.message);
    return false;
  }
  return true;
}

window.checkLicenseOnStartup = checkLicenseOnStartup;
window.refreshLicenseAndMaybeUnlock = refreshLicenseAndMaybeUnlock;
