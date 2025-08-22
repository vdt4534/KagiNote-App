import * as React from "react"
import { Card as ShadcnCard, CardHeader as ShadcnCardHeader, CardContent, CardFooter as ShadcnCardFooter } from "./card-new"
import { cn } from "@/lib/utils"

// Maintain exact API compatibility with existing Card component
// This allows gradual migration without breaking existing code

export const Card = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <ShadcnCard
    ref={ref}
    className={cn(
      "h-full flex flex-col", // Maintain flex behavior from RecordingScreen
      className
    )}
    {...props}
  />
))
Card.displayName = "Card"

export const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <ShadcnCardHeader
    ref={ref}
    className={cn(
      "flex-shrink-0 pb-2 sm:pb-3", // Match existing padding from RecordingScreen
      className
    )}
    {...props}
  />
))
CardHeader.displayName = "CardHeader"

// CardBody maintains the p-0 padding from RecordingScreen
export const CardBody = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <CardContent
    ref={ref}
    className={cn(
      "flex-1 min-h-0 p-0", // Preserve flex-1 and p-0 from RecordingScreen
      className
    )}
    {...props}
  />
))
CardBody.displayName = "CardBody"

export const CardFooter = ShadcnCardFooter

// Export all shadcn components for gradual adoption
export { CardTitle, CardDescription, CardContent } from "./card-new"