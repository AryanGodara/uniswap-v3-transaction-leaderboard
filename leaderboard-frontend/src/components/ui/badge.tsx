import * as React from "react"
import { cn } from "@/lib/utils"

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "secondary" | "destructive" | "outline"
}

function Badge({ className, variant = "default", ...props }: BadgeProps) {
  return (
    <div
      className={cn(
        "inline-flex items-center px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 retro-border",
        {
          "bg-primary text-primary-foreground retro-shadow-sm hover:bg-primary/80": variant === "default",
          "bg-secondary text-secondary-foreground retro-shadow-sm hover:bg-secondary/80": variant === "secondary",
          "bg-destructive text-destructive-foreground retro-shadow-sm hover:bg-destructive/80": variant === "destructive",
          "text-foreground retro-shadow-sm": variant === "outline",
        },
        className
      )}
      {...props}
    />
  )
}

export { Badge }
