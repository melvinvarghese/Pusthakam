import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { TextField, SelectField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { Chip } from "@/components/ui/Chip";
import { acquisitionsApi, configApi, toApiError } from "@/lib/api";
import type {
  VendorInput,
  BudgetInput,
  AcquisitionRequest,
  CreateRequestInput,
  PurchaseOrderItem,
  ReceiveResult,
} from "@/lib/api/types";
import { useSession, can } from "@/lib/session";
import { formatMinor } from "@/lib/money";

type Tab = "requests" | "orders" | "vendors" | "budgets";

export default function AcquisitionsPage() {
  const [tab, setTab] = useState<Tab>("requests");
  const { session } = useSession();
  const writable = can(session, "acquisitions.write");

  const overview = useQuery({ queryKey: ["config-overview"], queryFn: configApi.overview });
  const currency = overview.data?.library?.currency || "INR";

  return (
    <div className="space-y-6">
      <PageHeader
        title="Acquisitions"
        description="Book purchase recommendations, approval workflow, vendor registry, and purchase orders."
      />

      <div className="flex border-b border-line gap-2" role="tablist">
        <button
          role="tab"
          aria-selected={tab === "requests"}
          onClick={() => setTab("requests")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "requests" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Requests & Recommendations
        </button>
        <button
          role="tab"
          aria-selected={tab === "orders"}
          onClick={() => setTab("orders")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "orders" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Purchase Orders
        </button>
        <button
          role="tab"
          aria-selected={tab === "vendors"}
          onClick={() => setTab("vendors")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "vendors" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Vendors
        </button>
        <button
          role="tab"
          aria-selected={tab === "budgets"}
          onClick={() => setTab("budgets")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === "budgets" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Budgets
        </button>
      </div>

      {tab === "requests" && <RequestsTab writable={writable} currency={currency} />}
      {tab === "orders" && <OrdersTab writable={writable} currency={currency} />}
      {tab === "vendors" && <VendorsTab writable={writable} />}
      {tab === "budgets" && <BudgetsTab writable={writable} currency={currency} />}
    </div>
  );
}

// ---------------------------------------------------------------------------
// REQUESTS TAB
// ---------------------------------------------------------------------------

function RequestsTab({ writable, currency }: { writable: boolean; currency: string }) {
  const [statusFilter, setStatusFilter] = useState<string>("");
  const [newModalOpen, setNewModalOpen] = useState(false);
  const [approvingReq, setApprovingReq] = useState<AcquisitionRequest | null>(null);
  const [rejectingReq, setRejectingReq] = useState<AcquisitionRequest | null>(null);

  const query = useQuery({
    queryKey: ["acquisitions-requests", statusFilter],
    queryFn: () => acquisitionsApi.listRequests(statusFilter || null),
  });

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <label htmlFor="request-status-filter" className="text-xs font-medium text-muted">
            Status:
          </label>
          <select
            id="request-status-filter"
            aria-label="Status filter"
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="h-8 rounded border border-field-line bg-surface px-2 text-xs text-fg"
          >
            <option value="">All statuses</option>
            <option value="pending">Pending review</option>
            <option value="approved">Approved</option>
            <option value="ordered">Ordered</option>
            <option value="rejected">Rejected</option>
          </select>
        </div>
        {writable && (
          <Button variant="primary" size="sm" onClick={() => setNewModalOpen(true)}>
            New Request
          </Button>
        )}
      </div>

      <Panel>
        {query.isPending ? (
          <p className="p-6 text-sm text-muted">Loading requests…</p>
        ) : query.isError ? (
          <p className="p-6 text-sm text-danger">{toApiError(query.error).message}</p>
        ) : query.data.length === 0 ? (
          <p className="p-6 text-sm text-muted">No requests found.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Acquisition requests">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Title & Author</th>
                  <th className="px-4 py-2.5">Requested By</th>
                  <th className="px-4 py-2.5">Est. Price</th>
                  <th className="px-4 py-2.5">Copies</th>
                  <th className="px-4 py-2.5">Budget</th>
                  <th className="px-4 py-2.5">Status</th>
                  {writable && <th className="px-4 py-2.5 text-right">Actions</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {query.data.map((r) => (
                  <tr key={r.id} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5">
                      <div className="font-semibold text-fg">{r.title}</div>
                      {r.author && <div className="text-muted">{r.author}</div>}
                      {r.isbn && <div className="text-muted text-[11px]">ISBN: {r.isbn}</div>}
                    </td>
                    <td className="px-4 py-2.5">
                      <div>{r.requestedBy}</div>
                      {r.requestReason && <div className="text-muted italic">{r.requestReason}</div>}
                    </td>
                    <td className="px-4 py-2.5">
                      {r.estimatedPriceMinor != null ? formatMinor(r.estimatedPriceMinor, r.currency || currency) : "—"}
                    </td>
                    <td className="px-4 py-2.5">{r.copies}</td>
                    <td className="px-4 py-2.5">{r.budgetName || "—"}</td>
                    <td className="px-4 py-2.5">
                      <RequestStatusBadge status={r.status} />
                      {r.rejectionReason && <p className="text-[11px] text-danger mt-0.5">{r.rejectionReason}</p>}
                    </td>
                    {writable && (
                      <td className="px-4 py-2.5 text-right space-x-1">
                        {r.status === "pending" && (
                          <>
                            <Button size="sm" onClick={() => setApprovingReq(r)}>
                              Approve
                            </Button>
                            <Button size="sm" variant="secondary" className="text-danger border-danger hover:bg-danger/10" onClick={() => setRejectingReq(r)}>
                              Reject
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
      </Panel>

      {newModalOpen && <NewRequestModal onClose={() => setNewModalOpen(false)} currency={currency} />}
      {approvingReq && <ApproveRequestModal request={approvingReq} onClose={() => setApprovingReq(null)} />}
      {rejectingReq && <RejectRequestModal request={rejectingReq} onClose={() => setRejectingReq(null)} />}
    </div>
  );
}

function RequestStatusBadge({ status }: { status: string }) {
  switch (status) {
    case "pending":
      return <Chip tone="fine">Pending</Chip>;
    case "approved":
      return <span className="inline-block rounded px-2 py-0.5 text-xs font-medium bg-primary/10 text-primary border border-primary/20">Approved</span>;
    case "ordered":
      return <span className="inline-block rounded px-2 py-0.5 text-xs font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/20">Ordered</span>;
    case "rejected":
      return <Chip tone="overdue">Rejected</Chip>;
    default:
      return <Chip tone="neutral">{status}</Chip>;
  }
}

function NewRequestModal({ onClose, currency }: { onClose: () => void; currency: string }) {
  const qc = useQueryClient();
  const [title, setTitle] = useState("");
  const [author, setAuthor] = useState("");
  const [isbn, setIsbn] = useState("");
  const [publisher, setPublisher] = useState("");
  const [year, setYear] = useState("");
  const [price, setPrice] = useState("");
  const [copies, setCopies] = useState("1");
  const [requestedBy, setRequestedBy] = useState("");
  const [reason, setReason] = useState("");

  const create = useMutation({
    mutationFn: (input: CreateRequestInput) => acquisitionsApi.createRequest(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-requests"] });
      onClose();
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!title.trim() || !requestedBy.trim()) return;
    const estMinor = price ? Math.round(parseFloat(price) * 100) : null;
    create.mutate({
      title: title.trim(),
      author: author.trim() || null,
      isbn: isbn.trim() || null,
      publisher: publisher.trim() || null,
      publicationYear: year ? parseInt(year, 10) : null,
      estimatedPriceMinor: estMinor,
      currency,
      copies: parseInt(copies, 10) || 1,
      requestedBy: requestedBy.trim(),
      requestReason: reason.trim() || null,
    });
  };

  return (
    <Modal title="New Acquisition Request" onClose={onClose} wide>
      <form onSubmit={onSubmit} className="space-y-4">
        <TextField label="Title" value={title} onChange={(e) => setTitle(e.target.value)} required />
        <div className="grid grid-cols-2 gap-4">
          <TextField label="Author" value={author} onChange={(e) => setAuthor(e.target.value)} />
          <TextField label="ISBN" value={isbn} onChange={(e) => setIsbn(e.target.value)} />
        </div>
        <div className="grid grid-cols-2 gap-4">
          <TextField label="Publisher" value={publisher} onChange={(e) => setPublisher(e.target.value)} />
          <TextField label="Publication Year" type="number" value={year} onChange={(e) => setYear(e.target.value)} />
        </div>
        <div className="grid grid-cols-2 gap-4">
          <TextField
            label={`Estimated Unit Price (${currency})`}
            type="number"
            step="0.01"
            value={price}
            onChange={(e) => setPrice(e.target.value)}
          />
          <TextField label="Copies" type="number" min="1" value={copies} onChange={(e) => setCopies(e.target.value)} required />
        </div>
        <TextField
          label="Requested By (Patron / Department)"
          value={requestedBy}
          onChange={(e) => setRequestedBy(e.target.value)}
          required
        />
        <div>
          <label className="block text-[13px] font-medium mb-1">Reason / Notes</label>
          <textarea
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            rows={2}
            className="w-full rounded-md border border-field-line bg-surface p-2 text-xs text-fg"
          />
        </div>

        {create.isError && <p className="text-xs text-danger">{toApiError(create.error).message}</p>}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={create.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={create.isPending}>
            {create.isPending ? "Submitting…" : "Submit Request"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function ApproveRequestModal({ request, onClose }: { request: AcquisitionRequest; onClose: () => void }) {
  const qc = useQueryClient();
  const [budgetId, setBudgetId] = useState<string>("");

  const budgetsQuery = useQuery({
    queryKey: ["acquisitions-budgets", false],
    queryFn: () => acquisitionsApi.listBudgets(false),
  });

  const approve = useMutation({
    mutationFn: () => acquisitionsApi.approveRequest(request.id, budgetId ? Number(budgetId) : null),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-requests"] });
      onClose();
    },
  });

  return (
    <Modal title={`Approve Request: ${request.title}`} onClose={onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          approve.mutate();
        }}
        className="space-y-4"
      >
        <p className="text-xs text-muted">
          Approving marks this request ready for purchase order compilation. You can optionally link it to a budget allocation.
        </p>

        <SelectField
          label="Assign Budget (Optional)"
          value={budgetId}
          onChange={(e) => setBudgetId(e.target.value)}
          disabled={approve.isPending}
        >
          <option value="">No budget assigned</option>
          {budgetsQuery.data?.map((b) => (
            <option key={b.id} value={b.id}>
              {b.name} ({b.currency} {formatMinor(b.amountMinor - b.allocatedMinor, b.currency)} available)
            </option>
          ))}
        </SelectField>

        {approve.isError && <p className="text-xs text-danger">{toApiError(approve.error).message}</p>}

        <div className="flex justify-end gap-2 pt-2">
          <Button onClick={onClose} disabled={approve.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={approve.isPending}>
            {approve.isPending ? "Approving…" : "Approve Request"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function RejectRequestModal({ request, onClose }: { request: AcquisitionRequest; onClose: () => void }) {
  const qc = useQueryClient();
  const [reason, setReason] = useState("");

  const reject = useMutation({
    mutationFn: () => acquisitionsApi.rejectRequest(request.id, reason.trim() || null),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-requests"] });
      onClose();
    },
  });

  return (
    <Modal title={`Reject Request: ${request.title}`} onClose={onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          reject.mutate();
        }}
        className="space-y-4"
      >
        <div>
          <label className="block text-[13px] font-medium mb-1">Reason for Rejection</label>
          <textarea
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            rows={3}
            placeholder="e.g. Already available in collection, out of print, or exceeds budget."
            className="w-full rounded-md border border-field-line bg-surface p-2 text-xs text-fg"
          />
        </div>

        {reject.isError && <p className="text-xs text-danger">{toApiError(reject.error).message}</p>}

        <div className="flex justify-end gap-2 pt-2">
          <Button onClick={onClose} disabled={reject.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="secondary" className="text-danger border-danger hover:bg-danger/10" disabled={reject.isPending}>
            {reject.isPending ? "Rejecting…" : "Reject Request"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

// ---------------------------------------------------------------------------
// PURCHASE ORDERS TAB
// ---------------------------------------------------------------------------

function OrdersTab({ writable, currency }: { writable: boolean; currency: string }) {
  const [statusFilter, setStatusFilter] = useState<string>("");
  const [newOrderOpen, setNewOrderOpen] = useState(false);
  const [selectedOrderId, setSelectedOrderId] = useState<number | null>(null);

  const query = useQuery({
    queryKey: ["acquisitions-orders", statusFilter],
    queryFn: () => acquisitionsApi.listOrders(statusFilter || null),
  });

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <label htmlFor="order-status-filter" className="text-xs font-medium text-muted">
            Status:
          </label>
          <select
            id="order-status-filter"
            aria-label="Order status filter"
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="h-8 rounded border border-field-line bg-surface px-2 text-xs text-fg"
          >
            <option value="">All statuses</option>
            <option value="draft">Draft</option>
            <option value="ordered">Ordered</option>
            <option value="partially_received">Partially received</option>
            <option value="received">Received</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </div>
        {writable && (
          <Button variant="primary" size="sm" onClick={() => setNewOrderOpen(true)}>
            New Purchase Order
          </Button>
        )}
      </div>

      <Panel>
        {query.isPending ? (
          <p className="p-6 text-sm text-muted">Loading purchase orders…</p>
        ) : query.isError ? (
          <p className="p-6 text-sm text-danger">{toApiError(query.error).message}</p>
        ) : query.data.length === 0 ? (
          <p className="p-6 text-sm text-muted">No purchase orders found.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Purchase orders">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">PO Number</th>
                  <th className="px-4 py-2.5">Vendor</th>
                  <th className="px-4 py-2.5">Budget</th>
                  <th className="px-4 py-2.5">Items</th>
                  <th className="px-4 py-2.5">Total Amount</th>
                  <th className="px-4 py-2.5">Status</th>
                  <th className="px-4 py-2.5 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {query.data.map((o) => (
                  <tr key={o.id} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5 font-semibold text-fg">{o.orderNumber}</td>
                    <td className="px-4 py-2.5">{o.vendorName}</td>
                    <td className="px-4 py-2.5">{o.budgetName || "—"}</td>
                    <td className="px-4 py-2.5">{o.itemsCount} items</td>
                    <td className="px-4 py-2.5 font-medium">{formatMinor(o.totalMinor, currency)}</td>
                    <td className="px-4 py-2.5">
                      <OrderStatusBadge status={o.status} />
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <Button size="sm" onClick={() => setSelectedOrderId(o.id)}>
                        View Details
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Panel>

      {newOrderOpen && <NewOrderModal onClose={() => setNewOrderOpen(false)} currency={currency} />}
      {selectedOrderId != null && (
        <OrderDetailModal
          orderId={selectedOrderId}
          writable={writable}
          currency={currency}
          onClose={() => setSelectedOrderId(null)}
        />
      )}
    </div>
  );
}

function OrderStatusBadge({ status }: { status: string }) {
  switch (status) {
    case "draft":
      return <Chip tone="neutral">Draft</Chip>;
    case "ordered":
      return <span className="inline-block rounded px-2 py-0.5 text-xs font-medium bg-primary/10 text-primary border border-primary/20">Ordered</span>;
    case "partially_received":
      return <Chip tone="fine">Partial</Chip>;
    case "received":
      return <span className="inline-block rounded px-2 py-0.5 text-xs font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/20">Received</span>;
    case "cancelled":
      return <Chip tone="overdue">Cancelled</Chip>;
    default:
      return <Chip tone="neutral">{status}</Chip>;
  }
}

function NewOrderModal({ onClose, currency }: { onClose: () => void; currency: string }) {
  const qc = useQueryClient();
  const [orderNumber, setOrderNumber] = useState(`PO-${new Date().getFullYear()}-${String(Date.now()).slice(-4)}`);
  const [vendorId, setVendorId] = useState("");
  const [budgetId, setBudgetId] = useState("");
  const [invoiceRef, setInvoiceRef] = useState("");
  const [expectedAt, setExpectedAt] = useState("");
  const [notes, setNotes] = useState("");

  const [items, setItems] = useState<
    Array<{
      requestId?: number | null;
      title: string;
      author: string;
      isbn: string;
      quantity: number;
      unitPriceMinor: number;
      notes: string;
    }>
  >([
    {
      title: "",
      author: "",
      isbn: "",
      quantity: 1,
      unitPriceMinor: 0,
      notes: "",
    },
  ]);

  const vendorsQuery = useQuery({
    queryKey: ["acquisitions-vendors", false],
    queryFn: () => acquisitionsApi.listVendors(false),
  });

  const budgetsQuery = useQuery({
    queryKey: ["acquisitions-budgets", false],
    queryFn: () => acquisitionsApi.listBudgets(false),
  });

  const approvedReqs = useQuery({
    queryKey: ["acquisitions-requests", "approved"],
    queryFn: () => acquisitionsApi.listRequests("approved"),
  });

  const create = useMutation({
    mutationFn: acquisitionsApi.createOrder,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-orders"] });
      qc.invalidateQueries({ queryKey: ["acquisitions-requests"] });
      qc.invalidateQueries({ queryKey: ["acquisitions-budgets"] });
      onClose();
    },
  });

  const addItemFromRequest = (req: AcquisitionRequest) => {
    setItems((prev) => [
      ...prev.filter((i) => i.title.trim().length > 0),
      {
        requestId: req.id,
        title: req.title,
        author: req.author || "",
        isbn: req.isbn || "",
        quantity: req.copies,
        unitPriceMinor: req.estimatedPriceMinor || 0,
        notes: req.requestReason || "",
      },
    ]);
  };

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!orderNumber.trim() || !vendorId) return;
    const validItems = items.filter((i) => i.title.trim().length > 0);
    if (validItems.length === 0) return;

    create.mutate({
      orderNumber: orderNumber.trim(),
      vendorId: Number(vendorId),
      budgetId: budgetId ? Number(budgetId) : null,
      invoiceRef: invoiceRef.trim() || null,
      expectedAt: expectedAt.trim() || null,
      notes: notes.trim() || null,
      items: validItems.map((i) => ({
        requestId: i.requestId || null,
        title: i.title.trim(),
        author: i.author.trim() || null,
        isbn: i.isbn.trim() || null,
        quantity: i.quantity,
        unitPriceMinor: i.unitPriceMinor,
        notes: i.notes.trim() || null,
      })),
    });
  };

  const orderTotal = items.reduce((acc, i) => acc + i.quantity * i.unitPriceMinor, 0);

  return (
    <Modal title="Create Purchase Order" onClose={onClose} wide>
      <form onSubmit={onSubmit} className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <TextField
            label="PO Number"
            value={orderNumber}
            onChange={(e) => setOrderNumber(e.target.value)}
            required
          />
          <SelectField
            label="Vendor"
            value={vendorId}
            onChange={(e) => setVendorId(e.target.value)}
            required
          >
            <option value="">Select Vendor</option>
            {vendorsQuery.data?.map((v) => (
              <option key={v.id} value={v.id}>
                {v.name}
              </option>
            ))}
          </SelectField>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <SelectField
            label="Budget"
            value={budgetId}
            onChange={(e) => setBudgetId(e.target.value)}
          >
            <option value="">No budget</option>
            {budgetsQuery.data?.map((b) => (
              <option key={b.id} value={b.id}>
                {b.name} ({formatMinor(b.amountMinor - b.allocatedMinor, b.currency)} free)
              </option>
            ))}
          </SelectField>
          <TextField
            label="Expected Date"
            type="date"
            value={expectedAt}
            onChange={(e) => setExpectedAt(e.target.value)}
          />
          <TextField
            label="Invoice Reference"
            value={invoiceRef}
            onChange={(e) => setInvoiceRef(e.target.value)}
          />
        </div>

        <TextField
          label="Notes (Optional)"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
        />

        {approvedReqs.data && approvedReqs.data.length > 0 && (
          <div className="rounded border border-line bg-surface-alt p-3">
            <p className="text-xs font-semibold mb-2 text-fg">Approved requests available to order:</p>
            <div className="flex flex-wrap gap-2">
              {approvedReqs.data.map((req) => (
                <button
                  key={req.id}
                  type="button"
                  onClick={() => addItemFromRequest(req)}
                  className="rounded border border-field-line bg-surface px-2 py-1 text-xs hover:border-primary text-left"
                >
                  + {req.title} ({req.copies} copies)
                </button>
              ))}
            </div>
          </div>
        )}

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-fg">Order Items</span>
            <Button
              type="button"
              size="sm"
              onClick={() =>
                setItems((prev) => [
                  ...prev,
                  { title: "", author: "", isbn: "", quantity: 1, unitPriceMinor: 0, notes: "" },
                ])
              }
            >
              Add Item
            </Button>
          </div>

          <div className="space-y-2 max-h-56 overflow-y-auto pr-1">
            {items.map((item, idx) => (
              <div key={idx} className="grid grid-cols-12 gap-2 items-center bg-surface-alt/40 p-2 rounded">
                <input
                  placeholder="Book Title"
                  value={item.title}
                  onChange={(e) => {
                    const val = e.target.value;
                    setItems((prev) => prev.map((x, i) => (i === idx ? { ...x, title: val } : x)));
                  }}
                  className="col-span-4 h-8 rounded border border-field-line bg-surface px-2 text-xs"
                  required
                />
                <input
                  placeholder="Author"
                  value={item.author}
                  onChange={(e) => {
                    const val = e.target.value;
                    setItems((prev) => prev.map((x, i) => (i === idx ? { ...x, author: val } : x)));
                  }}
                  className="col-span-2 h-8 rounded border border-field-line bg-surface px-2 text-xs"
                />
                <input
                  placeholder="Qty"
                  type="number"
                  min="1"
                  value={item.quantity}
                  onChange={(e) => {
                    const val = parseInt(e.target.value, 10) || 1;
                    setItems((prev) => prev.map((x, i) => (i === idx ? { ...x, quantity: val } : x)));
                  }}
                  className="col-span-2 h-8 rounded border border-field-line bg-surface px-2 text-xs"
                />
                <input
                  placeholder={`Unit Price (${currency})`}
                  type="number"
                  step="0.01"
                  value={item.unitPriceMinor ? item.unitPriceMinor / 100 : ""}
                  onChange={(e) => {
                    const val = Math.round((parseFloat(e.target.value) || 0) * 100);
                    setItems((prev) => prev.map((x, i) => (i === idx ? { ...x, unitPriceMinor: val } : x)));
                  }}
                  className="col-span-3 h-8 rounded border border-field-line bg-surface px-2 text-xs"
                />
                <button
                  type="button"
                  onClick={() => setItems((prev) => prev.filter((_, i) => i !== idx))}
                  className="col-span-1 text-danger hover:text-danger/80 text-center font-bold"
                  title="Remove item"
                >
                  ×
                </button>
              </div>
            ))}
          </div>

          <div className="text-right text-xs font-semibold pt-2 text-fg">
            Order Total: {formatMinor(orderTotal, currency)}
          </div>
        </div>

        {create.isError && <p className="text-xs text-danger">{toApiError(create.error).message}</p>}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={create.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={create.isPending}>
            {create.isPending ? "Creating…" : "Save Purchase Order"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function OrderDetailModal({
  orderId,
  writable,
  currency,
  onClose,
}: {
  orderId: number;
  writable: boolean;
  currency: string;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [invoiceRefInput, setInvoiceRefInput] = useState("");
  const [receivingItem, setReceivingItem] = useState<PurchaseOrderItem | null>(null);

  const query = useQuery({
    queryKey: ["acquisitions-order-detail", orderId],
    queryFn: () => acquisitionsApi.getOrder(orderId),
  });

  const updateStatus = useMutation({
    mutationFn: ({ status, invoiceRef }: { status: string; invoiceRef?: string | null }) =>
      acquisitionsApi.updateOrderStatus(orderId, status, invoiceRef),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-orders"] });
      qc.invalidateQueries({ queryKey: ["acquisitions-order-detail", orderId] });
      qc.invalidateQueries({ queryKey: ["acquisitions-budgets"] });
    },
  });

  const order = query.data;
  const canReceive = writable && order && (order.status === "ordered" || order.status === "partially_received");

  return (
    <Modal title={`Purchase Order: ${order?.orderNumber || "…"}`} onClose={onClose} wide>
      {query.isPending ? (
        <p className="p-4 text-xs text-muted">Loading order details…</p>
      ) : query.isError ? (
        <p className="p-4 text-xs text-danger">{toApiError(query.error).message}</p>
      ) : !order ? null : (
        <div className="space-y-4 text-xs">
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 p-3 bg-surface-alt rounded">
            <div>
              <span className="text-muted block">Vendor</span>
              <span className="font-semibold">{order.vendorName}</span>
            </div>
            <div>
              <span className="text-muted block">Budget</span>
              <span>{order.budgetName || "None"}</span>
            </div>
            <div>
              <span className="text-muted block">Status</span>
              <OrderStatusBadge status={order.status} />
            </div>
            <div>
              <span className="text-muted block">Invoice Ref</span>
              <span>{order.invoiceRef || "None"}</span>
            </div>
          </div>

          <div>
            <span className="font-semibold block mb-2">Line Items</span>
            <table className="w-full text-left" aria-label="Purchase order items">
              <thead className="border-b border-line bg-surface-alt text-muted">
                <tr>
                  <th className="px-3 py-1.5">Title</th>
                  <th className="px-3 py-1.5">Author</th>
                  <th className="px-3 py-1.5">Ordered</th>
                  <th className="px-3 py-1.5">Received</th>
                  <th className="px-3 py-1.5">Unit Price</th>
                  <th className="px-3 py-1.5 text-right">Subtotal</th>
                  {canReceive && <th className="px-3 py-1.5 text-right">Action</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {order.items.map((item) => (
                  <tr key={item.id}>
                    <td className="px-3 py-1.5 font-medium">{item.title}</td>
                    <td className="px-3 py-1.5 text-muted">{item.author || "—"}</td>
                    <td className="px-3 py-1.5">{item.quantity}</td>
                    <td className="px-3 py-1.5">
                      {item.receivedQuantity} / {item.quantity}
                      {item.receivedQuantity >= item.quantity && <span className="ml-1 text-emerald-600 font-semibold">✓</span>}
                    </td>
                    <td className="px-3 py-1.5">{formatMinor(item.unitPriceMinor, currency)}</td>
                    <td className="px-3 py-1.5 text-right font-medium">
                      {formatMinor(item.quantity * item.unitPriceMinor, currency)}
                    </td>
                    {canReceive && (
                      <td className="px-3 py-1.5 text-right">
                        {item.quantity > item.receivedQuantity ? (
                          <Button size="sm" onClick={() => setReceivingItem(item)}>
                            Receive Stock
                          </Button>
                        ) : (
                          <span className="text-muted text-[11px]">Received</span>
                        )}
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
              <tfoot className="border-t border-line font-bold">
                <tr>
                  <td colSpan={canReceive ? 5 : 4} className="px-3 py-2 text-right">
                    Total:
                  </td>
                  <td className="px-3 py-2 text-right">{formatMinor(order.totalMinor, currency)}</td>
                  {canReceive && <td />}
                </tr>
              </tfoot>
            </table>
          </div>

          {writable && order.status === "draft" && (
            <div className="flex items-center gap-2 pt-2 border-t border-line">
              <input
                placeholder="Invoice reference (optional)"
                value={invoiceRefInput}
                onChange={(e) => setInvoiceRefInput(e.target.value)}
                className="h-8 rounded border border-field-line bg-surface px-2 text-xs flex-1"
              />
              <Button
                variant="primary"
                size="sm"
                disabled={updateStatus.isPending}
                onClick={() =>
                  updateStatus.mutate({ status: "ordered", invoiceRef: invoiceRefInput || undefined })
                }
              >
                Place Order
              </Button>
              <Button
                variant="secondary"
                className="text-danger border-danger hover:bg-danger/10"
                size="sm"
                disabled={updateStatus.isPending}
                onClick={() => updateStatus.mutate({ status: "cancelled" })}
              >
                Cancel PO
              </Button>
            </div>
          )}

          {updateStatus.isError && (
            <p className="text-xs text-danger">{toApiError(updateStatus.error).message}</p>
          )}

          <div className="flex justify-end pt-2">
            <Button onClick={onClose}>Close</Button>
          </div>
        </div>
      )}

      {receivingItem && (
        <ReceiveStockModal
          orderId={orderId}
          item={receivingItem}
          onClose={() => setReceivingItem(null)}
        />
      )}
    </Modal>
  );
}

function ReceiveStockModal({
  orderId,
  item,
  onClose,
}: {
  orderId: number;
  item: PurchaseOrderItem;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const remaining = item.quantity - item.receivedQuantity;
  const [quantity, setQuantity] = useState(String(remaining));
  const [categoryId, setCategoryId] = useState("");
  const [locationId, setLocationId] = useState("");
  const [result, setResult] = useState<ReceiveResult | null>(null);

  const config = useQuery({ queryKey: ["config-overview"], queryFn: configApi.overview });

  const receive = useMutation({
    mutationFn: () =>
      acquisitionsApi.receiveItems({
        orderId,
        itemId: item.id,
        quantity: parseInt(quantity, 10) || 1,
        itemCategoryId: categoryId ? Number(categoryId) : null,
        locationId: locationId ? Number(locationId) : null,
      }),
    onSuccess: (data) => {
      setResult(data);
      qc.invalidateQueries({ queryKey: ["acquisitions-orders"] });
      qc.invalidateQueries({ queryKey: ["acquisitions-order-detail", orderId] });
      qc.invalidateQueries({ queryKey: ["acquisitions-requests"] });
    },
  });

  return (
    <Modal title={`Receive Stock: ${item.title}`} onClose={onClose}>
      {result ? (
        <div className="space-y-4 text-xs">
          <div className="p-3 bg-emerald-500/10 text-emerald-700 rounded border border-emerald-500/20">
            <p className="font-semibold">Successfully received {result.createdCopies.length} copies!</p>
            <p className="text-muted mt-1">Generated Accession Numbers & Barcodes:</p>
            <ul className="list-disc pl-5 mt-1 font-mono">
              {result.createdCopies.map((c) => (
                <li key={c.copyId}>{c.accessionNo}</li>
              ))}
            </ul>
          </div>
          <div className="flex justify-end pt-2">
            <Button onClick={onClose} variant="primary">
              Done
            </Button>
          </div>
        </div>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            receive.mutate();
          }}
          className="space-y-4"
        >
          <div className="p-2 bg-surface-alt rounded text-xs">
            <span className="text-muted block">Total Ordered: {item.quantity} · Already Received: {item.receivedQuantity}</span>
            <span className="font-semibold block text-fg">Remaining to Receive: {remaining}</span>
          </div>

          <TextField
            label="Quantity to Receive"
            type="number"
            min="1"
            max={remaining}
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            required
          />

          <SelectField
            label="Item Category"
            value={categoryId}
            onChange={(e) => setCategoryId(e.target.value)}
          >
            <option value="">Default item category</option>
            {config.data?.itemCategories?.filter((c) => !c.archived).map((c) => (
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
            {config.data?.locations?.filter((l) => !l.archived).map((l) => (
              <option key={l.id} value={l.id}>
                {l.name}
              </option>
            ))}
          </SelectField>

          {receive.isError && (
            <p className="text-xs text-danger">{toApiError(receive.error).message}</p>
          )}

          <div className="flex justify-end gap-2 pt-2 border-t border-line">
            <Button onClick={onClose} disabled={receive.isPending}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" disabled={receive.isPending}>
              {receive.isPending ? "Receiving…" : "Confirm Receipt"}
            </Button>
          </div>
        </form>
      )}
    </Modal>
  );
}

// ---------------------------------------------------------------------------
// VENDORS TAB
// ---------------------------------------------------------------------------

function VendorsTab({ writable }: { writable: boolean }) {
  const qc = useQueryClient();
  const [includeArchived, setIncludeArchived] = useState(false);
  const [editingVendor, setEditingVendor] = useState<VendorInput | null>(null);

  const query = useQuery({
    queryKey: ["acquisitions-vendors", includeArchived],
    queryFn: () => acquisitionsApi.listVendors(includeArchived),
  });

  const archive = useMutation({
    mutationFn: ({ id, arch }: { id: number; arch: boolean }) =>
      acquisitionsApi.archiveVendor(id, arch),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["acquisitions-vendors"] }),
  });

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <label className="flex items-center gap-2 text-xs text-muted">
          <input
            type="checkbox"
            checked={includeArchived}
            onChange={(e) => setIncludeArchived(e.target.checked)}
            className="rounded border-field-line"
          />
          Show archived vendors
        </label>
        {writable && (
          <Button
            variant="primary"
            size="sm"
            onClick={() =>
              setEditingVendor({
                name: "",
                code: "",
                contactName: "",
                email: "",
                phone: "",
                address: "",
                notes: "",
              })
            }
          >
            Add Vendor
          </Button>
        )}
      </div>

      <Panel>
        {query.isPending ? (
          <p className="p-6 text-sm text-muted">Loading vendors…</p>
        ) : query.isError ? (
          <p className="p-6 text-sm text-danger">{toApiError(query.error).message}</p>
        ) : query.data.length === 0 ? (
          <p className="p-6 text-sm text-muted">No vendors registered.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Vendors">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Name</th>
                  <th className="px-4 py-2.5">Code</th>
                  <th className="px-4 py-2.5">Contact</th>
                  <th className="px-4 py-2.5">Email</th>
                  <th className="px-4 py-2.5">Phone</th>
                  <th className="px-4 py-2.5">Status</th>
                  {writable && <th className="px-4 py-2.5 text-right">Actions</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {query.data.map((v) => (
                  <tr key={v.id} className="hover:bg-surface-alt/50">
                    <td className="px-4 py-2.5 font-semibold text-fg">{v.name}</td>
                    <td className="px-4 py-2.5">{v.code || "—"}</td>
                    <td className="px-4 py-2.5">{v.contactName || "—"}</td>
                    <td className="px-4 py-2.5">{v.email || "—"}</td>
                    <td className="px-4 py-2.5">{v.phone || "—"}</td>
                    <td className="px-4 py-2.5">
                      {v.archived ? <Chip tone="overdue">Archived</Chip> : <Chip tone="neutral">Active</Chip>}
                    </td>
                    {writable && (
                      <td className="px-4 py-2.5 text-right space-x-1">
                        <Button
                          size="sm"
                          onClick={() =>
                            setEditingVendor({
                              id: v.id,
                              name: v.name,
                              code: v.code || "",
                              contactName: v.contactName || "",
                              email: v.email || "",
                              phone: v.phone || "",
                              address: v.address || "",
                              notes: v.notes || "",
                            })
                          }
                        >
                          Edit
                        </Button>
                        <Button
                          size="sm"
                          onClick={() => archive.mutate({ id: v.id, arch: !v.archived })}
                        >
                          {v.archived ? "Restore" : "Archive"}
                        </Button>
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Panel>

      {editingVendor && (
        <VendorModal vendor={editingVendor} onClose={() => setEditingVendor(null)} />
      )}
    </div>
  );
}

function VendorModal({ vendor, onClose }: { vendor: VendorInput; onClose: () => void }) {
  const qc = useQueryClient();
  const [name, setName] = useState(vendor.name || "");
  const [code, setCode] = useState(vendor.code || "");
  const [contactName, setContactName] = useState(vendor.contactName || "");
  const [email, setEmail] = useState(vendor.email || "");
  const [phone, setPhone] = useState(vendor.phone || "");
  const [address, setAddress] = useState(vendor.address || "");
  const [notes, setNotes] = useState(vendor.notes || "");

  const save = useMutation({
    mutationFn: () =>
      acquisitionsApi.saveVendor({
        id: vendor.id,
        name: name.trim(),
        code: code.trim() || null,
        contactName: contactName.trim() || null,
        email: email.trim() || null,
        phone: phone.trim() || null,
        address: address.trim() || null,
        notes: notes.trim() || null,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-vendors"] });
      onClose();
    },
  });

  return (
    <Modal title={vendor.id ? "Edit Vendor" : "Add Vendor"} onClose={onClose} wide>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!name.trim()) return;
          save.mutate();
        }}
        className="space-y-4"
      >
        <div className="grid grid-cols-2 gap-4">
          <TextField label="Vendor Name" value={name} onChange={(e) => setName(e.target.value)} required />
          <TextField label="Vendor Code (Short)" value={code} onChange={(e) => setCode(e.target.value)} />
        </div>
        <div className="grid grid-cols-3 gap-4">
          <TextField label="Contact Person" value={contactName} onChange={(e) => setContactName(e.target.value)} />
          <TextField label="Email" type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
          <TextField label="Phone" value={phone} onChange={(e) => setPhone(e.target.value)} />
        </div>
        <TextField label="Address" value={address} onChange={(e) => setAddress(e.target.value)} />
        <div>
          <label className="block text-[13px] font-medium mb-1">Notes</label>
          <textarea
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            rows={2}
            className="w-full rounded-md border border-field-line bg-surface p-2 text-xs text-fg"
          />
        </div>

        {save.isError && <p className="text-xs text-danger">{toApiError(save.error).message}</p>}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={save.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={save.isPending}>
            {save.isPending ? "Saving…" : "Save Vendor"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

// ---------------------------------------------------------------------------
// BUDGETS TAB
// ---------------------------------------------------------------------------

function BudgetsTab({ writable, currency }: { writable: boolean; currency: string }) {
  const qc = useQueryClient();
  const [includeArchived, setIncludeArchived] = useState(false);
  const [editingBudget, setEditingBudget] = useState<BudgetInput | null>(null);

  const query = useQuery({
    queryKey: ["acquisitions-budgets", includeArchived],
    queryFn: () => acquisitionsApi.listBudgets(includeArchived),
  });

  const archive = useMutation({
    mutationFn: ({ id, arch }: { id: number; arch: boolean }) =>
      acquisitionsApi.archiveBudget(id, arch),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["acquisitions-budgets"] }),
  });

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <label className="flex items-center gap-2 text-xs text-muted">
          <input
            type="checkbox"
            checked={includeArchived}
            onChange={(e) => setIncludeArchived(e.target.checked)}
            className="rounded border-field-line"
          />
          Show archived budgets
        </label>
        {writable && (
          <Button
            variant="primary"
            size="sm"
            onClick={() =>
              setEditingBudget({
                name: "",
                code: "",
                amountMinor: 0,
                currency,
                fiscalYear: `${new Date().getFullYear()}-${new Date().getFullYear() + 1}`,
              })
            }
          >
            Add Budget
          </Button>
        )}
      </div>

      <Panel>
        {query.isPending ? (
          <p className="p-6 text-sm text-muted">Loading budgets…</p>
        ) : query.isError ? (
          <p className="p-6 text-sm text-danger">{toApiError(query.error).message}</p>
        ) : query.data.length === 0 ? (
          <p className="p-6 text-sm text-muted">No acquisition budgets set up.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs" aria-label="Acquisition budgets">
              <thead className="border-b border-line bg-surface-alt font-medium text-muted">
                <tr>
                  <th className="px-4 py-2.5">Name</th>
                  <th className="px-4 py-2.5">Code</th>
                  <th className="px-4 py-2.5">Fiscal Year</th>
                  <th className="px-4 py-2.5">Total Amount</th>
                  <th className="px-4 py-2.5">Allocated</th>
                  <th className="px-4 py-2.5">Remaining</th>
                  <th className="px-4 py-2.5">Status</th>
                  {writable && <th className="px-4 py-2.5 text-right">Actions</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-line">
                {query.data.map((b) => {
                  const rem = b.amountMinor - b.allocatedMinor;
                  return (
                    <tr key={b.id} className="hover:bg-surface-alt/50">
                      <td className="px-4 py-2.5 font-semibold text-fg">{b.name}</td>
                      <td className="px-4 py-2.5">{b.code || "—"}</td>
                      <td className="px-4 py-2.5">{b.fiscalYear || "—"}</td>
                      <td className="px-4 py-2.5">{formatMinor(b.amountMinor, b.currency)}</td>
                      <td className="px-4 py-2.5">{formatMinor(b.allocatedMinor, b.currency)}</td>
                      <td className={`px-4 py-2.5 font-medium ${rem < 0 ? "text-danger" : "text-fg"}`}>
                        {formatMinor(rem, b.currency)}
                      </td>
                      <td className="px-4 py-2.5">
                        {b.archived ? <Chip tone="overdue">Archived</Chip> : <Chip tone="neutral">Active</Chip>}
                      </td>
                      {writable && (
                        <td className="px-4 py-2.5 text-right space-x-1">
                          <Button
                            size="sm"
                            onClick={() =>
                              setEditingBudget({
                                id: b.id,
                                name: b.name,
                                code: b.code || "",
                                amountMinor: b.amountMinor,
                                currency: b.currency,
                                fiscalYear: b.fiscalYear || "",
                              })
                            }
                          >
                            Edit
                          </Button>
                          <Button
                            size="sm"
                            onClick={() => archive.mutate({ id: b.id, arch: !b.archived })}
                          >
                            {b.archived ? "Restore" : "Archive"}
                          </Button>
                        </td>
                      )}
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </Panel>

      {editingBudget && (
        <BudgetModal budget={editingBudget} defaultCurrency={currency} onClose={() => setEditingBudget(null)} />
      )}
    </div>
  );
}

function BudgetModal({
  budget,
  defaultCurrency,
  onClose,
}: {
  budget: BudgetInput;
  defaultCurrency: string;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [name, setName] = useState(budget.name || "");
  const [code, setCode] = useState(budget.code || "");
  const [amount, setAmount] = useState(budget.amountMinor ? String(budget.amountMinor / 100) : "");
  const [fiscalYear, setFiscalYear] = useState(budget.fiscalYear || "");

  const save = useMutation({
    mutationFn: () => {
      const minor = Math.round((parseFloat(amount) || 0) * 100);
      return acquisitionsApi.saveBudget({
        id: budget.id,
        name: name.trim(),
        code: code.trim() || null,
        amountMinor: minor,
        currency: budget.currency || defaultCurrency,
        fiscalYear: fiscalYear.trim() || null,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["acquisitions-budgets"] });
      onClose();
    },
  });

  return (
    <Modal title={budget.id ? "Edit Budget" : "Add Budget"} onClose={onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!name.trim() || !amount) return;
          save.mutate();
        }}
        className="space-y-4"
      >
        <TextField label="Budget Name" value={name} onChange={(e) => setName(e.target.value)} required />
        <div className="grid grid-cols-2 gap-4">
          <TextField label="Code (e.g. GF-2026)" value={code} onChange={(e) => setCode(e.target.value)} />
          <TextField label="Fiscal Year" value={fiscalYear} onChange={(e) => setFiscalYear(e.target.value)} />
        </div>
        <TextField
          label={`Total Amount (${budget.currency || defaultCurrency})`}
          type="number"
          step="0.01"
          min="0"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          required
        />

        {save.isError && <p className="text-xs text-danger">{toApiError(save.error).message}</p>}

        <div className="flex justify-end gap-2 pt-2 border-t border-line">
          <Button onClick={onClose} disabled={save.isPending}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={save.isPending}>
            {save.isPending ? "Saving…" : "Save Budget"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
