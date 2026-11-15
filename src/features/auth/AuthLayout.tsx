import type { ReactNode } from "react";
import { BrandMark } from "@/components/BrandMark";

/** Full-screen frame for the screens shown before the app: setup and sign-in. */
export function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-full flex-col items-center justify-center gap-6 bg-ink p-6">
      <div className="flex items-center gap-3 text-white">
        <BrandMark />
        <div>
          <div className="text-[15px] font-bold leading-[18px]">Pusthakam</div>
          <div className="text-xs text-rail-mute">A Better Way to Manage Libraries.</div>
        </div>
      </div>
      {children}
    </div>
  );
}
