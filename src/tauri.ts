// Typed Tauri command/event contracts.
// Frontend <-> Rust boundary. Commands are declared here and implemented in
// src-tauri/src/commands.rs. Events are declared here and emitted from Rust.

export interface DockConfig {
  id: string;
  name: string;
  monitorId: string;
  edge: 'top' | 'bottom' | 'left' | 'right' | 'floating';
  x: number;
  y: number;
  width: number;
  height: number;
  iconSize: number;
  gap: number;
  theme: 'light' | 'dark' | 'system';
  background: 'solid' | 'transparent' | 'mica' | 'acrylic';
  autoHide: boolean;
  collapseToButton: boolean;
  hideWhenFullscreen: boolean;
  scale: number;
  orientation: 'horizontal' | 'vertical';
}

export interface DockItem {
  id: string;
  type: 'app' | 'folder' | 'file' | 'url' | 'separator';
  label: string;
  path?: string;
  aumid?: string;
  url?: string;
  args?: string[];
  iconUrl?: string;
  customIconUrl?: string;
  order: number;
}

export interface CommandResponse<T = unknown> {
  ok: boolean;
  data?: T;
  error?: string;
}

// Commands (invoked from frontend -> Rust)
export interface Commands {
  'dock:list': () => Promise<CommandResponse<DockConfig[]>>;
  'dock:create': (config: Partial<DockConfig>) => Promise<CommandResponse<DockConfig>>;
  'dock:update': (id: string, config: Partial<DockConfig>) => Promise<CommandResponse<DockConfig>>;
  'dock:delete': (id: string) => Promise<CommandResponse<void>>;
  'dock:items': (dockId: string) => Promise<CommandResponse<DockItem[]>>;
  'item:launch': (itemId: string) => Promise<CommandResponse<void>>;
}

// Events (emitted from Rust -> frontend)
export interface Events {
  'dock:created': DockConfig;
  'dock:updated': DockConfig;
  'dock:deleted': { id: string };
  'app:running': { appId: string; running: boolean };
}
