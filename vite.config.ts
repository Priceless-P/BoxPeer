import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import net from 'net';

// Default port to use
const defaultPort = 1420;

// Function to find an available port synchronously
function getAvailablePort(startPort: number): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(startPort, () => {
      server.once('close', () => resolve(startPort));
      server.close();
    });
    server.on('error', () => resolve(getAvailablePort(startPort + 1)));
  });
}

// Use a synchronous method to determine the port before exporting the config
let port: number = defaultPort;
getAvailablePort(defaultPort).then(availablePort => {
  port = availablePort;
}).catch(error => {
  console.error('Failed to find an available port:', error);
});

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: port,
    strictPort: false, // Allow Vite to choose a different port if the specified one is in use
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
