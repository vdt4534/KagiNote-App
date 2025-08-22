# Scripts Directory

## For Developers Only

### download_sherpa_models.sh

Downloads the public Sherpa-ONNX speaker diarization models for development and bundling with the app.

**Important Notes:**
- These are **PUBLIC** models from Sherpa-ONNX GitHub releases
- **NO authentication required** - no Hugging Face or other accounts needed
- Models are downloaded to `src-tauri/resources/models/diarization/`
- These models ship bundled with the final app

**Usage:**
```bash
./scripts/download_sherpa_models.sh
```

The script downloads:
1. **Segmentation Model** (6MB) - Speech/silence detection
2. **Embedding Model** (71MB) - 3D-Speaker voice embeddings

## For End Users

**End users don't need to run any scripts.** The models are bundled with the application and work offline out of the box.