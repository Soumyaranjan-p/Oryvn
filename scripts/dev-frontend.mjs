// Dev frontend launcher used as Tauri's `beforeDevCommand`.
//
// Why this exists:
//   `tauri dev` needs the Vite dev server on http://localhost:5173. Starting
//   `npm run dev` blindly fails with "Port 5173 is already in use" when a Vite
//   instance is already running (common when the user has `npm run dev` open in
//   another terminal, or after a previous crash left the port bound).
//
// Behaviour:
//   - Port free      -> spawn `vite` as a child and pipe its output.
//   - Port in use    -> assume a dev server is already serving there, log that,
//                       and stay alive so Tauri keeps the command running.
//
// Tauri polls devUrl itself, so it will proceed as soon as the server responds.

import net from 'node:net';
import { spawn } from 'node:child_process';

// Vite binds to "localhost", which on this machine resolves to IPv6 ::1
// (not 127.0.0.1). Probe both loopback families so we detect an existing
// dev server regardless of how it was bound.
const HOSTS = ['localhost', '127.0.0.1', '::1'];
const PORT = 5173;

function canConnect(host, port) {
  return new Promise((resolve) => {
    const socket = net.connect({ host, port });
    socket.setTimeout(1000);
    socket.on('connect', () => {
      socket.destroy();
      resolve(true);
    });
    const fail = () => {
      socket.destroy();
      resolve(false);
    };
    socket.on('error', fail);
    socket.on('timeout', fail);
  });
}

async function isPortInUse(port) {
  for (const host of HOSTS) {
    if (await canConnect(host, port)) {
      console.log(`[dev-frontend] detected existing server on ${host}:${port}`);
      return true;
    }
  }
  return false;
}

async function main() {
  if (await isPortInUse(PORT)) {
    console.log(
      `[dev-frontend] Port ${PORT} already in use - reusing existing dev server.`,
    );
    console.log('[dev-frontend] (Stop any other Vite instance if startup fails.)');
    // Stay alive so Tauri treats the command as running.
    await new Promise(() => {});
    return;
  }

  console.log(`[dev-frontend] Starting Vite on port ${PORT}...`);
  const child = spawn('npm', ['run', 'dev'], {
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  const forward = (signal) => {
    if (!child.killed) child.kill(signal);
  };
  process.on('SIGINT', () => forward('SIGINT'));
  process.on('SIGTERM', () => forward('SIGTERM'));

  child.on('exit', (code) => process.exit(code ?? 0));
}

main().catch((err) => {
  console.error('[dev-frontend] failed:', err);
  process.exit(1);
});
