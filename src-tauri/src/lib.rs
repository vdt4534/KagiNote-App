//! KagiNote - Privacy-focused meeting transcription application
//! 
//! This Tauri application provides local audio capture, voice activity detection,
//! and speech recognition for meeting transcription with complete privacy.

pub mod audio;
pub mod asr;

// Removed unused import

// Tauri commands for frontend integration
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::start_audio_capture,
            commands::stop_audio_capture,
            commands::transcribe_audio,
            commands::get_audio_devices,
            commands::get_system_info,
            commands::start_transcription,
            commands::stop_transcription
        ])
        .setup(|_app| {
            // Initialize logging
            tracing_subscriber::fmt::init();
            
            // Perform any initialization here
            tauri::async_runtime::spawn(async {
                // Initialize audio and ASR systems
                if let Err(e) = initialize_systems().await {
                    tracing::error!("Failed to initialize systems: {}", e);
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn initialize_systems() -> anyhow::Result<()> {
    // Initialize core systems
    tracing::info!("Initializing KagiNote systems...");
    
    // Validate audio system
    audio::capture::AudioCaptureService::validate_system()?;
    
    // Pre-load models if available
    // asr::whisper::WhisperEngine::preload_models().await?;
    
    tracing::info!("KagiNote systems initialized successfully");
    Ok(())
}
