import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Modal } from '@/components/ui/Modal';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Card, CardBody } from '@/components/ui/Card';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface ModelInfo {
  id: string;
  name: string;
  size: string;
  quality: 'Standard' | 'High Accuracy' | 'Turbo';
  description: string;
  ramRequirement: string;
  isDownloaded: boolean;
  isLoading?: boolean;
}

export interface NewMeetingModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartRecording: (config: MeetingConfig) => void;
  models?: ModelInfo[];
  microphones?: string[];
  isLoading?: boolean;
}

export interface MeetingConfig {
  title: string;
  participants: string;
  modelId: string;
  language: string;
  autoDetectLanguages: boolean;
  microphone: string;
}

const defaultModels: ModelInfo[] = [
  {
    id: 'turbo',
    name: 'Turbo',
    size: '1.2GB',
    quality: 'Turbo',
    description: 'Fast processing',
    ramRequirement: '4GB RAM',
    isDownloaded: true,
  },
  {
    id: 'standard',
    name: 'Standard',
    size: '1.5GB',
    quality: 'Standard',
    description: 'Balanced quality',
    ramRequirement: '8GB RAM',
    isDownloaded: true,
  },
  {
    id: 'high-accuracy',
    name: 'High Accuracy',
    size: '2.4GB',
    quality: 'High Accuracy',
    description: 'Best quality',
    ramRequirement: '16GB RAM',
    isDownloaded: false,
  },
];

const languages = [
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Spanish' },
  { code: 'fr', name: 'French' },
  { code: 'de', name: 'German' },
  { code: 'it', name: 'Italian' },
  { code: 'pt', name: 'Portuguese' },
  { code: 'ja', name: 'Japanese' },
  { code: 'zh', name: 'Chinese' },
];

export const NewMeetingModal: React.FC<NewMeetingModalProps> = ({
  isOpen,
  onClose,
  onStartRecording,
  models = defaultModels,
  microphones = ['MacBook Pro Microphone', 'External USB Microphone'],
  isLoading = false,
}) => {
  const [config, setConfig] = useState<MeetingConfig>({
    title: '',
    participants: '',
    modelId: 'standard',
    language: 'en',
    autoDetectLanguages: true,
    microphone: microphones[0] || '',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const selectedModel = models.find(m => m.id === config.modelId);

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!config.title.trim()) {
      newErrors.title = 'Meeting title is required';
    }

    if (!selectedModel?.isDownloaded) {
      newErrors.model = 'Selected model is not downloaded';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = () => {
    if (validateForm()) {
      onStartRecording(config);
    }
  };

  const handleModelSelect = (modelId: string) => {
    setConfig(prev => ({ ...prev, modelId }));
    // Clear any model-related errors
    if (errors.model) {
      setErrors(prev => ({ ...prev, model: '' }));
    }
  };

  const getModelStatusIcon = (model: ModelInfo) => {
    if (model.isLoading) return <Icon name="clock" size="sm" className="animate-spin" />;
    if (model.isDownloaded) return <Icon name="check-circle" size="sm" className="text-secondary-600" />;
    return <Icon name="x-circle" size="sm" className="text-warning-600" />;
  };

  const getModelStatusText = (model: ModelInfo) => {
    if (model.isLoading) return 'Downloading...';
    if (model.isDownloaded) return 'Downloaded and ready';
    return 'Not downloaded';
  };

  const getQualityColor = (quality: string): "primary" | "secondary" | "warning" | "error" | "neutral" => {
    switch (quality) {
      case 'High Accuracy':
        return 'secondary';
      case 'Turbo':
        return 'warning';
      default:
        return 'primary';
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="New Meeting Setup"
      size="lg"
      className="max-w-2xl"
    >
      <div className="space-y-6">
        {/* Meeting Information */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
            Meeting Information
          </h3>
          
          <div className="space-y-4">
            <Input
              label="Meeting Title"
              value={config.title}
              onChange={(e) => setConfig(prev => ({ ...prev, title: e.target.value }))}
              placeholder="Weekly Team Sync"
              error={!!errors.title}
              helperText={errors.title}
              disabled={isLoading}
            />
            
            <Input
              label="Participants (optional)"
              value={config.participants}
              onChange={(e) => setConfig(prev => ({ ...prev, participants: e.target.value }))}
              placeholder="John, Sarah, Mike"
              disabled={isLoading}
            />
          </div>
        </div>

        {/* Transcription Quality */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
            Transcription Quality
          </h3>
          
          {errors.model && (
            <p className="text-error-600 dark:text-error-400 text-sm">
              {errors.model}
            </p>
          )}
          
          <div className="grid gap-3">
            {models.map((model) => (
              <Card
                key={model.id}
                className={cn(
                  'cursor-pointer transition-all',
                  config.modelId === model.id
                    ? 'border-primary-300 bg-primary-50 dark:border-primary-700 dark:bg-primary-900/20'
                    : 'hover:border-neutral-300 dark:hover:border-neutral-600',
                  !model.isDownloaded && 'opacity-60'
                )}
                onClick={() => handleModelSelect(model.id)}
              >
                <CardBody className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className={cn(
                        'w-4 h-4 rounded-full border-2 transition-colors',
                        config.modelId === model.id
                          ? 'border-primary-600 bg-primary-600'
                          : 'border-neutral-300 dark:border-neutral-600'
                      )}>
                        {config.modelId === model.id && (
                          <div className="w-full h-full rounded-full bg-white dark:bg-neutral-900 scale-50" />
                        )}
                      </div>
                      
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-neutral-900 dark:text-neutral-100">
                            {model.name}
                          </span>
                          <Badge variant={getQualityColor(model.quality)} size="sm">
                            {model.quality}
                          </Badge>
                        </div>
                        <p className="text-sm text-neutral-600 dark:text-neutral-400">
                          {model.description} • {model.ramRequirement}
                        </p>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <span className="text-sm text-neutral-500 dark:text-neutral-400">
                        {model.size}
                      </span>
                      {getModelStatusIcon(model)}
                    </div>
                  </div>
                  
                  <div className="mt-2 flex items-center gap-1 text-xs">
                    {getModelStatusIcon(model)}
                    <span className={cn(
                      model.isDownloaded 
                        ? 'text-secondary-600 dark:text-secondary-400'
                        : 'text-warning-600 dark:text-warning-400'
                    )}>
                      {getModelStatusText(model)}
                    </span>
                  </div>
                </CardBody>
              </Card>
            ))}
          </div>
        </div>

        {/* Language Settings */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
            Language Settings
          </h3>
          
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
                Primary Language
              </label>
              <select
                value={config.language}
                onChange={(e) => setConfig(prev => ({ ...prev, language: e.target.value }))}
                className="w-full px-3 py-2 border border-neutral-300 dark:border-neutral-600 rounded-md bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100"
                disabled={isLoading}
              >
                {languages.map((lang) => (
                  <option key={lang.code} value={lang.code}>
                    {lang.name}
                  </option>
                ))}
              </select>
            </div>
            
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="auto-detect"
                checked={config.autoDetectLanguages}
                onChange={(e) => setConfig(prev => ({ ...prev, autoDetectLanguages: e.target.checked }))}
                className="w-4 h-4 text-primary-600 border-neutral-300 rounded focus:ring-primary-500"
                disabled={isLoading}
              />
              <label htmlFor="auto-detect" className="text-sm text-neutral-700 dark:text-neutral-300">
                Auto-detect others
              </label>
            </div>
          </div>
        </div>

        {/* System Status */}
        <div className="space-y-3 p-4 bg-neutral-50 dark:bg-neutral-800 rounded-lg">
          <div className="flex items-center gap-2">
            <Icon name="check-circle" size="sm" className="text-secondary-600" />
            <span className="text-sm text-neutral-700 dark:text-neutral-300">
              Microphone: {config.microphone}
            </span>
          </div>
          
          {selectedModel && (
            <div className="flex items-center gap-2">
              <Icon name="check-circle" size="sm" className="text-secondary-600" />
              <span className="text-sm text-neutral-700 dark:text-neutral-300">
                Model loaded: {selectedModel.name} ({selectedModel.size})
              </span>
            </div>
          )}
          
          <div className="flex items-center gap-2">
            <Icon name="check-circle" size="sm" className="text-secondary-600" />
            <span className="text-sm text-neutral-700 dark:text-neutral-300">
              Available RAM: 8.2GB
            </span>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-700">
          <Button
            variant="ghost"
            onClick={onClose}
            disabled={isLoading}
          >
            Cancel
          </Button>
          
          <Button
            variant="primary"
            onClick={handleSubmit}
            disabled={isLoading || !selectedModel?.isDownloaded}
            isLoading={isLoading}
            className="min-w-[140px]"
          >
            Start Recording
          </Button>
        </div>
      </div>
    </Modal>
  );
};