figma.showUI(__html__, { width: 320, height: 160 });

let reqCounter = 0;

function firstImageFillIndex(node) {
  if (!("fills" in node) || !Array.isArray(node.fills)) return -1;
  return node.fills.findIndex((f) => f.type === "IMAGE");
}

async function startRemoval() {
  const sel = figma.currentPage.selection;
  if (sel.length === 0) { figma.notify("Select an image layer first."); return; }
  if (sel.length > 1) { figma.notify("Select exactly one image layer."); return; }
  const node = sel[0];
  if (firstImageFillIndex(node) === -1) {
    figma.notify("This layer has no image fill to process.");
    return;
  }
  const bytes = await node.exportAsync({ format: "PNG" });
  const requestId = `r${++reqCounter}`;
  figma.ui.postMessage({ type: "remove", requestId, bytes });
}

figma.ui.onmessage = (msg) => {
  if (msg.type === "health-ok") {
    figma.notify(`DropBG connected (model: ${msg.model})`);
  } else if (msg.type === "health-error") {
    figma.notify(`DropBG not reachable: ${msg.message}`);
  } else if (msg.type === "run-clicked") {
    startRemoval();
  } else if (msg.type === "remove-error") {
    const code = msg.status ? ` (${msg.status})` : "";
    figma.notify(`Background removal failed${code}: ${msg.message}`);
  } else if (msg.type === "remove-ok") {
    // Task 6 applies msg.bytes.
    figma.notify("Cutout received.");
  }
};

figma.ui.postMessage({ type: "health-check" });
