/**
 * End-to-End Tests for Speaker Diarization Workflow
 * 
 * These tests validate the complete user journey for speaker diarization functionality.
 * ALL TESTS WILL FAIL initially because the features don't exist yet.
 * Tests define the expected user experience and drive implementation.
 */

import { test, expect, Page } from '@playwright/test';

test.describe('Speaker Diarization E2E Tests', () => {
  
  test.beforeEach(async ({ page }) => {
    // Start the application
    await page.goto('/');
    await expect(page.locator('[data-testid="app-title"]')).toContainText('KagiNote');
  });

  /// Test complete meeting recording with speaker identification
  /// WILL FAIL - speaker diarization recording doesn't exist
  test('should record meeting and identify multiple speakers', async ({ page }) => {
    // Start a new meeting with speaker diarization enabled
    await page.click('[data-testid="new-meeting-button"]');
    await page.fill('[data-testid="meeting-title-input"]', 'Team Standup with Speakers');
    
    // Enable speaker diarization
    await page.check('[data-testid="enable-speaker-diarization-checkbox"]');
    await expect(page.locator('[data-testid="enable-speaker-diarization-checkbox"]')).toBeChecked();
    
    // Start recording
    await page.click('[data-testid="start-recording-button"]');
    await expect(page.locator('[data-testid="recording-status"]')).toContainText('Recording with Speaker Detection');
    
    // Simulate audio input that would be processed by backend
    // In real scenario, this would involve actual microphone input
    await page.evaluate(() => {
      // Simulate backend sending speaker diarization updates
      window.dispatchEvent(new CustomEvent('speaker-detected', {
        detail: { speakerId: 'speaker_1', confidence: 0.92, timestamp: 5.2 }
      }));
    });
    
    // Wait for first speaker to be detected
    await expect(page.locator('[data-testid="speaker-count"]')).toContainText('1');
    await expect(page.locator('[data-testid="speaker-speaker_1"]')).toBeVisible();
    
    // Simulate second speaker detection
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('speaker-detected', {
        detail: { speakerId: 'speaker_2', confidence: 0.88, timestamp: 12.7 }
      }));
    });
    
    // Wait for second speaker
    await expect(page.locator('[data-testid="speaker-count"]')).toContainText('2');
    await expect(page.locator('[data-testid="speaker-speaker_2"]')).toBeVisible();
    
    // Continue recording for a bit more
    await page.waitForTimeout(5000);
    
    // Stop recording
    await page.click('[data-testid="stop-recording-button"]');
    await expect(page.locator('[data-testid="recording-status"]')).toContainText('Processing speakers...');
    
    // Wait for processing to complete
    await expect(page.locator('[data-testid="recording-status"]')).toContainText('Recording complete');
    
    // Verify final results
    await expect(page.locator('[data-testid="final-speaker-count"]')).toContainText('2 speakers identified');
    await expect(page.locator('[data-testid="speaker-confidence-speaker_1"]')).toContainText('92%');
    await expect(page.locator('[data-testid="speaker-confidence-speaker_2"]')).toContainText('88%');
  });

  /// Test real-time speaker display during recording
  /// WILL FAIL - real-time speaker display doesn't exist
  test('should show real-time speaker information during recording', async ({ page }) => {
    await startRecordingWithSpeakers(page);
    
    // Verify initial state
    await expect(page.locator('[data-testid="live-speakers-panel"]')).toBeVisible();
    await expect(page.locator('[data-testid="current-speaker-indicator"]')).toContainText('Detecting speakers...');
    
    // Simulate speaker activity
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('speaker-activity', {
        detail: { 
          speakerId: 'speaker_1', 
          isActive: true, 
          confidence: 0.94,
          text: 'Good morning everyone, let\'s start the meeting'
        }
      }));
    });
    
    // Check real-time updates
    await expect(page.locator('[data-testid="current-speaker"]')).toContainText('Speaker 1');
    await expect(page.locator('[data-testid="current-speaker-confidence"]')).toContainText('94%');
    await expect(page.locator('[data-testid="live-transcript"]')).toContainText('Good morning everyone');
    
    // Speaker indicator should show as active
    await expect(page.locator('[data-testid="speaker-activity-speaker_1"]')).toHaveClass(/active/);
    
    // Simulate speaker change
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('speaker-activity', {
        detail: { 
          speakerId: 'speaker_2', 
          isActive: true, 
          confidence: 0.87,
          text: 'Thanks for organizing this meeting'
        }
      }));
    });
    
    // Verify speaker change
    await expect(page.locator('[data-testid="current-speaker"]')).toContainText('Speaker 2');
    await expect(page.locator('[data-testid="speaker-activity-speaker_1"]')).not.toHaveClass(/active/);
    await expect(page.locator('[data-testid="speaker-activity-speaker_2"]')).toHaveClass(/active/);
  });

  /// Test speaker renaming workflow
  /// WILL FAIL - speaker renaming doesn't exist
  test('should allow renaming speakers during and after recording', async ({ page }) => {
    await startRecordingWithSpeakers(page);
    await simulateTwoSpeakersDetected(page);
    
    // Rename first speaker during recording
    await page.click('[data-testid="speaker-menu-speaker_1"]');
    await page.click('[data-testid="rename-speaker-button"]');
    
    // Should open rename dialog
    await expect(page.locator('[data-testid="speaker-rename-dialog"]')).toBeVisible();
    await page.fill('[data-testid="speaker-name-input"]', 'Alice');
    await page.click('[data-testid="save-speaker-name-button"]');
    
    // Verify name change
    await expect(page.locator('[data-testid="speaker-name-speaker_1"]')).toContainText('Alice');
    
    // Stop recording
    await page.click('[data-testid="stop-recording-button"]');
    
    // Rename second speaker after recording
    await page.click('[data-testid="speaker-menu-speaker_2"]');
    await page.click('[data-testid="rename-speaker-button"]');
    await page.fill('[data-testid="speaker-name-input"]', 'Bob');
    await page.click('[data-testid="save-speaker-name-button"]');
    
    // Verify both names are preserved
    await expect(page.locator('[data-testid="speaker-name-speaker_1"]')).toContainText('Alice');
    await expect(page.locator('[data-testid="speaker-name-speaker_2"]')).toContainText('Bob');
    
    // Names should appear in transcript
    await expect(page.locator('[data-testid="transcript-with-speakers"]')).toContainText('Alice:');
    await expect(page.locator('[data-testid="transcript-with-speakers"]')).toContainText('Bob:');
  });

  /// Test speaker color assignment and customization
  /// WILL FAIL - speaker colors don't exist
  test('should assign unique colors to speakers and allow customization', async ({ page }) => {
    await startRecordingWithSpeakers(page);
    await simulateMultipleSpeakersDetected(page, 4);
    
    // Verify unique colors are assigned
    const speaker1Color = await page.locator('[data-testid="speaker-color-speaker_1"]').evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    const speaker2Color = await page.locator('[data-testid="speaker-color-speaker_2"]').evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    
    expect(speaker1Color).not.toBe(speaker2Color);
    
    // Change speaker color
    await page.click('[data-testid="speaker-color-button-speaker_1"]');
    await expect(page.locator('[data-testid="color-picker"]')).toBeVisible();
    
    await page.click('[data-testid="color-option-red"]');
    
    // Verify color change
    const newColor = await page.locator('[data-testid="speaker-color-speaker_1"]').evaluate(el => 
      window.getComputedStyle(el).backgroundColor
    );
    expect(newColor).toContain('rgb(220, 38, 38)'); // Red color
    
    // Color should be reflected in transcript
    await page.click('[data-testid="stop-recording-button"]');
    
    const transcriptSegment = page.locator('[data-testid="transcript-segment-speaker_1"]').first();
    const segmentBorderColor = await transcriptSegment.evaluate(el => 
      window.getComputedStyle(el).borderLeftColor
    );
    expect(segmentBorderColor).toContain('rgb(220, 38, 38)');
  });

  /// Test speaker statistics and analytics
  /// WILL FAIL - speaker statistics don't exist
  test('should display comprehensive speaker statistics', async ({ page }) => {
    await recordMeetingWithMultipleSpeakers(page);
    
    // Open statistics panel
    await page.click('[data-testid="speaker-statistics-button"]');
    await expect(page.locator('[data-testid="speaker-statistics-panel"]')).toBeVisible();
    
    // Verify basic statistics
    await expect(page.locator('[data-testid="total-speakers"]')).toContainText('3');
    await expect(page.locator('[data-testid="total-meeting-time"]')).toMatch(/\d+m \d+s/);
    await expect(page.locator('[data-testid="average-confidence"]')).toMatch(/\d+%/);
    
    // Verify speaker distribution chart
    await expect(page.locator('[data-testid="speaker-distribution-chart"]')).toBeVisible();
    
    // Verify individual speaker stats
    const speaker1Stats = page.locator('[data-testid="speaker-stats-speaker_1"]');
    await expect(speaker1Stats.locator('[data-testid="speech-time"]')).toMatch(/\d+m \d+s/);
    await expect(speaker1Stats.locator('[data-testid="speech-percentage"]')).toMatch(/\d+%/);
    await expect(speaker1Stats.locator('[data-testid="segments-count"]')).toMatch(/\d+ segments/);
    
    // Test export statistics
    await page.click('[data-testid="export-statistics-button"]');
    const downloadPromise = page.waitForEvent('download');
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/speaker-statistics.*\.json/);
  });

  /// Test transcript export with speaker information
  /// WILL FAIL - transcript export with speakers doesn't exist
  test('should export transcript with speaker information', async ({ page }) => {
    await recordMeetingWithMultipleSpeakers(page);
    
    // Rename speakers for better export
    await renameSpeaker(page, 'speaker_1', 'Project Manager');
    await renameSpeaker(page, 'speaker_2', 'Lead Developer');
    await renameSpeaker(page, 'speaker_3', 'UX Designer');
    
    // Test different export formats
    await page.click('[data-testid="export-transcript-button"]');
    await expect(page.locator('[data-testid="export-dialog"]')).toBeVisible();
    
    // Text format with speakers
    await page.check('[data-testid="include-speakers-checkbox"]');
    await page.check('[data-testid="include-timestamps-checkbox"]');
    await page.selectOption('[data-testid="export-format-select"]', 'txt');
    
    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-confirm-button"]');
    const download = await downloadPromise;
    
    expect(download.suggestedFilename()).toMatch(/transcript.*\.txt/);
    
    // Verify export content (in a real scenario, we'd check file contents)
    await expect(page.locator('[data-testid="export-preview"]')).toContainText('Project Manager:');
    await expect(page.locator('[data-testid="export-preview"]')).toContainText('Lead Developer:');
    await expect(page.locator('[data-testid="export-preview"]')).toContainText('UX Designer:');
  });

  /// Test speaker search and filtering
  /// WILL FAIL - speaker search doesn't exist
  test('should allow searching and filtering by speakers', async ({ page }) => {
    await recordMeetingWithMultipleSpeakers(page);
    await renameSpeaker(page, 'speaker_1', 'Alice Johnson');
    await renameSpeaker(page, 'speaker_2', 'Bob Smith');
    await renameSpeaker(page, 'speaker_3', 'Carol Davis');
    
    // Test speaker search
    await page.fill('[data-testid="speaker-search-input"]', 'Alice');
    await expect(page.locator('[data-testid="speaker-Alice Johnson"]')).toBeVisible();
    await expect(page.locator('[data-testid="speaker-Bob Smith"]')).not.toBeVisible();
    await expect(page.locator('[data-testid="speaker-Carol Davis"]')).not.toBeVisible();
    
    // Clear search
    await page.fill('[data-testid="speaker-search-input"]', '');
    
    // Test transcript filtering by speaker
    await page.click('[data-testid="filter-by-speaker-button"]');
    await page.click('[data-testid="speaker-filter-Alice Johnson"]');
    
    // Only Alice's segments should be visible
    await expect(page.locator('[data-testid="transcript-segment"]')).toHaveCount(
      await page.locator('[data-testid="transcript-segment"][data-speaker-id="speaker_1"]').count()
    );
    
    // Test multiple speaker selection
    await page.click('[data-testid="speaker-filter-Bob Smith"]', { modifierKey: 'Ctrl' });
    
    // Should show Alice and Bob segments
    const aliceSegments = await page.locator('[data-testid="transcript-segment"][data-speaker-id="speaker_1"]').count();
    const bobSegments = await page.locator('[data-testid="transcript-segment"][data-speaker-id="speaker_2"]').count();
    await expect(page.locator('[data-testid="transcript-segment"]')).toHaveCount(aliceSegments + bobSegments);
  });

  /// Test error handling and edge cases
  /// WILL FAIL - error handling doesn't exist
  test('should handle speaker diarization errors gracefully', async ({ page }) => {
    // Start recording
    await page.click('[data-testid="new-meeting-button"]');
    await page.check('[data-testid="enable-speaker-diarization-checkbox"]');
    await page.click('[data-testid="start-recording-button"]');
    
    // Simulate backend error
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('speaker-diarization-error', {
        detail: { error: 'Model loading failed', code: 'MODEL_LOAD_ERROR' }
      }));
    });
    
    // Should show error message
    await expect(page.locator('[data-testid="speaker-error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="speaker-error-message"]')).toContainText('Speaker identification temporarily unavailable');
    
    // Should offer fallback options
    await expect(page.locator('[data-testid="continue-without-speakers-button"]')).toBeVisible();
    await expect(page.locator('[data-testid="retry-speakers-button"]')).toBeVisible();
    
    // Test retry functionality
    await page.click('[data-testid="retry-speakers-button"]');
    await expect(page.locator('[data-testid="speaker-status"]')).toContainText('Retrying speaker detection...');
    
    // Simulate successful retry
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('speaker-diarization-ready', {
        detail: { status: 'ready' }
      }));
    });
    
    await expect(page.locator('[data-testid="speaker-status"]')).toContainText('Speaker detection active');
  });

  /// Test performance with long meetings
  /// WILL FAIL - performance optimization doesn't exist
  test('should handle long meetings with many speakers efficiently', async ({ page }) => {
    // Start long meeting
    await page.click('[data-testid="new-meeting-button"]');
    await page.check('[data-testid="enable-speaker-diarization-checkbox"]');
    await page.click('[data-testid="start-recording-button"]');
    
    // Simulate 1-hour meeting with 8 speakers
    await simulateLongMeetingWithManySpeakers(page, 3600, 8);
    
    // Check UI remains responsive
    const startTime = Date.now();
    await page.click('[data-testid="speaker-statistics-button"]');
    const responseTime = Date.now() - startTime;
    expect(responseTime).toBeLessThan(1000); // Should respond within 1 second
    
    // Verify all speakers are identified
    await expect(page.locator('[data-testid="total-speakers"]')).toContainText('8');
    
    // Check memory usage indicators
    const memoryUsage = await page.evaluate(() => {
      return (performance as any).memory?.usedJSHeapSize || 0;
    });
    
    // Should not exceed reasonable memory limits (100MB)
    expect(memoryUsage).toBeLessThan(100 * 1024 * 1024);
    
    // Stop recording should complete quickly
    const stopStartTime = Date.now();
    await page.click('[data-testid="stop-recording-button"]');
    await expect(page.locator('[data-testid="recording-status"]')).toContainText('Recording complete');
    const stopTime = Date.now() - stopStartTime;
    expect(stopTime).toBeLessThan(5000); // Should complete within 5 seconds
  });

  /// Test accessibility features
  /// WILL FAIL - accessibility features don't exist
  test('should be accessible to screen readers and keyboard users', async ({ page }) => {
    await recordMeetingWithMultipleSpeakers(page);
    
    // Test keyboard navigation
    await page.keyboard.press('Tab');
    await expect(page.locator(':focus')).toHaveAttribute('data-testid', 'speaker-speaker_1');
    
    await page.keyboard.press('Tab');
    await expect(page.locator(':focus')).toHaveAttribute('data-testid', 'speaker-speaker_2');
    
    // Test ARIA labels
    await expect(page.locator('[data-testid="speaker-speaker_1"]')).toHaveAttribute('aria-label', /Speaker 1.*confidence/);
    
    // Test screen reader announcements
    const announcements = await page.evaluate(() => {
      return document.querySelector('[aria-live="polite"]')?.textContent;
    });
    expect(announcements).toMatch(/speakers? detected/i);
    
    // Test high contrast mode compatibility
    await page.emulateMedia({ colorScheme: 'dark' });
    const speakerCard = page.locator('[data-testid="speaker-speaker_1"]');
    const contrast = await speakerCard.evaluate(el => {
      const style = window.getComputedStyle(el);
      const bgColor = style.backgroundColor;
      const textColor = style.color;
      // Basic contrast check (simplified)
      return { bgColor, textColor };
    });
    
    expect(contrast.bgColor).not.toBe(contrast.textColor);
  });

  // Helper functions for test setup
  async function startRecordingWithSpeakers(page: Page) {
    await page.click('[data-testid="new-meeting-button"]');
    await page.check('[data-testid="enable-speaker-diarization-checkbox"]');
    await page.click('[data-testid="start-recording-button"]');
  }

  async function simulateTwoSpeakersDetected(page: Page) {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('speaker-detected', {
        detail: { speakerId: 'speaker_1', confidence: 0.92, timestamp: 5.2 }
      }));
      window.dispatchEvent(new CustomEvent('speaker-detected', {
        detail: { speakerId: 'speaker_2', confidence: 0.88, timestamp: 12.7 }
      }));
    });
    
    await expect(page.locator('[data-testid="speaker-count"]')).toContainText('2');
  }

  async function simulateMultipleSpeakersDetected(page: Page, speakerCount: number) {
    for (let i = 1; i <= speakerCount; i++) {
      await page.evaluate((index) => {
        window.dispatchEvent(new CustomEvent('speaker-detected', {
          detail: { 
            speakerId: `speaker_${index}`, 
            confidence: 0.85 + (Math.random() * 0.15), 
            timestamp: index * 10 
          }
        }));
      }, i);
    }
    
    await expect(page.locator('[data-testid="speaker-count"]')).toContainText(speakerCount.toString());
  }

  async function recordMeetingWithMultipleSpeakers(page: Page) {
    await startRecordingWithSpeakers(page);
    await simulateMultipleSpeakersDetected(page, 3);
    await page.waitForTimeout(2000); // Simulate recording time
    await page.click('[data-testid="stop-recording-button"]');
    await expect(page.locator('[data-testid="recording-status"]')).toContainText('Recording complete');
  }

  async function renameSpeaker(page: Page, speakerId: string, newName: string) {
    await page.click(`[data-testid="speaker-menu-${speakerId}"]`);
    await page.click('[data-testid="rename-speaker-button"]');
    await page.fill('[data-testid="speaker-name-input"]', newName);
    await page.click('[data-testid="save-speaker-name-button"]');
    await expect(page.locator(`[data-testid="speaker-name-${speakerId}"]`)).toContainText(newName);
  }

  async function simulateLongMeetingWithManySpeakers(page: Page, durationSeconds: number, speakerCount: number) {
    // Simulate periodic speaker activity over time
    const segments = Math.floor(durationSeconds / 30); // 30-second segments
    
    for (let i = 0; i < segments; i++) {
      const speakerId = `speaker_${(i % speakerCount) + 1}`;
      await page.evaluate((data) => {
        window.dispatchEvent(new CustomEvent('speaker-activity', {
          detail: {
            speakerId: data.speakerId,
            timestamp: data.timestamp,
            confidence: 0.8 + (Math.random() * 0.2)
          }
        }));
      }, { speakerId, timestamp: i * 30 });
      
      // Batch updates to avoid overwhelming the UI
      if (i % 10 === 0) {
        await page.waitForTimeout(100);
      }
    }
  }
});