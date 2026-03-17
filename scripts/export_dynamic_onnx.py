#!/usr/bin/env python3
"""
Export BiRefNet_dynamic to ONNX with dynamic input resolution (256-2304px).

Usage:
    pip install torch transformers onnx onnxruntime onnxconverter-common
    python scripts/export_dynamic_onnx.py

Output: scripts/birefnet_dynamic_fp16.onnx
Copy the output file to your DropBG model directory (~/Downloads/DropBG/).

Note: This model supports dynamic input resolution, meaning images can be
processed at their native resolution (rounded to nearest multiple of 32)
without resize artifacts. The ONNX export uses dynamic axes for H and W.
"""

import torch
import os

MODEL_ID = "ZhengPeng7/BiRefNet_dynamic"
OUTPUT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_FP32 = os.path.join(OUTPUT_DIR, "birefnet_dynamic_fp16.onnx.tmp")
OUTPUT_FP16 = os.path.join(OUTPUT_DIR, "birefnet_dynamic_fp16.onnx")
DEFAULT_SIZE = 1024


def main():
    print("Loading model: " + MODEL_ID)
    from transformers import AutoModelForImageSegmentation

    model = AutoModelForImageSegmentation.from_pretrained(
        MODEL_ID, trust_remote_code=True
    )
    model.set_mode("eval")

    print("Creating dummy input: 1x3x" + str(DEFAULT_SIZE) + "x" + str(DEFAULT_SIZE))
    dummy_input = torch.randn(1, 3, DEFAULT_SIZE, DEFAULT_SIZE)

    print("Exporting to ONNX with dynamic H/W axes...")
    torch.onnx.export(
        model,
        dummy_input,
        OUTPUT_FP32,
        opset_version=17,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={
            "input": {0: "batch_size", 2: "height", 3: "width"},
            "output": {0: "batch_size", 2: "height", 3: "width"},
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

        os.remove(OUTPUT_FP32)
        print("Removed fp32 temp file. Final output: " + OUTPUT_FP16)
    except ImportError:
        print("onnxconverter-common not installed. Renaming fp32 model.")
        os.rename(OUTPUT_FP32, OUTPUT_FP16)
        print("Final output: " + OUTPUT_FP16)

    print("")
    print("Done! Copy the .onnx file to ~/Downloads/DropBG/")
    print("")
    print("Note: This model supports dynamic resolution (256-2304px).")
    print("DropBG will process images at their native resolution (rounded to 32px)")
    print("instead of resizing everything to 1024x1024, reducing artifacts.")


if __name__ == "__main__":
    main()
