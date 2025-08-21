# On-Device Meeting Transcription – Technical Strategy (Windows & macOS, Multilingual)

**Audience:** Product + Engineering + Sales Enablement  
**Goal:** Ship a privacy-first meeting transcriber that runs fully on users' machines (Windows/macOS), supporting multilingual meetings with focus on English and Japanese. Users can select accuracy/speed tiers to match their hardware. No cloud inference by default.

**Version:** 2.0 - Revised based on technical review  
**Date:** January 2025

---

## 1) Core Principles & Goals

- **Privacy by Default:** All audio capture, ASR, VAD, diarization, and export run locally. No data leaves device without explicit user consent.
- **Multilingual First:** Support 100+ languages with optimized paths for English and Japanese.
- **Quality Over Speed:** Default to accuracy with options for faster processing.
- **Real-time with Refinement:** Stream transcripts with 1-2 second latency, background refinement for accuracy.
- **Smart Resource Management:** Automatic quality adjustment based on hardware capabilities and thermal state.
- **Modular Architecture:** Native core (Rust/C++) with swappable UI shells.

---

## 2) Model Portfolio & Tiers

### User-Selectable Quality Tiers

#### **Standard (Balanced Performance)**
- **Model:** Whisper Medium (INT8 on CPU, FP16 on GPU)
- **Engine:** faster-whisper (CTranslate2)
- **Languages:** 99 languages with good accuracy
- **Use Case:** Daily meetings, good hardware
- **Expected RTF:** 0.7-1.0× on modern CPUs

#### **High Accuracy**
- **Model:** Whisper Large-v3
- **Engine:** faster-whisper (CTranslate2) 
- **Languages:** 100 languages, best accuracy
- **Use Case:** Critical meetings, technical content
- **Expected RTF:** 1.2-2.0× on CPU, 0.3-0.5× on GPU

#### **High Accuracy Turbo**
- **Model:** Whisper Large-v3-Turbo
- **Engine:** faster-whisper (CTranslate2)
- **Languages:** 100 languages, 6× faster than Large-v3
- **Use Case:** Best overall for GPU systems
- **Expected RTF:** 0.5-0.8× on CPU, 0.1-0.3× on GPU

### Optional Performance Modes

#### **Draft Mode (User-Activated)**
- **Models:** Whisper Base or Small (INT8)
- **Warning:** "Draft quality - for quick previews only"
- **Use Case:** Very low-spec machines, quick notes

#### **Language-Specific Accelerators**
- **Japanese CPU Boost:** ReazonSpeech k2-v2 (ONNX)
  - 10× faster than Whisper for Japanese
  - Auto-offered when Japanese detected on CPU-only systems
- **English Fast Mode:** Whisper Base.en/Small.en (INT8)
  - 2× faster than multilingual equivalents
  - Offered for English-only meetings

### VAD Integration (Mandatory)
- **Model:** Silero-VAD v5 (1.5MB ONNX model)
- **Purpose:** Voice activity detection, prevents hallucination
- **Performance:** <1% CPU usage, 10-20ms latency
- **Integration:** Runs before ALL ASR models

---

## 3) Platform Optimization Matrix

| Platform | Configuration | Standard | High Accuracy | Turbo | Diarization |
|----------|--------------|----------|---------------|-------|-------------|
| **Windows (CPU-only)** | 8+ cores, 16GB RAM | Medium INT8 (CT2) | Large-v3 INT8 (async) | L-v3-Turbo INT8 | sherpa-onnx |
| **Windows (Intel iGPU)** | With OpenVINO | Medium INT8 + OpenVINO | Large-v3 (async) | L-v3-Turbo INT8 | sherpa-onnx |
| **Windows (NVIDIA)** | 4GB+ VRAM | Medium FP16 CUDA | Large-v3 FP16 CUDA | L-v3-Turbo FP16 | pyannote 3.1 |
| **macOS (Intel)** | 16GB+ RAM | whisper.cpp Medium | Large-v3 (async) | L-v3-Turbo | sherpa-onnx |
| **macOS (Apple Silicon)** | M1/M2/M3 | Medium Q5_K (Metal) | Large-v3 Q5_K | L-v3-Turbo Q5_K | sherpa-onnx |

### Language-Specific Routing

| Detected Language | CPU Deployment | GPU Deployment |
|-------------------|---------------|----------------|
| Japanese | ReazonSpeech k2-v2 → Whisper (fallback) | Whisper Turbo |
| English | Whisper Medium.en INT8 | Whisper Turbo |
| Mixed/Other | Whisper Medium INT8 | Whisper Turbo |
| Unknown | Whisper Medium (multilingual) | Whisper Turbo |

---

## 4) Real-Time Processing Pipeline

### 4.1) VAD-Based Dynamic Chunking

```
Audio Stream → Silero-VAD → Speech Detection → Dynamic Chunking → ASR
                    ↓
              Silence (skip)
```

**VAD Configuration:**
- Speech threshold: 0.5 (probability)
- Min speech duration: 0.5 seconds
- Max speech duration: 30 seconds  
- Silence for split: 0.5 seconds
- Context padding: 400ms before/after

### 4.2) Two-Pass Architecture

#### **Pass 1: Real-Time (Display immediately)**
- Process VAD-detected segments
- 10-second sliding window with 2-second overlap
- Display with ~1 second latency
- Include last 50 words as prompt context

#### **Pass 2: Refinement (Background)**
- Re-process last 30 seconds during pauses
- Full context conditioning
- Speaker-specific language models
- Update display with refined text

### 4.3) Context-Aware Streaming

```python
class ContextualTranscriptionPipeline:
    def __init__(self):
        self.vad = SileroVAD(threshold=0.5)
        self.audio_buffer = RingBuffer(30_seconds)
        self.context_buffer = []  # Last 100 words
        self.pending_segments = []
        
    def process_chunk(self, audio_chunk):
        # VAD preprocessing
        if self.vad.is_speech(audio_chunk):
            self.pending_segments.append(audio_chunk)
            
            # Check for natural boundary
            if self.should_process():
                return self.transcribe_with_context()
        
    def transcribe_with_context(self):
        # Include overlap and context
        audio = self.prepare_audio_with_overlap()
        prompt = f"Context: {' '.join(self.context_buffer[-50:])}"
        
        result = self.whisper_model.transcribe(
            audio,
            prompt=prompt,
            temperature=0.0,  # Deterministic
            condition_on_previous_text=True
        )
        
        # Merge and remove duplicates
        return self.merge_with_previous(result)
```

---

## 5) Diarization Strategy

### 5.1) Dual-Mode Diarization

#### **CPU Mode: sherpa-onnx**
- Lightweight speaker segmentation
- 3-5% CPU usage
- 2-3 speakers max (optimal)
- Real-time capable

#### **GPU Mode: pyannote 3.1**
- High accuracy, multi-speaker
- Requires HuggingFace token
- 10+ speakers supported
- Higher latency (processes in chunks)

### 5.2) Speaker Embedding Pipeline

1. **Initial Clustering:** ECAPA-TDNN embeddings
2. **Online Refinement:** Update speaker profiles during meeting
3. **Stabilization:** Median filtering, prevent speaker flipping
4. **Language Association:** Track language preference per speaker

---

## 6) Audio Capture Architecture

### 6.1) Windows Audio Pipeline
- **Primary:** WASAPI loopback for system audio
- **Secondary:** WDM for microphone capture
- **Fallback:** Virtual audio cable (if WASAPI blocked)
- **Split Channels:** Separate "You" from remote participants

### 6.2) macOS Audio Pipeline
- **Primary:** Core Audio + BlackHole (guided install)
- **Alternative:** Browser extension with Native Messaging
- **Fallback:** Screen recording API with audio

### 6.3) Audio Preprocessing
- Resample to 16kHz (Whisper requirement)
- Mono downmix for transcription
- Preserve stereo for speaker separation
- AGC and noise suppression (optional)

---

## 7) Language Detection & Routing

### 7.1) Multi-Stage Language Detection

```python
class LanguageRouter:
    def __init__(self):
        self.detection_model = WhisperModel("base", compute_type="int8")
        self.language_cache = {}  # Speaker -> Language mapping
        
    def detect_and_route(self, audio_segment, speaker_id=None):
        # Check cache first
        if speaker_id and speaker_id in self.language_cache:
            return self.language_cache[speaker_id]
        
        # Quick detection on first 5 seconds
        result = self.detection_model.detect_language(audio_segment[:5*16000])
        
        if result.confidence > 0.8:
            # High confidence - route to optimized model
            if result.language == "ja" and self.is_cpu_only:
                return "reazon_k2"  # ReazonSpeech
            elif result.language == "en":
                return "whisper_en"  # English-optimized
            else:
                return "whisper_multi"  # Multilingual
        else:
            # Low confidence - use multilingual
            return "whisper_multi"
```

### 7.2) User Controls
- **Auto-Detect** (default): Smart routing based on detection
- **Force Language:** Lock to specific language model
- **Multilingual Mode:** Always use multilingual models
- **Per-Speaker:** Different models per speaker (advanced)

---

## 8) Hardware Requirements & Auto-Configuration

### 8.1) Minimum Requirements

#### **CPU-Only Systems**
- Processor: 6-core modern CPU (Intel 10th gen / AMD Ryzen 3000+)
- RAM: 16GB (8GB models + 8GB system)
- Storage: 8GB for models
- **Expected Performance:** Real-time with Medium model

#### **GPU-Accelerated**
- Same as above PLUS:
- GPU: 4GB VRAM minimum (GTX 1650 or better)
- CUDA 12.0+ or ROCm 5.0+

### 8.2) Recommended Configuration

#### **Smooth Real-Time Experience**
- CPU: 8-core with AVX2
- RAM: 24GB
- GPU: 6GB VRAM (RTX 3060 or better)
- **Capability:** Real-time with Large-v3-Turbo + diarization

### 8.3) Auto-Configuration Logic

```python
def auto_configure_models(system_info):
    config = ModelConfig()
    
    # Check thermal state
    if system_info.cpu_temp > 80:
        config.downgrade_tier()
    
    # Check available memory
    available_ram = system_info.free_ram
    if available_ram < 4_000:  # Less than 4GB
        config.model = "whisper_base"  # Minimal model
        config.enable_swap = True
    elif available_ram < 8_000:
        config.model = "whisper_medium_int8"
        config.batch_size = 1
    
    # Check GPU availability
    if system_info.has_gpu and system_info.vram >= 4_000:
        config.device = "cuda"
        config.compute_type = "float16"
        config.enable_pyannote = True
    else:
        config.device = "cpu"
        config.compute_type = "int8"
        config.diarization = "sherpa-onnx"
    
    return config
```

---

## 9) Memory Management & Performance

### 9.1) Model Loading Strategy

#### **Lazy Loading**
- Load only required models
- Keep base model for detection
- Swap based on detected languages

#### **Memory Pressure Response**
```python
if available_ram < 4GB:
    unload_unused_models()
if available_ram < 2GB:
    downgrade_to_smaller_model()
if available_ram < 1GB:
    prompt_cloud_fallback()  # With user permission
```

### 9.2) Thermal Management

- Monitor CPU/GPU temperature
- Automatic quality reduction at 80°C
- Force cooldown periods at 90°C
- User notification of thermal throttling

### 9.3) Performance Metrics

| Metric | Target | Acceptable | Poor |
|--------|--------|------------|------|
| RTF (Real-time factor) | <0.5 | 0.5-1.0 | >1.0 |
| Latency to first word | <1s | 1-2s | >2s |
| Memory usage | <4GB | 4-8GB | >8GB |
| CPU usage (average) | <50% | 50-75% | >75% |
| Diarization Error Rate | <10% | 10-20% | >20% |

---

## 10) Export & Integration

### 10.1) Export Formats

#### **Basic Formats**
- TXT: Plain text with timestamps
- SRT: Subtitle format with timing
- VTT: WebVTT for web players
- JSON: Structured data with metadata

#### **Advanced Formats**
- TTML: Broadcast subtitle standard
- ASS: Advanced SubStation with styling
- Word-level JSON: With confidence scores
- Speaker-separated: Individual files per speaker

### 10.2) Progressive Export

1. **Draft Export:** Available immediately (Pass 1 results)
2. **Refined Export:** After Pass 2 completion
3. **Final Export:** With manual corrections

### 10.3) Integration APIs

```python
# WebSocket API for real-time streaming
ws://localhost:8765/transcribe

# REST API for batch processing
POST /api/transcribe
GET /api/status/{job_id}
GET /api/export/{job_id}?format=srt

# Native messaging for browser extension
chrome-extension://[id]/native-messaging
```

---

## 11) Implementation Roadmap

### Phase 0: MVP (4-6 weeks)
- [x] Windows/macOS audio capture
- [x] Whisper Medium + VAD integration  
- [x] Basic sherpa-onnx diarization
- [x] TXT/SRT export
- [x] Simple UI with model selection

### Phase 1: Production Ready (8-10 weeks)
- [ ] Large-v3-Turbo integration
- [ ] Two-pass refinement pipeline
- [ ] pyannote for GPU systems
- [ ] Language detection & routing
- [ ] ReazonSpeech for Japanese
- [ ] Advanced export formats
- [ ] Thermal management

### Phase 2: Enhanced Features (12-14 weeks)
- [ ] Browser extension
- [ ] Custom vocabulary support
- [ ] Real-time translation
- [ ] Meeting summarization (local LLM)
- [ ] Cloud backup (encrypted)
- [ ] Team collaboration features

---

## 12) Testing & Benchmarks

### 12.1) Test Matrix

| Test Scenario | Models | Languages | Duration | Speakers |
|---------------|--------|-----------|----------|----------|
| Business Meeting | All tiers | EN/JP/Mixed | 30 min | 4 |
| Technical Presentation | Large-v3 | EN+JP terms | 60 min | 1 |
| Interview | Turbo | Accented EN | 45 min | 2 |
| Conference Call | Medium | Multilingual | 90 min | 8 |

### 12.2) Performance Benchmarks

```python
def benchmark_configuration(audio_file, model_config):
    metrics = {
        'rtf': measure_real_time_factor(),
        'wer': calculate_word_error_rate(),
        'der': calculate_diarization_error_rate(),
        'memory_peak': get_peak_memory(),
        'cpu_average': get_average_cpu(),
        'first_word_latency': measure_latency(),
        'thermal_throttle_events': count_throttles()
    }
    return metrics
```

### 12.3) Acceptance Criteria

- **RTF < 1.0** for Standard tier on minimum spec
- **WER < 10%** for English, **CER < 8%** for Japanese
- **DER < 15%** for 2-4 speakers
- **Memory < 8GB** peak usage
- **No crashes** in 4-hour continuous operation

---

## 13) Risk Mitigation

### 13.1) Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Thermal throttling | Performance degradation | Auto-downgrade, cooling periods |
| Memory exhaustion | Application crash | Model swapping, memory limits |
| Audio capture blocked | No transcription | Multiple capture methods |
| Language misdetection | Wrong model used | Confidence thresholds, user override |
| Diarization failure | Mixed speakers | Fallback to single speaker |

### 13.2) Legal & Compliance

- **Model Licenses:** Document all model licenses
- **Audio Privacy:** No retention without consent  
- **GDPR Compliance:** Data deletion on request
- **Accessibility:** Screen reader support
- **Export Compliance:** No encryption export

### 13.3) User Experience Risks

- **Complexity:** Provide "Easy Mode" with auto-config
- **Poor hardware:** Clear minimum requirements
- **Language mixing:** Educate on multilingual mode
- **Accuracy expectations:** Set realistic expectations

---

## 14) Security & Privacy

### 14.1) Data Protection

- **Local-Only by Default:** No network calls without permission
- **Encryption at Rest:** AES-256 for stored transcripts
- **Memory Protection:** Secure wiping of audio buffers
- **No Telemetry:** Opt-in anonymous usage stats only

### 14.2) Model Security

- **Checksum Verification:** Validate model integrity
- **Sandboxed Execution:** Models run in restricted environment
- **No Auto-Updates:** User controls all updates
- **Local Model Storage:** Encrypted model cache

---

## 15) Open Questions & Decisions Needed

1. **Model Bundling:** Include models in installer (larger) or download on-demand?
2. **Cloud Fallback:** Offer optional cloud API for low-spec devices?
3. **Pricing Tiers:** Free tier limitations? Premium features?
4. **Enterprise Features:** Central management? Compliance logging?
5. **Mobile Support:** iOS/Android apps in Phase 3?
6. **Translation:** Real-time translation in Phase 2 or 3?
7. **Summarization:** Local LLM (Llama) or cloud API?

---

## Appendix A: Model Specifications

| Model | Size (INT8) | Size (FP16) | Parameters | RTF (CPU) | RTF (GPU) |
|-------|-------------|-------------|------------|-----------|-----------|
| Whisper Base | 39 MB | 74 MB | 39M | 0.3× | 0.05× |
| Whisper Small | 166 MB | 244 MB | 39M | 0.5× | 0.08× |
| Whisper Medium | 769 MB | 1.5 GB | 769M | 1.0× | 0.15× |
| Whisper Large-v3 | 1.6 GB | 3.1 GB | 1.55B | 2.0× | 0.3× |
| Whisper Large-v3-Turbo | 809 MB | 1.6 GB | 809M | 0.8× | 0.12× |
| ReazonSpeech k2-v2 | 636 MB | N/A | 636M | 0.3× | N/A |

## Appendix B: VAD Configuration Examples

```yaml
# Meeting Configuration
meeting:
  threshold: 0.5
  min_speech_duration_ms: 500
  max_speech_duration_ms: 30000
  min_silence_duration_ms: 500
  speech_pad_ms: 400

# Presentation Configuration  
presentation:
  threshold: 0.4
  min_speech_duration_ms: 1000
  max_speech_duration_ms: 60000
  min_silence_duration_ms: 1000
  speech_pad_ms: 600

# Interview Configuration
interview:
  threshold: 0.5
  min_speech_duration_ms: 300
  max_speech_duration_ms: 20000
  min_silence_duration_ms: 300
  speech_pad_ms: 300
```

## Appendix C: Quick Command Reference

```bash
# Convert Whisper model to CTranslate2 INT8
ct2-transformers-converter --model openai/whisper-large-v3-turbo \
  --output_dir models/whisper-turbo-int8 \
  --quantization int8 \
  --copy_files tokenizer.json preprocessor_config.json

# Run with faster-whisper
python transcribe.py --model models/whisper-turbo-int8 \
  --compute_type int8 \
  --vad_filter true \
  --language auto

# Benchmark performance
python benchmark.py --audio test.wav \
  --models "medium,large-v3,turbo" \
  --metrics "rtf,wer,memory"
```

---

**END OF DOCUMENT**

*Last Updated: January 2025*  
*Version: 2.0*  
*Status: Ready for Technical Review*