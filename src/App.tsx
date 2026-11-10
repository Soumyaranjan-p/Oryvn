import { APP_NAME } from './brand';
import { invoke } from '@tauri-apps/api/core';
import Dock from './components/Dock';

function isDockMode(): boolean {
  if (typeof window === 'undefined') return false;
  const params = new URLSearchParams(window.location.search);
  return params.has('dock');
}

async function createDock() {
  try {
    await invoke('spike_a_create_dock');
  } catch (err) {
    console.error('Failed to create dock:', err);
  }
}

async function setupMultiMonitor() {
  try {
    await invoke('spike_b_setup');
    alert('Spike B: check console for multi-monitor dock placements.');
  } catch (err) {
    console.error('Spike B failed:', err);
    alert('Spike B failed: ' + String(err));
  }
}

async function toggleAutohide() {
  try {
    await invoke('spike_c_toggle_autohide', {
      dockLabel: 'dock-spike-a',
      edge: 'bottom',
      enabled: true,
    });
    alert('Spike C: autohide polling thread started (check logs).');
  } catch (err) {
    console.error('Spike C failed:', err);
    alert('Spike C failed: ' + String(err));
  }
}

async function registerAppbar() {
  try {
    await invoke('spike_d_register_appbar');
    alert('Spike D: appbar registered on bottom edge.');
  } catch (err) {
    console.error('Spike D failed:', err);
    alert('Spike D failed: ' + String(err));
  }
}

async function removeAppbar() {
  try {
    await invoke('spike_d_remove_appbar');
    alert('Spike D: appbar removed.');
  } catch (err) {
    console.error('Spike D failed:', err);
    alert('Spike D failed: ' + String(err));
  }
}

async function openNDocks(count: number) {
  try {
    const res = await invoke('spike_e_open_n_docks', { count });
    console.log('Spike E opened docks:', res);
    alert('Spike E: opened ' + count + ' docks (check task manager / process explorer).');
  } catch (err) {
    console.error('Spike E failed:', err);
    alert('Spike E failed: ' + String(err));
  }
}

export default function App() {
  const dock = isDockMode();

  if (dock) {
    return <Dock />;
  }

  return (
    <div className="w-full h-full bg-transparent flex flex-col items-center justify-center p-8">
      <div className="max-w-md text-center space-y-3">
        <h1 className="text-2xl font-semibold text-slate-900 dark:text-slate-100">
          {APP_NAME} — Phase 0 (risk spikes)
        </h1>
        <p className="text-slate-600 dark:text-slate-400">
          Bootstrapped Tauri v2 + React + Tailwind.
        </p>
        <button
          onClick={createDock}
          className="px-6 py-3 bg-blue-600 text-white rounded-xl text-sm font-medium hover:bg-blue-700 active:scale-95 transition-transform"
        >
          Create Dock (Spike A)
        </button>
        <button
          onClick={setupMultiMonitor}
          className="px-6 py-3 bg-indigo-600 text-white rounded-xl text-sm font-medium hover:bg-indigo-700 active:scale-95 transition-transform"
        >
          Multi-Monitor Docks (Spike B)
        </button>
        <button
          onClick={toggleAutohide}
          className="px-6 py-3 bg-green-600 text-white rounded-xl text-sm font-medium hover:bg-green-700 active:scale-95 transition-transform"
        >
          Auto-Hide Polling (Spike C)
        </button>
        <button
          onClick={registerAppbar}
          className="px-6 py-3 bg-purple-600 text-white rounded-xl text-sm font-medium hover:bg-purple-700 active:scale-95 transition-transform"
        >
          AppBar Docking (Spike D)
        </button>
        <button
          onClick={removeAppbar}
          className="px-6 py-3 bg-red-600 text-white rounded-xl text-sm font-medium hover:bg-red-700 active:scale-95 transition-transform"
        >
          Remove AppBar (Spike D cleanup)
        </button>
        <div className="mt-4 border-t pt-4 text-left text-xs text-slate-500 dark:text-slate-400 space-y-2">
          <div className="font-semibold text-slate-700 dark:text-slate-300">Debug (Spike E)</div>
          <button
            onClick={() => openNDocks(1)}
            className="px-4 py-2 bg-gray-600 text-white rounded text-xs font-medium hover:bg-gray-700 active:scale-95 transition-transform"
          >
            Open 1 Dock
          </button>
          <button
            onClick={() => openNDocks(3)}
            className="px-4 py-2 bg-gray-600 text-white rounded text-xs font-medium hover:bg-gray-700 active:scale-95 transition-transform"
          >
            Open 3 Docks
          </button>
          <button
            onClick={() => openNDocks(5)}
            className="px-4 py-2 bg-gray-600 text-white rounded text-xs font-medium hover:bg-gray-700 active:scale-95 transition-transform"
          >
            Open 5 Docks
          </button>
          <p className="text-[10px] text-slate-500 dark:text-slate-500 mt-1">
            Debug-only (debug_assertions). Measure per-dock WebView2 memory in Task
            Manager / Process Explorer.
          </p>
        </div>
        <p className="mt-6 text-xs text-slate-500 dark:text-slate-400">
          Spikes A&ndash;E are throwaway risk-spike code under
          "src-tauri/src/spikes/". Each is verified independently before Phase 1.
        </p>
      </div>
    </div>
  );
}
