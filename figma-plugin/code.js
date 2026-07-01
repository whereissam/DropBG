// Main thread. Has the Figma API; cannot fetch.
figma.showUI(__html__, { width: 320, height: 160 });

// Route messages from the UI iframe.
figma.ui.onmessage = (msg) => {
  if (msg.type === "health-ok") {
    figma.notify(`DropBG connected (model: ${msg.model})`);
  } else if (msg.type === "health-error") {
    figma.notify(`DropBG not reachable: ${msg.message}`);
  }
  // "remove-ok" / "remove-error" handled in Task 5–6.
};

// Kick off a health check on open.
figma.ui.postMessage({ type: "health-check" });
