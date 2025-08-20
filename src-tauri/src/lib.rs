//! KagiNote - Privacy-focused meeting transcription application
//! 
//! This Tauri application provides local audio capture, voice activity detection,
//! and speech recognition for meeting transcription with complete privacy.

pub mod audio;
pub mod asr;

use tauri::Manager;

// Tauri commands for frontend integration
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::start_audio_capture,
            commands::stop_audio_capture,
            commands::transcribe_audio,
            commands::get_audio_devices,
            commands::get_system_info,
            commands::start_transcription,
            commands::stop_transcription,
            commands::get_active_sessions,
            commands::cleanup_session,
            commands::emergency_stop_all
        ])
        .setup(|app| {
            // Initialize logging
            tracing_subscriber::fmt::init();
            
            // Store app handle for cleanup on exit
            let app_handle = app.handle().clone();
            
            // Perform any initialization here
            tauri::async_runtime::spawn(async {
                // Initialize audio and ASR systems
                if let Err(e) = initialize_systems().await {
                    tracing::error!("Failed to initialize systems: {}", e);
                }
            });
            
            // Register cleanup handler for when app is about to exit
            // Note: For Tauri v2, window event handling may be different
            // For now, we'll handle cleanup in a different way or skip this specific handler
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn initialize_systems() -> anyhow::Result<()> {
    // Initialize core systems
    tracing::info!("Initializing KagiNote systems...");
    
    // Validate audio system
    if let Err(e) = audio::capture::AudioCaptureService::validate_system() {
        tracing::warn!("Audio system validation failed: {}", e);
    }
    
    // Pre-load models if available
    // asr::whisper::WhisperEngine::preload_models().await?;
    
    tracing::info!("KagiNote systems initialized successfully");
    Ok(())
}

/// Cleanup app state when the application is shutting down
async fn cleanup_app_state(app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    tracing::info!("Cleaning up KagiNote app state...");
    
    let state = app_handle.state::<commands::AppState>();
    
    // Stop all active audio capture services
    let mut audio_capture_guard = state.audio_capture_service.lock().await;
    if let Some(mut capture_service) = audio_capture_guard.take() {
        if let Err(e) = capture_service.stop_capture().await {
            tracing::warn!("Failed to stop audio capture during cleanup: {}", e);
        }
    }
    drop(audio_capture_guard);
    
    // Clear all active sessions
    let mut sessions_guard = state.active_sessions.lock().await;
    let session_count = sessions_guard.len();
    sessions_guard.clear();
    drop(sessions_guard);
    
    // Whisper engine will be dropped automatically when the app state is dropped
    
    tracing::info!("Cleaned up {} active sessions and audio services", session_count);
    Ok(())
}
