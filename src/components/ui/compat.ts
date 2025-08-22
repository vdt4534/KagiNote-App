// Compatibility layer for smooth shadcn/ui migration
// This file provides a single source of truth for UI components
// allowing gradual migration from custom components to shadcn/ui

// Export new shadcn/ui-based components as the default
export { Button, buttonVariants } from './button-new';
export type { ButtonProps } from './button-new';

export { Badge, badgeVariants } from './badge-new';
export type { BadgeProps } from './badge-new';

export { Input } from './input-new';
export type { InputProps } from './input-new';

export { Label } from './label-new';

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardDescription,
  CardContent,
  CardBody, // Alias for backward compatibility
} from './card-new';

export {
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
} from './select-new';

export {
  Sheet,
  SheetPortal,
  SheetOverlay,
  SheetTrigger,
  SheetClose,
  SheetContent,
  SheetHeader,
  SheetFooter,
  SheetTitle,
  SheetDescription,
} from './sheet-new';

// Components not yet migrated (keeping original exports)
export { Icon } from './Icon';
export type { IconProps } from './Icon';

export { Modal } from './Modal';
export type { ModalProps } from './Modal';

export { LoadingSpinner } from './LoadingSpinner';
export type { LoadingSpinnerProps } from './LoadingSpinner';

export { Toast } from './Toast';
export type { ToastProps } from './Toast';

export { default as ToastContainer } from './ToastContainer';
export { ToastProvider, useToast, createDiarizationToasts } from './ToastContainer';

// Export utility function
export { cn } from '@/lib/utils';