# DropBG Local (Figma dev plugin)

Thin companion that sends the selected image to a locally running DropBG app
and replaces it with the background removed. No cloud upload.

## Use
1. Launch the DropBG desktop app (its localhost API listens on `127.0.0.1:8765`).
2. In Figma desktop: **Plugins → Development → Import plugin from manifest…**
   and pick `figma-plugin/manifest.json`.
3. Select one image layer, run **DropBG Local**, click **Remove background**.

The localhost host is declared under `devAllowedDomains`, so this works as a
development plugin without publishing. To publish, move the host to
`allowedDomains` and add a `reasoning` string (Figma requires it).
