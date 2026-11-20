import { useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Panel, PanelHeader } from "@/components/ui/Panel";
import { ApiError, catalogueApi, configApi } from "@/lib/api";
import { LibraryCalendarPanel } from "./LibraryCalendarPanel";
import { OptionalModules } from "./OptionalModules";
import { LibraryCustomFields } from "./LibraryCustomFields";
import { LanSettingsPanel } from "./LanSettingsPanel";
import type { AccessionFormat, ArchiveKind, ConfigOverview, Library, Named, Policy, Shelf } from "@/lib/api/types";
import { formatMinor } from "@/lib/money";
import { can, useSession } from "@/lib/session";
import {
  AccessionFormatDialog, ConfirmDeletePolicy, LibraryDialog, LIST_LABELS, ListDialog, PolicyDialog, PROFILE_LABELS, ShelfDialog,
  type ListKind,
} from "./dialogs";

type Dialog =
  | { type: "library"; library: Library }
  | { type: "list"; kind: ListKind; item?: Named }
  | { type: "shelf"; shelf?: Shelf }
  | { type: "policy"; policy?: Policy }
  | { type: "deletePolicy"; id: number }
  | { type: "accession"; format: AccessionFormat }
  | null;

function Th({ children }: { children?: ReactNode }) {
  return <th className="h-8 border-b border-line px-4 text-left text-[11px] font-semibold uppercase tracking-wide text-muted">{children}</th>;
}
function Td({ children, className = "" }: { children?: ReactNode; className?: string }) {
  return <td className={`border-b border-line px-4 py-2 align-middle ${className}`}>{children}</td>;
}

function Empty({ children }: { children: ReactNode }) {
  return <p className="px-4 py-6 text-center text-muted">{children}</p>;
}

export function SettingsPage() {
  const { session } = useSession();
  const canEdit = can(session, "admin.settings");
  const qc = useQueryClient();
  const config = useQuery({ queryKey: ["config"], queryFn: configApi.overview, retry: false });
  const accession = useQuery({ queryKey: ["accession-format"], queryFn: catalogueApi.accessionFormat, retry: false });
  const [dialog, setDialog] = useState<Dialog>(null);
  const close = () => setDialog(null);

  const archive = useMutation({
    mutationFn: (v: { kind: ArchiveKind; id: number; archived: boolean }) => configApi.setArchived(v.kind, v.id, v.archived),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["config"] }),
  });
  const remove = useMutation({
    mutationFn: (id: number) => configApi.deletePolicy(id),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["config"] });
      close();
    },
  });

  if (config.isPending) return <div className="p-6 text-muted" role="status">Loading settings…</div>;
  if (config.isError || !config.data) {
    return (
      <div className="p-6" role="alert">
        <p className="text-danger">{config.error instanceof ApiError ? config.error.message : "Could not load settings."}</p>
      </div>
    );
  }
  const c: ConfigOverview = config.data;
  const currency = c.library?.currency ?? "INR";
  const memberName = (id: number) => c.memberCategories.find((m) => m.id === id)?.name ?? "?";
  const itemName = (id: number) => c.itemCategories.find((m) => m.id === id)?.name ?? "?";
  const locationName = (id: number) => c.locations.find((m) => m.id === id)?.name ?? "?";

  const ArchiveBtn = ({ kind, id, archived }: { kind: ArchiveKind; id: number; archived: boolean }) => (
    <Button size="sm" variant="ghost" disabled={archive.isPending} onClick={() => archive.mutate({ kind, id, archived: !archived })}>
      {archived ? "Restore" : "Archive"}
    </Button>
  );

  const listPanel = (kind: ListKind, rows: (Named & { membershipDays?: number })[]) => {
    const labels = LIST_LABELS[kind];
    return (
      <Panel aria-labelledby={`${kind}-h`}>
        <PanelHeader
          id={`${kind}-h`}
          title={labels.plural}
          aside={canEdit && <Button size="sm" onClick={() => setDialog({ type: "list", kind })}>Add {labels.singular}</Button>}
        />
        {rows.length === 0 ? (
          <Empty>No {labels.plural.toLowerCase()} yet.</Empty>
        ) : (
          <table className="w-full table-fixed border-collapse">
            <thead><tr><Th>Name</Th>{kind === "memberCategory" && <Th>Membership</Th>}<Th>Status</Th><Th /></tr></thead>
            <tbody>
              {rows.map((r) => (
                <tr key={r.id} className={r.archived ? "text-muted" : undefined}>
                  <Td>
                    <span className="font-medium">{r.name}</span>
                    {r.description && <span className="block text-xs text-muted">{r.description}</span>}
                  </Td>
                  {kind === "memberCategory" && <Td>{r.membershipDays} days</Td>}
                  <Td>{r.archived ? <Chip>Archived</Chip> : "Active"}</Td>
                  <Td className="whitespace-nowrap text-right">
                    {canEdit && (
                      <>
                        <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "list", kind, item: r })}>Edit</Button>
                        <ArchiveBtn kind={kind} id={r.id} archived={r.archived} />
                      </>
                    )}
                  </Td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Panel>
    );
  };

  return (
    <div className="mx-auto max-w-[1100px] space-y-5 p-6">
      <PageHeader
        title="Library Settings"
        description={canEdit ? "Everything about how your library works. Changes apply immediately." : "You can view these settings but not change them."}
      />

      {archive.isError && <p role="alert" className="text-danger">{archive.error instanceof ApiError ? archive.error.message : "Could not update."}</p>}
      <OptionalModules canEdit={canEdit} />

      <Panel aria-labelledby="library-h">
        <PanelHeader
          id="library-h"
          title="Library"
          aside={canEdit && c.library && <Button size="sm" onClick={() => setDialog({ type: "library", library: c.library! })}>Edit</Button>}
        />
        {c.library ? (
          <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 p-4">
            <dt className="text-muted">Name</dt><dd className="font-medium">{c.library.name}</dd>
            <dt className="text-muted">Type</dt><dd>{PROFILE_LABELS[c.library.profile]}</dd>
            <dt className="text-muted">Address</dt><dd>{c.library.address ?? "Not set"}</dd>
            <dt className="text-muted">Contact</dt><dd>{c.library.contact ?? "Not set"}</dd>
            <dt className="text-muted">Currency</dt><dd>{c.library.currency}</dd>
          </dl>
        ) : (
          <Empty>The library has not been set up yet.</Empty>
        )}
      </Panel>

      <LibraryCustomFields canEdit={canEdit} />
      <LibraryCalendarPanel canEdit={canEdit} />
      <LanSettingsPanel />
      {listPanel("memberCategory", c.memberCategories)}
      {listPanel("itemCategory", c.itemCategories)}

      <Panel aria-labelledby="policy-h">
        <PanelHeader id="policy-h" title="Loan rules" aside={canEdit && <Button size="sm" onClick={() => setDialog({ type: "policy" })}>Add loan rule</Button>} />
        {c.policies.length === 0 ? (
          <Empty>No loan rules yet. A member category can only borrow item categories that have a rule.</Empty>
        ) : (
          <table className="w-full table-fixed border-collapse">
            <thead><tr><Th>Member</Th><Th>Item</Th><Th>Loan</Th><Th>At a time</Th><Th>Renewals</Th><Th>Fine / day</Th><Th /></tr></thead>
            <tbody>
              {c.policies.map((p) => (
                <tr key={p.id}>
                  <Td className="font-medium">{memberName(p.memberCategoryId)}</Td>
                  <Td>{itemName(p.itemCategoryId)}</Td>
                  <Td>{p.loanDays} days</Td>
                  <Td>{p.maxLoans}</Td>
                  <Td>{p.maxRenewals === 0 ? "None" : `${p.maxRenewals} × ${p.renewalDays} days`}</Td>
                  <Td>{p.finePerDayMinor === 0 ? "None" : formatMinor(p.finePerDayMinor, currency)}</Td>
                  <Td className="whitespace-nowrap text-right">
                    {canEdit && (
                      <>
                        <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "policy", policy: p })}>Edit</Button>
                        <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "deletePolicy", id: p.id })}>Delete</Button>
                      </>
                    )}
                  </Td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Panel>

      {listPanel("collection", c.collections)}
      {listPanel("location", c.locations)}

      {accession.data && (
        <Panel aria-labelledby="accession-h">
          <PanelHeader
            id="accession-h"
            title="Accession numbers"
            aside={canEdit && <Button size="sm" onClick={() => setDialog({ type: "accession", format: accession.data })}>Edit</Button>}
          />
          <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 p-4">
            <dt className="text-muted">Next number</dt>
            <dd className="font-mono">{`${accession.data.prefix}${String(accession.data.next).padStart(accession.data.padding, "0")}`}</dd>
            <dt className="text-muted">Format</dt>
            <dd>{accession.data.prefix ? `Prefix "${accession.data.prefix}", ` : "No prefix, "}{accession.data.padding} digits</dd>
          </dl>
        </Panel>
      )}

      <Panel aria-labelledby="shelf-h">
        <PanelHeader id="shelf-h" title="Shelves" aside={canEdit && <Button size="sm" onClick={() => setDialog({ type: "shelf" })}>Add shelf</Button>} />
        {c.shelves.length === 0 ? (
          <Empty>No shelves yet. Add shelves to record exactly where books are kept.</Empty>
        ) : (
          <table className="w-full table-fixed border-collapse">
            <thead><tr><Th>Location</Th><Th>Shelf</Th><Th>Status</Th><Th /></tr></thead>
            <tbody>
              {c.shelves.map((s) => (
                <tr key={s.id} className={s.archived ? "text-muted" : undefined}>
                  <Td>{locationName(s.locationId)}</Td>
                  <Td><span className="font-medium">{s.code}</span>{s.description && <span className="block text-xs text-muted">{s.description}</span>}</Td>
                  <Td>{s.archived ? <Chip>Archived</Chip> : "Active"}</Td>
                  <Td className="whitespace-nowrap text-right">
                    {canEdit && (
                      <>
                        <Button size="sm" variant="ghost" onClick={() => setDialog({ type: "shelf", shelf: s })}>Edit</Button>
                        <ArchiveBtn kind="shelf" id={s.id} archived={s.archived} />
                      </>
                    )}
                  </Td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Panel>

      {dialog?.type === "library" && <LibraryDialog library={dialog.library} onClose={close} />}
      {dialog?.type === "list" && <ListDialog kind={dialog.kind} item={dialog.item} onClose={close} />}
      {dialog?.type === "shelf" && <ShelfDialog shelf={dialog.shelf} locations={c.locations} onClose={close} />}
      {dialog?.type === "policy" && <PolicyDialog policy={dialog.policy} config={c} onClose={close} />}
      {dialog?.type === "accession" && <AccessionFormatDialog format={dialog.format} onClose={close} />}
      {dialog?.type === "deletePolicy" && (
        <ConfirmDeletePolicy
          pending={remove.isPending}
          error={remove.isError ? (remove.error instanceof ApiError ? remove.error.message : "Could not delete.") : null}
          onConfirm={() => remove.mutate(dialog.id)}
          onClose={close}
        />
      )}
    </div>
  );
}

