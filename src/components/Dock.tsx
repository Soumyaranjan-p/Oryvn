import { useState, useCallback } from 'react';

// Spike A: dock window UI with hover magnification.
// Runs inside a transparent, borderless Tauri window.
// Animate only transform + opacity (GPU-friendly, no layout thrashing).

const DOCK_ITEMS = [
  { id: '1', label: 'VS Code', icon: '💻' },
  { id: '2', label: 'Terminal', icon: '⚡' },
  { id: '3', label: 'Browser', icon: '🌐' },
  { id: '4', label: 'Folder', icon: '📁' },
  { id: '5', label: 'Settings', icon: '⚙️' },
];

export default function Dock() {
  const [hovered, setHovered] = useState<string | null>(null);

  const enter = useCallback((id: string) => setHovered(id), []);
  const leave = useCallback(() => setHovered(null), []);

  const itemScale = (id: string) => (hovered === id ? 1.25 : 1);
  const itemOpacity = (id: string) => (hovered === id ? 1 : 0.85);

  return (
    <div
      className="fixed inset-0 pointer-events-none z-[99999] bg-transparent"
      style={{ height: '100vh', width: '100vw', background: 'transparent' }}
    >
      {/* Dock bar: bottom edge, centered */}
      <div
        className="absolute bottom-8 left-1/2 -translate-x-1/2 flex items-end gap-2"
        style={{ height: '100%', background: 'transparent' }}
      >
        {DOCK_ITEMS.map((item) => (
          <div
            key={item.id}
            className="relative group"
            onMouseEnter={() => enter(item.id)}
            onMouseLeave={leave}
          >
            <button
              className="w-16 h-16 rounded-2xl flex items-center justify-center text-xl transition-transform duration-200 ease-out bg-transparent"
              style={{
                transform: `scale(${itemScale(item.id)})`,
                opacity: itemOpacity(item.id),
                transition: 'transform 200ms cubic-bezier(0.23, 1, 0.32, 1), opacity 200ms ease',
                background: 'transparent',
              }}
              onMouseEnter={() => enter(item.id)}
              onMouseLeave={leave}
              title={item.label}
            >
              <span className="text-3xl">{item.icon}</span>
            </button>
            <span className="absolute -bottom-8 left-1/2 -translate-x-1/2 whitespace-nowrap text-xs text-slate-700 dark:text-slate-300 opacity-0 group-hover:opacity-100 transition-opacity">
              {item.label}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
