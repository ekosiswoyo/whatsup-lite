import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import './style.css';

const autostart = document.querySelector<HTMLInputElement>('#autostart')!;
const minimize = document.querySelector<HTMLInputElement>('#minimize')!;
const status = document.querySelector<HTMLParagraphElement>('#status')!;

async function load(): Promise<void> {
  const settings = await invoke<{ minimize_to_tray: boolean }>('get_settings');
  minimize.checked = settings.minimize_to_tray;
  autostart.checked = await isEnabled();
}

autostart.addEventListener('change', async () => {
  try {
    await (autostart.checked ? enable() : disable());
    status.textContent = 'Pengaturan startup disimpan.';
  } catch (error) {
    autostart.checked = !autostart.checked;
    status.textContent = `Gagal mengubah startup: ${String(error)}`;
  }
});

minimize.addEventListener('change', async () => {
  await invoke('set_minimize_to_tray', { enabled: minimize.checked });
  status.textContent = 'Pengaturan tray disimpan.';
});

document.querySelector('#clear')!.addEventListener('click', async () => {
  if (!confirm('Hapus seluruh cookie, cache, dan sesi WhatsApp?')) return;
  await invoke('clear_session');
  status.textContent = 'Sesi dihapus. WhatsApp Web dimuat ulang.';
});

document.querySelector('#close')!.addEventListener('click', () => getCurrentWindow().hide());

load().catch((error) => { status.textContent = `Gagal membaca pengaturan: ${String(error)}`; });
