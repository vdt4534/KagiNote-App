#!/usr/bin/env python3
"""
Download and convert pyannote models for KagiNote
Requires: pip install huggingface_hub torch onnx
"""

import os
import sys
import torch
import onnx
from pathlib import Path
from huggingface_hub import hf_hub_download, login

# Your Hugging Face token
HF_TOKEN = os.getenv("HF_TOKEN")  # Set your Hugging Face token as environment variable

def download_pyannote_models():
    """Download pyannote models from Hugging Face"""
    
    # Login to Hugging Face
    print("Logging in to Hugging Face...")
    login(token=HF_TOKEN)
    
    # Set up paths
    models_dir = Path.home() / "Library" / "Application Support" / "KagiNote" / "models" / "diarization"
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Downloading models to {models_dir}")
    
    # Download segmentation model (pyannote/segmentation-3.0)
    print("\n1. Downloading segmentation model...")
    try:
        seg_model_path = hf_hub_download(
            repo_id="pyannote/segmentation-3.0",
            filename="pytorch_model.bin",
            token=HF_TOKEN,
            cache_dir=models_dir / "cache"
        )
        print(f"   Downloaded to: {seg_model_path}")
        
        # Copy to our location
        import shutil
        shutil.copy(seg_model_path, models_dir / "segmentation.pt")
        print(f"   Copied to: {models_dir / 'segmentation.pt'}")
        
    except Exception as e:
        print(f"   ERROR downloading segmentation: {e}")
        return False
    
    # Download embedding model (pyannote/embedding)
    print("\n2. Downloading embedding model...")
    try:
        emb_model_path = hf_hub_download(
            repo_id="pyannote/embedding",
            filename="pytorch_model.bin",
            token=HF_TOKEN,
            cache_dir=models_dir / "cache"
        )
        print(f"   Downloaded to: {emb_model_path}")
        
        # Copy to our location
        import shutil
        shutil.copy(emb_model_path, models_dir / "embedding.pt")
        print(f"   Copied to: {models_dir / 'embedding.pt'}")
        
    except Exception as e:
        print(f"   ERROR downloading embedding: {e}")
        # Try alternative model
        print("   Trying alternative embedding model (wespeaker)...")
        try:
            emb_model_path = hf_hub_download(
                repo_id="pyannote/wespeaker-voxceleb-resnet34-LM",
                filename="pytorch_model.bin",
                token=HF_TOKEN,
                cache_dir=models_dir / "cache"
            )
            print(f"   Downloaded to: {emb_model_path}")
            
            import shutil
            shutil.copy(emb_model_path, models_dir / "embedding.pt")
            print(f"   Copied to: {models_dir / 'embedding.pt'}")
        except Exception as e2:
            print(f"   ERROR downloading alternative: {e2}")
            return False
    
    print("\n3. Models downloaded successfully!")
    print(f"   Segmentation: {models_dir / 'segmentation.pt'}")
    print(f"   Embedding: {models_dir / 'embedding.pt'}")
    
    # Note: Converting PyTorch to ONNX requires the model architecture
    # which is complex for pyannote models. For now, we'll use the PyTorch files
    # and implement a Python bridge to use them.
    
    print("\nNOTE: Models are in PyTorch format (.pt)")
    print("To use with ONNX Runtime, we need to either:")
    print("1. Use a Python bridge with pyannote.audio library")
    print("2. Export to ONNX using the model architecture (complex)")
    
    return True

def verify_models():
    """Verify downloaded models"""
    models_dir = Path.home() / "Library" / "Application Support" / "KagiNote" / "models" / "diarization"
    
    seg_path = models_dir / "segmentation.pt"
    emb_path = models_dir / "embedding.pt"
    
    print("\n4. Verifying models...")
    
    if seg_path.exists():
        size_mb = seg_path.stat().st_size / (1024 * 1024)
        print(f"   ✓ Segmentation model: {size_mb:.1f} MB")
    else:
        print(f"   ✗ Segmentation model not found")
        
    if emb_path.exists():
        size_mb = emb_path.stat().st_size / (1024 * 1024)
        print(f"   ✓ Embedding model: {size_mb:.1f} MB")
    else:
        print(f"   ✗ Embedding model not found")

if __name__ == "__main__":
    print("=== PyAnnote Model Downloader for KagiNote ===\n")
    
    if download_pyannote_models():
        verify_models()
        print("\n✅ Success! Models are ready for use with Python bridge.")
    else:
        print("\n❌ Failed to download models. Check your Hugging Face token and internet connection.")
        sys.exit(1)