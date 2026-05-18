#!/usr/bin/env python3
"""
Export BiRefNet_HR-matting to ONNX (fp16) at 2048x2048 resolution.

Usage:
    pip install torch transformers onnx onnxruntime onnxconverter-common
    python scripts/export_hr_matting_onnx.py

Output: scripts/birefnet_hr_matting_fp16.onnx (~900 MB)
Copy the output file to your DropBG model directory (~/Downloads/DropBG/).

Notes:
- HR-matting is the high-resolution matting variant trained at 2048x2048.
- Inference uses ~4x more memory than the 1024-input models — close other
  apps before running heavy batches, especially on machines with <16 GB RAM.
"""

import torch
import os

MODEL_ID = "ZhengPeng7/BiRefNet_HR-matting"
OUTPUT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_FP32 = os.path.join(OUTPUT_DIR, "birefnet_hr_matting.onnx")
OUTPUT_FP16 = os.path.join(OUTPUT_DIR, "birefnet_hr_matting_fp16.onnx")
INPUT_SIZE = 2048


def main():
    print("Loading model: " + MODEL_ID)
    from transformers import AutoModelForImageSegmentation

    model = AutoModelForImageSegmentation.from_pretrained(
        MODEL_ID, trust_remote_code=True
    )
    model.set_mode("eval")

    print("Creating dummy input: 1x3x" + str(INPUT_SIZE) + "x" + str(INPUT_SIZE))
    dummy_input = torch.randn(1, 3, INPUT_SIZE, INPUT_SIZE)

    print("Exporting to ONNX (fp32): " + OUTPUT_FP32)
    torch.onnx.export(
        model,
        dummy_input,
        OUTPUT_FP32,
        opset_version=17,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={
            "input": {0: "batch_size"},
            "output": {0: "batch_size"},
        },
    )

    fp32_size = os.path.getsize(OUTPUT_FP32) / (1024 * 1024)
    print("FP32 model exported: " + str(round(fp32_size, 1)) + " MB")

    # Convert to fp16
    print("Converting to fp16: " + OUTPUT_FP16)
    try:
        from onnxconverter_common import float16
        import onnx

        model_fp32 = onnx.load(OUTPUT_FP32)
        model_fp16 = float16.convert_float_to_float16(model_fp32)
        onnx.save(model_fp16, OUTPUT_FP16)

        fp16_size = os.path.getsize(OUTPUT_FP16) / (1024 * 1024)
        print("FP16 model exported: " + str(round(fp16_size, 1)) + " MB")

        # Clean up fp32
        os.remove(OUTPUT_FP32)
        print("Removed fp32 model. Final output: " + OUTPUT_FP16)
    except ImportError:
        print("onnxconverter-common not installed. Keeping fp32 model.")
        print("Install with: pip install onnxconverter-common")
        print("Final output: " + OUTPUT_FP32)

    print("")
    print("Done! Copy the .onnx file to ~/Downloads/DropBG/")


if __name__ == "__main__":
    main()
