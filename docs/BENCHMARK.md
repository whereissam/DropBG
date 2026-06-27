# DropBG Internal Benchmark (Phase 11.6)

There is **no trustworthy public benchmark** that answers the only question that
matters here: *which model + backend + post-processing path produces the best
cutout for the DropBG / macOS pipeline?* Public leaderboards score raw models on
generic datasets — not our FP16 ONNX exports, not Core ML execution, not our
edge-refinement and decontamination passes.

So we maintain a small **private** evaluation set and a fixed protocol. Create a
local `bench/` folder for it — it is **not committed** (the whole directory is
git-ignored, since the images may be licensed, personal, or client work). Only
this protocol lives in the repo.

Suggested local layout:

```
bench/                      # git-ignored — create locally
├── portraits/  hair/  pets/  products/
├── glass/  shadows/  thin-lines/  low-contrast/
├── manifest.csv            # filename,category,source,license,notes
├── results.csv             # one row per (image, model, post-path) review
└── _out/                   # per-model batch outputs
```

## The set: 50–100 images

Aim for 50 to start, growing toward 100. Spread across the categories below;
each is a place models commonly *fail differently*, so a model that wins overall
can still lose a category.

| Category | Target | What it stresses |
|---|---:|---|
| `portraits` | 10–15 | faces, skin, soft hair outline |
| `hair` | 8–12 | flyaway strands, fine alpha, contrast vs background |
| `pets` | 6–10 | fur, whiskers, mixed textures |
| `products` | 8–12 | hard clean edges, reflections, e-commerce framing |
| `glass` | 5–8 | transparency, refraction, partial alpha |
| `shadows` | 5–8 | cast/contact shadows — keep vs drop decisions |
| `thin-lines` | 4–6 | wires, jewelry chains, plant stems, antennae |
| `low-contrast` | 5–8 | subject color ≈ background color |

Per image, record provenance in the manifest so the set stays auditable and
license-clean.

## Protocol

1. **Inputs.** Drop each category folder into DropBG's batch mode, once per model
   under test (the curated lineup: Apple Vision, BiRefNet Lite / General /
   Dynamic Matting / HR-matting, BEN2, RMBG 2.0). Save each run to
   `bench/_out/<model>/<category>/`.
2. **Backend.** Before timing a model, run **Settings → Inference Backend →
   Benchmark Backends** so each model uses its fastest *correct* backend on the
   test machine (Phase 11.2a). Record the chosen backend + median ms.
3. **Post-processing variants.** For the edge-sensitive categories (`hair`,
   `pets`, `glass`, `thin-lines`) also capture: base, **+HR Edges** (11.4), and
   **+Decontaminate** (11.5), so we can see what each pass actually buys.
4. **Scoring.** No ground-truth masks, so score by side-by-side human review on
   a 1–5 scale for: *edge fidelity*, *fringe/halo*, *holes/spill*, *shadow
   handling*. Capture composites on **white and on mid-gray** — fringe hides on
   white.
5. **Record.** One row per (image, model, post-path) in `bench/results.csv`
   (template committed). Aggregate per category to pick mode defaults.

## What the results decide

- Whether **BiRefNet Dynamic Matting** beats General enough to become the
  **Balanced/Quality default** (the held promotion from Phase 11.1 / 11.2b).
- Whether **BEN2** earns more than its current "experimental — benchmark first"
  status, or stays there.
- Default post-processing per **mode** (e.g. auto-Decontaminate in Product mode).
- Per-`{model, device}` backend choices already persisted by 11.2a.

## Out of scope (research backlog only)

Heavy / high-port-risk models stay in the [research backlog](MODEL_RESEARCH.md#research-backlog-not-shipped)
and are **not** added to the picker just to benchmark them — evaluate from
upstream demos first, port only if a category win is demonstrated.
