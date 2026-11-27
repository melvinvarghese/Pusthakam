import { openExternal } from "@/lib/api";
import { formatCopyrightSpan, supportUrl, type CopyrightPlacement } from "@/lib/copyright";

export function CopyrightNotice({ placement }: { placement: CopyrightPlacement }) {
  const href = supportUrl(placement);
  const span = formatCopyrightSpan();

  return (
    <span>
      © {span}{" "}
      <a
        href={href}
        target="_blank"
        rel="noopener noreferrer"
        title="Pusthakam Library Management System by Melvin Varghese — Support & Documentation"
        className="underline decoration-current/40 underline-offset-2 transition-colors hover:text-white hover:decoration-current"
        onClick={(e) => {
          e.preventDefault();
          void openExternal(href);
        }}
      >
        Melvin Varghese
      </a>
    </span>
  );
}
