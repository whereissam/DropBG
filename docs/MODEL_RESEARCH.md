# Background Removal Model Research (March 2026)

## Current Models in DropBG

| Model | Variant | Size | Input | License | Strength |
|---|---|---|---|---|---|
| BiRefNet Lite | `Lite` | ~200 MB | 1024² | Apache 2.0 | Fast, good for most images |
| BiRefNet Full | `Full` | ~900 MB | 1024² | Apache 2.0 | Complex backgrounds, high detail |
| BEN2 | `BEN2` | ~219 MB | 1024² | MIT | Hair & fine edges via CGM pipeline |
| RMBG 2.0 | `RMBG2` | ~514 MB | 1024² | CC BY-NC 4.0 | Best overall quality (gated) |
| MODNet | `MODNet` | ~13 MB | 512² | Apache 2.0 | Lightweight portraits (outdated) |

## State of the Art (2025-2026)

### BiRefNet Family (Apache 2.0)

The BiRefNet family is the current SOTA for background removal. Key variants:

| Variant | ONNX Available? | Size | Input | Best For |
|---|---|---|---|---|
| **BiRefNet** (base) | Yes | ~490 MB | 1024² | General purpose |
| **BiRefNet Lite** | Yes | ~200 MB | 1024² | Speed/lightweight |
| **BiRefNet Portrait** | Yes (onnx-community) | ~490 MB | 1024² | Portrait/people segmentation |
| **BiRefNet General (epoch 244)** | Yes (onnx-community) | ~490 MB | 1024² | Newer training, better general quality |
| BiRefNet-matting | **No ONNX yet** | — | 2048² | True alpha mattes (hair, fur, glass) |
| BiRefNet_dynamic | **No ONNX yet** | — | 256-2304 | Any resolution without resize artifacts |
| BiRefNet_HR | **No ONNX yet** | — | 2048² | High-resolution images |

### BRIA RMBG 2.0

- Built on BiRefNet architecture with proprietary training data
- Claims 90% usable results vs 85% for vanilla BiRefNet
- **License:** CC BY-NC 4.0 (commercial requires paid license from BRIA)
- Excellent on complex/cluttered backgrounds and e-commerce products
- No RMBG 3.0 as of March 2026

### BEN2 (MIT)

- Confidence Guided Matting (CGM) refiner targets low-confidence pixels
- Good at hair matting and 4K processing
- MIT license makes it most commercially attractive high-quality option
- Full CGM pipeline is ~1.76 GB on ONNX

### SAM 3 (Meta, Nov 2025)

- Open-vocabulary text-based prompting ("red bottle", "cat")
- Multi-instance output, unified image/video
- **Not a background remover** — requires prompts (points, boxes, or text)
- Produces binary masks, not alpha mattes
- Would need separate matting step for transparency

### ViTMatte (MIT)

- True alpha matting (hair, fur, glass, smoke)
- **Requires trimap input** — needs two-stage pipeline
- Could pair with BiRefNet: coarse mask → ViTMatte refinement

### InSPyReNet (MIT)

- Designed for high-res salient object detection
- Good for portraits but not as strong as BiRefNet for general objects
- Community integrations exist (ComfyUI, rembg)

## Best Model Per Scenario

| Scenario | Recommended | Notes |
|---|---|---|
| General objects | RMBG 2.0 or BiRefNet General | RMBG 2.0 leads benchmarks |
| Portraits/people | **BiRefNet Portrait** | Replaces MODNet — much higher quality |
| Hair, fur, transparency | BiRefNet-matting or BEN2 CGM | Matting has no ONNX yet |
| Complex backgrounds | RMBG 2.0 | Proprietary training data excels here |
| Speed/lightweight | BiRefNet Lite | Already in DropBG |
| Commercial use | BEN2 (MIT), BiRefNet (Apache 2.0) | RMBG 2.0 is NC only |

## What We Added

Based on this research, we added to DropBG:

1. **BiRefNet Portrait** (~490 MB) — specialized portrait model, replaces MODNet for face/people use cases
2. **BiRefNet General** (~490 MB) — newer training epoch (244), improved general quality over BiRefNet Full

## Known Issues with Current Models

- **Binary masks vs alpha mattes:** Most models output binary segmentation, not true alpha. Hair and semi-transparent edges get hard-clipped. BiRefNet-matting would fix this but has no ONNX conversion yet.
- **Fixed input resolution:** Resizing large images to 1024px loses fine detail. BiRefNet_dynamic would handle native resolution but also has no ONNX conversion yet.
- **MODNet is outdated:** 13MB model from 2020, low quality by 2026 standards. BiRefNet Portrait is the modern replacement.

## Future Opportunities

- **Convert BiRefNet-matting to ONNX** — would give true alpha matte output for hair/fur/transparency. Source: `ZhengPeng7/BiRefNet-matting`
- **Convert BiRefNet_dynamic to ONNX** — would handle any resolution (256-2304px). Source: `ZhengPeng7/BiRefNet_dynamic`
- **Two-stage pipeline** — use BiRefNet for coarse mask → ViTMatte for refined alpha on edges
- **SAM 3 integration** — text-prompted object extraction ("remove the cat") as a differentiated feature

## ONNX Sources

| Model | HuggingFace Repo | fp16 URL |
|---|---|---|
| BiRefNet Lite | `onnx-community/BiRefNet_lite-ONNX` | `.../onnx/model_fp16.onnx` |
| BiRefNet Full | `onnx-community/BiRefNet-ONNX` | `.../onnx/model_fp16.onnx` |
| BiRefNet Portrait | `onnx-community/BiRefNet-portrait-ONNX` | `.../onnx/model_fp16.onnx` |
| BiRefNet General | `onnx-community/BiRefNet-general-epoch_244` | `.../onnx/model_fp16.onnx` |
| BEN2 | `onnx-community/BEN2-ONNX` | `.../onnx/model_fp16.onnx` |
| RMBG 2.0 | `briaai/RMBG-2.0` | `.../onnx/model_fp16.onnx` (gated) |
| MODNet | `Xenova/modnet` | `.../onnx/model_fp16.onnx` |
