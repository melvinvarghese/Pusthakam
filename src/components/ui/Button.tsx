import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

export const buttonStyles = cva(
  "inline-flex shrink-0 items-center justify-center gap-1.5 whitespace-nowrap rounded-md border font-medium transition-colors disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        primary: "border-primary bg-primary font-semibold text-primary-fg hover:opacity-90 active:opacity-80",
        secondary: "border-line-strong bg-surface text-fg hover:bg-band active:bg-line",
        ghost: "border-transparent text-muted hover:bg-band hover:text-fg",
      },
      size: {
        sm: "h-[26px] px-2.5 text-xs",
        md: "h-8 px-3 text-[13px]",
        lg: "h-9 px-4 text-[13px]",
      },
    },
    defaultVariants: { variant: "secondary", size: "md" },
  },
);

type Props = ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof buttonStyles>;

export function Button({ className, variant, size, type = "button", ...rest }: Props) {
  return <button type={type} className={cn(buttonStyles({ variant, size }), className)} {...rest} />;
}
