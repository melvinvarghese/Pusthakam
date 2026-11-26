import { useTranslation } from "@/lib/translations";
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
import { announce } from "@/lib/announcer";
import { can, useSession } from "@/lib/session";
import type { TranslationValues } from "@/lib/translations";
import { OpenLoans } from "./OpenLoans";
import { OverdueLoans } from "./OverdueLoans";
import { DueSlip } from "./DueSlip";
import { FindBook } from "./FindBook";
import { FindMember } from "./FindMember";

const Reservations = lazy(() => import("./Reservations"));
const Fines = lazy(() => import("./Fines"));
const LoanImport = lazy(() => import("./LoanImport"));
const RenewalImport = lazy(() => import("./RenewalImport"));
const HistoryImport = lazy(() => import("./HistoryImport"));

const ACTIONS = {
  issue: { label: "Issue", review: "Review loan", title: "Confirm issue", confirm: "Issue copy" },
  return: { label: "Return", review: "Find open loan", title: "Confirm return", confirm: "Return copy" },
  renew: { label: "Renew", review: "Review renewal", title: "Confirm renewal", confirm: "Renew loan" },
} as const;

type Mode = keyof typeof ACTIONS;
type View = Mode | "overdue" | "reservations" | "fines";
type CompletionMessage = { key: string; values: TranslationValues; fine?: string; condition?: string };
type Review =
  | { mode: "issue"; memberNumber: string; barcode: string; details: IssuePreview }
  | { mode: "return"; details: ReturnPreview; condition: ReturnCondition }
  | { mode: "renew"; details: RenewalPreview };

export function CirculationPage() {
  const { t } = useTranslation();
  const { session } = useSession();
  const client = useQueryClient();
  const formRef = useRef<HTMLFormElement>(null);
  const actionGroup = useRef<HTMLDivElement>(null);
  const [mode, setMode] = useState<View>("issue");
  const [review, setReview] = useState<Review | null>(null);
  const [message, setMessage] = useState<CompletionMessage | null>(null);
  const messageText = (value: CompletionMessage) =>
    t(value.key, { ...value.values, condition: t(value.condition ?? "") })
      + (value.fine ? ` ${t("Overdue fine: {amount}.", { amount: value.fine })}` : "");
  const [completedLoan, setCompletedLoan] = useState<Extract<Review, { mode: "issue" | "renew" }> | null>(null);
  const [slipOpen, setSlipOpen] = useState(false);
  const [findingBook, setFindingBook] = useState(false);
  const [findingMember, setFindingMember] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [renewalImportOpen, setRenewalImportOpen] = useState(false);
  const [historyImportOpen, setHistoryImportOpen] = useState(false);
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
      void client.invalidateQueries({ queryKey: ["circulation", "open-loans"] });
      void client.invalidateQueries({ queryKey: ["circulation", "reservations"] });
      if (fine) {
        void client.invalidateQueries({ queryKey: ["circulation", "fines"] });
        void client.invalidateQueries({ queryKey: ["member-fines"] });
      }
      void client.invalidateQueries({ queryKey: ["member-loans"] });
      const { title, dueOn } = value.details;
      const keys = {
        issue: "Issued {title}. Due {dueOn}.",
        return: "Returned {title}.",
        renew: "Renewed {title}. Now due {dueOn}.",
      };
      const completed: CompletionMessage = {
        key: value.mode === "return" && value.condition !== "available"
          ? "Closed the loan for {title}. Marked {condition}." : keys[value.mode],
        values: { title, dueOn },
        condition: value.mode === "return" ? value.condition : undefined,
        fine: fine ? formatMinor(fine.amountMinor, fine.currency) : undefined,
      };
      setMessage(completed);
      announce(messageText(completed), "polite");
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
    return <p className="p-6" role="alert">{t("You do not have permission to issue, return, or renew loans.")}</p>;
  }

  function selectMode(value: View) {
    prepare.reset();
    complete.reset();
    setMessage(null);
    setCompletedLoan(null);
    setFindingBook(false);
    setMode(value);
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (mode === "overdue" || mode === "reservations" || mode === "fines") return;
    const data = new FormData(event.currentTarget);
    setMessage(null);
    setCompletedLoan(null);
    complete.reset();
    prepare.mutate({ mode, memberNumber: String(data.get("memberNumber") ?? ""), barcode: String(data.get("barcode") ?? "") });
  }

  const error = prepare.error ?? complete.error;
  return (
    <div className="mx-auto max-w-[1100px] space-y-4 p-6">
      <PageHeader title={t("Circulation")} />
      <div ref={actionGroup} className="flex flex-wrap gap-2" role="group" aria-label={t("Circulation action")}>
        {can(session, "admin.settings") && <Button onClick={() => setImportOpen(true)}>{t("Preview loan import")}</Button>}
        {can(session, "admin.settings") && <Button onClick={() => setRenewalImportOpen(true)}>{t("Import renewal history")}</Button>}
        {can(session, "admin.settings") && <Button onClick={() => setHistoryImportOpen(true)}>{t("Import reservations and fines")}</Button>}
        {(["issue", "return", "renew"] as const).map((action) => (
          <Button key={action} aria-pressed={mode === action} variant={mode === action ? "primary" : "secondary"}
            disabled={prepare.isPending} onClick={() => selectMode(action)}>{t(ACTIONS[action].label)}</Button>
        ))}
        <Button aria-pressed={mode === "overdue"} variant={mode === "overdue" ? "primary" : "secondary"}
          disabled={prepare.isPending} onClick={() => selectMode("overdue")}>{t("Overdue")}</Button>
        <Button aria-pressed={mode === "reservations"} variant={mode === "reservations" ? "primary" : "secondary"}
          disabled={prepare.isPending} onClick={() => selectMode("reservations")}>{t("Reservations")}</Button>
        <Button aria-pressed={mode === "fines"} variant={mode === "fines" ? "primary" : "secondary"}
          disabled={prepare.isPending} onClick={() => selectMode("fines")}>{t("Fines")}</Button>
      </div>
      {importOpen && <Suspense fallback={<p role="status">{t("Loading import preview.")}</p>}><LoanImport onClose={() => setImportOpen(false)} /></Suspense>}
      {renewalImportOpen && <Suspense fallback={<p role="status">{t("Loading renewal import.")}</p>}><RenewalImport onClose={() => setRenewalImportOpen(false)} /></Suspense>}
      {historyImportOpen && <Suspense fallback={<p role="status">{t("Loading history import.")}</p>}><HistoryImport onClose={() => setHistoryImportOpen(false)} /></Suspense>}
      {mode === "fines" ? (
        <Suspense fallback={<p role="status" className="text-muted">{t("Loading fines.")}</p>}>
          <Fines />
        </Suspense>
      ) : mode === "reservations" ? (
        <Suspense fallback={<p role="status" className="text-muted">{t("Loading reservations.")}</p>}>
          <Reservations />
        </Suspense>
      ) : mode === "overdue" ? <>
        <OverdueLoans busy={prepare.isPending} onReturn={(loan) => {
          complete.reset();
          setMessage(null);
          prepare.mutate({ mode: "return", memberNumber: "", barcode: loan.barcode });
        }} />
        {prepare.isPending && <p role="status">{t("Looking up loan.")}</p>}
        {!review && error && <p role="alert" className="text-danger">{toApiError(error).message}</p>}
      </> : <Panel className="max-w-xl p-4">
        <form key={mode} ref={formRef} onSubmit={submit} className="space-y-4">
          <fieldset disabled={prepare.isPending} className="space-y-3">
            {mode === "issue" && <TextField label={t("Member number")} name="memberNumber" required maxLength={64} autoComplete="off"
              onKeyDown={(event) => {
                if (event.key !== "Enter" || !event.currentTarget.value.trim()) return;
                event.preventDefault();
                formRef.current?.querySelector<HTMLInputElement>('input[name="barcode"]')?.focus();
              }} />}
            {mode === "issue" && can(session, "members.read") && (
              <Button type="button" size="sm" onClick={() => setFindingMember(true)}>{t("Find member by name")}</Button>
            )}
            <TextField label={t("Book barcode or accession number")} name="barcode" required maxLength={40} autoComplete="off" />
            {mode === "issue" && can(session, "catalogue.read") && <Button type="button" size="sm" onClick={() => setFindingBook(true)}>{t("Find book by name")}</Button>}
          </fieldset>
          <Button type="submit" variant="primary" disabled={prepare.isPending}>
            {prepare.isPending ? t("Looking up…") : t(ACTIONS[mode].review)}
          </Button>
        </form>
        {!review && error && <p role="alert" className="mt-3 text-danger">{toApiError(error).message}</p>}
      </Panel>}
      {(mode === "return" || mode === "renew") && <OpenLoans key={mode} action={mode} busy={prepare.isPending || complete.isPending} onSelect={(barcode) => {
        setMessage(null); setCompletedLoan(null); complete.reset();
        prepare.mutate({ mode, memberNumber: "", barcode });
      }} />}
      {mode === "issue" && findingBook && <FindBook onClose={() => setFindingBook(false)} onSelect={(barcode) => {
        const input = formRef.current?.querySelector<HTMLInputElement>('input[name="barcode"]');
        if (input) input.value = barcode;
        setFindingBook(false);
        requestAnimationFrame(() => formRef.current?.querySelector<HTMLButtonElement>('button[type="submit"]')?.focus());
      }} />}
      {mode === "issue" && findingMember && <FindMember onClose={() => setFindingMember(false)}
        onSelect={(number) => {
          const input = formRef.current?.querySelector<HTMLInputElement>('input[name="memberNumber"]');
          if (input) input.value = number;
          setFindingMember(false);
          requestAnimationFrame(() => formRef.current?.querySelector<HTMLInputElement>('input[name="barcode"]')?.focus());
        }} />}
      {message && <div className="flex flex-wrap items-center gap-3">
        <p role="status">{messageText(message)}</p>
        {completedLoan && <Button size="sm" onClick={() => setSlipOpen(true)}>{t("View due slip")}</Button>}
      </div>}
      {slipOpen && completedLoan && (
        <DueSlip loan={completedLoan.details} renewed={completedLoan.mode === "renew"} onClose={() => setSlipOpen(false)} />
      )}
      {review && (
        <Modal title={t(ACTIONS[review.mode].title)}
          onClose={() => { if (!complete.isPending) setReview(null); }}
          footer={<>
            <Button disabled={complete.isPending} onClick={() => setReview(null)}>{t("Cancel")}</Button>
            <Button variant="primary" disabled={complete.isPending || complete.isError} onClick={() => complete.mutate(review)}>
              {review.mode === "return" && review.condition === "lost" ? t("Close loan as lost") : t(ACTIONS[review.mode].confirm)}
            </Button>
          </>}>
          <dl className="grid grid-cols-[6rem_1fr] gap-2">
            <dt className="text-muted">{t("Title")}</dt><dd>{review.details.title}</dd>
            <dt className="text-muted">{t("Member")}</dt><dd>{review.details.memberName}</dd>
            {review.mode === "renew" && <><dt className="text-muted">{t("Current due")}</dt><dd>{review.details.previousDueOn}</dd></>}
            <dt className="text-muted">{review.mode === "renew" ? t("New due date") : t("Due date")}</dt><dd>{review.details.dueOn}</dd>
          </dl>
          {review.mode === "return" && review.details.estimatedFine && (
            <p className="text-sm">{t("Estimated overdue fine")} {formatMinor(
              review.details.estimatedFine.amountMinor, review.details.estimatedFine.currency,
            )}</p>
          )}
          {review.mode === "return" && can(session, "circulation.manage") && (
            <SelectField label={t("Copy condition")} value={review.condition} disabled={complete.isPending}
              onChange={(event) => setReview({ ...review, condition: event.target.value as ReturnCondition })}>
              <option value="available">{t("Available")}</option>
              <option value="damaged">{t("Damaged")}</option>
              <option value="lost">{t("Lost")}</option>
            </SelectField>
          )}
          {complete.error && <p role="alert" className="text-danger">{toApiError(complete.error).message}</p>}
        </Modal>
      )}
    </div>
  );
}
