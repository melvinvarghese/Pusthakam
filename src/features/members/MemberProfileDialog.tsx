import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { prepareImage as preparePhoto } from "@/lib/images";
import { formatMinor } from "@/lib/money";
import { membersApi, toApiError } from "@/lib/api";
import type { Member } from "@/lib/api/types";
import { useTranslation } from "@/lib/translations";

interface Props {
  member: Member;
  canEdit: boolean;
  hasLibrary: boolean;
  onClose: () => void;
  onLoans: () => void;
  onFines: () => void;
  onCard: () => void;
  onEdit: () => void;
}

type ProfileTab = "overview" | "holds" | "activity";

const KIND_LABELS: Record<string, string> = {
  loan: "Loan",
  return: "Return",
  renewal: "Renewal",
  fine: "Fine",
  hold: "Hold",
  admin: "Admin",
};

export function MemberProfileDialog({
  member,
  canEdit,
  hasLibrary,
  onClose,
  onLoans,
  onFines,
  onCard,
  onEdit,
}: Props) {
  const { t } = useTranslation();
  const client = useQueryClient();
  const [tab, setTab] = useState<ProfileTab>("overview");
  const [username, setUsername] = useState(member.opacUsername ?? "");
  const [password, setPassword] = useState("");
  const [clearPassword, setClearPassword] = useState(false);
  const [credNotice, setCredNotice] = useState<string | null>(null);
  const [expiresOn, setExpiresOn] = useState(member.expiresOn);
  const [renewNotice, setRenewNotice] = useState<string | null>(null);

  const photo = useQuery({
    queryKey: ["member-photo", member.id],
    queryFn: () => membersApi.getPhoto(member.id),
    retry: false,
  });

  const summary = useQuery({
    queryKey: ["member-summary", member.id],
    queryFn: () => membersApi.summary(member.id),
    retry: false,
  });

  const holds = useQuery({
    queryKey: ["member-holds", member.id],
    queryFn: () => membersApi.holds(member.id),
    enabled: tab === "holds",
    retry: false,
  });

  const activity = useQuery({
    queryKey: ["member-activity", member.id],
    queryFn: () => membersApi.activity(member.id),
    enabled: tab === "activity",
    retry: false,
  });

  const savePhoto = useMutation({
    mutationFn: async (file: File | null) => {
      const data = file ? await preparePhoto(file) : null;
      await membersApi.setPhoto(member.id, data);
    },
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ["member-photo", member.id] });
    },
  });

  const renew = useMutation({
    mutationFn: () => membersApi.renew(member.id),
    onSuccess: async (newExpiry) => {
      setExpiresOn(newExpiry);
      setRenewNotice(t("Membership renewed through {date}.", { date: newExpiry }));
      await client.invalidateQueries({ queryKey: ["member-summary", member.id] });
      await client.invalidateQueries({ queryKey: ["member-activity", member.id] });
      await client.invalidateQueries({ queryKey: ["members"] });
    },
  });

  const saveCreds = useMutation({
    mutationFn: () =>
      membersApi.setOpacCredentials(
        member.id,
        username.trim() || null,
        password.trim() || null,
        clearPassword,
      ),
    onSuccess: async () => {
      setPassword("");
      setClearPassword(false);
      setCredNotice(t("OPAC login credentials saved."));
      await client.invalidateQueries({ queryKey: ["member-activity", member.id] });
      await client.invalidateQueries({ queryKey: ["members"] });
    },
  });

  function submitCreds(e: FormEvent) {
    e.preventDefault();
    setCredNotice(null);
    saveCreds.mutate();
  }

  return (
    <Modal title={t("Member profile")} onClose={onClose} wide footer={<Button onClick={onClose}>{t("Close")}</Button>}>
      <div className="space-y-4">
        {/* Tabs */}
        <div role="tablist" aria-label={t("Member profile sections")} className="flex gap-2 border-b border-line pb-2">
          {([
            ["overview", t("Overview")],
            ["holds", summary.data ? `${t("Holds")} (${summary.data.activeHolds})` : t("Holds")],
            ["activity", t("Activity")],
          ] as [ProfileTab, string][]).map(([id, label]) => (
            <Button
              key={id}
              type="button"
              role="tab"
              aria-selected={tab === id}
              size="sm"
              variant={tab === id ? "primary" : "ghost"}
              onClick={() => setTab(id)}
            >
              {label}
            </Button>
          ))}
        </div>

        {tab === "overview" && (
          <>
            {/* Header: Photo + Core Profile */}
            <div className="flex flex-wrap items-start gap-4 rounded-md border border-line bg-band/40 p-4">
              <div className="flex flex-col items-center gap-2">
                {photo.data ? (
                  <img
                    src={`data:image/png;base64,${photo.data}`}
                    alt={member.name}
                    className="size-24 rounded-md border border-line bg-surface object-contain"
                  />
                ) : (
                  <div className="grid size-24 place-items-center rounded-md border border-dashed border-line bg-surface text-xs text-muted">
                    {photo.isPending ? t("Loading…") : t("No photo")}
                  </div>
                )}
                {canEdit && (
                  <div className="flex flex-col items-center gap-1">
                    <label className="cursor-pointer text-xs font-medium text-accent underline">
                      {photo.data ? t("Change photo") : t("Add photo")}
                      <input
                        type="file"
                        accept=".png,.jpg,.jpeg,image/png,image/jpeg"
                        className="sr-only"
                        disabled={savePhoto.isPending}
                        onChange={(event) => {
                          const file = event.target.files?.[0];
                          event.target.value = "";
                          if (file) savePhoto.mutate(file);
                        }}
                      />
                    </label>
                    {photo.data && (
                      <button
                        type="button"
                        disabled={savePhoto.isPending}
                        onClick={() => savePhoto.mutate(null)}
                        className="text-xs text-danger underline"
                      >
                        {t("Remove photo")}
                      </button>
                    )}
                  </div>
                )}
              </div>

              <div className="min-w-0 flex-1 space-y-2">
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <div>
                    <h2 className="text-lg font-bold leading-tight">{member.name}</h2>
                    <p className="font-mono text-xs text-muted">
                      {member.memberNumber} · {member.categoryName}
                      {member.department ? ` · ${member.department}` : ""}
                    </p>
                  </div>
                  {summary.data && (
                    <span className={`rounded-full px-2.5 py-0.5 text-xs font-semibold ${summary.data.eligibleToBorrow ? "bg-ok-bg text-ok" : "bg-danger/10 text-danger"}`}>
                      {summary.data.eligibleToBorrow ? t("Eligible to borrow") : t("Ineligible / Expired")}
                    </span>
                  )}
                </div>

                <dl className="grid grid-cols-2 gap-x-4 gap-y-1.5 text-sm sm:grid-cols-3">
                  <div>
                    <dt className="text-xs text-muted">{t("Email")}</dt>
                    <dd className="break-all">{member.email || "—"}</dd>
                  </div>
                  <div>
                    <dt className="text-xs text-muted">{t("Phone")}</dt>
                    <dd>{member.phone || "—"}</dd>
                  </div>
                  <div>
                    <dt className="text-xs text-muted">{t("Start date")}</dt>
                    <dd>{member.joinedOn || "—"}</dd>
                  </div>
                  <div>
                    <dt className="text-xs text-muted">{t("Expiry date")}</dt>
                    <dd>{expiresOn}</dd>
                  </div>
                  <div className="col-span-2">
                    <dt className="text-xs text-muted">{t("Address")}</dt>
                    <dd>{member.address || "—"}</dd>
                  </div>
                </dl>

                <div className="flex flex-wrap gap-2 pt-1">
                  <Button size="sm" variant="secondary" onClick={() => { onClose(); onLoans(); }}>{t("Loans")}</Button>
                  <Button size="sm" variant="secondary" onClick={() => { onClose(); onFines(); }}>{t("Fines")}</Button>
                  {canEdit && (
                    <>
                      <Button size="sm" variant="secondary" disabled={!hasLibrary} onClick={() => { onClose(); onCard(); }}>{t("Card")}</Button>
                      <Button size="sm" variant="secondary" onClick={() => { onClose(); onEdit(); }}>{t("Edit details")}</Button>
                      <Button size="sm" variant="secondary" disabled={renew.isPending} onClick={() => { setRenewNotice(null); renew.mutate(); }}>
                        {renew.isPending
                          ? t("Renewing…")
                          : summary.data
                            ? t("Renew (+{days}d)", { days: summary.data.membershipDays })
                            : t("Renew membership")}
                      </Button>
                    </>
                  )}
                </div>
                {renewNotice && <p role="status" className="text-xs text-ok">{renewNotice}</p>}
                {renew.error && <p role="alert" className="text-xs text-danger">{toApiError(renew.error).message}</p>}
              </div>
            </div>

            {/* Live Summary Strip */}
            {summary.data && (
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-5">
                <div className="rounded-md border border-line bg-surface p-2.5">
                  <div className="text-[11px] text-muted">{t("Active loans")}</div>
                  <div className="text-base font-bold">
                    {summary.data.activeLoans}
                    {summary.data.maxLoansLimit !== null && (
                      <span className="text-xs font-normal text-muted"> / {summary.data.maxLoansLimit}</span>
                    )}
                  </div>
                </div>
                <div className="rounded-md border border-line bg-surface p-2.5">
                  <div className="text-[11px] text-muted">{t("Overdue")}</div>
                  <div className={`text-base font-bold ${summary.data.overdueLoans > 0 ? "text-danger" : ""}`}>
                    {summary.data.overdueLoans}
                  </div>
                </div>
                <div className="rounded-md border border-line bg-surface p-2.5">
                  <div className="text-[11px] text-muted">{t("Active holds")}</div>
                  <div className="text-base font-bold">{summary.data.activeHolds}</div>
                </div>
                <div className="rounded-md border border-line bg-surface p-2.5">
                  <div className="text-[11px] text-muted">{t("Total borrowed")}</div>
                  <div className="text-base font-bold">{summary.data.totalLoans}</div>
                </div>
                <div className="rounded-md border border-line bg-surface p-2.5">
                  <div className="text-[11px] text-muted">{t("Unpaid fines")}</div>
                  <div className={`text-sm font-bold ${summary.data.unpaidFines.length > 0 ? "text-danger" : ""}`}>
                    {summary.data.unpaidFines.length === 0
                      ? "0"
                      : summary.data.unpaidFines.map((b) => formatMinor(b.outstandingMinor, b.currency)).join(", ")}
                  </div>
                </div>
              </div>
            )}

            {/* Staff-Only Internal Notes */}
            {member.notes && (
              <div className="rounded-md border border-line bg-band/30 p-3">
                <div className="text-xs font-semibold text-muted">{t("Internal librarian notes (staff only)")}</div>
                <p className="mt-1 whitespace-pre-wrap text-sm">{member.notes}</p>
              </div>
            )}

            {savePhoto.error && <p role="alert" className="text-xs text-danger">{toApiError(savePhoto.error).message}</p>}

            {/* OPAC Portal Login Credentials */}
            <section className="space-y-3 rounded-md border border-line p-4">
              <div>
                <h3 className="text-sm font-semibold">{t("OPAC Portal Login")}</h3>
                <p className="text-xs text-muted">
                  {t("Assign a username and password/PIN so this member can sign into the public OPAC to view loans, fines, and place holds.")}
                </p>
              </div>

              {canEdit ? (
                <form onSubmit={submitCreds} className="space-y-3">
                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <TextField
                      label={t("OPAC username")}
                      value={username}
                      onChange={(e) => setUsername(e.target.value)}
                      placeholder={member.memberNumber}
                      maxLength={40}
                      hint={t("Optional; member number also works")}
                    />
                    <TextField
                      label={member.hasOpacPassword ? t("New password or PIN") : t("Password or PIN")}
                      type="password"
                      value={password}
                      onChange={(e) => { setPassword(e.target.value); if (e.target.value) setClearPassword(false); }}
                      placeholder={member.hasOpacPassword ? t("Leave blank to keep existing") : t("At least 4 characters")}
                      maxLength={128}
                    />
                  </div>
                  {member.hasOpacPassword && (
                    <label className="flex items-center gap-2 text-xs text-muted">
                      <input
                        type="checkbox"
                        checked={clearPassword}
                        onChange={(e) => { setClearPassword(e.target.checked); if (e.target.checked) setPassword(""); }}
                      />
                      <span>{t("Clear password (allow sign-in with phone or email instead)")}</span>
                    </label>
                  )}
                  {credNotice && <p role="status" className="text-xs text-ok">{credNotice}</p>}
                  {saveCreds.error && <p role="alert" className="text-xs text-danger">{toApiError(saveCreds.error).message}</p>}
                  <div className="flex justify-end">
                    <Button type="submit" size="sm" disabled={saveCreds.isPending}>
                      {saveCreds.isPending ? t("Saving…") : t("Save OPAC login")}
                    </Button>
                  </div>
                </form>
              ) : (
                <p className="text-sm text-muted">
                  {t("OPAC username")}: <span className="font-mono">{member.opacUsername || member.memberNumber}</span>
                  {" · "}
                  {member.hasOpacPassword ? t("Password set") : t("Uses phone or email")}
                </p>
              )}
            </section>
          </>
        )}

        {tab === "holds" && (
          <div className="space-y-3">
            {holds.isPending && <p role="status" className="text-sm text-muted">{t("Loading member holds…")}</p>}
            {holds.isError && <p role="alert" className="text-sm text-danger">{toApiError(holds.error).message}</p>}
            {holds.data && holds.data.length === 0 && (
              <p className="rounded-md border border-line p-4 text-sm text-muted">{t("This member has no reservations or holds.")}</p>
            )}
            {holds.data && holds.data.length > 0 && (
              <ul className="divide-y divide-line rounded-md border border-line">
                {holds.data.map((h) => (
                  <li key={h.id} className="flex flex-wrap items-center justify-between gap-2 p-3 text-sm">
                    <div className="min-w-0">
                      <div className="font-medium">{h.title}</div>
                      <div className="font-mono text-xs text-muted">
                        {h.barcode} · {t("Placed")} {new Date(h.placedAt).toLocaleDateString()}
                        {h.expiresAt && !h.closedAt ? ` · ${t("Pickup by")} ${new Date(h.expiresAt).toLocaleDateString()}` : ""}
                        {h.closedAt ? ` · ${t("Closed")} ${new Date(h.closedAt).toLocaleDateString()}` : ""}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {h.queuePosition !== null && (
                        <span className="rounded bg-band px-2 py-0.5 font-mono text-xs">#{h.queuePosition}</span>
                      )}
                      <span className="rounded-full bg-band px-2.5 py-0.5 text-xs font-medium capitalize">{h.status}</span>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {tab === "activity" && (
          <div className="space-y-3">
            {activity.isPending && <p role="status" className="text-sm text-muted">{t("Loading activity timeline…")}</p>}
            {activity.isError && <p role="alert" className="text-sm text-danger">{toApiError(activity.error).message}</p>}
            {activity.data && activity.data.length === 0 && (
              <p className="rounded-md border border-line p-4 text-sm text-muted">{t("No recorded activity for this member yet.")}</p>
            )}
            {activity.data && activity.data.length > 0 && (
              <ul className="divide-y divide-line rounded-md border border-line">
                {activity.data.map((item) => (
                  <li key={item.id} className="flex flex-wrap items-start justify-between gap-2 p-3 text-sm">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="rounded bg-band px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide text-muted">
                          {t(KIND_LABELS[item.kind] ?? item.kind)}
                        </span>
                        <span className="font-medium">{item.title}</span>
                      </div>
                      <p className="mt-0.5 text-xs text-muted">
                        {item.detail}
                        {item.staffName ? ` · ${t("Staff:")} ${item.staffName}` : ""}
                      </p>
                    </div>
                    <span className="whitespace-nowrap text-xs text-muted">
                      {new Date(item.occurredAt).toLocaleString()}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </div>
    </Modal>
  );
}
