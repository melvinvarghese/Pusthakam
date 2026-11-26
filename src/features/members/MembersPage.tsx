import { useTranslation } from "@/lib/translations";
import { lazy, Suspense, useDeferredValue, useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/PageHeader";
import { Pager } from "@/components/Pager";
import { Button } from "@/components/ui/Button";
import { SelectField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel } from "@/components/ui/Panel";
import { configApi, membersApi, toApiError } from "@/lib/api";
import type { Member, MemberInput, Named } from "@/lib/api/types";
import { can, useSession } from "@/lib/session";
import { MemberFines } from "./MemberFines";
import { MemberLoanHistory } from "./MemberLoanHistory";
import { MemberCard } from "./MemberCard";

const KohaMemberImport = lazy(() => import("./KohaMemberImport"));

function MemberForm({ member, categories, onClose }: {
  member?: Member;
  categories: Named[];
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const client = useQueryClient();
  const save = useMutation({
    mutationFn: membersApi.save,
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ["members"] });
      onClose();
    },
  });
  const choices = categories.filter((category) => !category.archived || category.id === member?.categoryId);
  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const input: MemberInput = {
      id: member?.id ?? null,
      memberNumber: String(form.get("number") ?? ""),
      name: String(form.get("name") ?? ""),
      categoryId: Number(form.get("category")),
      email: String(form.get("email") ?? "") || null,
      phone: String(form.get("phone") ?? "") || null,
      expiresOn: String(form.get("expiry") ?? "") || null,
    };
    save.mutate(input);
  }

  return (
    <Modal title={member ? t("Edit member") : t("Register member")} onClose={save.isPending ? () => {} : onClose}>
      <form onSubmit={submit} className="space-y-4">
        <fieldset disabled={save.isPending} className="space-y-4">
          <TextField label={t("Name")} name="name" defaultValue={member?.name} maxLength={160} required />
          <TextField label={t("Member number")} name="number" defaultValue={member?.memberNumber} maxLength={64} required />
          <SelectField label={t("Category")} name="category" defaultValue={member?.categoryId ?? ""} required>
            <option value="" disabled>{t("Choose a category")}</option>
            {choices.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}
          </SelectField>
          <div className="grid grid-cols-2 gap-3">
            <TextField label={t("Email")} name="email" type="email" defaultValue={member?.email ?? ""} maxLength={254} />
            <TextField label={t("Phone")} name="phone" type="tel" defaultValue={member?.phone ?? ""} maxLength={40} />
          </div>
          <TextField label={t("Expiry date")} name="expiry" type="date" defaultValue={member?.expiresOn}
            required={!!member} hint={member ? undefined : t("Leave blank to use the category’s membership period.")} />
        </fieldset>
        {save.error && <p role="alert" className="text-danger">{toApiError(save.error).message}</p>}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" onClick={onClose} disabled={save.isPending}>{t("Cancel")}</Button>
          <Button type="submit" disabled={save.isPending}>{save.isPending ? t("Saving…") : t("Save member")}</Button>
        </div>
      </form>
    </Modal>
  );
}

export function MembersPage() {
  const { t } = useTranslation();
  const { session } = useSession();
  const canRead = can(session, "members.read");
  const canEdit = can(session, "members.write");
  const client = useQueryClient();
  const [filter, setFilter] = useState({ search: "", page: 1, archived: false });
  const deferred = useDeferredValue(filter);
  const [form, setForm] = useState<{ member?: Member } | null>(null);
  const [fineMember, setFineMember] = useState<Member | null>(null);
  const [loanHistory, setLoanHistory] = useState<Member | null>(null);
  const [card, setCard] = useState<Member | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  const [confirm, setConfirm] = useState<{ member: Member; archive: boolean } | null>(null);
  const members = useQuery({
    queryKey: ["members", deferred],
    queryFn: () => membersApi.list(deferred.search, deferred.page, deferred.archived),
    enabled: canRead,
    retry: false,
  });
  const config = useQuery({ queryKey: ["config"], queryFn: configApi.overview, enabled: canRead && canEdit, retry: false });
  const categories = config.data?.memberCategories ?? [];
  const archive = useMutation({
    mutationFn: ({ id, archived }: { id: number; archived: boolean }) => membersApi.archive(id, archived),
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ["members"] });
      setConfirm(null);
    },
  });

  if (!canRead) return <p role="alert" className="p-6">{t("You do not have permission to view members.")}</p>;
  const hasCategory = categories.some((category) => !category.archived);
  return (
    <div className="mx-auto max-w-[1100px] space-y-4 p-6">
      <PageHeader title={t("Members")} />
      <div className="staff-member-toolbar flex items-end gap-3">
        <TextField label={t("Search members")} type="search" placeholder={t("Name or member number")} value={filter.search}
          onChange={(event) => setFilter({ ...filter, search: event.target.value, page: 1 })} />
        <SelectField label={t("Status")} value={filter.archived ? "archived" : "active"}
          onChange={(event) => setFilter({ ...filter, archived: event.target.value === "archived", page: 1 })}>
          <option value="active">{t("Active")}</option>
          <option value="archived">{t("Archived")}</option>
        </SelectField>
        {canEdit && <div className="ml-auto flex gap-2">
          <Button disabled={!hasCategory} onClick={() => setImportOpen(true)}>{t("Import Koha patrons")}</Button>
          <Button disabled={!hasCategory} onClick={() => setForm({})}>{t("Register member")}</Button>
        </div>}
      </div>
      {canEdit && config.isError && <p role="alert" className="text-danger">{toApiError(config.error).message}</p>}
      {canEdit && config.isSuccess && !hasCategory && (
        <p className="text-muted">{t("An active member category is required to register members.")}{" "}
          {can(session, "admin.settings") && <Link className="underline" to="/settings">{t("Open Library Settings")}</Link>}
        </p>
      )}
      {members.isPending && <p role="status" className="text-muted">{t("Loading members.")}</p>}
      {members.isError && <p role="alert" className="text-danger">{toApiError(members.error).message}</p>}
      {members.data && (
        <Panel>
          {members.data.rows.length === 0 ? (
            <p className="p-6 text-center text-muted">{filter.search ? t("No matching members.") : filter.archived ? t("No archived members.") : t("No members registered yet.")}</p>
          ) : (
            <table role="table" className="staff-card-table staff-member-table w-full border-collapse text-left text-sm">
              <thead className="border-b border-line text-xs text-muted">
                <tr>{["Member number", "Name", "Category", "Expiry date", "Actions"].map((label) => (
                  <th key={label} scope="col" className="px-4 py-2">{t(label)}</th>
                ))}</tr>
              </thead>
              <tbody>
                {members.data.rows.map((member) => (
                  <tr key={member.id} className="border-b border-line last:border-b-0">
                    <td data-mobile-label={t("Member number")} className="px-4 py-2 font-mono">{member.memberNumber}</td>
                    <td className="px-4 py-2">
                      <div>{member.name}</div>
                      {(member.email || member.phone) && <div className="text-xs text-muted">{[member.email, member.phone].filter(Boolean).join(" · ")}</div>}
                    </td>
                    <td data-mobile-label={t("Category")} className="px-4 py-2">{member.categoryName}</td>
                    <td data-mobile-label={t("Expiry date")} className="px-4 py-2">{member.expiresOn}</td>
                    <td className="whitespace-nowrap px-4 py-2">
                      <Button size="sm" variant="ghost" aria-label={t("Loans for {name} ({number})", { name: member.name, number: member.memberNumber })}
                        onClick={() => setLoanHistory(member)}>{t("Loans")}</Button>
                      <Button size="sm" variant="ghost" aria-label={t("Fines for {name} ({number})", { name: member.name, number: member.memberNumber })} onClick={() => setFineMember(member)}>{t("Fines")}</Button>
                      {canEdit && <>
                        {!deferred.archived && <>
                          <Button size="sm" variant="ghost" disabled={!config.data?.library}
                            aria-label={t("Card for {name} ({number})", { name: member.name, number: member.memberNumber })}
                            onClick={() => setCard(member)}>{t("Card")}</Button>
                          <Button size="sm" variant="ghost" disabled={!config.isSuccess}
                            onClick={() => setForm({ member })}>{t("Edit")}</Button>
                        </>}
                        <Button size="sm" variant="ghost" onClick={() => {
                          archive.reset();
                          setConfirm({ member, archive: !deferred.archived });
                        }}>
                          {deferred.archived ? t("Restore") : t("Archive")}
                        </Button>
                      </>}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          <Pager page={members.data.page} pageSize={members.data.pageSize} total={members.data.total}
            onPage={(page) => setFilter({ ...filter, page })} />
        </Panel>
      )}
      {form && <MemberForm member={form.member} categories={categories} onClose={() => setForm(null)} />}
      {fineMember && <MemberFines member={fineMember} onClose={() => setFineMember(null)} />}
      {loanHistory && <MemberLoanHistory member={loanHistory} onClose={() => setLoanHistory(null)} />}
      {card && config.data?.library && (
        <MemberCard member={card} libraryName={config.data.library.name} onClose={() => setCard(null)} />
      )}
      {importOpen && (
        <Suspense fallback={<p role="status" className="text-muted">{t("Loading patron import.")}</p>}>
          <KohaMemberImport categories={categories} onClose={() => setImportOpen(false)} />
        </Suspense>
      )}
      {confirm && (
        <Modal title={confirm.archive ? t("Archive member") : t("Restore member")} onClose={() => { if (!archive.isPending) setConfirm(null); }}
          footer={<>
            <Button variant="ghost" disabled={archive.isPending} onClick={() => setConfirm(null)}>{t("Cancel")}</Button>
            <Button disabled={archive.isPending} onClick={() => archive.mutate({ id: confirm.member.id, archived: confirm.archive })}>
              {confirm.archive ? t("Archive member") : t("Restore member")}
            </Button>
          </>}>
          <p>{confirm.member.name} ({confirm.member.memberNumber})</p>
          {archive.error && <p role="alert" className="text-danger">{toApiError(archive.error).message}</p>}
        </Modal>
      )}
    </div>
  );
}
