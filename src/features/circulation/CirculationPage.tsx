import { lazy, Suspense, useRef, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { circulationApi, toApiError } from "@/lib/api";
import type { IssuePreview, RenewalPreview, ReturnCondition, ReturnPreview } from "@/lib/api/types";
import { formatMinor } from "@/lib/money";
import { can, useSession } from "@/lib/session";
import { OverdueLoans } from "./OverdueLoans";
import { DueSlip } from "./DueSlip";

const Reservations = lazy(() => import("./Reservations"));
const Fines = lazy(() => import("./Fines"));

const ACTIONS = {
  issue: { label: "Issue", review: "Review loan", title: "Confirm issue", confirm: "Issue copy" },
  return: { label: "Return", review: "Find open loan", title: "Confirm return", confirm: "Return copy" },
  renew: { label: "Renew", review: "Review renewal", title: "Confirm renewal", confirm: "Renew loan" },
} as const;

type Mode = keyof typeof ACTIONS;
type View = Mode | "overdue" | "reservations" | "fines";
type Review =
  | { mode: "issue"; memberNumber: string; barcode: string; details: IssuePreview }
  | { mode: "return"; details: ReturnPreview; condition: ReturnCondition }
  | { mode: "renew"; details: RenewalPreview };

export function CirculationPage() {
  const { session } = useSession();
  const client = useQueryClient();
  const formRef = useRef<HTMLFormElement>(null);
  const actionGroup = useRef<HTMLDivElement>(null);
  const [mode, setMode] = useState<View>("issue");
  const [review, setReview] = useState<Review | null>(null);
  const [message, setMessage] = useState("");
  const [completedLoan, setCompletedLoan] = useState<Extract<Review, { mode: "issue" | "renew" }> | null>(null);
  const [slipOpen, setSlipOpen] = useState(false);
  const prepare = useMutation({
    mutationFn: async ({ mode, memberNumber, barcode }: { mode: Mode; memberNumber: string; barcode: string }): Promise<Review> => {
      switch (mode) {
        case "issue": return { mode, memberNumber, barcode, details: await circulationApi.prepareIssue(memberNumber, barcode) };
        case "return": return { mode, details: await circulationApi.prepareReturn(barcode), condition: "available" };
        case "renew": return { mode, details: await circulationApi.prepareRenewal(barcode) };
      }
    },
    onSuccess: setReview,
  });
  const complete = useMutation({
    mutationFn: async (value: Review) => {
      switch (value.mode) {
        case "issue": return circulationApi.issue(value.memberNumber, value.barcode, value.details);
        case "return": return circulationApi.returnCopy(value.details.loanId, value.condition);
        case "renew": return circulationApi.renew(value.details);
      }
    },
    onSuccess: (fine, value) => {
      void client.invalidateQueries({ queryKey: ["catalogue"] });
      void client.invalidateQueries({ queryKey: ["circulation", "overdue"] });
      void client.invalidateQueries({ queryKey: ["circulation", "reservations"] });
      if (fine) void client.invalidateQueries({ queryKey: ["circulation", "fines"] });
      void client.invalidateQueries({ queryKey: ["member-loans"] });
      const { title, dueOn } = value.details;
      const messages = {
        issue: `Issued ${title}. Due ${dueOn}.`,
        return: `Returned ${title}.${fine ? ` Overdue fine: ${formatMinor(fine.amountMinor, fine.currency)}.` : ""}`,
        renew: `Renewed ${title}. Now due ${dueOn}.`,
      };
      const conditionMessage = value.mode === "return" && value.condition !== "available"
        ? `Closed the loan for ${title}. Marked ${value.condition}.${fine ? ` Overdue fine: ${formatMinor(fine.amountMinor, fine.currency)}.` : ""}`
        : messages[value.mode];
      setMessage(conditionMessage);
      setCompletedLoan(value.mode === "return" ? null : value);
      setReview(null);
      const barcode = formRef.current?.querySelector<HTMLInputElement>('input[name="barcode"]');
      if (barcode) barcode.value = "";
      requestAnimationFrame(() => {
        const target = barcode ?? actionGroup.current?.querySelector<HTMLButtonElement>('button[aria-pressed="true"]');
        target?.focus();
      });
    },
  });
  if (!can(session, "circulation.issue")) {
    return <p className="p-6" role="alert">You do not have permission to issue, return, or renew loans.</p>;
  }

  function selectMode(value: View) {
    prepare.reset();
    complete.reset();
    setMessage("");
    setCompletedLoan(null);
    setMode(value);
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (mode === "overdue" || mode === "reservations" || mode === "fines") return;
    const data = new FormData(event.currentTarget);
    setMessage("");
    setCompletedLoan(null);
    complete.reset();
    prepare.mutate({ mode, memberNumber: String(data.get("memberNumber") ?? ""), barcode: String(data.get("barcode") ?? "") });
  }

  const error = prepare.error ?? complete.error;
  return (
    <div className="mx-auto max-w-[1100px] space-y-4 p-6">
      <PageHeader title="Circulation" />
      <div ref={actionGroup} className="flex gap-2" role="group" aria-label="Circulation action">
        {(["issue", "return", "renew"] as const).map((action) => (
          <Button key={action} aria-pressed={mode === action} variant={mode === action ? "primary" : "secondary"}
            disabled={prepare.isPending} onClick={() => selectMode(action)}>{ACTIONS[action].label}</Button>
        ))}
        <Button aria-pressed={mode === "overdue"} variant={mode === "overdue" ? "primary" : "secondary"}
          disabled={prepare.isPending} onClick={() => selectMode("overdue")}>Overdue</Button>
        <Button aria-pressed={mode === "reservations"} variant={mode === "reservations" ? "primary" : "secondary"}
          disabled={prepare.isPending} onClick={() => selectMode("reservations")}>Reservations</Button>
        <Button aria-pressed={mode === "fines"} variant={mode === "fines" ? "primary" : "secondary"}
          disabled={prepare.isPending} onClick={() => selectMode("fines")}>Fines</Button>
      </div>
      {mode === "fines" ? (
        <Suspense fallback={<p role="status" className="text-muted">Loading fines.</p>}>
          <Fines />
        </Suspense>
      ) : mode === "reservations" ? (
        <Suspense fallback={<p role="status" className="text-muted">Loading reservations.</p>}>
          <Reservations />
        </Suspense>
      ) : mode === "overdue" ? <>
        <OverdueLoans busy={prepare.isPending} onReturn={(loan) => {
          complete.reset();
          setMessage("");
          prepare.mutate({ mode: "return", memberNumber: "", barcode: loan.barcode });
        }} />
        {prepare.isPending && <p role="status">Looking up loan.</p>}
        {!review && error && <p role="alert" className="text-danger">{toApiError(error).message}</p>}
      </> : <Panel className="max-w-xl p-4">
        <form key={mode} ref={formRef} onSubmit={submit} className="space-y-4">
          <fieldset disabled={prepare.isPending} className="space-y-3">
            {mode === "issue" && <TextField label="Member number" name="memberNumber" required maxLength={64} autoComplete="off"
              onKeyDown={(event) => {
                if (event.key !== "Enter" || !event.currentTarget.value.trim()) return;
                event.preventDefault();
                formRef.current?.querySelector<HTMLInputElement>('input[name="barcode"]')?.focus();
              }} />}
            <TextField label="Copy barcode" name="barcode" required maxLength={40} autoComplete="off" />
          </fieldset>
          <Button type="submit" variant="primary" disabled={prepare.isPending}>
            {prepare.isPending ? "Looking up…" : ACTIONS[mode].review}
          </Button>
        </form>
        {!review && error && <p role="alert" className="mt-3 text-danger">{toApiError(error).message}</p>}
      </Panel>}
      {message && <div className="flex flex-wrap items-center gap-3">
        <p role="status">{message}</p>
        {completedLoan && <Button size="sm" onClick={() => setSlipOpen(true)}>View due slip</Button>}
      </div>}
      {slipOpen && completedLoan && (
        <DueSlip loan={completedLoan.details} renewed={completedLoan.mode === "renew"} onClose={() => setSlipOpen(false)} />
      )}
      {review && (
        <Modal title={ACTIONS[review.mode].title}
          onClose={() => { if (!complete.isPending) setReview(null); }}
          footer={<>
            <Button disabled={complete.isPending} onClick={() => setReview(null)}>Cancel</Button>
            <Button variant="primary" disabled={complete.isPending || complete.isError} onClick={() => complete.mutate(review)}>
              {review.mode === "return" && review.condition === "lost" ? "Close loan as lost" : ACTIONS[review.mode].confirm}
            </Button>
          </>}>
          <dl className="grid grid-cols-[6rem_1fr] gap-2">
            <dt className="text-muted">Title</dt><dd>{review.details.title}</dd>
            <dt className="text-muted">Member</dt><dd>{review.details.memberName}</dd>
            {review.mode === "renew" && <><dt className="text-muted">Current due</dt><dd>{review.details.previousDueOn}</dd></>}
            <dt className="text-muted">{review.mode === "renew" ? "New due date" : "Due date"}</dt><dd>{review.details.dueOn}</dd>
          </dl>
          {review.mode === "return" && review.details.estimatedFine && (
            <p className="text-sm">Estimated overdue fine {formatMinor(
              review.details.estimatedFine.amountMinor, review.details.estimatedFine.currency,
            )}</p>
          )}
          {review.mode === "return" && can(session, "circulation.manage") && (
            <SelectField label="Copy condition" value={review.condition} disabled={complete.isPending}
              onChange={(event) => setReview({ ...review, condition: event.target.value as ReturnCondition })}>
              <option value="available">Available</option>
              <option value="damaged">Damaged</option>
              <option value="lost">Lost</option>
            </SelectField>
          )}
          {complete.error && <p role="alert" className="text-danger">{toApiError(complete.error).message}</p>}
        </Modal>
      )}
    </div>
  );
}
