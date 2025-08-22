#!/usr/bin/env python3
"""
Download pyannote models and export them to ONNX format for use with Rust
"""

import os
import sys
import torch
import numpy as np
from pathlib import Path
from pyannote.audio import Model, Inference

def export_pyannote_models():
    """Download and export pyannote models to ONNX"""
    
    print("=== Exporting PyAnnote Models to ONNX ===\n")
    
    # Set up paths
    models_dir = Path.home() / "Library" / "Application Support" / "KagiNote" / "models" / "diarization"
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Target directory: {models_dir}\n")
    
    # 1. Download and export segmentation model
    print("1. Loading pyannote/segmentation-3.0...")
    try:
        model = Model.from_pretrained("pyannote/segmentation-3.0")
        print("   ✓ Model loaded successfully")
        
        # Create dummy input for export (batch_size=1, channels=1, time_frames)
        # Segmentation models typically expect mel-spectrogram features
        # The model expects (batch, channels, frames) 
        dummy_input = torch.randn(1, 1, 80, 300)  # 80 mel bins, 300 time frames
        
        print("2. Exporting to ONNX...")
        onnx_path = models_dir / "segmentation.onnx"
        
        # Export to ONNX
        torch.onnx.export(
            model,
            dummy_input,
            str(onnx_path),
            export_params=True,
            opset_version=11,
            do_constant_folding=True,
            input_names=['input'],
            output_names=['output'],
            dynamic_axes={
                'input': {0: 'batch_size', 3: 'time'},
                'output': {0: 'batch_size', 2: 'time'}
            }
        )
        
        print(f"   ✓ Exported to: {onnx_path}")
        print(f"   ✓ Size: {onnx_path.stat().st_size / 1024 / 1024:.1f} MB")
        
    except Exception as e:
        print(f"   ✗ Failed: {e}")
        print("\n   This might be because the model architecture is complex.")
        print("   Let's try a different approach...")
        
        # Alternative: Use the Inference wrapper which might be simpler
        try:
            print("\n3. Trying alternative export method...")
            from pyannote.audio import Inference
            
            model = Model.from_pretrained("pyannote/segmentation-3.0")
            
            # Save the PyTorch model for now
            pt_path = models_dir / "segmentation.pt"
            torch.save(model.state_dict(), pt_path)
            print(f"   ✓ Saved PyTorch model to: {pt_path}")
            print(f"   ✓ Size: {pt_path.stat().st_size / 1024 / 1024:.1f} MB")
            
            # For ONNX export, we need the model architecture
            # This is complex for pyannote models
            print("\n   NOTE: Direct ONNX export is complex for pyannote models.")
            print("   The models use custom architectures that need special handling.")
            
        except Exception as e2:
            print(f"   ✗ Alternative also failed: {e2}")
    
    # 2. Try to get embedding model
    print("\n4. Loading embedding model...")
    try:
        # Try the public segmentation model first (doesn't require special access)
        model = Model.from_pretrained("pyannote/segmentation")
        print("   ✓ Loaded public segmentation model")
        
        pt_path = models_dir / "segmentation_public.pt"
        torch.save(model.state_dict(), pt_path)
        print(f"   ✓ Saved to: {pt_path}")
        
    except Exception as e:
        print(f"   ✗ Failed: {e}")
    
    print("\n5. Summary:")
    print("   PyAnnote models have complex architectures that are not easily")
    print("   exported to ONNX directly. The models use:")
    print("   - Custom SincNet layers")
    print("   - LSTM/GRU layers with specific configurations")
    print("   - Multi-scale temporal convolutions")
    print("\n   For production use, we need to either:")
    print("   1. Use pyannote-audio Python library via subprocess (recommended)")
    print("   2. Reimplement the model architecture in ONNX")
    print("   3. Use a pre-converted ONNX model from the community")

if __name__ == "__main__":
    export_pyannote_models()