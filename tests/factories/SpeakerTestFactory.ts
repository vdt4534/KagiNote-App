/**
 * Speaker Diarization Test Data Factory
 * 
 * This factory creates comprehensive test data for speaker diarization functionality.
 * All test data is designed to validate functionality that DOES NOT YET EXIST.
 * This follows TDD principles - tests define the contract for implementation.
 */

import { AudioData } from './AudioTestFactory';

export interface SpeakerEmbedding {
  speakerId: string;
  embedding: number[]; // 512-dimensional embedding vector
  confidence: number;
  extractedAt: number; // timestamp
}

export interface SpeakerSegment {
  startTime: number;
  endTime: number;
  speakerId: string;
  confidence: number;
  text?: string;
  embedding?: SpeakerEmbedding;
}

export interface SpeakerDiarizationResult {
  speakers: Map<string, SpeakerProfile>;
  segments: SpeakerSegment[];
  processingTime: number;
  totalSpeakers: number;
  overallConfidence: number;
}

export interface SpeakerProfile {
  id: string;
  displayName: string;
  voiceCharacteristics: {
    pitch: number; // Hz
    formantF1: number; // Hz
    formantF2: number; // Hz
    speakingRate: number; // words per minute
  };
  embeddings: SpeakerEmbedding[];
  totalSpeechTime: number;
  lastActive: number;
  confidence: number;
}

export interface MultiSpeakerScenario {
  audio: AudioData;
  groundTruth: {
    speakers: SpeakerProfile[];
    segments: SpeakerSegment[];
    speakerChanges: number;
    avgSegmentLength: number;
  };
  complexity: 'simple' | 'medium' | 'complex' | 'extreme';
  expectedProcessingTime: number; // milliseconds
}

export class SpeakerTestFactory {
  
  /**
   * Creates a simple 2-speaker conversation scenario
   * Tests basic speaker separation and identification
   */
  static createTwoSpeakerConversation(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Alice', {
        pitch: 220, // Female voice
        formantF1: 800,
        formantF2: 1200,
        speakingRate: 150
      }),
      this.createSpeakerProfile('speaker_2', 'Bob', {
        pitch: 120, // Male voice  
        formantF1: 500,
        formantF2: 1000,
        speakingRate: 140
      })
    ];

    const segments = [
      { startTime: 0, endTime: 3, speakerId: 'speaker_1', confidence: 0.95, text: 'Hello, how are you today?' },
      { startTime: 3.5, endTime: 6, speakerId: 'speaker_2', confidence: 0.92, text: 'I am doing well, thank you.' },
      { startTime: 7, endTime: 10, speakerId: 'speaker_1', confidence: 0.94, text: 'That is great to hear.' },
      { startTime: 10.5, endTime: 13, speakerId: 'speaker_2', confidence: 0.91, text: 'How has your week been?' }
    ];

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 4,
        avgSegmentLength: 2.6
      },
      complexity: 'simple',
      expectedProcessingTime: 2000 // 2 seconds for 13 seconds of audio
    };
  }

  /**
   * Creates a business meeting with 4 speakers
   * Tests handling multiple distinct voices
   */
  static createBusinessMeeting(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Manager', {
        pitch: 180,
        formantF1: 750,
        formantF2: 1150,
        speakingRate: 160
      }),
      this.createSpeakerProfile('speaker_2', 'Developer', {
        pitch: 250,
        formantF1: 850,
        formantF2: 1250,
        speakingRate: 145
      }),
      this.createSpeakerProfile('speaker_3', 'Designer', {
        pitch: 140,
        formantF1: 550,
        formantF2: 1050,
        speakingRate: 135
      }),
      this.createSpeakerProfile('speaker_4', 'QA Engineer', {
        pitch: 200,
        formantF1: 780,
        formantF2: 1180,
        speakingRate: 155
      })
    ];

    const segments = this.generateMeetingSegments(300, speakers); // 5 minute meeting

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: segments.length,
        avgSegmentLength: 300 / segments.length
      },
      complexity: 'medium',
      expectedProcessingTime: 30000 // 30 seconds for 5 minutes of audio
    };
  }

  /**
   * Creates challenging overlapping speech scenario
   * Tests speaker separation during simultaneous speech
   */
  static createOverlappingSpeechScenario(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Interviewer', {
        pitch: 175,
        formantF1: 720,
        formantF2: 1120,
        speakingRate: 150
      }),
      this.createSpeakerProfile('speaker_2', 'Candidate', {
        pitch: 210,
        formantF1: 800,
        formantF2: 1200,
        speakingRate: 165
      })
    ];

    // Include overlapping segments
    const segments = [
      { startTime: 0, endTime: 5, speakerId: 'speaker_1', confidence: 0.88, text: 'Tell me about your background' },
      { startTime: 4.5, endTime: 7, speakerId: 'speaker_2', confidence: 0.75, text: 'I have five years of experience' }, // Overlap
      { startTime: 8, endTime: 12, speakerId: 'speaker_2', confidence: 0.92, text: 'working with React and TypeScript' },
      { startTime: 11.5, endTime: 14, speakerId: 'speaker_1', confidence: 0.82, text: 'That sounds impressive' }, // Overlap
    ];

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 4,
        avgSegmentLength: 3.5
      },
      complexity: 'complex',
      expectedProcessingTime: 8000 // 8 seconds for 14 seconds of audio (overlaps slow processing)
    };
  }

  /**
   * Creates extreme stress test with 8 speakers
   * Tests maximum capacity handling
   */
  static createLargeGroupMeeting(): MultiSpeakerScenario {
    const speakers = Array.from({ length: 8 }, (_, i) => 
      this.createSpeakerProfile(`speaker_${i + 1}`, `Participant ${i + 1}`, {
        pitch: 120 + (i * 20), // Vary pitch from 120-260 Hz
        formantF1: 500 + (i * 50),
        formantF2: 1000 + (i * 50),
        speakingRate: 130 + (i * 5)
      })
    );

    const segments = this.generateLargeGroupSegments(3600, speakers); // 1 hour meeting

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: segments.length,
        avgSegmentLength: 3600 / segments.length
      },
      complexity: 'extreme',
      expectedProcessingTime: 60000 // 1 minute for 1 hour of audio
    };
  }

  /**
   * Creates similar-sounding speakers test
   * Tests ability to distinguish between similar voices
   */
  static createSimilarVoicesScenario(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Twin A', {
        pitch: 180,
        formantF1: 750,
        formantF2: 1150,
        speakingRate: 150
      }),
      this.createSpeakerProfile('speaker_2', 'Twin B', {
        pitch: 182, // Very similar pitch
        formantF1: 755, // Very similar formants
        formantF2: 1155,
        speakingRate: 152 // Very similar speaking rate
      })
    ];

    const segments = [
      { startTime: 0, endTime: 4, speakerId: 'speaker_1', confidence: 0.75, text: 'We sound very similar' },
      { startTime: 5, endTime: 9, speakerId: 'speaker_2', confidence: 0.72, text: 'Yes, people often confuse us' },
      { startTime: 10, endTime: 14, speakerId: 'speaker_1', confidence: 0.78, text: 'But there are subtle differences' },
      { startTime: 15, endTime: 19, speakerId: 'speaker_2', confidence: 0.74, text: 'If you listen carefully enough' }
    ];

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 4,
        avgSegmentLength: 4.25
      },
      complexity: 'complex',
      expectedProcessingTime: 12000 // 12 seconds for 19 seconds of audio (difficult case)
    };
  }

  /**
   * Creates noisy environment diarization test
   * Tests robustness in challenging audio conditions
   */
  static createNoisyEnvironmentScenario(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Presenter', {
        pitch: 165,
        formantF1: 700,
        formantF2: 1100,
        speakingRate: 140
      }),
      this.createSpeakerProfile('speaker_2', 'Audience Member', {
        pitch: 190,
        formantF1: 780,
        formantF2: 1180,
        speakingRate: 160
      })
    ];

    const segments = [
      { startTime: 0, endTime: 8, speakerId: 'speaker_1', confidence: 0.65, text: 'Welcome to our presentation today' },
      { startTime: 10, endTime: 15, speakerId: 'speaker_2', confidence: 0.58, text: 'Can you speak louder please?' },
      { startTime: 17, endTime: 25, speakerId: 'speaker_1', confidence: 0.70, text: 'Of course, let me increase the volume' },
    ];

    return {
      audio: this.generateNoisyMultiSpeakerAudio(segments, speakers, -15), // -15dB SNR
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 3,
        avgSegmentLength: 8.3
      },
      complexity: 'complex',
      expectedProcessingTime: 15000 // 15 seconds for 25 seconds of noisy audio
    };
  }

  /**
   * Creates test data for speaker re-identification
   * Tests consistency of speaker IDs across long sessions
   */
  static createSpeakerReidentificationScenario(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Host', {
        pitch: 170,
        formantF1: 720,
        formantF2: 1120,
        speakingRate: 155
      }),
      this.createSpeakerProfile('speaker_2', 'Guest', {
        pitch: 195,
        formantF2: 800,
        formantF2: 1200,
        speakingRate: 145
      })
    ];

    // Create segments with large gaps to test re-identification
    const segments = [
      { startTime: 0, endTime: 5, speakerId: 'speaker_1', confidence: 0.92, text: 'Welcome to the show' },
      { startTime: 6, endTime: 10, speakerId: 'speaker_2', confidence: 0.90, text: 'Thank you for having me' },
      // Large gap - 5 minutes of different content
      { startTime: 310, endTime: 315, speakerId: 'speaker_1', confidence: 0.89, text: 'Now back to our discussion' }, // Should still identify as speaker_1
      { startTime: 316, endTime: 320, speakerId: 'speaker_2', confidence: 0.87, text: 'Yes, where we left off' }, // Should still identify as speaker_2
    ];

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 4,
        avgSegmentLength: 5
      },
      complexity: 'medium',
      expectedProcessingTime: 20000 // 20 seconds for 320 seconds of sparse audio
    };
  }

  // Helper methods for creating test data

  private static createSpeakerProfile(
    id: string, 
    displayName: string, 
    characteristics: any
  ): SpeakerProfile {
    return {
      id,
      displayName,
      voiceCharacteristics: characteristics,
      embeddings: [],
      totalSpeechTime: 0,
      lastActive: Date.now(),
      confidence: 0.90
    };
  }

  private static generateMeetingSegments(
    durationSeconds: number, 
    speakers: SpeakerProfile[]
  ): SpeakerSegment[] {
    const segments: SpeakerSegment[] = [];
    const avgSegmentLength = 8; // 8 seconds average
    const totalSegments = Math.floor(durationSeconds / avgSegmentLength);
    
    const meetingPhrases = [
      "Let's begin today's meeting",
      "Thank you all for joining",
      "Our quarterly results show",
      "I'd like to discuss the implementation",
      "The design looks promising",
      "What are your thoughts on this?",
      "I agree with that assessment",
      "Let's move to the next item",
      "Any questions before we continue?",
      "That's a good point to consider",
      "We should schedule a follow-up",
      "I'll take that as an action item"
    ];

    let currentTime = 0;
    for (let i = 0; i < totalSegments; i++) {
      const speaker = speakers[i % speakers.length];
      const segmentLength = avgSegmentLength + (Math.random() * 4 - 2); // ±2 seconds variation
      const text = meetingPhrases[i % meetingPhrases.length];
      
      segments.push({
        startTime: currentTime,
        endTime: currentTime + segmentLength,
        speakerId: speaker.id,
        confidence: 0.85 + (Math.random() * 0.15), // 0.85-1.0
        text
      });
      
      currentTime += segmentLength + (Math.random() * 2); // Small pause between speakers
    }
    
    return segments;
  }

  private static generateLargeGroupSegments(
    durationSeconds: number,
    speakers: SpeakerProfile[]
  ): SpeakerSegment[] {
    const segments: SpeakerSegment[] = [];
    const avgSegmentLength = 15; // Longer segments in large groups
    const totalSegments = Math.floor(durationSeconds / avgSegmentLength);
    
    let currentTime = 0;
    for (let i = 0; i < totalSegments; i++) {
      // More realistic speaker distribution - some speak more than others
      const speakerIndex = this.weightedSpeakerSelection(speakers.length, i);
      const speaker = speakers[speakerIndex];
      const segmentLength = avgSegmentLength + (Math.random() * 10 - 5);
      
      segments.push({
        startTime: currentTime,
        endTime: currentTime + segmentLength,
        speakerId: speaker.id,
        confidence: 0.80 + (Math.random() * 0.20), // Slightly lower confidence with more speakers
        text: `Speaker ${speakerIndex + 1} presenting their point`
      });
      
      currentTime += segmentLength + (Math.random() * 3);
    }
    
    return segments;
  }

  private static weightedSpeakerSelection(speakerCount: number, segmentIndex: number): number {
    // Simulate realistic meeting dynamics - some speakers more active
    const weights = Array.from({ length: speakerCount }, (_, i) => {
      if (i < 2) return 0.3; // First two speakers are very active (60%)
      if (i < 4) return 0.2; // Next two are moderately active (40%)
      return 0.1; // Remaining speakers are less active
    });
    
    // Add some randomness
    const random = Math.random();
    let cumulative = 0;
    
    for (let i = 0; i < weights.length; i++) {
      cumulative += weights[i];
      if (random < cumulative) return i;
    }
    
    return 0; // Fallback
  }

  private static generateMultiSpeakerAudio(
    segments: SpeakerSegment[], 
    speakers: SpeakerProfile[]
  ): AudioData {
    const maxEndTime = Math.max(...segments.map(s => s.endTime));
    const sampleRate = 16000;
    const totalSamples = Math.floor(maxEndTime * sampleRate);
    const samples = new Float32Array(totalSamples);
    
    // Generate audio for each segment based on speaker characteristics
    for (const segment of segments) {
      const speaker = speakers.find(s => s.id === segment.speakerId);
      if (!speaker) continue;
      
      const segmentAudio = this.generateSpeakerAudio(
        segment.endTime - segment.startTime,
        speaker.voiceCharacteristics,
        sampleRate
      );
      
      const startIndex = Math.floor(segment.startTime * sampleRate);
      for (let i = 0; i < segmentAudio.length && (startIndex + i) < totalSamples; i++) {
        samples[startIndex + i] += segmentAudio[i];
      }
    }
    
    return {
      sampleRate,
      channels: 1,
      samples,
      timestamp: Date.now(),
      sourceChannel: 'mixed',
      durationSeconds: maxEndTime
    };
  }

  private static generateNoisyMultiSpeakerAudio(
    segments: SpeakerSegment[],
    speakers: SpeakerProfile[],
    snrDb: number
  ): AudioData {
    const cleanAudio = this.generateMultiSpeakerAudio(segments, speakers);
    
    // Add background noise
    const noiseLevel = Math.pow(10, snrDb / 20);
    for (let i = 0; i < cleanAudio.samples.length; i++) {
      const noise = (Math.random() * 2 - 1) * noiseLevel;
      cleanAudio.samples[i] = Math.tanh(cleanAudio.samples[i] + noise);
    }
    
    return cleanAudio;
  }

  private static generateSpeakerAudio(
    duration: number,
    characteristics: any,
    sampleRate: number
  ): Float32Array {
    const samples = new Float32Array(duration * sampleRate);
    const { pitch, formantF1, formantF2 } = characteristics;
    
    for (let i = 0; i < samples.length; i++) {
      const t = i / sampleRate;
      
      // Generate speech-like signal with speaker-specific characteristics
      let sample = 0;
      sample += 0.4 * Math.sin(2 * Math.PI * pitch * t);
      sample += 0.3 * Math.sin(2 * Math.PI * formantF1 * t);
      sample += 0.2 * Math.sin(2 * Math.PI * formantF2 * t);
      
      // Apply speech envelope
      const envelope = this.speechEnvelope(t, duration, characteristics.speakingRate);
      samples[i] = sample * envelope * 0.2;
    }
    
    return samples;
  }

  private static speechEnvelope(t: number, duration: number, speakingRate: number): number {
    const syllableRate = speakingRate / 60 * 2; // Convert WPM to syllables per second
    const syllableTime = t * syllableRate;
    const syllablePhase = syllableTime % 1;
    
    // Speech vs silence pattern based on speaking rate
    const isSpeech = syllablePhase < 0.7;
    return isSpeech ? Math.sin(Math.PI * syllablePhase / 0.7) : 0;
  }

  /**
   * Creates ONNX model validation test data
   * Tests that bundled models can be loaded and provide valid outputs
   */
  static createONNXModelValidationScenario(): MultiSpeakerScenario {
    const speakers = [
      this.createSpeakerProfile('speaker_1', 'Model Test Voice A', {
        pitch: 190,
        formantF1: 760,
        formantF2: 1160,
        speakingRate: 150
      }),
      this.createSpeakerProfile('speaker_2', 'Model Test Voice B', {
        pitch: 155,
        formantF1: 620,
        formantF2: 1020,
        speakingRate: 145
      })
    ];

    const segments = [
      { startTime: 0, endTime: 3, speakerId: 'speaker_1', confidence: 0.88, text: 'Testing model loading and inference' },
      { startTime: 4, endTime: 7, speakerId: 'speaker_2', confidence: 0.85, text: 'Validating ONNX model outputs' },
      { startTime: 8, endTime: 11, speakerId: 'speaker_1', confidence: 0.90, text: 'Models should produce valid embeddings' },
    ];

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 3,
        avgSegmentLength: 3.0
      },
      complexity: 'simple',
      expectedProcessingTime: 3000 // 3 seconds for model loading validation
    };
  }

  /**
   * Creates memory pressure test scenario
   * Tests diarization behavior under memory constraints
   */
  static createMemoryPressureScenario(): MultiSpeakerScenario {
    const speakers = Array.from({ length: 6 }, (_, i) => 
      this.createSpeakerProfile(`speaker_${i + 1}`, `Memory Test ${i + 1}`, {
        pitch: 130 + (i * 25),
        formantF1: 550 + (i * 60),
        formantF2: 1050 + (i * 60),
        speakingRate: 135 + (i * 8)
      })
    );

    // Create many short segments to stress memory allocation
    const segments = Array.from({ length: 200 }, (_, i) => ({
      startTime: i * 1.5,
      endTime: (i * 1.5) + 1.2,
      speakerId: `speaker_${(i % 6) + 1}`,
      confidence: 0.75 + (Math.random() * 0.2),
      text: `Memory test segment ${i + 1}`
    }));

    return {
      audio: this.generateMultiSpeakerAudio(segments, speakers),
      groundTruth: {
        speakers,
        segments,
        speakerChanges: 200,
        avgSegmentLength: 1.2
      },
      complexity: 'extreme',
      expectedProcessingTime: 45000 // 45 seconds for 300 seconds of dense audio
    };
  }

  /**
   * Creates concurrent session test data
   * Tests handling multiple simultaneous diarization sessions
   */
  static createConcurrentSessionsScenario(): MultiSpeakerScenario[] {
    return Array.from({ length: 5 }, (_, sessionIndex) => {
      const speakers = [
        this.createSpeakerProfile(`session${sessionIndex}_speaker_1`, `Session ${sessionIndex} Person A`, {
          pitch: 160 + (sessionIndex * 10),
          formantF1: 700 + (sessionIndex * 30),
          formantF2: 1100 + (sessionIndex * 30),
          speakingRate: 145 + (sessionIndex * 5)
        }),
        this.createSpeakerProfile(`session${sessionIndex}_speaker_2`, `Session ${sessionIndex} Person B`, {
          pitch: 190 + (sessionIndex * 12),
          formantF1: 750 + (sessionIndex * 35),
          formantF2: 1150 + (sessionIndex * 35),
          speakingRate: 150 + (sessionIndex * 5)
        })
      ];

      const segments = [
        { startTime: 0, endTime: 5, speakerId: `session${sessionIndex}_speaker_1`, confidence: 0.87, text: `Session ${sessionIndex} conversation start` },
        { startTime: 6, endTime: 11, speakerId: `session${sessionIndex}_speaker_2`, confidence: 0.84, text: `Session ${sessionIndex} response` },
        { startTime: 12, endTime: 17, speakerId: `session${sessionIndex}_speaker_1`, confidence: 0.89, text: `Session ${sessionIndex} continuation` },
      ];

      return {
        audio: this.generateMultiSpeakerAudio(segments, speakers),
        groundTruth: {
          speakers,
          segments,
          speakerChanges: 3,
          avgSegmentLength: 5.0
        },
        complexity: 'medium',
        expectedProcessingTime: 8000 // 8 seconds per session
      };
    });
  }

  /**
   * Creates error recovery test scenario
   * Tests graceful handling of various error conditions
   */
  static createErrorRecoveryScenario(): {
    corruptedAudio: MultiSpeakerScenario;
    insufficientAudio: MultiSpeakerScenario;
    invalidConfig: MultiSpeakerScenario;
  } {
    const baseSpeaker = this.createSpeakerProfile('error_test_speaker', 'Error Test Voice', {
      pitch: 175,
      formantF1: 725,
      formantF2: 1125,
      speakingRate: 148
    });

    return {
      corruptedAudio: {
        audio: this.generateCorruptedAudio(10.0), // 10 seconds of corrupted audio
        groundTruth: {
          speakers: [baseSpeaker],
          segments: [],
          speakerChanges: 0,
          avgSegmentLength: 0
        },
        complexity: 'extreme',
        expectedProcessingTime: -1 // Should fail gracefully
      },

      insufficientAudio: {
        audio: this.generateMultiSpeakerAudio(
          [{ startTime: 0, endTime: 0.3, speakerId: 'error_test_speaker', confidence: 0.4, text: 'Too short' }],
          [baseSpeaker]
        ),
        groundTruth: {
          speakers: [baseSpeaker],
          segments: [],
          speakerChanges: 0,
          avgSegmentLength: 0.3
        },
        complexity: 'simple',
        expectedProcessingTime: -1 // Should fail due to insufficient audio
      },

      invalidConfig: {
        audio: this.generateMultiSpeakerAudio(
          [{ startTime: 0, endTime: 5, speakerId: 'error_test_speaker', confidence: 0.8, text: 'Valid audio but invalid config' }],
          [baseSpeaker]
        ),
        groundTruth: {
          speakers: [baseSpeaker],
          segments: [],
          speakerChanges: 0,
          avgSegmentLength: 5.0
        },
        complexity: 'simple',
        expectedProcessingTime: -1 // Should fail due to invalid configuration
      }
    };
  }

  private static generateCorruptedAudio(duration: number): AudioData {
    const sampleRate = 16000;
    const totalSamples = Math.floor(duration * sampleRate);
    const samples = new Float32Array(totalSamples);
    
    // Generate corrupted audio with NaN and Infinity values
    for (let i = 0; i < samples.length; i++) {
      if (Math.random() < 0.1) {
        samples[i] = Math.random() < 0.5 ? NaN : Infinity;
      } else {
        samples[i] = (Math.random() - 0.5) * 2; // Normal audio
      }
    }
    
    return {
      sampleRate,
      channels: 1,
      samples,
      timestamp: Date.now(),
      sourceChannel: 'corrupted',
      durationSeconds: duration
    };
  }
}

/**
 * Performance test scenarios for speaker diarization
 * These define the acceptance criteria that implementation must meet
 */
export const SpeakerDiarizationPerformanceScenarios = {
  realTimeProcessing: {
    description: 'Process speaker diarization in < 1 minute for 1 hour audio',
    target: 60000, // milliseconds
    testData: () => SpeakerTestFactory.createLargeGroupMeeting(),
    measure: 'total_processing_time'
  },
  
  speakerAccuracy: {
    description: 'Achieve >90% speaker identification accuracy',
    target: 0.90,
    testData: () => SpeakerTestFactory.createBusinessMeeting(),
    measure: 'speaker_identification_accuracy'
  },
  
  similarVoicesHandling: {
    description: 'Distinguish similar voices with >75% accuracy',
    target: 0.75,
    testData: () => SpeakerTestFactory.createSimilarVoicesScenario(),
    measure: 'similar_voices_accuracy'
  },
  
  noisyEnvironmentRobustness: {
    description: 'Maintain >60% accuracy in noisy conditions',
    target: 0.60,
    testData: () => SpeakerTestFactory.createNoisyEnvironmentScenario(),
    measure: 'noisy_accuracy'
  },
  
  reidentificationConsistency: {
    description: 'Maintain consistent speaker IDs across session',
    target: 0.95,
    testData: () => SpeakerTestFactory.createSpeakerReidentificationScenario(),
    measure: 'speaker_consistency'
  },

  onnxModelPerformance: {
    description: 'Load and run ONNX models within performance targets',
    target: 3000, // 3 seconds max for model operations
    testData: () => SpeakerTestFactory.createONNXModelValidationScenario(),
    measure: 'model_loading_and_inference_time'
  },

  memoryEfficiency: {
    description: 'Handle memory pressure without crashes or excessive usage',
    target: 500, // 500MB max memory usage
    testData: () => SpeakerTestFactory.createMemoryPressureScenario(),
    measure: 'peak_memory_usage_mb'
  },

  concurrentSessions: {
    description: 'Handle 5 concurrent diarization sessions successfully',
    target: 5, // Number of successful concurrent sessions
    testData: () => SpeakerTestFactory.createConcurrentSessionsScenario(),
    measure: 'successful_concurrent_sessions'
  },

  errorRecovery: {
    description: 'Gracefully handle and recover from various error conditions',
    target: 1.0, // 100% error recovery (no crashes)
    testData: () => SpeakerTestFactory.createErrorRecoveryScenario(),
    measure: 'error_recovery_success_rate'
  }
};