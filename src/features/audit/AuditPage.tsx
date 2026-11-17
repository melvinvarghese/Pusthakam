import { useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";
import { auditApi, toApiError } from "@/lib/api";
import type { AuditQuery } from "@/lib/api/types";
import { can, useSession } from "@/lib/session";

export function AuditPage() {
  const { session } = useSession();
  const allowed = can(session, "audit.read");
  const [query, setQuery] = useState<AuditQuery>({ page: 1 });
  const result = useQuery({ queryKey: ["audit", query], queryFn: () => auditApi.search(query), enabled: allowed, retry: false });

  function applyFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const fields = new FormData(event.currentTarget);
    const from = String(fields.get("from") ?? "");
    const to = String(fields.get("to") ?? "");
    const actor = String(fields.get("actor") ?? "").trim();
    const action = String(fields.get("action") ?? "").trim();
    const next = { from, to, actor, action, page: 1 };
    if (query.page === 1 && from === query.from && to === query.to && actor === query.actor && action === query.action) {
      void result.refetch();
    } else {
      setQuery(next);
    }
  }

  if (!allowed) return <p role="alert" className="p-6">You do not have permission to view the audit log.</p>;
  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader title="Audit Log" />
      <form className="flex flex-wrap items-end gap-3" onSubmit={applyFilters}>
        <TextField label="Start date" name="from" type="date" />
        <TextField label="End date" name="to" type="date" />
        <TextField label="Staff name" name="actor" maxLength={120} />
        <TextField label="Action code" name="action" maxLength={120} placeholder="catalogue.record_archived" />
        <Button type="submit" disabled={result.isFetching}>Search audit log</Button>
      </form>
      {result.isPending && <p role="status" className="text-muted">Loading audit log.</p>}
      {result.isError && <p role="alert" className="text-danger">{toApiError(result.error).message}</p>}
      {result.data && (
        <Panel>
          {result.data.rows.length === 0 ? <p className="p-6 text-muted">No audit entries match.</p> : (
            <>
              <div className="overflow-x-auto">
                <table aria-label="Audit entries" className="w-full border-collapse text-left text-sm">
                  <thead className="border-b border-line text-xs text-muted"><tr>
                    {["Date and time", "Staff", "Action", "Record", "Summary"].map((label) => (
                      <th key={label} scope="col" className="px-4 py-2">{label}</th>
                    ))}
                  </tr></thead>
                  <tbody>{result.data.rows.map((entry) => (
                    <tr key={entry.id} className="border-b border-line last:border-b-0">
                      <td className="whitespace-nowrap px-4 py-2"><time dateTime={entry.occurredAt}>{new Date(entry.occurredAt).toLocaleString()}</time></td>
                      <td className="px-4 py-2">{entry.actor ?? "System"}</td>
                      <td className="px-4 py-2 font-mono text-xs">{entry.action}</td>
                      <td className="px-4 py-2 text-xs">{entry.entityType}{entry.entityId !== null && ` #${entry.entityId}`}</td>
                      <td className="px-4 py-2">{entry.summary}</td>
                    </tr>
                  ))}</tbody>
                </table>
              </div>
              <Pager page={result.data.page} pageSize={result.data.pageSize} total={result.data.total}
                onPage={(page) => setQuery((previous) => ({ ...previous, page }))} />
            </>
          )}
        </Panel>
      )}
    </div>
  );
}
