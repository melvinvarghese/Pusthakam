import { useLocation } from "react-router-dom";
import { Hammer } from "lucide-react";
import { ALL_NAV_ITEMS } from "@/app/nav";
import { EmptyState } from "@/components/EmptyState";

/** Honest placeholder for modules that are planned but not built. Never presented as a finished module. */
export function NotAvailable() {
  const { pathname } = useLocation();
  const item = ALL_NAV_ITEMS.find((i) => i.path === pathname);
  return (
    <div>
      <EmptyState
        icon={Hammer}
        title={`${item?.label ?? "This section"} is not available yet`}
        description={
          item
            ? `This module is planned for development phase ${item.phase} and is not part of this version.`
            : "This page does not exist."
        }
      />
    </div>
  );
}
