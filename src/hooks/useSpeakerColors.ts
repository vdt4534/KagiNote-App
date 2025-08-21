import { useMemo } from 'react';

/**
 * Speaker color palette optimized for accessibility and contrast
 * Colors ensure good readability in both light and dark modes
 */
const SPEAKER_COLORS = [
  '#3B82F6', // Blue
  '#10B981', // Green 
  '#F59E0B', // Amber
  '#EF4444', // Red
  '#8B5CF6', // Violet
  '#06B6D4', // Cyan
  '#F97316', // Orange
  '#84CC16', // Lime
  '#EC4899', // Pink
  '#6366F1', // Indigo
];

/**
 * Color blind friendly palette as fallback
 */
const COLORBLIND_COLORS = [
  '#1f77b4', // Blue
  '#ff7f0e', // Orange
  '#2ca02c', // Green
  '#d62728', // Red
  '#9467bd', // Purple
  '#8c564b', // Brown
  '#e377c2', // Pink
  '#7f7f7f', // Gray
  '#bcbd22', // Olive
  '#17becf', // Cyan
];

export interface SpeakerColorConfig {
  id: string;
  color: string;
  textColor: string;
  borderColor: string;
}

export interface UseSpeakerColorsOptions {
  colorBlindMode?: boolean;
  customColors?: Record<string, string>;
}

/**
 * Hook for generating and managing consistent speaker colors
 * Ensures each speaker gets a unique, accessible color
 */
export function useSpeakerColors(
  speakerIds: string[],
  options: UseSpeakerColorsOptions = {}
) {
  const { colorBlindMode = false, customColors = {} } = options;

  const speakerColors = useMemo(() => {
    const colors = colorBlindMode ? COLORBLIND_COLORS : SPEAKER_COLORS;
    const result = new Map<string, SpeakerColorConfig>();

    speakerIds.forEach((speakerId, index) => {
      // Use custom color if provided, otherwise use palette
      const baseColor = customColors[speakerId] || colors[index % colors.length];
      
      result.set(speakerId, {
        id: speakerId,
        color: baseColor,
        textColor: getContrastTextColor(baseColor),
        borderColor: baseColor,
      });
    });

    return result;
  }, [speakerIds, colorBlindMode, customColors]);

  const getSpeakerColor = (speakerId: string): SpeakerColorConfig => {
    return speakerColors.get(speakerId) || {
      id: speakerId,
      color: '#6B7280', // Gray fallback
      textColor: '#FFFFFF',
      borderColor: '#6B7280',
    };
  };

  const updateSpeakerColor = (speakerId: string, newColor: string) => {
    // This would typically update a global state or localStorage
    // For now, return the config with the new color
    return {
      id: speakerId,
      color: newColor,
      textColor: getContrastTextColor(newColor),
      borderColor: newColor,
    };
  };

  const getAvailableColors = () => {
    const colors = colorBlindMode ? COLORBLIND_COLORS : SPEAKER_COLORS;
    return colors.map(color => ({
      value: color,
      label: colorToName(color),
      disabled: Array.from(speakerColors.values()).some(config => config.color === color),
    }));
  };

  return {
    speakerColors,
    getSpeakerColor,
    updateSpeakerColor,
    getAvailableColors,
  };
}

/**
 * Determines if a color needs light or dark text for good contrast
 */
function getContrastTextColor(hexColor: string): string {
  // Remove # if present
  const color = hexColor.replace('#', '');
  
  // Convert to RGB
  const r = parseInt(color.substr(0, 2), 16);
  const g = parseInt(color.substr(2, 2), 16);
  const b = parseInt(color.substr(4, 2), 16);
  
  // Calculate luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  
  // Return black text for light colors, white for dark
  return luminance > 0.5 ? '#000000' : '#FFFFFF';
}

/**
 * Convert hex color to human readable name
 */
function colorToName(hexColor: string): string {
  const colorNames: Record<string, string> = {
    '#3B82F6': 'Blue',
    '#10B981': 'Green',
    '#F59E0B': 'Amber', 
    '#EF4444': 'Red',
    '#8B5CF6': 'Violet',
    '#06B6D4': 'Cyan',
    '#F97316': 'Orange',
    '#84CC16': 'Lime',
    '#EC4899': 'Pink',
    '#6366F1': 'Indigo',
    '#1f77b4': 'Blue',
    '#ff7f0e': 'Orange',
    '#2ca02c': 'Green',
    '#d62728': 'Red',
    '#9467bd': 'Purple',
    '#8c564b': 'Brown',
    '#e377c2': 'Pink',
    '#7f7f7f': 'Gray',
    '#bcbd22': 'Olive',
    '#17becf': 'Cyan',
  };

  return colorNames[hexColor] || 'Custom';
}