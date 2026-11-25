import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { TextField, SelectField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Chip } from "@/components/ui/Chip";
import { serialsApi, configApi, toApiError } from "@/lib/api";
import type {
  SerialSubscription,
  SaveSubscriptionInput,
  SerialIssue,
  CheckInResult,
  RenewSubscriptionInput,
} from "@/lib/api/types";
import { useSession, can } from "@/lib/session";
import { formatMinor } from "@/lib/money";
import { cn } from "@/lib/utils";

type Tab = "subscriptions" | "issues" | "missing" | "expenditure";

export default function PeriodicalsPage() {
  const [tab, setTab] = useState<Tab>("subscriptions");
  const [selectedSubId, setSelectedSubId] = useState<number | null>(null);
  const { session } = useSession();
  const writable = can(session, "serials.write");

  const overview = useQuery({ queryKey: ["config-overview"], queryFn: configApi.overview });
  const currency = overview.data?.library?.currency || "INR";

  return (
    <div className="staff-periodicals space-y-6">
      <PageHeader
        title="Periodicals & Serials"
        description="Journals, magazines and newspapers: subscriptions, automated issue schedules, receiving check-in and claim tracking."
      />

      <div className="flex border-b border-line gap-2" role="tablist">
        <button
          role="tab"
          aria-selected={tab === "subscriptions"}
          onClick={() => setTab("subscriptions")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "subscriptions" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Subscriptions
        </button>
        <button
          role="tab"
          aria-selected={tab === "issues"}
          onClick={() => setTab("issues")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "issues" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Issue Check-In
        </button>
        <button
          role="tab"
          aria-selected={tab === "missing"}
          onClick={() => setTab("missing")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "missing" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Missing & Claims
        </button>
        <button
          role="tab"
          aria-selected={tab === "expenditure"}
          onClick={() => setTab("expenditure")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "expenditure" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Expenditure & Renewals
        </button>
      </div>

      {tab === "subscriptions" && (
        <SubscriptionsTab
          writable={writable}
          currency={currency}
          onViewIssues={(id) => {
            setSelectedSubId(id);
            setTab("issues");
          }}
        />
      )}

      {tab === "issues" && (
        <IssuesTab
          writable={writable}
          currency={currency}
          selectedSubId={selectedSubId}
          onSelectSubId={setSelectedSubId}
        />
      )}

      {tab === "missing" && (
        <MissingTab writable={writable} />
      )}

      {tab === "expenditure" && (
        <ExpenditureTab currency={currency} />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Subscriptions Tab
// ---------------------------------------------------------------------------

function SubscriptionsTab({
  writable,
  currency,
  onViewIssues,
}: {
  writable: boolean;
  currency: string;
  onViewIssues: (subId: number) => void;
}) {
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("all");
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingSub, setEditingSub] = useState<SerialSubscription | null>(null);
  const [renewingSub, setRenewingSub] = useState<SerialSubscription | null>(null);
  const [historySub, setHistorySub] = useState<SerialSubscription | null>(null);

  const qc = useQueryClient();
  const query = useQuery({
    queryKey: ["serials-subscriptions", { status, search }],
    queryFn: () => serialsApi.listSubscriptions({ status, search }),
  });

  const expiringQuery = useQuery({
    queryKey: ["serials-expiring"],
    queryFn: () => serialsApi.listExpiring(30),
  });

  const cancelMutation = useMutation({
    mutationFn: (id: number) => serialsApi.cancelSubscription(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      qc.invalidateQueries({ queryKey: ["serials-expiring"] });
      qc.invalidateQueries({ queryKey: ["serials-expenditure"] });
    },
  });

  return (
    <div className="space-y-4">
      {expiringQuery.data && expiringQuery.data.length > 0 && (
        <div
          role="alert"
          className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-warn/40 bg-warn-bg px-4 py-3 text-xs text-warn"
        >
          <div>
            <span className="font-bold">Due for Renewal: </span>
            {expiringQuery.data.length} {expiringQuery.data.length === 1 ? "subscription is" : "subscriptions are"} expiring within 30 days or overdue.
          </div>
          <div className="flex flex-wrap gap-2">
            {expiringQuery.data.slice(0, 3).map((item) => (
              <Button
                key={item.subscription.id}
                size="sm"
                onClick={() => setRenewingSub(item.subscription)}
              >
                Renew {item.subscription.title}
              </Button>
            ))}
          </div>
        </div>
      )}

      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <input
            type="search"
            placeholder="Search by title or ISSN…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="h-8 w-64 rounded border border-field-line bg-surface px-2.5 text-xs text-fg"
            aria-label="Search subscriptions"
          />
          <div className="flex items-center gap-1.5 text-xs">
            <label htmlFor="sub-status-filter" className="text-muted">Status:</label>
            <select
              id="sub-status-filter"
              value={status}
              onChange={(e) => setStatus(e.target.value)}
              className="h-8 rounded border border-field-line bg-surface px-2 text-xs text-fg"
            >
              <option value="all">All statuses</option>
              <option value="active">Active</option>
              <option value="expired">Expired</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </div>
        </div>

        {writable && (
          <Button size="sm" variant="primary" onClick={() => { setEditingSub(null); setIsModalOpen(true); }}>
            New Subscription
          </Button>
        )}
      </div>

      <section className="rounded-lg border border-line bg-surface">
        {query.isPending ? (
          <p className="p-6 text-xs text-muted">Loading subscriptions…</p>
        ) : query.isError ? (
          <p className="p-6 text-xs text-danger">{toApiError(query.error).message}</p>
        ) : query.data.length === 0 ? (
          <p className="p-6 text-xs text-muted">No subscriptions found matching the filter.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Serial subscriptions">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Title & Publisher</th>
                  <th className="px-4 py-2.5">Frequency</th>
                  <th className="px-4 py-2.5">Period</th>
                  <th className="px-4 py-2.5">Issues Received</th>
                  <th className="px-4 py-2.5">Cost</th>
                  <th className="px-4 py-2.5">Status</th>
                  <th className="px-4 py-2.5 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {query.data.map((sub) => (
                  <tr key={sub.id} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5">
                      <div className="font-semibold text-fg">{sub.title}</div>
                      {sub.publisher && <div className="text-muted">{sub.publisher}</div>}
                      {sub.issn && <div className="text-muted text-[11px]">ISSN: {sub.issn}</div>}
                    </td>
                    <td className="px-4 py-2.5 capitalize">{sub.frequency}</td>
                    <td className="px-4 py-2.5">
                      <div>{sub.startDate} to {sub.endDate}</div>
                    </td>
                    <td className="px-4 py-2.5">
                      <span className="font-medium">{sub.receivedIssuesCount}</span> / {sub.totalIssuesCount}
                      {sub.missingIssuesCount > 0 && (
                        <span className="ml-1 text-danger font-semibold" title="Missing or claimed issues">
                          ({sub.missingIssuesCount} missing)
                        </span>
                      )}
                    </td>
                    <td className="px-4 py-2.5">
                      {formatMinor(sub.costMinor, sub.currency || currency)}
                    </td>
                    <td className="px-4 py-2.5">
                      <SubscriptionStatusBadge status={sub.status} />
                    </td>
                    <td className="px-4 py-2.5 text-right space-x-1 whitespace-nowrap">
                      <Button size="sm" onClick={() => onViewIssues(sub.id)}>
                        Issues
                      </Button>
                      <Button size="sm" onClick={() => setHistorySub(sub)}>
                        Renewals
                      </Button>
                      {writable && (sub.status === "active" || sub.status === "expired") && (
                        <Button size="sm" onClick={() => setRenewingSub(sub)}>
                          Renew
                        </Button>
                      )}
                      {writable && sub.status === "active" && (
                        <Button
                          size="sm"
                          variant="secondary"
                          className="text-danger border-danger hover:bg-danger/10"
                          disabled={cancelMutation.isPending}
                          onClick={() => {
                            if (confirm(`Cancel subscription for "${sub.title}"?`)) {
                              cancelMutation.mutate(sub.id);
                            }
                          }}
                        >
                          Cancel
                        </Button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {isModalOpen && (
        <SubscriptionModal
          initialData={editingSub}
          onClose={() => { setIsModalOpen(false); setEditingSub(null); }}
        />
      )}

      {renewingSub && (
        <RenewModal
          subscription={renewingSub}
          onClose={() => setRenewingSub(null)}
        />
      )}

      {historySub && (
        <RenewalHistoryModal
          subscription={historySub}
          onClose={() => setHistorySub(null)}
          onRenew={() => setRenewingSub(historySub)}
          writable={writable}
          currency={currency}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Issues Tab
// ---------------------------------------------------------------------------

function IssuesTab({
  writable,
  currency: _currency,
  selectedSubId,
  onSelectSubId,
}: {
  writable: boolean;
  currency: string;
  selectedSubId: number | null;
  onSelectSubId: (id: number | null) => void;
}) {
  const [status, setStatus] = useState("all");
  const [checkingInIssue, setCheckingInIssue] = useState<SerialIssue | null>(null);
  const [isAddIssueOpen, setIsAddIssueOpen] = useState(false);

  const qc = useQueryClient();
  const subsQuery = useQuery({
    queryKey: ["serials-subscriptions", { status: "all" }],
    queryFn: () => serialsApi.listSubscriptions(),
  });

  const issuesQuery = useQuery({
    queryKey: ["serials-issues", { subscriptionId: selectedSubId, status }],
    queryFn: () => serialsApi.listIssues({
      subscriptionId: selectedSubId,
      status: status === "all" ? undefined : status,
    }),
  });

  const updateStatusMutation = useMutation({
    mutationFn: ({ id, newStatus }: { id: number; newStatus: string }) =>
      serialsApi.updateIssueStatus(id, newStatus),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
      qc.invalidateQueries({ queryKey: ["serials-missing"] });
    },
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-1.5 text-xs">
            <label htmlFor="issue-sub-filter" className="text-muted">Subscription:</label>
            <select
              id="issue-sub-filter"
              value={selectedSubId ?? ""}
              onChange={(e) => onSelectSubId(e.target.value ? Number(e.target.value) : null)}
              className="h-8 max-w-xs rounded border border-field-line bg-surface px-2 text-xs text-fg"
            >
              <option value="">All subscriptions</option>
              {subsQuery.data?.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.title} ({s.frequency})
                </option>
              ))}
            </select>
          </div>

          <div className="flex items-center gap-1.5 text-xs">
            <label htmlFor="issue-status-filter" className="text-muted">Status:</label>
            <select
              id="issue-status-filter"
              value={status}
              onChange={(e) => setStatus(e.target.value)}
              className="h-8 rounded border border-field-line bg-surface px-2 text-xs text-fg"
            >
              <option value="all">All</option>
              <option value="expected">Expected</option>
              <option value="received">Received</option>
              <option value="missing">Missing</option>
              <option value="claimed">Claimed</option>
            </select>
          </div>
        </div>

        {writable && selectedSubId && (
          <Button size="sm" variant="secondary" onClick={() => setIsAddIssueOpen(true)}>
            + Add Ad-hoc Issue
          </Button>
        )}
      </div>

      <section className="rounded-lg border border-line bg-surface">
        {issuesQuery.isPending ? (
          <p className="p-6 text-xs text-muted">Loading issues…</p>
        ) : issuesQuery.isError ? (
          <p className="p-6 text-xs text-danger">{toApiError(issuesQuery.error).message}</p>
        ) : issuesQuery.data.length === 0 ? (
          <p className="p-6 text-xs text-muted">No issues found matching the selected filter.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Periodical issues">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Issue Number</th>
                  <th className="px-4 py-2.5">Subscription</th>
                  <th className="px-4 py-2.5">Expected Date</th>
                  <th className="px-4 py-2.5">Received Date</th>
                  <th className="px-4 py-2.5">Accession / Barcode</th>
                  <th className="px-4 py-2.5">Status</th>
                  {writable && <th className="px-4 py-2.5 text-right">Actions</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {issuesQuery.data.map((issue) => (
                  <tr key={issue.id} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5 font-semibold text-fg">{issue.issueNumber}</td>
                    <td className="px-4 py-2.5">{issue.subscriptionTitle}</td>
                    <td className="px-4 py-2.5">{issue.expectedDate}</td>
                    <td className="px-4 py-2.5">{issue.receivedDate || "—"}</td>
                    <td className="px-4 py-2.5">
                      {issue.accessionNo ? (
                        <div>
                          <span className="font-mono">{issue.accessionNo}</span>
                          {issue.barcode && <span className="text-muted text-[11px] block">{issue.barcode}</span>}
                        </div>
                      ) : (
                        "—"
                      )}
                    </td>
                    <td className="px-4 py-2.5">
                      <IssueStatusBadge status={issue.status} />
                      {issue.claimCount > 0 && (
                        <span className="text-[11px] text-danger block mt-0.5">
                          Claimed ({issue.claimCount}x)
                        </span>
                      )}
                    </td>
                    {writable && (
                      <td className="px-4 py-2.5 text-right space-x-1 whitespace-nowrap">
                        {issue.status !== "received" && (
                          <>
                            <Button size="sm" variant="primary" onClick={() => setCheckingInIssue(issue)}>
                              Check In
                            </Button>
                            {issue.status !== "missing" && issue.status !== "claimed" && (
                              <Button
                                size="sm"
                                onClick={() => updateStatusMutation.mutate({ id: issue.id, newStatus: "missing" })}
                              >
                                Missing
                              </Button>
                            )}
                            <Button
                              size="sm"
                              onClick={() => updateStatusMutation.mutate({ id: issue.id, newStatus: "claimed" })}
                            >
                              Claim
                            </Button>
                          </>
                        )}
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {checkingInIssue && (
        <CheckInModal
          issue={checkingInIssue}
          onClose={() => setCheckingInIssue(null)}
        />
      )}

      {isAddIssueOpen && selectedSubId && (
        <AddIssueModal
          subscriptionId={selectedSubId}
          onClose={() => setIsAddIssueOpen(false)}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Missing & Claimed Tab
// ---------------------------------------------------------------------------

function MissingTab({ writable }: { writable: boolean }) {
  const [checkingInIssue, setCheckingInIssue] = useState<SerialIssue | null>(null);
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["serials-missing"],
    queryFn: serialsApi.listMissingIssues,
  });

  const claimMutation = useMutation({
    mutationFn: (id: number) => serialsApi.updateIssueStatus(id, "claimed"),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["serials-missing"] });
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
    },
  });

  return (
    <div className="space-y-4">
      <section className="rounded-lg border border-line bg-surface">
        <div className="border-b border-line px-4 py-3">
          <h2 className="text-sm font-semibold">Missing & Overdue Issues</h2>
          <p className="text-xs text-muted">
            Expected issues that have passed their expected delivery date or have been flagged for supplier claims.
          </p>
        </div>
        {query.isPending ? (
          <p className="p-4 text-xs text-muted">Checking for missing issues…</p>
        ) : query.isError ? (
          <p className="p-4 text-xs text-danger">{toApiError(query.error).message}</p>
        ) : query.data.length === 0 ? (
          <p className="p-4 text-xs text-muted">No missing or overdue issues currently recorded. All issues on schedule!</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Missing serial issues">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Title</th>
                  <th className="px-4 py-2.5">Issue Number</th>
                  <th className="px-4 py-2.5">Expected Date</th>
                  <th className="px-4 py-2.5">Status</th>
                  <th className="px-4 py-2.5">Claims Sent</th>
                  {writable && <th className="px-4 py-2.5 text-right">Actions</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {query.data.map((issue) => (
                  <tr key={issue.id} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5 font-semibold text-fg">{issue.subscriptionTitle}</td>
                    <td className="px-4 py-2.5 font-medium">{issue.issueNumber}</td>
                    <td className="px-4 py-2.5 text-danger font-medium">{issue.expectedDate}</td>
                    <td className="px-4 py-2.5">
                      <IssueStatusBadge status={issue.status} />
                    </td>
                    <td className="px-4 py-2.5">
                      {issue.claimCount > 0 ? (
                        <span>
                          {issue.claimCount} (last on {issue.claimDate || "—"})
                        </span>
                      ) : (
                        "None"
                      )}
                    </td>
                    {writable && (
                      <td className="px-4 py-2.5 text-right space-x-1 whitespace-nowrap">
                        <Button
                          size="sm"
                          disabled={claimMutation.isPending}
                          onClick={() => claimMutation.mutate(issue.id)}
                        >
                          Send Claim
                        </Button>
                        <Button
                          size="sm"
                          variant="primary"
                          onClick={() => setCheckingInIssue(issue)}
                        >
                          Check In
                        </Button>
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {checkingInIssue && (
        <CheckInModal
          issue={checkingInIssue}
          onClose={() => setCheckingInIssue(null)}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Modals
// ---------------------------------------------------------------------------

function SubscriptionModal({
  initialData,
  onClose,
}: {
  initialData?: SerialSubscription | null;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [title, setTitle] = useState(initialData?.title || "");
  const [issn, setIssn] = useState(initialData?.issn || "");
  const [publisher, setPublisher] = useState(initialData?.publisher || "");
  const [frequency, setFrequency] = useState(initialData?.frequency || "monthly");
  const [startDate, setStartDate] = useState(initialData?.startDate || new Date().toISOString().slice(0, 10));
  const [endDate, setEndDate] = useState(
    initialData?.endDate ||
      new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10)
  );
  const [cost, setCost] = useState(initialData ? String(initialData.costMinor / 100) : "0");
  const [autoGen, setAutoGen] = useState(true);
  const [prefix, setPrefix] = useState("Issue ");
  const [notes, setNotes] = useState(initialData?.notes || "");

  const saveMutation = useMutation({
    mutationFn: (input: SaveSubscriptionInput) => serialsApi.saveSubscription(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      onClose();
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!title.trim() || !startDate || !endDate) return;

    const costMinor = Math.round((parseFloat(cost) || 0) * 100);
    saveMutation.mutate({
      id: initialData?.id,
      title: title.trim(),
      issn: issn.trim() || null,
      publisher: publisher.trim() || null,
      frequency,
      startDate,
      endDate,
      costMinor,
      notes: notes.trim() || null,
      autoGenerateIssues: initialData ? false : autoGen,
      issuePatternPrefix: prefix.trim() || null,
    });
  };

  return (
    <Modal
      title={initialData ? "Edit Subscription" : "New Periodical Subscription"}
      onClose={onClose}
      wide
    >
      <form onSubmit={onSubmit} className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <TextField
            label="Publication Title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            autoFocus
          />
          <TextField
            label="ISSN (e.g. 0028-0836)"
            value={issn}
            onChange={(e) => setIssn(e.target.value)}
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <TextField
            label="Publisher"
            value={publisher}
            onChange={(e) => setPublisher(e.target.value)}
          />
          <SelectField
            label="Publication Frequency"
            value={frequency}
            onChange={(e) => setFrequency(e.target.value)}
          >
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
            <option value="biweekly">Bi-weekly (fortnightly)</option>
            <option value="monthly">Monthly</option>
            <option value="bimonthly">Bi-monthly</option>
            <option value="quarterly">Quarterly</option>
            <option value="semiannual">Semi-annual</option>
            <option value="annual">Annual</option>
            <option value="irregular">Irregular / Monograph</option>
          </SelectField>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <TextField
            label="Start Date"
            type="date"
            value={startDate}
            onChange={(e) => setStartDate(e.target.value)}
            required
          />
          <TextField
            label="End Date"
            type="date"
            value={endDate}
            onChange={(e) => setEndDate(e.target.value)}
            required
          />
          <TextField
            label="Subscription Cost"
            type="number"
            step="0.01"
            value={cost}
            onChange={(e) => setCost(e.target.value)}
          />
        </div>

        {!initialData && frequency !== "irregular" && (
          <div className="p-3 bg-surface-alt rounded border border-line space-y-2 text-xs">
            <label className="flex items-center gap-2 font-medium cursor-pointer">
              <input
                type="checkbox"
                checked={autoGen}
                onChange={(e) => setAutoGen(e.target.checked)}
                className="rounded border-line"
              />
              Automatically generate expected issues schedule
            </label>
            {autoGen && (
              <div className="pt-1">
                <TextField
                  label="Issue Number Prefix"
                  value={prefix}
                  onChange={(e) => setPrefix(e.target.value)}
                  placeholder="e.g. Issue , Vol 1 No "
                />
              </div>
            )}
          </div>
        )}

        <TextField
          label="Notes (Optional)"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />

        {saveMutation.isError && (
          <p className="text-xs text-danger">{toApiError(saveMutation.error).message}</p>
        )}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={saveMutation.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Saving…" : "Save Subscription"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function CheckInModal({
  issue,
  onClose,
}: {
  issue: SerialIssue;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [receivedDate, setReceivedDate] = useState(new Date().toISOString().slice(0, 10));
  const [accessionCopy, setAccessionCopy] = useState(false);
  const [categoryId, setCategoryId] = useState("");
  const [locationId, setLocationId] = useState("");
  const [notes, setNotes] = useState("");
  const [result, setResult] = useState<CheckInResult | null>(null);

  const overview = useQuery({ queryKey: ["config-overview"], queryFn: configApi.overview });

  const checkInMutation = useMutation({
    mutationFn: serialsApi.checkInIssue,
    onSuccess: (res) => {
      setResult(res);
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
      qc.invalidateQueries({ queryKey: ["serials-missing"] });
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    checkInMutation.mutate({
      issueId: issue.id,
      receivedDate: receivedDate || undefined,
      accessionCopy,
      itemCategoryId: categoryId ? Number(categoryId) : undefined,
      locationId: locationId ? Number(locationId) : undefined,
      notes: notes.trim() || undefined,
    });
  };

  return (
    <Modal title={`Check In Issue: ${issue.issueNumber}`} onClose={onClose}>
      {result ? (
        <div className="space-y-4 text-xs">
          <div className="p-3 bg-emerald-50 text-emerald-900 border border-emerald-200 rounded">
            <p className="font-semibold text-sm">Issue checked in successfully!</p>
            <p className="mt-1">
              {issue.subscriptionTitle} · {issue.issueNumber} marked as received on {result.receivedDate}.
            </p>
          </div>

          {result.accessionNo && (
            <div className="p-3 bg-surface-alt rounded border border-line">
              <span className="text-muted block">Physical Copy Created:</span>
              <span className="font-mono text-sm font-bold block mt-1">
                Accession No: {result.accessionNo}
              </span>
              {result.barcode && (
                <span className="text-muted block font-mono">Barcode: {result.barcode}</span>
              )}
            </div>
          )}

          <div className="flex justify-end pt-2">
            <Button variant="primary" onClick={onClose}>
              Done
            </Button>
          </div>
        </div>
      ) : (
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="p-2.5 bg-surface-alt rounded text-xs">
            <span className="font-semibold block text-fg">{issue.subscriptionTitle}</span>
            <span className="text-muted block">
              Issue: {issue.issueNumber} · Expected Delivery: {issue.expectedDate}
            </span>
          </div>

          <TextField
            label="Received Date"
            type="date"
            value={receivedDate}
            onChange={(e) => setReceivedDate(e.target.value)}
            required
          />

          <div className="p-3 bg-surface-alt rounded border border-line space-y-3 text-xs">
            <label className="flex items-center gap-2 font-medium cursor-pointer">
              <input
                type="checkbox"
                checked={accessionCopy}
                onChange={(e) => setAccessionCopy(e.target.checked)}
                className="rounded border-line"
              />
              Add copy to Accession Register (barcode & accession number)
            </label>

            {accessionCopy && (
              <div className="grid grid-cols-2 gap-3 pt-2">
                <SelectField
                  label="Category"
                  value={categoryId}
                  onChange={(e) => setCategoryId(e.target.value)}
                >
                  <option value="">Default category</option>
                  {overview.data?.itemCategories?.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </SelectField>
                <SelectField
                  label="Location"
                  value={locationId}
                  onChange={(e) => setLocationId(e.target.value)}
                >
                  <option value="">Default location</option>
                  {overview.data?.locations?.map((l) => (
                    <option key={l.id} value={l.id}>
                      {l.name}
                    </option>
                  ))}
                </SelectField>
              </div>
            )}
          </div>

          <TextField
            label="Notes (Optional)"
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
          />

          {checkInMutation.isError && (
            <p className="text-xs text-danger">{toApiError(checkInMutation.error).message}</p>
          )}

          <div className="flex justify-end gap-2 pt-2 border-t border-line">
            <Button onClick={onClose} disabled={checkInMutation.isPending}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={checkInMutation.isPending}>
              {checkInMutation.isPending ? "Checking in…" : "Confirm Check In"}
            </Button>
          </div>
        </form>
      )}
    </Modal>
  );
}

function AddIssueModal({
  subscriptionId,
  onClose,
}: {
  subscriptionId: number;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [issueNumber, setIssueNumber] = useState("");
  const [expectedDate, setExpectedDate] = useState(new Date().toISOString().slice(0, 10));
  const [notes, setNotes] = useState("");

  const addMutation = useMutation({
    mutationFn: serialsApi.addIssue,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
      onClose();
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!issueNumber.trim() || !expectedDate) return;
    addMutation.mutate({
      subscriptionId,
      issueNumber: issueNumber.trim(),
      expectedDate,
      notes: notes.trim() || null,
    });
  };

  return (
    <Modal title="Add Ad-hoc Periodical Issue" onClose={onClose}>
      <form onSubmit={onSubmit} className="space-y-4">
        <TextField
          label="Issue Number or Designation"
          value={issueNumber}
          onChange={(e) => setIssueNumber(e.target.value)}
          placeholder="e.g. Special Edition No. 1, Supplement 2026"
          required
          autoFocus
        />
        <TextField
          label="Expected Delivery Date"
          type="date"
          value={expectedDate}
          onChange={(e) => setExpectedDate(e.target.value)}
          required
        />
        <TextField
          label="Notes (Optional)"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />

        {addMutation.isError && (
          <p className="text-xs text-danger">{toApiError(addMutation.error).message}</p>
        )}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={addMutation.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={addMutation.isPending}>
            {addMutation.isPending ? "Adding…" : "Add Issue"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function RenewModal({
  subscription,
  onClose,
}: {
  subscription: SerialSubscription;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [startDate, setStartDate] = useState(subscription.endDate);
  const [endDate, setEndDate] = useState(
    new Date(new Date(subscription.endDate).getTime() + 365 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10)
  );
  const [cost, setCost] = useState(String(subscription.costMinor / 100));
  const [autoGen, setAutoGen] = useState(true);
  const [prefix, setPrefix] = useState("Issue ");
  const [notes, setNotes] = useState("");

  const renewMutation = useMutation({
    mutationFn: (input: RenewSubscriptionInput) => serialsApi.renewSubscription(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["serials-subscriptions"] });
      qc.invalidateQueries({ queryKey: ["serials-issues"] });
      onClose();
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!startDate || !endDate) return;
    const costMinor = Math.round((parseFloat(cost) || 0) * 100);
    renewMutation.mutate({
      subscriptionId: subscription.id,
      startDate,
      endDate,
      costMinor,
      notes: notes.trim() || null,
      autoGenerateIssues: autoGen,
      issuePatternPrefix: prefix.trim() || null,
    });
  };

  return (
    <Modal title={`Renew Subscription: ${subscription.title}`} onClose={onClose}>
      <form onSubmit={onSubmit} className="space-y-4">
        <div className="p-2.5 bg-surface-alt rounded text-xs">
          <span className="text-muted block">Current Expiry: {subscription.endDate}</span>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <TextField
            label="Renewal Start Date"
            type="date"
            value={startDate}
            onChange={(e) => setStartDate(e.target.value)}
            required
          />
          <TextField
            label="New End Date"
            type="date"
            value={endDate}
            onChange={(e) => setEndDate(e.target.value)}
            required
          />
        </div>

        <TextField
          label="Renewal Cost"
          type="number"
          step="0.01"
          value={cost}
          onChange={(e) => setCost(e.target.value)}
        />

        <div className="p-3 bg-surface-alt rounded border border-line space-y-2 text-xs">
          <label className="flex items-center gap-2 font-medium cursor-pointer">
            <input
              type="checkbox"
              checked={autoGen}
              onChange={(e) => setAutoGen(e.target.checked)}
              className="rounded border-line"
            />
            Generate issues schedule for renewal period
          </label>
          {autoGen && (
            <TextField
              label="Issue Number Prefix"
              value={prefix}
              onChange={(e) => setPrefix(e.target.value)}
            />
          )}
        </div>

        <TextField
          label="Notes (Optional)"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />

        {renewMutation.isError && (
          <p className="text-xs text-danger">{toApiError(renewMutation.error).message}</p>
        )}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={renewMutation.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={renewMutation.isPending}>
            {renewMutation.isPending ? "Renewing…" : "Confirm Renewal"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

// ---------------------------------------------------------------------------
// Badges
// ---------------------------------------------------------------------------

function SubscriptionStatusBadge({ status }: { status: string }) {
  switch (status) {
    case "active":
      return <Chip tone="neutral">Active</Chip>;
    case "expired":
      return <Chip tone="overdue">Expired</Chip>;
    case "cancelled":
      return <Chip tone="neutral">Cancelled</Chip>;
    default:
      return <Chip tone="neutral">{status}</Chip>;
  }
}

function IssueStatusBadge({ status }: { status: string }) {
  switch (status) {
    case "received":
      return <Chip tone="neutral">Received</Chip>;
    case "expected":
      return <Chip tone="neutral">Expected</Chip>;
    case "missing":
      return <Chip tone="overdue">Missing</Chip>;
    case "claimed":
      return <Chip tone="fine">Claimed</Chip>;
    case "cancelled":
      return <Chip tone="neutral">Cancelled</Chip>;
    default:
      return <Chip tone="neutral">{status}</Chip>;
  }
}

// ---------------------------------------------------------------------------
// Renewal History Modal
// ---------------------------------------------------------------------------

function RenewalHistoryModal({
  subscription,
  onClose,
  onRenew,
  writable,
  currency,
}: {
  subscription: SerialSubscription;
  onClose: () => void;
  onRenew: () => void;
  writable: boolean;
  currency: string;
}) {
  const query = useQuery({
    queryKey: ["serials-renewals", subscription.id],
    queryFn: () => serialsApi.listRenewals(subscription.id),
  });

  const renewals = query.data || [];
  const totalRenewalSpend = renewals.reduce((acc, r) => acc + r.costMinor, 0);

  return (
    <Modal title={`Renewal History: ${subscription.title}`} onClose={onClose}>
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-muted border-b border-line pb-3">
          <div>
            <span>Current Coverage: </span>
            <span className="font-semibold text-fg">{subscription.startDate} to {subscription.endDate}</span>
          </div>
          <div>
            <span>Renewals Count: </span>
            <span className="font-semibold text-fg">{renewals.length}</span>
            <span className="ml-3">Cumulative Renewal Spend: </span>
            <span className="font-semibold text-fg">{formatMinor(totalRenewalSpend, currency)}</span>
          </div>
        </div>

        {query.isPending ? (
          <p className="p-4 text-xs text-muted">Loading renewal records…</p>
        ) : query.isError ? (
          <p className="p-4 text-xs text-danger">{toApiError(query.error).message}</p>
        ) : renewals.length === 0 ? (
          <p className="p-4 text-xs text-muted">No renewals recorded for this subscription yet.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Renewal history table">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-3 py-2">Renewal Period</th>
                  <th className="px-3 py-2">Cost</th>
                  <th className="px-3 py-2">Budget</th>
                  <th className="px-3 py-2">Notes</th>
                  <th className="px-3 py-2">Recorded At</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {renewals.map((ren) => (
                  <tr key={ren.id} className="hover:bg-surface-alt/50">
                    <td className="px-3 py-2 font-medium">
                      {ren.startDate} to {ren.endDate}
                    </td>
                    <td className="px-3 py-2 font-medium">
                      {formatMinor(ren.costMinor, ren.currency || currency)}
                    </td>
                    <td className="px-3 py-2 text-muted">{ren.budgetName || "None"}</td>
                    <td className="px-3 py-2 text-muted">{ren.notes || "—"}</td>
                    <td className="px-3 py-2 text-muted">{ren.createdAt ? ren.createdAt.slice(0, 10) : "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        <div className="flex justify-end gap-2 border-t border-line pt-3">
          <Button size="sm" onClick={onClose}>Close</Button>
          {writable && (
            <Button size="sm" variant="primary" onClick={() => { onClose(); onRenew(); }}>
              Renew Subscription
            </Button>
          )}
        </div>
      </div>
    </Modal>
  );
}

// ---------------------------------------------------------------------------
// Expenditure & Reconciliation Tab
// ---------------------------------------------------------------------------

function ExpenditureTab({ currency }: { currency: string }) {
  const currentYear = new Date().getFullYear().toString();
  const [fromDate, setFromDate] = useState(`${currentYear}-01-01`);
  const [toDate, setToDate] = useState(`${currentYear}-12-31`);

  const query = useQuery({
    queryKey: ["serials-expenditure", { fromDate, toDate }],
    queryFn: () => serialsApi.getExpenditure({
      fromDate: fromDate || null,
      toDate: toDate || null,
    }),
  });

  const data = query.data;
  const initialItems = (data?.items || []).filter((i) => i.itemType === "initial");
  const renewalItems = (data?.items || []).filter((i) => i.itemType === "renewal");

  const initialTotal = initialItems.reduce((acc, i) => acc + i.costMinor, 0);
  const renewalTotal = renewalItems.reduce((acc, i) => acc + i.costMinor, 0);

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-1.5 text-xs">
            <label htmlFor="exp-from-date" className="text-muted">From:</label>
            <input
              id="exp-from-date"
              type="date"
              value={fromDate}
              onChange={(e) => setFromDate(e.target.value)}
              className="h-8 rounded border border-field-line bg-surface px-2 text-xs text-fg"
            />
          </div>
          <div className="flex items-center gap-1.5 text-xs">
            <label htmlFor="exp-to-date" className="text-muted">To:</label>
            <input
              id="exp-to-date"
              type="date"
              value={toDate}
              onChange={(e) => setToDate(e.target.value)}
              className="h-8 rounded border border-field-line bg-surface px-2 text-xs text-fg"
            />
          </div>
          <Button
            size="sm"
            onClick={() => {
              setFromDate(`${currentYear}-01-01`);
              setToDate(`${currentYear}-12-31`);
            }}
          >
            Current Year
          </Button>
          <Button
            size="sm"
            onClick={() => {
              setFromDate("");
              setToDate("");
            }}
          >
            All Time
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <div className="rounded-lg border border-line bg-surface p-4">
          <div className="text-xs font-medium text-muted">Total Serials Expenditure</div>
          <div className="mt-1 text-xl font-bold text-fg">
            {formatMinor(data?.totalSpendMinor || 0, data?.currency || currency)}
          </div>
          <div className="mt-1 text-[11px] text-muted">
            {data?.itemCount || 0} cost events in period
          </div>
        </div>

        <div className="rounded-lg border border-line bg-surface p-4">
          <div className="text-xs font-medium text-muted">New Subscriptions Spend</div>
          <div className="mt-1 text-xl font-bold text-fg">
            {formatMinor(initialTotal, data?.currency || currency)}
          </div>
          <div className="mt-1 text-[11px] text-muted">
            {initialItems.length} initial subscriptions
          </div>
        </div>

        <div className="rounded-lg border border-line bg-surface p-4">
          <div className="text-xs font-medium text-muted">Renewals Spend</div>
          <div className="mt-1 text-xl font-bold text-fg">
            {formatMinor(renewalTotal, data?.currency || currency)}
          </div>
          <div className="mt-1 text-[11px] text-muted">
            {renewalItems.length} renewals processed
          </div>
        </div>
      </div>

      <section className="rounded-lg border border-line bg-surface">
        <div className="border-b border-line px-4 py-3">
          <h2 className="text-sm font-semibold">Spending Reconciliation Ledger</h2>
          <p className="text-xs text-muted">
            Itemized expenditures for new subscriptions and renewals falling within the selected date range.
          </p>
        </div>

        {query.isPending ? (
          <p className="p-6 text-xs text-muted">Loading serials expenditure…</p>
        ) : query.isError ? (
          <p className="p-6 text-xs text-danger">{toApiError(query.error).message}</p>
        ) : (data?.items || []).length === 0 ? (
          <p className="p-6 text-xs text-muted">No serial expenditures found for the selected date range.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Serials expenditure ledger">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Title & ISSN</th>
                  <th className="px-4 py-2.5">Type</th>
                  <th className="px-4 py-2.5">Effective Date</th>
                  <th className="px-4 py-2.5">Coverage Period</th>
                  <th className="px-4 py-2.5">Budget</th>
                  <th className="px-4 py-2.5">Cost</th>
                  <th className="px-4 py-2.5">Notes</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {data!.items.map((item) => (
                  <tr key={`${item.itemType}-${item.id}`} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5">
                      <div className="font-semibold text-fg">{item.title}</div>
                      {item.issn && <div className="text-muted text-[11px]">ISSN: {item.issn}</div>}
                    </td>
                    <td className="px-4 py-2.5">
                      <span className={cn(
                        "inline-block rounded px-2 py-0.5 text-[11px] font-semibold",
                        item.itemType === "renewal" ? "bg-accent/10 text-accent" : "bg-muted/10 text-muted"
                      )}>
                        {item.itemType === "renewal" ? "Renewal" : "Initial Sub"}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 font-medium">{item.date}</td>
                    <td className="px-4 py-2.5 text-muted">{item.startDate} to {item.endDate}</td>
                    <td className="px-4 py-2.5 text-muted">{item.budgetName || "None"}</td>
                    <td className="px-4 py-2.5 font-bold text-fg">
                      {formatMinor(item.costMinor, item.currency || currency)}
                    </td>
                    <td className="px-4 py-2.5 text-muted">{item.notes || "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}
