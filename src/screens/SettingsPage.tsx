import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Card, CardHeader, CardBody } from '@/components/ui/card-compat';
import { Button } from '@/components/ui/button-compat';
import { Badge } from '@/components/ui/badge-compat';
import { Input } from '@/components/ui/input-new';
import { Label } from '@/components/ui/label-new';
import { Icon } from '@/components/ui/Icon';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/select-new';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ModelInfo } from './NewMeetingModal';

export interface AppSettings {
  // General
  theme: 'light' | 'dark' | 'system';
  language: string;
  autoSaveInterval: number;
  defaultSaveLocation: string;
  startupBehavior: 'dashboard' | 'last-opened' | 'new-recording';
  
  // Recording
  defaultMicrophone: string;
  sampleRate: number;
  bitDepth: number;
  vadThreshold: number;
  bufferSize: number;
  autoPauseOnSilence: boolean;
  silenceThreshold: number;
  
  // Transcription
  defaultModel: 'turbo' | 'standard' | 'high-accuracy';
  autoDetectLanguage: boolean;
  defaultLanguage: string;
  punctuation: boolean;
  profanityFilter: boolean;
  timestampFormat: 'seconds' | 'hh:mm:ss' | 'mm:ss';
  
  // Speaker Diarization
  enableDiarization: boolean;
  maxSpeakers: number;
  speakerSimilarity: number;
  autoAssignNames: boolean;
  
  // Privacy & Security
  encryptData: boolean;
  autoDeleteDays: number;
  analyticsOptOut: boolean;
  
  // Import/Export
  defaultExportFormat: 'pdf' | 'txt' | 'docx' | 'srt' | 'json';
  includeTimestamps: boolean;
  includeSpeakerLabels: boolean;
  exportQuality: 'low' | 'medium' | 'high';
}

export interface SettingsPageProps {
  settings: AppSettings;
  onUpdateSettings: (settings: Partial<AppSettings>) => void;
  onSave: () => void;
  onCancel: () => void;
  models?: ModelInfo[];
  microphones?: string[];
  className?: string;
}

const defaultSettings: AppSettings = {
  // General
  theme: 'system',
  language: 'en',
  autoSaveInterval: 5,
  defaultSaveLocation: '~/Documents/KagiNote',
  startupBehavior: 'dashboard',
  
  // Recording
  defaultMicrophone: 'Default',
  sampleRate: 48000,
  bitDepth: 16,
  vadThreshold: 0.5,
  bufferSize: 4096,
  autoPauseOnSilence: false,
  silenceThreshold: 3,
  
  // Transcription
  defaultModel: 'standard',
  autoDetectLanguage: true,
  defaultLanguage: 'en',
  punctuation: true,
  profanityFilter: false,
  timestampFormat: 'mm:ss',
  
  // Speaker Diarization
  enableDiarization: true,
  maxSpeakers: 8,
  speakerSimilarity: 0.7,
  autoAssignNames: false,
  
  // Privacy & Security
  encryptData: false,
  autoDeleteDays: 0,
  analyticsOptOut: true,
  
  // Import/Export
  defaultExportFormat: 'pdf',
  includeTimestamps: true,
  includeSpeakerLabels: true,
  exportQuality: 'high',
};

export const SettingsPage: React.FC<SettingsPageProps> = ({
  settings = defaultSettings,
  onUpdateSettings,
  onSave,
  onCancel,
  models = [],
  microphones = ['Default', 'Built-in Microphone', 'External Microphone'],
  className,
}) => {
  const [localSettings, setLocalSettings] = useState<AppSettings>(settings);
  const [activeTab, setActiveTab] = useState('general');
  const [hasChanges, setHasChanges] = useState(false);

  const handleSettingChange = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) => {
    setLocalSettings(prev => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const handleSave = () => {
    onUpdateSettings(localSettings);
    onSave();
    setHasChanges(false);
  };

  const handleCancel = () => {
    setLocalSettings(settings);
    setHasChanges(false);
    onCancel();
  };

  const handleReset = (category: string) => {
    if (!confirm(`Reset ${category} settings to defaults?`)) return;
    
    const categoryDefaults = Object.keys(defaultSettings)
      .filter(key => {
        switch (category) {
          case 'general':
            return ['theme', 'language', 'autoSaveInterval', 'defaultSaveLocation', 'startupBehavior'].includes(key);
          case 'recording':
            return key.includes('Microphone') || key.includes('sample') || key.includes('vad') || key.includes('buffer') || key.includes('silence');
          case 'transcription':
            return key.includes('Model') || key.includes('Language') || key.includes('punctuation') || key.includes('profanity') || key.includes('timestamp');
          case 'diarization':
            return key.includes('diarization') || key.includes('speaker') || key.includes('Speaker');
          case 'privacy':
            return key.includes('encrypt') || key.includes('delete') || key.includes('analytics');
          case 'export':
            return key.includes('export') || key.includes('Export') || key.includes('timestamps') || key.includes('labels');
          default:
            return false;
        }
      })
      .reduce((acc, key) => ({
        ...acc,
        [key]: defaultSettings[key as keyof AppSettings]
      }), {});
    
    setLocalSettings(prev => ({ ...prev, ...categoryDefaults }));
    setHasChanges(true);
  };

  // Calculate model storage usage
  const calculateStorageUsage = () => {
    const downloadedModels = models.filter(m => m.isDownloaded);
    const totalSize = downloadedModels.reduce((sum, m) => {
      const size = parseFloat(m.size.replace('GB', ''));
      return sum + size;
    }, 0);
    return totalSize.toFixed(1);
  };

  return (
    <div className={cn('h-full flex flex-col', className)}>
      {/* Header */}
      <div className="flex-shrink-0 flex items-center justify-between pb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
          Settings
        </h1>
        {hasChanges && (
          <Badge variant="warning" size="sm">
            <Icon name="exclamation-triangle" size="sm" className="mr-1" />
            Unsaved changes
          </Badge>
        )}
      </div>

      {/* Settings Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col">
        <TabsList className="grid w-full grid-cols-3 lg:grid-cols-7">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="recording">Recording</TabsTrigger>
          <TabsTrigger value="transcription">Transcription</TabsTrigger>
          <TabsTrigger value="diarization">Speakers</TabsTrigger>
          <TabsTrigger value="models">Models</TabsTrigger>
          <TabsTrigger value="privacy">Privacy</TabsTrigger>
          <TabsTrigger value="export">Export</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-auto pt-6">
          {/* General Settings */}
          <TabsContent value="general" className="space-y-6">
            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Appearance</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <Label htmlFor="theme">Theme</Label>
                  <Select 
                    value={localSettings.theme} 
                    onValueChange={(value) => handleSettingChange('theme', value as AppSettings['theme'])}
                  >
                    <SelectTrigger id="theme">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="light">Light</SelectItem>
                      <SelectItem value="dark">Dark</SelectItem>
                      <SelectItem value="system">System</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label htmlFor="language">Language</Label>
                  <Select 
                    value={localSettings.language} 
                    onValueChange={(value) => handleSettingChange('language', value)}
                  >
                    <SelectTrigger id="language">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="en">English</SelectItem>
                      <SelectItem value="ja">日本語</SelectItem>
                      <SelectItem value="es">Español</SelectItem>
                      <SelectItem value="fr">Français</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardBody>
            </Card>

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Behavior</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <Label htmlFor="startup">Startup Behavior</Label>
                  <Select 
                    value={localSettings.startupBehavior} 
                    onValueChange={(value) => handleSettingChange('startupBehavior', value as AppSettings['startupBehavior'])}
                  >
                    <SelectTrigger id="startup">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="dashboard">Show Dashboard</SelectItem>
                      <SelectItem value="last-opened">Restore Last Session</SelectItem>
                      <SelectItem value="new-recording">Start New Recording</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label htmlFor="autosave">Auto-save Interval (minutes)</Label>
                  <Input
                    id="autosave"
                    type="number"
                    min="1"
                    max="60"
                    value={localSettings.autoSaveInterval}
                    onChange={(e) => handleSettingChange('autoSaveInterval', parseInt(e.target.value))}
                  />
                </div>
                <div>
                  <Label htmlFor="save-location">Default Save Location</Label>
                  <div className="flex gap-2">
                    <Input
                      id="save-location"
                      type="text"
                      value={localSettings.defaultSaveLocation}
                      onChange={(e) => handleSettingChange('defaultSaveLocation', e.target.value)}
                    />
                    <Button variant="outline">
                      <Icon name="folder" size="sm" />
                    </Button>
                  </div>
                </div>
              </CardBody>
            </Card>
          </TabsContent>

          {/* Recording Settings */}
          <TabsContent value="recording" className="space-y-6">
            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Audio Input</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <Label htmlFor="microphone">Default Microphone</Label>
                  <Select 
                    value={localSettings.defaultMicrophone} 
                    onValueChange={(value) => handleSettingChange('defaultMicrophone', value)}
                  >
                    <SelectTrigger id="microphone">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {microphones.map(mic => (
                        <SelectItem key={mic} value={mic}>{mic}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label htmlFor="sample-rate">Sample Rate (Hz)</Label>
                  <Select 
                    value={localSettings.sampleRate.toString()} 
                    onValueChange={(value) => handleSettingChange('sampleRate', parseInt(value))}
                  >
                    <SelectTrigger id="sample-rate">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="16000">16,000 Hz</SelectItem>
                      <SelectItem value="44100">44,100 Hz</SelectItem>
                      <SelectItem value="48000">48,000 Hz</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label htmlFor="bit-depth">Bit Depth</Label>
                  <Select 
                    value={localSettings.bitDepth.toString()} 
                    onValueChange={(value) => handleSettingChange('bitDepth', parseInt(value))}
                  >
                    <SelectTrigger id="bit-depth">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="16">16-bit</SelectItem>
                      <SelectItem value="24">24-bit</SelectItem>
                      <SelectItem value="32">32-bit</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardBody>
            </Card>

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Voice Activity Detection</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label htmlFor="vad-threshold">VAD Sensitivity</Label>
                    <span className="text-sm text-gray-500">{Math.round(localSettings.vadThreshold * 100)}%</span>
                  </div>
                  <input
                    id="vad-threshold"
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={localSettings.vadThreshold}
                    onChange={(e) => handleSettingChange('vadThreshold', parseFloat(e.target.value))}
                    className="w-full"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="auto-pause">Auto-pause on Silence</Label>
                  <button
                    id="auto-pause"
                    onClick={() => handleSettingChange('autoPauseOnSilence', !localSettings.autoPauseOnSilence)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.autoPauseOnSilence ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.autoPauseOnSilence ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                {localSettings.autoPauseOnSilence && (
                  <div>
                    <Label htmlFor="silence-threshold">Silence Duration (seconds)</Label>
                    <Input
                      id="silence-threshold"
                      type="number"
                      min="1"
                      max="10"
                      value={localSettings.silenceThreshold}
                      onChange={(e) => handleSettingChange('silenceThreshold', parseInt(e.target.value))}
                    />
                  </div>
                )}
              </CardBody>
            </Card>
          </TabsContent>

          {/* Transcription Settings */}
          <TabsContent value="transcription" className="space-y-6">
            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">AI Model</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <Label htmlFor="default-model">Default Model</Label>
                  <Select 
                    value={localSettings.defaultModel} 
                    onValueChange={(value) => handleSettingChange('defaultModel', value as AppSettings['defaultModel'])}
                  >
                    <SelectTrigger id="default-model">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="turbo">Turbo (Fast)</SelectItem>
                      <SelectItem value="standard">Standard (Balanced)</SelectItem>
                      <SelectItem value="high-accuracy">High Accuracy (Best)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="auto-detect">Auto-detect Language</Label>
                  <button
                    id="auto-detect"
                    onClick={() => handleSettingChange('autoDetectLanguage', !localSettings.autoDetectLanguage)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.autoDetectLanguage ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.autoDetectLanguage ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                {!localSettings.autoDetectLanguage && (
                  <div>
                    <Label htmlFor="default-language">Default Language</Label>
                    <Select 
                      value={localSettings.defaultLanguage} 
                      onValueChange={(value) => handleSettingChange('defaultLanguage', value)}
                    >
                      <SelectTrigger id="default-language">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="en">English</SelectItem>
                        <SelectItem value="ja">Japanese</SelectItem>
                        <SelectItem value="es">Spanish</SelectItem>
                        <SelectItem value="fr">French</SelectItem>
                        <SelectItem value="de">German</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                )}
              </CardBody>
            </Card>

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Processing Options</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="punctuation">Add Punctuation</Label>
                  <button
                    id="punctuation"
                    onClick={() => handleSettingChange('punctuation', !localSettings.punctuation)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.punctuation ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.punctuation ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="profanity">Profanity Filter</Label>
                  <button
                    id="profanity"
                    onClick={() => handleSettingChange('profanityFilter', !localSettings.profanityFilter)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.profanityFilter ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.profanityFilter ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                <div>
                  <Label htmlFor="timestamp-format">Timestamp Format</Label>
                  <Select 
                    value={localSettings.timestampFormat} 
                    onValueChange={(value) => handleSettingChange('timestampFormat', value as AppSettings['timestampFormat'])}
                  >
                    <SelectTrigger id="timestamp-format">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="seconds">Seconds (45.2)</SelectItem>
                      <SelectItem value="mm:ss">Minutes:Seconds (00:45)</SelectItem>
                      <SelectItem value="hh:mm:ss">Hours:Minutes:Seconds (00:00:45)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardBody>
            </Card>
          </TabsContent>

          {/* Speaker Diarization Settings */}
          <TabsContent value="diarization" className="space-y-6">
            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Speaker Detection</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label htmlFor="enable-diarization">Enable Speaker Diarization</Label>
                  <button
                    id="enable-diarization"
                    onClick={() => handleSettingChange('enableDiarization', !localSettings.enableDiarization)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.enableDiarization ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.enableDiarization ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                {localSettings.enableDiarization && (
                  <>
                    <div>
                      <Label htmlFor="max-speakers">Maximum Speakers</Label>
                      <Input
                        id="max-speakers"
                        type="number"
                        min="2"
                        max="10"
                        value={localSettings.maxSpeakers}
                        onChange={(e) => handleSettingChange('maxSpeakers', parseInt(e.target.value))}
                      />
                    </div>
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <Label htmlFor="speaker-similarity">Speaker Similarity Threshold</Label>
                        <span className="text-sm text-gray-500">{Math.round(localSettings.speakerSimilarity * 100)}%</span>
                      </div>
                      <input
                        id="speaker-similarity"
                        type="range"
                        min="0.5"
                        max="0.9"
                        step="0.1"
                        value={localSettings.speakerSimilarity}
                        onChange={(e) => handleSettingChange('speakerSimilarity', parseFloat(e.target.value))}
                        className="w-full"
                      />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="auto-assign">Auto-assign Speaker Names</Label>
                      <button
                        id="auto-assign"
                        onClick={() => handleSettingChange('autoAssignNames', !localSettings.autoAssignNames)}
                        className={cn(
                          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                          localSettings.autoAssignNames ? 'bg-blue-600' : 'bg-gray-200'
                        )}
                      >
                        <span
                          className={cn(
                            'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                            localSettings.autoAssignNames ? 'translate-x-6' : 'translate-x-1'
                          )}
                        />
                      </button>
                    </div>
                  </>
                )}
              </CardBody>
            </Card>
          </TabsContent>

          {/* Models & Storage */}
          <TabsContent value="models" className="space-y-6">
            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">AI Models</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                {models.map(model => (
                  <div key={model.id} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{model.name}</span>
                        <Badge variant={model.quality === 'High Accuracy' ? 'success' : model.quality === 'Turbo' ? 'warning' : 'default'} size="sm">
                          {model.quality}
                        </Badge>
                      </div>
                      <div className="text-sm text-gray-500 mt-1">
                        {model.description} • {model.size} • {model.ramRequirement}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {model.isDownloaded ? (
                        <>
                          <Badge variant="success" size="sm">
                            <Icon name="check" size="sm" className="mr-1" />
                            Downloaded
                          </Badge>
                          <Button variant="ghost" size="sm">
                            <Icon name="trash" size="sm" />
                          </Button>
                        </>
                      ) : (
                        <Button variant="outline" size="sm">
                          <Icon name="download" size="sm" className="mr-1" />
                          Download
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </CardBody>
            </Card>

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Storage Usage</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <div className="flex justify-between mb-2">
                    <span className="text-sm text-gray-600">Models</span>
                    <span className="text-sm font-medium">{calculateStorageUsage()} GB</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div 
                      className="bg-blue-600 h-2 rounded-full"
                      style={{ width: `${(parseFloat(calculateStorageUsage()) / 10) * 100}%` }}
                    />
                  </div>
                </div>
                <Button variant="outline" className="w-full">
                  <Icon name="folder" size="sm" className="mr-2" />
                  Change Model Storage Location
                </Button>
                <Button variant="outline" className="w-full">
                  <Icon name="trash" size="sm" className="mr-2" />
                  Clear Cache
                </Button>
              </CardBody>
            </Card>
          </TabsContent>

          {/* Privacy & Security */}
          <TabsContent value="privacy" className="space-y-6">
            <Card>
              <CardHeader>
                <div className="flex items-center gap-2">
                  <Icon name="shield-check" size="base" className="text-green-600" />
                  <h2 className="text-lg font-semibold">Privacy Settings</h2>
                </div>
              </CardHeader>
              <CardBody className="space-y-4">
                <div className="p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                  <div className="flex items-center gap-2 text-green-700 dark:text-green-300">
                    <Icon name="check-circle" size="sm" />
                    <span className="font-medium">100% Local Processing</span>
                  </div>
                  <p className="text-sm text-green-600 dark:text-green-400 mt-1">
                    All transcription happens on your device. No data is sent to external servers.
                  </p>
                </div>
                
                <div className="flex items-center justify-between">
                  <Label htmlFor="encrypt">Encrypt Stored Data</Label>
                  <button
                    id="encrypt"
                    onClick={() => handleSettingChange('encryptData', !localSettings.encryptData)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.encryptData ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.encryptData ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                
                <div>
                  <Label htmlFor="auto-delete">Auto-delete Recordings After (days)</Label>
                  <Input
                    id="auto-delete"
                    type="number"
                    min="0"
                    max="365"
                    value={localSettings.autoDeleteDays}
                    onChange={(e) => handleSettingChange('autoDeleteDays', parseInt(e.target.value))}
                    placeholder="0 = Never delete"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    Set to 0 to keep recordings indefinitely
                  </p>
                </div>
                
                <div className="flex items-center justify-between">
                  <Label htmlFor="analytics">Opt-out of Anonymous Analytics</Label>
                  <button
                    id="analytics"
                    onClick={() => handleSettingChange('analyticsOptOut', !localSettings.analyticsOptOut)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.analyticsOptOut ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.analyticsOptOut ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
              </CardBody>
            </Card>

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold text-red-600">Danger Zone</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <Button variant="destructive" className="w-full">
                  <Icon name="trash" size="sm" className="mr-2" />
                  Clear All Transcripts
                </Button>
                <Button variant="destructive" className="w-full">
                  <Icon name="exclamation-triangle" size="sm" className="mr-2" />
                  Reset All Settings
                </Button>
              </CardBody>
            </Card>
          </TabsContent>

          {/* Import/Export Settings */}
          <TabsContent value="export" className="space-y-6">
            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Export Preferences</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <div>
                  <Label htmlFor="export-format">Default Export Format</Label>
                  <Select 
                    value={localSettings.defaultExportFormat} 
                    onValueChange={(value) => handleSettingChange('defaultExportFormat', value as AppSettings['defaultExportFormat'])}
                  >
                    <SelectTrigger id="export-format">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="pdf">PDF Document</SelectItem>
                      <SelectItem value="txt">Plain Text</SelectItem>
                      <SelectItem value="docx">Word Document</SelectItem>
                      <SelectItem value="srt">SubRip Subtitle</SelectItem>
                      <SelectItem value="json">JSON Data</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="flex items-center justify-between">
                  <Label htmlFor="include-timestamps">Include Timestamps</Label>
                  <button
                    id="include-timestamps"
                    onClick={() => handleSettingChange('includeTimestamps', !localSettings.includeTimestamps)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.includeTimestamps ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.includeTimestamps ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                
                <div className="flex items-center justify-between">
                  <Label htmlFor="include-speakers">Include Speaker Labels</Label>
                  <button
                    id="include-speakers"
                    onClick={() => handleSettingChange('includeSpeakerLabels', !localSettings.includeSpeakerLabels)}
                    className={cn(
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                      localSettings.includeSpeakerLabels ? 'bg-blue-600' : 'bg-gray-200'
                    )}
                  >
                    <span
                      className={cn(
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        localSettings.includeSpeakerLabels ? 'translate-x-6' : 'translate-x-1'
                      )}
                    />
                  </button>
                </div>
                
                <div>
                  <Label htmlFor="export-quality">Export Quality</Label>
                  <Select 
                    value={localSettings.exportQuality} 
                    onValueChange={(value) => handleSettingChange('exportQuality', value as AppSettings['exportQuality'])}
                  >
                    <SelectTrigger id="export-quality">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="low">Low (Smaller file size)</SelectItem>
                      <SelectItem value="medium">Medium (Balanced)</SelectItem>
                      <SelectItem value="high">High (Best quality)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardBody>
            </Card>

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Backup & Restore</h2>
              </CardHeader>
              <CardBody className="space-y-4">
                <Button variant="outline" className="w-full">
                  <Icon name="download" size="sm" className="mr-2" />
                  Export All Settings
                </Button>
                <Button variant="outline" className="w-full">
                  <Icon name="upload" size="sm" className="mr-2" />
                  Import Settings
                </Button>
              </CardBody>
            </Card>
          </TabsContent>
        </div>
      </Tabs>

      {/* Footer Actions */}
      <div className="flex-shrink-0 flex items-center justify-between pt-6 border-t border-gray-200 dark:border-gray-700">
        <Button
          variant="ghost"
          onClick={() => handleReset(activeTab)}
        >
          Reset {activeTab.charAt(0).toUpperCase() + activeTab.slice(1)}
        </Button>
        <div className="flex gap-3">
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={!hasChanges}
          >
            Cancel
          </Button>
          <Button
            variant="default"
            onClick={handleSave}
            disabled={!hasChanges}
          >
            <Icon name="check" size="sm" className="mr-2" />
            Save Changes
          </Button>
        </div>
      </div>
    </div>
  );
};