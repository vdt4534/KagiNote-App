/**
 * End-to-end tests for speaker diarization functionality
 * 
 * These tests simulate real user workflows with speaker identification
 */

import { test, expect } from '@playwright/test';

// Mock data for testing
const mockMeetingConfig = {
  title: 'Test Meeting with Speakers',
  modelId: 'standard',
  language: 'en',
  enableSpeakerDiarization: true,
};

const mockSpeakerEvents = [
  {
    speakerId: 'speaker_1',
    displayName: 'Alice',
    confidence: 0.95,
    voiceCharacteristics: {
      pitch: 220.0,
      formantF1: 900.0,
      formantF2: 2100.0,
      speakingRate: 160.0,
    },
    isActive: true,
    color: '#3B82F6',
  },
  {
    speakerId: 'speaker_2',
    displayName: 'Bob', 
    confidence: 0.92,
    voiceCharacteristics: {
      pitch: 150.0,
      formantF1: 600.0,
      formantF2: 1700.0,
      speakingRate: 140.0,
    },
    isActive: true,
    color: '#10B981',
  },
];

test.describe('Speaker Diarization E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to app
    await page.goto('/');
    
    // Wait for app to be ready
    await expect(page.getByText('KagiNote')).toBeVisible();
  });

  test('should display speaker detection during recording', async ({ page }) => {
    // Start a new meeting
    await page.getByText('New Meeting').click();
    
    // Configure meeting with speaker diarization
    await page.fill('[data-testid="meeting-title"]', mockMeetingConfig.title);
    await page.selectOption('[data-testid="model-select"]', mockMeetingConfig.modelId);
    await page.check('[data-testid="enable-diarization"]');
    
    // Start recording
    await page.getByText('Start Recording').click();
    
    // Wait for recording screen
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Simulate speaker detection by emitting events
    await page.evaluate((speakers) => {
      // Simulate receiving speaker-update events from backend
      speakers.forEach((speaker, index) => {
        setTimeout(() => {
          const event = new CustomEvent('speaker-detected', {
            detail: {
              ...speaker,
              sessionId: 'test-session',
              timestamp: Date.now(),
            }
          });
          window.dispatchEvent(event);
        }, (index + 1) * 1000); // Stagger the events
      });
    }, mockSpeakerEvents);
    
    // Check that speakers appear in the UI
    await expect(page.getByTestId('speaker-card-speaker_1')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('speaker-card-speaker_2')).toBeVisible({ timeout: 6000 });
    
    // Verify speaker information
    await expect(page.getByText('Alice')).toBeVisible();
    await expect(page.getByText('Bob')).toBeVisible();
    await expect(page.getByText('2 Speakers Detected')).toBeVisible();
  });

  test('should allow speaker renaming during recording', async ({ page }) => {
    // Start recording with speakers
    await page.getByText('New Meeting').click();
    await page.fill('[data-testid="meeting-title"]', 'Speaker Rename Test');
    await page.check('[data-testid="enable-diarization"]');
    await page.getByText('Start Recording').click();
    
    // Wait for recording and simulate speaker detection
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Simulate speaker detection
    await page.evaluate((speaker) => {
      const event = new CustomEvent('speaker-detected', {
        detail: {
          ...speaker,
          sessionId: 'rename-test-session',
          timestamp: Date.now(),
        }
      });
      window.dispatchEvent(event);
    }, mockSpeakerEvents[0]);
    
    // Wait for speaker to appear
    await expect(page.getByTestId('speaker-card-speaker_1')).toBeVisible();
    
    // Click on speaker name to rename
    await page.getByTestId('speaker-name-button-speaker_1').click();
    
    // Verify rename dialog appears
    await expect(page.getByTestId('speaker-rename-dialog')).toBeVisible();
    
    // Enter new name
    await page.fill('input[type="text"]', 'John Smith');
    await page.getByText('Save').click();
    
    // Verify dialog closes and name is updated
    await expect(page.getByTestId('speaker-rename-dialog')).not.toBeVisible();
    await expect(page.getByText('John Smith')).toBeVisible();
  });

  test('should show speaker color customization', async ({ page }) => {
    // Start recording
    await page.getByText('New Meeting').click();
    await page.fill('[data-testid="meeting-title"]', 'Color Customization Test');
    await page.check('[data-testid="enable-diarization"]');
    await page.getByText('Start Recording').click();
    
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Simulate speaker detection
    await page.evaluate((speaker) => {
      const event = new CustomEvent('speaker-detected', {
        detail: {
          ...speaker,
          sessionId: 'color-test-session',
          timestamp: Date.now(),
        }
      });
      window.dispatchEvent(event);
    }, mockSpeakerEvents[0]);
    
    // Wait for speaker to appear
    await expect(page.getByTestId('speaker-card-speaker_1')).toBeVisible();
    
    // Click on color button
    await page.getByTestId('speaker-color-button-speaker_1').click();
    
    // Verify color picker appears
    await expect(page.getByTestId('speaker-color-picker')).toBeVisible();
    
    // Select a different color
    await page.getByTestId('color-option-red').click();
    
    // Verify color picker closes
    await expect(page.getByTestId('speaker-color-picker')).not.toBeVisible();
    
    // Verify the color change is reflected (this would need to check the actual style)
    const colorButton = page.getByTestId('speaker-color-button-speaker_1');
    const style = await colorButton.getAttribute('style');
    expect(style).toContain('#DC2626'); // Red color
  });

  test('should display transcript with speaker labels', async ({ page }) => {
    // Start recording
    await page.getByText('New Meeting').click();
    await page.fill('[data-testid="meeting-title"]', 'Transcript Test');
    await page.check('[data-testid="enable-diarization"]');
    await page.getByText('Start Recording').click();
    
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Simulate transcription events with speaker information
    await page.evaluate(() => {
      // Simulate a transcription update with speaker info
      const transcriptEvent = new CustomEvent('transcription-update', {
        detail: {
          sessionId: 'transcript-test-session',
          segment: {
            text: 'Hello, this is a test transcription.',
            startTime: 0.0,
            endTime: 3.0,
            speaker: 'speaker_1',
            speakerId: 'speaker_1',
            confidence: 0.95,
          },
          updateType: 'new',
          processingPass: 1,
        }
      });
      window.dispatchEvent(transcriptEvent);
    });
    
    // Check that transcript appears with speaker label
    await expect(page.getByText('Hello, this is a test transcription.')).toBeVisible();
    await expect(page.getByText('Speaker 1')).toBeVisible(); // Default speaker name
  });

  test('should handle graceful degradation when diarization fails', async ({ page }) => {
    // Mock diarization failure
    await page.route('**/diarize_audio_segment', route => {
      route.fulfill({
        status: 500,
        body: JSON.stringify({ error: 'Diarization service unavailable' }),
      });
    });
    
    // Start recording
    await page.getByText('New Meeting').click();
    await page.fill('[data-testid="meeting-title"]', 'Fallback Test');
    await page.check('[data-testid="enable-diarization"]');
    await page.getByText('Start Recording').click();
    
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Even with diarization failure, transcription should work with default speaker
    await page.evaluate(() => {
      const transcriptEvent = new CustomEvent('transcription-update', {
        detail: {
          sessionId: 'fallback-test-session',
          segment: {
            text: 'Fallback transcription without diarization.',
            startTime: 0.0,
            endTime: 3.0,
            speaker: 'speaker_1', // Default fallback
            confidence: 0.90,
          },
          updateType: 'new',
          processingPass: 1,
        }
      });
      window.dispatchEvent(transcriptEvent);
    });
    
    // Should show transcription with default speaker
    await expect(page.getByText('Fallback transcription without diarization.')).toBeVisible();
    await expect(page.getByText('Speaker 1')).toBeVisible();
    
    // Should not show multiple speakers
    await expect(page.getByText('1 Speakers Detected')).toBeVisible();
  });

  test('should persist speaker information after stopping recording', async ({ page }) => {
    // Complete recording workflow with speakers
    await page.getByText('New Meeting').click();
    await page.fill('[data-testid="meeting-title"]', 'Persistence Test');
    await page.check('[data-testid="enable-diarization"]');
    await page.getByText('Start Recording').click();
    
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Simulate speakers and transcription
    await page.evaluate((speakers) => {
      speakers.forEach((speaker, index) => {
        setTimeout(() => {
          // Speaker detection
          const speakerEvent = new CustomEvent('speaker-detected', {
            detail: {
              ...speaker,
              sessionId: 'persistence-test-session',
              timestamp: Date.now(),
            }
          });
          window.dispatchEvent(speakerEvent);
          
          // Transcription from this speaker
          setTimeout(() => {
            const transcriptEvent = new CustomEvent('transcription-update', {
              detail: {
                sessionId: 'persistence-test-session',
                segment: {
                  text: `This is ${speaker.displayName} speaking.`,
                  startTime: index * 3.0,
                  endTime: (index + 1) * 3.0,
                  speaker: speaker.speakerId,
                  speakerId: speaker.speakerId,
                  confidence: speaker.confidence,
                },
                updateType: 'new',
                processingPass: 1,
              }
            });
            window.dispatchEvent(transcriptEvent);
          }, 200);
        }, index * 1000);
      });
    }, mockSpeakerEvents);
    
    // Wait for speakers to appear
    await expect(page.getByText('2 Speakers Detected')).toBeVisible({ timeout: 10000 });
    
    // Stop recording
    await page.getByText('Stop Recording').click();
    
    // Should return to dashboard
    await expect(page.getByText('100% Local Privacy • No Cloud Required')).toBeVisible();
    
    // Check that meeting was saved with speaker information
    await expect(page.getByText('Persistence Test')).toBeVisible();
    
    // The meeting card should show multiple speakers
    const meetingCard = page.getByText('Persistence Test').locator('xpath=ancestor::div[contains(@class, "meeting-card") or contains(@class, "card")]');
    await expect(meetingCard.getByText('2')).toBeVisible(); // Speaker count
  });

  test('should show speaker statistics and timing', async ({ page }) => {
    // Start recording and simulate a conversation between speakers
    await page.getByText('New Meeting').click();
    await page.fill('[data-testid="meeting-title"]', 'Statistics Test');
    await page.check('[data-testid="enable-diarization"]');
    await page.getByText('Start Recording').click();
    
    await expect(page.getByText('Live Recording')).toBeVisible();
    
    // Simulate conversation with timing information
    await page.evaluate(() => {
      const events = [
        { speaker: 'speaker_1', text: 'Hello everyone.', start: 0, end: 2, name: 'Alice' },
        { speaker: 'speaker_2', text: 'Hi Alice, how are you?', start: 3, end: 6, name: 'Bob' },
        { speaker: 'speaker_1', text: 'I am doing well, thank you.', start: 7, end: 10, name: 'Alice' },
        { speaker: 'speaker_2', text: 'That is great to hear.', start: 11, end: 14, name: 'Bob' },
      ];
      
      events.forEach((event, index) => {
        setTimeout(() => {
          // Speaker detection
          const speakerEvent = new CustomEvent('speaker-detected', {
            detail: {
              speakerId: event.speaker,
              displayName: event.name,
              confidence: 0.94,
              isActive: true,
              sessionId: 'statistics-test-session',
              timestamp: Date.now(),
              color: event.speaker === 'speaker_1' ? '#3B82F6' : '#10B981',
            }
          });
          window.dispatchEvent(speakerEvent);
          
          // Transcription
          setTimeout(() => {
            const transcriptEvent = new CustomEvent('transcription-update', {
              detail: {
                sessionId: 'statistics-test-session',
                segment: {
                  text: event.text,
                  startTime: event.start,
                  endTime: event.end,
                  speaker: event.speaker,
                  speakerId: event.speaker,
                  confidence: 0.94,
                },
                updateType: 'new',
                processingPass: 1,
              }
            });
            window.dispatchEvent(transcriptEvent);
          }, 100);
        }, index * 1500);
      });
    });
    
    // Wait for conversation to complete
    await expect(page.getByText('That is great to hear.')).toBeVisible({ timeout: 10000 });
    
    // Check speaker statistics (this would depend on actual implementation)
    await expect(page.getByText('2 Speakers Detected')).toBeVisible();
    
    // Both speakers should have speaking time and confidence displayed
    const aliceCard = page.getByTestId('speaker-card-speaker_1');
    const bobCard = page.getByTestId('speaker-card-speaker_2');
    
    await expect(aliceCard).toContainText('Alice');
    await expect(bobCard).toContainText('Bob');
    
    // Each card should show some statistics (time and confidence)
    await expect(aliceCard).toContainText('94%'); // Confidence
    await expect(bobCard).toContainText('94%'); // Confidence
  });
});