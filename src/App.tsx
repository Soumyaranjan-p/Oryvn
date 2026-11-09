import { APP_NAME } from './brand';
import { invoke } from '@tauri-apps/api/core';
import Dock from './components/Dock';

function isDockMode(): boolean {
  if (typeof window === 'undefined') return false;
  return window.location.search.includes('dock=1');
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
    const res = await invoke('spike_b_setup');
    console.log('Spike B result:', res);
    alert('Spike B: check console for multi-monitor dock placements.');
  } catch (err) {
    console.error('Spike B failed:', err);
    alert('Spike B failed: ' + String(err));
  }
}

export default function App() {
  const dock = isDockMode();

  if (dock) {
    return <Dock />;
  }

  return (
    <div className="w-full h-full bg-transparent flex flex-col items-center justify-center p-8">
      <div className="max-w-md text-center space-y-4">
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
        <p className="mt-6 text-xs text-slate-500 dark:text-slate-400">
          Spike A: transparent borderless always-on-top dock with
          WS_EX_TOOLWINDOW + WS_EX_NOACTIVATE. Spike B: one dock per monitor +
          WM_DISPLAYCHANGE listener.
        </p>
      </div>
    </div>
  );
}
