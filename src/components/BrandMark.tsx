/**
 * The Pusthakam logo mark: a miniature spine label. Decorative (the wordmark beside it carries the name).
 * The band uses the literature class colour; it is a logo, not data.
 */
export function BrandMark() {
  return (
    <span
      aria-hidden="true"
      className="relative flex h-[30px] w-[26px] shrink-0 flex-col justify-end rounded-b-[3px] bg-[var(--label-stock)] px-[3px] pb-[3px] font-mono text-[8px] font-bold leading-[9px] text-[var(--label-ink)]"
    >
      <span className="absolute inset-x-0 top-0 h-1 rounded-t-[2px] bg-[var(--cls-8)]" />
      <span>PUS</span>
      <span className="font-normal">THK</span>
    </span>
  );
}
