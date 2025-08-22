#!/usr/bin/env python3
"""
Download proper ONNX models for pyannote diarization
These are the actual ONNX models that can be used with ort in Rust
"""

import os
import sys
from pathlib import Path
from huggingface_hub import hf_hub_download, login

# Your Hugging Face token
HF_TOKEN = os.getenv("HF_TOKEN")  # Set your Hugging Face token as environment variable

def download_onnx_models():
    """Download ONNX versions of pyannote models"""
    
    print("=== Downloading ONNX Models for pyannote ===\n")
    
    # Login to Hugging Face
    print("1. Logging in to Hugging Face...")
    login(token=HF_TOKEN)
    
    # Set up paths
    models_dir = Path.home() / "Library" / "Application Support" / "KagiNote" / "models" / "diarization"
    models_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"2. Target directory: {models_dir}\n")
    
    # First, try to get the segmentation model ONNX file
    print("3. Downloading segmentation model (ONNX format)...")
    try:
        # The segmentation-3.0 model requires acceptance
        # After you accept it on HuggingFace, this will work
        seg_onnx = hf_hub_download(
            repo_id="pyannote/segmentation-3.0",
            filename="model.onnx",  # Try the ONNX file
            token=HF_TOKEN,
            cache_dir=models_dir / "cache"
        )
        
        # Copy to our location
        import shutil
        target_seg = models_dir / "segmentation.onnx"
        shutil.copy(seg_onnx, target_seg)
        print(f"   ✓ Saved to: {target_seg}")
        print(f"   ✓ Size: {target_seg.stat().st_size / 1024 / 1024:.1f} MB")
        
    except Exception as e:
        print(f"   ✗ Failed: {e}")
        print("\n   IMPORTANT: You need to accept the model agreement at:")
        print("   https://huggingface.co/pyannote/segmentation-3.0")
        print("   Then run this script again.\n")
        return False
    
    # Try to get an embedding/speaker model
    print("\n4. Downloading speaker embedding model (ONNX format)...")
    
    # Try different models that might have ONNX versions
    models_to_try = [
        ("pyannote/embedding", "model.onnx"),
        ("pyannote/wespeaker-voxceleb-resnet34-LM", "model.onnx"),
        ("speechbrain/spkrec-ecapa-voxceleb", "model.onnx"),
    ]
    
    embedding_downloaded = False
    for repo_id, filename in models_to_try:
        try:
            print(f"   Trying {repo_id}...")
            emb_onnx = hf_hub_download(
                repo_id=repo_id,
                filename=filename,
                token=HF_TOKEN,
                cache_dir=models_dir / "cache"
            )
            
            # Copy to our location
            import shutil
            target_emb = models_dir / "embedding.onnx"
            shutil.copy(emb_onnx, target_emb)
            print(f"   ✓ Saved to: {target_emb}")
            print(f"   ✓ Size: {target_emb.stat().st_size / 1024 / 1024:.1f} MB")
            embedding_downloaded = True
            break
            
        except Exception as e:
            print(f"   - {repo_id} failed: {str(e)[:100]}")
            continue
    
    if not embedding_downloaded:
        print("\n   ⚠ Could not download embedding model in ONNX format")
        print("   We may need to convert from PyTorch to ONNX")
    
    print("\n5. Checking downloaded files...")
    
    # Verify what we have
    seg_path = models_dir / "segmentation.onnx"
    emb_path = models_dir / "embedding.onnx"
    
    if seg_path.exists():
        # Check if it's actually an ONNX file
        with open(seg_path, 'rb') as f:
            header = f.read(8)
            if header[:4] == b'\x08\x01\x12\x00' or header[:4] == b'PK\x03\x04':
                print(f"   ✓ Segmentation model is valid ONNX: {seg_path.stat().st_size / 1024 / 1024:.1f} MB")
            else:
                content = f.read(100).decode('utf-8', errors='ignore')
                if 'Access to model' in content:
                    print("   ✗ Segmentation file is an error message, not a model!")
                    print("   You need to accept the agreement at: https://huggingface.co/pyannote/segmentation-3.0")
                    return False
    
    if emb_path.exists():
        print(f"   ✓ Embedding model downloaded: {emb_path.stat().st_size / 1024 / 1024:.1f} MB")
    
    return True

if __name__ == "__main__":
    success = download_onnx_models()
    
    if success:
        print("\n✅ Models downloaded successfully!")
        print("Now the Rust code can load these ONNX models with ort.")
    else:
        print("\n❌ Failed to download some models.")
        print("\nNEXT STEPS:")
        print("1. Go to https://huggingface.co/pyannote/segmentation-3.0")
        print("2. Click 'Agree and access repository'")
        print("3. Run this script again")
        sys.exit(1)