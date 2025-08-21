import * as React from "react"
import { cn } from "@/lib/utils"

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "outline" | "ghost" | "destructive"
  size?: "default" | "sm" | "lg"
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "default", size = "default", ...props }, ref) => {
    return (
      <button
        className={cn(
          // Base RetroUI neobrutalist styles with enhanced animations
          "inline-flex items-center justify-center whitespace-nowrap font-bold transition-all duration-150 ease-out disabled:pointer-events-none disabled:opacity-50 retro-border",
          // Enhanced press animation - moves down and right, shadow disappears
          "active:translate-x-1 active:translate-y-1 active:shadow-none transform-gpu",
          // Hover effects with subtle scaling
          "hover:scale-[1.02] hover:transition-transform hover:duration-100",
          // Focus states for accessibility
          "focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-ring",
          // Variants with improved styling
          {
            "bg-primary text-primary-foreground retro-shadow hover:bg-primary/90 hover:shadow-[6px_6px_0px_0px_#000000]": variant === "default",
            "bg-background text-foreground retro-shadow hover:bg-accent hover:shadow-[6px_6px_0px_0px_#000000]": variant === "outline",
            "hover:bg-accent hover:text-accent-foreground shadow-none border-none": variant === "ghost",
            "bg-destructive text-destructive-foreground retro-shadow hover:bg-destructive/90 hover:shadow-[6px_6px_0px_0px_#000000]": variant === "destructive",
          },
          // Sizes with proper RetroUI proportions
          {
            "h-11 px-6 py-2 text-sm": size === "default",
            "h-9 px-4 text-xs": size === "sm", 
            "h-14 px-8 text-base": size === "lg",
          },
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button }
