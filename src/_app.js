// Initialize Tauri
const { invoke } = window.__TAURI__;

// Export invoke for use in Rust code
window.invoke = invoke;

// Handle UI events
window.addEventListener('DOMContentLoaded', () => {
  // Initialize logging
  console.log('Anarchy Inference UI initialized');
});

// Export UI helper functions
window.createWindow = async (title, width, height) => {
  return await invoke('create_window', { title, width, height });
};

window.executeCode = async (code) => {
  return await invoke('execute_code', { code });
};
