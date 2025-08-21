import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { useSpeakerColors } from '@/hooks/useSpeakerColors';

export interface SpeakerColorPickerProps {
  currentColor?: string;
  onColorSelect: (color: string) => void;
  onClose: () => void;
  isOpen: boolean;
  colorBlindMode?: boolean;
  excludeColors?: string[];
  className?: string;
}

const DEFAULT_COLORS = [
  { value: '#3B82F6', name: 'Blue' },
  { value: '#10B981', name: 'Green' },
  { value: '#F59E0B', name: 'Amber' },
  { value: '#EF4444', name: 'Red' },
  { value: '#8B5CF6', name: 'Violet' },
  { value: '#06B6D4', name: 'Cyan' },
  { value: '#F97316', name: 'Orange' },
  { value: '#84CC16', name: 'Lime' },
  { value: '#EC4899', name: 'Pink' },
  { value: '#6366F1', name: 'Indigo' },
];

const COLORBLIND_COLORS = [
  { value: '#1f77b4', name: 'Blue' },
  { value: '#ff7f0e', name: 'Orange' },
  { value: '#2ca02c', name: 'Green' },
  { value: '#d62728', name: 'Red' },
  { value: '#9467bd', name: 'Purple' },
  { value: '#8c564b', name: 'Brown' },
  { value: '#e377c2', name: 'Pink' },
  { value: '#7f7f7f', name: 'Gray' },
  { value: '#bcbd22', name: 'Olive' },
  { value: '#17becf', name: 'Cyan' },
];

export const SpeakerColorPicker: React.FC<SpeakerColorPickerProps> = ({
  currentColor,
  onColorSelect,
  onClose,
  isOpen,
  colorBlindMode = false,
  excludeColors = [],
  className = '',
}) => {
  const [selectedColor, setSelectedColor] = useState(currentColor || '');
  
  const colors = colorBlindMode ? COLORBLIND_COLORS : DEFAULT_COLORS;

  const handleColorClick = (color: string) => {
    setSelectedColor(color);
  };

  const handleConfirm = () => {
    if (selectedColor) {
      onColorSelect(selectedColor);
    }
    onClose();
  };

  const handleCancel = () => {
    setSelectedColor(currentColor || '');
    onClose();
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div 
        data-testid="speaker-color-picker"
        className={cn(
          'bg-white dark:bg-neutral-900 p-6 rounded-lg shadow-xl max-w-sm w-full mx-4',
          className
        )}
      >
        <h3 className="text-lg font-semibold mb-4 text-neutral-900 dark:text-neutral-100">
          Choose Speaker Color
        </h3>

        {/* Color Grid */}
        <div className="grid grid-cols-5 gap-3 mb-6">
          {colors.map((color) => {
            const isExcluded = excludeColors.includes(color.value);
            const isSelected = selectedColor === color.value;
            const isCurrent = currentColor === color.value;

            return (
              <button
                key={color.value}
                data-testid={`color-option-${color.name.toLowerCase()}`}
                onClick={() => !isExcluded && handleColorClick(color.value)}
                disabled={isExcluded}
                className={cn(
                  'w-10 h-10 rounded-full border-2 transition-all duration-200',
                  'hover:scale-110 focus:outline-none focus:ring-2 focus:ring-primary-500',
                  isSelected && 'ring-2 ring-neutral-900 dark:ring-neutral-100 ring-offset-2',
                  isCurrent && !isSelected && 'ring-1 ring-neutral-400 ring-offset-1',
                  isExcluded && 'opacity-50 cursor-not-allowed grayscale'
                )}
                style={{ 
                  backgroundColor: color.value,
                  borderColor: isSelected ? '#000' : 'transparent'
                }}
                title={isExcluded ? `${color.name} (already in use)` : color.name}
                aria-label={`Select ${color.name} color`}
              />
            );
          })}
        </div>

        {/* Preview */}
        {selectedColor && (
          <div className="mb-4 p-3 border border-neutral-200 dark:border-neutral-700 rounded-md">
            <div className="flex items-center gap-3">
              <div
                className="w-4 h-4 rounded-full"
                style={{ backgroundColor: selectedColor }}
              />
              <span className="text-sm text-neutral-600 dark:text-neutral-400">
                Preview: Speaker with {colors.find(c => c.value === selectedColor)?.name.toLowerCase()} color
              </span>
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-2 justify-end">
          <Button
            variant="ghost"
            onClick={handleCancel}
          >
            Cancel
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!selectedColor}
          >
            Apply Color
          </Button>
        </div>

        {/* Accessibility Note */}
        {colorBlindMode && (
          <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-2">
            Color blind friendly palette active
          </p>
        )}
      </div>
    </div>
  );
};