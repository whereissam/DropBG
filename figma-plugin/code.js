figma.showUI(__html__, { width: 320, height: 160 });

let reqCounter = 0;
let activeNode = null;

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
  activeNode = node;
  const bytes = await node.exportAsync({ format: "PNG" });
  const requestId = `r${++reqCounter}`;
  figma.ui.postMessage({ type: "remove", requestId, bytes });
}

function applyCutout(bytes) {
  const node = activeNode;
  if (!node) { figma.notify("No target layer."); return; }
  const idx = firstImageFillIndex(node);
  if (idx === -1) { figma.notify("Target layer lost its image fill."); return; }

  // Hidden backup of the original (keeps its fills).
  const backup = node.clone();
  backup.visible = false;
  backup.name = `${node.name} (original)`;

  // Replace only the first IMAGE fill; preserve all other fills.
  const image = figma.createImage(bytes);
  const fills = JSON.parse(JSON.stringify(node.fills)); // clone the readonly array
  fills[idx] = { type: "IMAGE", scaleMode: "FILL", imageHash: image.hash };
  node.fills = fills;

  figma.currentPage.selection = [node];
  figma.notify("Background removed.");
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
    applyCutout(msg.bytes);
  }
};

figma.ui.postMessage({ type: "health-check" });
