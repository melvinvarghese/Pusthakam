import { Link, useLocation } from "react-router-dom";
import { ALL_NAV_ITEMS } from "@/app/nav";
import { PageHeader } from "@/components/PageHeader";
import { Panel } from "@/components/ui/Panel";
import { buttonStyles } from "@/components/ui/Button";

/** Honest placeholder for modules that are planned but not built. Never presented as a finished module. */
export function NotAvailable() {
  const { pathname } = useLocation();
  const item = ALL_NAV_ITEMS.find((i) => i.path === pathname);

  if (!item) {
    return (
      <div className="mx-auto max-w-[1100px] space-y-5 p-6">
        <PageHeader title="Page not found" description="This page does not exist in Pusthakam." />
        <Link to="/" className={buttonStyles({ variant: "primary" })}>Back to Dashboard</Link>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title={item.label} description={item.summary} />
      <Panel className="flex max-w-xl items-start gap-4 p-5">
        {/* Empty label: a marker for "nothing catalogued here yet"; it shows no data. */}
        <span className="call-label" data-ghost="" aria-hidden="true"><span /><span /><span /></span>
        <div>
          <h2 className="text-base font-semibold">{item.label} is not available yet</h2>
          <p className="mt-1 text-muted">
            This module is planned for development phase {item.phase} and is not part of this version.
          </p>
          <Link to="/" className={`${buttonStyles({ variant: "secondary" })} mt-4`}>Back to Dashboard</Link>
        </div>
      </Panel>
    </div>
  );
}
