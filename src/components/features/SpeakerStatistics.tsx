import React from 'react';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface SpeakerStats {
  totalSpeakers: number;
  totalSpeechTime: number;
  averageConfidence: number;
  speakerDistribution: Map<string, {
    name: string;
    percentage: number;
    duration: number;
    color?: string;
  }>;
}

export interface SpeakerStatisticsProps {
  stats: SpeakerStats;
  showChart?: boolean;
  className?: string;
}

export const SpeakerStatistics: React.FC<SpeakerStatisticsProps> = ({
  stats,
  showChart = true,
  className = '',
}) => {
  const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = Math.floor(seconds % 60);

    if (hours > 0) {
      return `${hours}h ${minutes}m ${remainingSeconds}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${remainingSeconds}s`;
    } else {
      return `${remainingSeconds}s`;
    }
  };

  const formatConfidence = (confidence: number): string => {
    return `${Math.round(confidence * 100)}%`;
  };

  const sortedSpeakers = Array.from(stats.speakerDistribution.entries())
    .sort((a, b) => b[1].percentage - a[1].percentage);

  return (
    <div className={cn('space-y-6', className)}>
      {/* Summary Stats */}
      <div 
        className="grid grid-cols-1 md:grid-cols-3 gap-4"
        aria-label="Speaker statistics summary"
      >
        <div className="bg-white dark:bg-neutral-900 p-4 rounded-lg border border-neutral-200 dark:border-neutral-700">
          <div className="flex items-center gap-2 mb-2">
            <Icon name="users" size="sm" className="text-primary-600 dark:text-primary-400" />
            <span className="text-sm font-medium text-neutral-600 dark:text-neutral-400">
              Total Speakers
            </span>
          </div>
          <p className="text-2xl font-bold text-neutral-900 dark:text-neutral-100">
            {stats.totalSpeakers} Speakers
          </p>
        </div>

        <div className="bg-white dark:bg-neutral-900 p-4 rounded-lg border border-neutral-200 dark:border-neutral-700">
          <div className="flex items-center gap-2 mb-2">
            <Icon name="clock" size="sm" className="text-secondary-600 dark:text-secondary-400" />
            <span className="text-sm font-medium text-neutral-600 dark:text-neutral-400">
              Total Speech Time
            </span>
          </div>
          <p className="text-2xl font-bold text-neutral-900 dark:text-neutral-100">
            {formatDuration(stats.totalSpeechTime)}
          </p>
        </div>

        <div className="bg-white dark:bg-neutral-900 p-4 rounded-lg border border-neutral-200 dark:border-neutral-700">
          <div className="flex items-center gap-2 mb-2">
            <Icon name="chart-bar" size="sm" className="text-warning-600 dark:text-warning-400" />
            <span className="text-sm font-medium text-neutral-600 dark:text-neutral-400">
              Average Confidence
            </span>
          </div>
          <p className="text-2xl font-bold text-neutral-900 dark:text-neutral-100">
            {formatConfidence(stats.averageConfidence)}
          </p>
        </div>
      </div>

      {/* Screen Reader Summary */}
      <div className="sr-only" aria-live="polite">
        {stats.totalSpeakers} speakers detected with average confidence of {Math.round(stats.averageConfidence * 100)} percent
      </div>

      {/* Speaker Distribution Chart */}
      {showChart && (
        <div className="bg-white dark:bg-neutral-900 p-4 rounded-lg border border-neutral-200 dark:border-neutral-700">
          <h4 className="text-lg font-semibold mb-4 text-neutral-900 dark:text-neutral-100">
            Speaking Time Distribution
          </h4>
          
          <div 
            data-testid="speaker-distribution-chart"
            className="space-y-3"
            aria-label="Speaker distribution chart"
          >
            {/* Visual Bar Chart */}
            <div className="relative h-8 bg-neutral-100 dark:bg-neutral-800 rounded-full overflow-hidden">
              {sortedSpeakers.reduce((acc, [speakerId, speaker], index) => {
                let leftPosition = 0;
                for (let i = 0; i < index; i++) {
                  leftPosition += sortedSpeakers[i][1].percentage;
                }
                
                const segment = (
                  <div
                    key={speakerId}
                    data-testid={`chart-segment-${speakerId}`}
                    className="absolute top-0 h-full transition-all duration-300"
                    style={{
                      left: `${leftPosition}%`,
                      width: `${speaker.percentage}%`,
                      backgroundColor: speaker.color || '#6B7280',
                    }}
                    title={`${speaker.name}: ${speaker.percentage.toFixed(1)}%`}
                  />
                );
                
                return [...acc, segment];
              }, [] as React.ReactElement[])}
            </div>

            {/* Legend */}
            <div className="space-y-2">
              {sortedSpeakers.map(([speakerId, speaker]) => (
                <div key={speakerId} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className="w-3 h-3 rounded-full"
                      style={{ backgroundColor: speaker.color || '#6B7280' }}
                    />
                    <span className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
                      {speaker.name} - {speaker.percentage.toFixed(1)}%
                    </span>
                  </div>
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">
                    {formatDuration(speaker.duration)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Detailed Breakdown */}
      <div className="bg-white dark:bg-neutral-900 p-4 rounded-lg border border-neutral-200 dark:border-neutral-700">
        <h4 className="text-lg font-semibold mb-4 text-neutral-900 dark:text-neutral-100">
          Detailed Breakdown
        </h4>
        
        <div className="space-y-3">
          {sortedSpeakers.map(([speakerId, speaker]) => (
            <div 
              key={speakerId}
              className="flex items-center justify-between p-3 bg-neutral-50 dark:bg-neutral-800 rounded-md"
            >
              <div className="flex items-center gap-3">
                <div
                  className="w-4 h-4 rounded-full border border-neutral-300 dark:border-neutral-600"
                  style={{ backgroundColor: speaker.color || '#6B7280' }}
                />
                <div>
                  <p className="font-medium text-neutral-900 dark:text-neutral-100">
                    {speaker.name}
                  </p>
                  <p className="text-sm text-neutral-600 dark:text-neutral-400">
                    {formatDuration(speaker.duration)} • {speaker.percentage.toFixed(1)}%
                  </p>
                </div>
              </div>
              
              <div className="text-right">
                <div className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
                  {speaker.percentage.toFixed(1)}%
                </div>
                <div className="w-24 h-2 bg-neutral-200 dark:bg-neutral-700 rounded-full overflow-hidden">
                  <div
                    className="h-full transition-all duration-300"
                    style={{
                      width: `${speaker.percentage}%`,
                      backgroundColor: speaker.color || '#6B7280',
                    }}
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Insights */}
      {stats.totalSpeakers > 1 && (
        <div className="bg-blue-50 dark:bg-blue-950 p-4 rounded-lg border border-blue-200 dark:border-blue-800">
          <div className="flex items-start gap-3">
            <Icon name="information-circle" size="sm" className="text-blue-600 dark:text-blue-400 mt-0.5" />
            <div>
              <h5 className="font-medium text-blue-900 dark:text-blue-100 mb-1">
                Meeting Insights
              </h5>
              <ul className="text-sm text-blue-800 dark:text-blue-200 space-y-1">
                <li>
                  Most active speaker: {sortedSpeakers[0][1].name} ({sortedSpeakers[0][1].percentage.toFixed(1)}%)
                </li>
                {stats.averageConfidence < 0.8 && (
                  <li>
                    Audio quality may affect speaker identification accuracy
                  </li>
                )}
                {stats.totalSpeakers > 5 && (
                  <li>
                    Large group detected - consider using participant names for clarity
                  </li>
                )}
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};