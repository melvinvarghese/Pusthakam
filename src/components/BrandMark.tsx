import logoUrl from "@/assets/pusthakam-logo.png";

/** Decorative beside the Pusthakam wordmark. */
export function BrandMark({ size = "default" }: { size?: "default" | "splash" }) {
  return (
    <img
      src={logoUrl}
      alt=""
      aria-hidden="true"
      width={size === "splash" ? 160 : 40}
      height={size === "splash" ? 160 : 40}
      className={size === "splash" ? "h-36 w-36 shrink-0 object-contain sm:h-40 sm:w-40" : "h-10 w-10 shrink-0 object-contain"}
    />
  );
}
