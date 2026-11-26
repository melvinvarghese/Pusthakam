import { useTranslation } from "@/lib/translations";
import { LanguageSelect } from "@/components/LanguageSelect";
import type { ReactNode } from "react";
import { BrandMark } from "@/components/BrandMark";

/** Full-screen frame for the screens shown before the app: setup and sign-in. */
export function AuthLayout({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  return (
    <div className="staff-auth flex min-h-full flex-col items-center justify-center gap-6 bg-ink p-6">
      <div className="flex items-center gap-3 text-white">
        <BrandMark />
        <div>
          <div className="text-[15px] font-bold leading-[18px]">Pusthakam</div>
          <div className="text-xs text-rail-mute">{t("A Better Way to Manage Libraries.")}</div>
        </div>
      </div>
      <div className="w-[min(24rem,100%)] rounded-lg bg-surface p-4 text-fg"><LanguageSelect /></div>
      {children}
    </div>
  );
}
