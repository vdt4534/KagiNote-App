#!/bin/bash

# Download Sherpa-ONNX speaker diarization models
# These are proper ONNX models that work with ort in Rust

MODELS_DIR="$HOME/Library/Application Support/KagiNote/models/diarization"
mkdir -p "$MODELS_DIR"

echo "=== Downloading Sherpa-ONNX Speaker Diarization Models ==="
echo "Target: $MODELS_DIR"
echo ""

# Download speaker segmentation model (VAD/segmentation)
echo "1. Downloading speaker segmentation model..."
cd "$MODELS_DIR"

# Download the pyannote segmentation model from sherpa-onnx
wget -q --show-progress \
  "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-segmentation-models/sherpa-onnx-pyannote-segmentation-3-0.tar.bz2" \
  -O segmentation.tar.bz2

echo "   Extracting..."
tar xf segmentation.tar.bz2
mv sherpa-onnx-pyannote-segmentation-3-0/model.onnx segmentation.onnx
rm -rf sherpa-onnx-pyannote-segmentation-3-0 segmentation.tar.bz2

echo "   ✓ Segmentation model saved to: segmentation.onnx"
echo "   ✓ Size: $(du -h segmentation.onnx | cut -f1)"

# Download speaker embedding model
echo ""
echo "2. Downloading speaker embedding model..."
wget -q --show-progress \
  "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx" \
  -O embedding.onnx

echo "   ✓ Embedding model saved to: embedding.onnx"
echo "   ✓ Size: $(du -h embedding.onnx | cut -f1)"

echo ""
echo "✅ Models downloaded successfully!"
echo ""
echo "These are proper ONNX models that can be loaded with ort in Rust."
ls -lh "$MODELS_DIR"/*.onnx