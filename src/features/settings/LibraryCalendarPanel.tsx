import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { configApi, toApiError } from "@/lib/api";
import type { Holiday } from "@/lib/api/types";
import { Button } from "@/components/ui/Button";
import { CheckField, TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Panel, PanelHeader } from "@/components/ui/Panel";

const DAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
export function LibraryCalendarPanel({ canEdit }: { canEdit: boolean }) {
  const client = useQueryClient();
  const calendar = useQuery({ queryKey: ["library-calendar"], queryFn: configApi.calendar, retry: false });
  const [draft, setDraft] = useState<number[] | null>(null);
  const [editing, setEditing] = useState<{ id: number | null; date: string; name: string } | null>(null);
  const [removing, setRemoving] = useState<Holiday | null>(null);
  const refresh = () => client.invalidateQueries({ queryKey: ["library-calendar"] });
  const weekly = useMutation({ mutationFn: configApi.saveClosedWeekdays, onSuccess: async () => { setDraft(null); await refresh(); } });
  const save = useMutation({ mutationFn: configApi.saveHoliday, onSuccess: async () => { setEditing(null); await refresh(); } });
  const remove = useMutation({ mutationFn: configApi.removeHoliday, onSuccess: async () => { setRemoving(null); await refresh(); } });
  const days = draft ?? calendar.data?.closedWeekdays ?? [];
  return <Panel aria-labelledby="calendar-heading">
    <PanelHeader id="calendar-heading" title="Library calendar" aside={canEdit && <Button size="sm" disabled={!calendar.data || calendar.isError} onClick={() => { save.reset(); setEditing({ id: null, date: "", name: "" }); }}>Add holiday</Button>} />
    <div className="space-y-4 p-4">
      <p className="text-xs text-muted">Issue and renewal due dates move forward to the next open day when they fall on a closed weekday or holiday. Existing due dates stay unchanged. Fine rules are unchanged.</p>
      {calendar.isPending && <p role="status">Loading calendar…</p>}
      {calendar.isError && <div role="alert"><p className="text-danger">{toApiError(calendar.error).message}</p><Button onClick={() => void calendar.refetch()}>Retry calendar</Button></div>}
      {calendar.data && !calendar.isError && <>
        <fieldset disabled={!canEdit || weekly.isPending}>
          <legend className="mb-2 text-sm font-semibold">Weekly closed days</legend>
          <div className="flex flex-wrap gap-4">{DAYS.map((day, index) => <CheckField key={day} label={day} checked={days.includes(index)} onChange={(e) => { const checked = e.target.checked; setDraft(checked ? [...days, index] : days.filter((d) => d !== index)); weekly.reset(); }} />)}</div>
        </fieldset>
        {canEdit && <Button disabled={draft === null || weekly.isPending || days.length === 7} onClick={() => weekly.mutate(days)}>Save closed days</Button>}
        {days.length === 7 && <p role="alert" className="text-danger">Leave at least one weekday open.</p>}
        {weekly.isSuccess && draft === null && <p role="status">Closed days saved.</p>}
        {weekly.isError && <p role="alert" className="text-danger">{toApiError(weekly.error).message}</p>}
        {calendar.data.holidays.length === 0 ? <p className="text-sm text-muted">No holiday dates configured.</p> : <table className="w-full text-sm">
          <thead><tr className="text-left"><th className="py-2">Date</th><th>Holiday</th>{canEdit && <th><span className="sr-only">Actions</span></th>}</tr></thead>
          <tbody>{calendar.data.holidays.map((holiday) => <tr key={holiday.id} className="border-t border-line"><td className="py-2">{holiday.date}</td><td>{holiday.name}</td>{canEdit && <td className="text-right">
            <Button size="sm" variant="ghost" aria-label={`Edit holiday ${holiday.name}`} onClick={() => { save.reset(); setEditing(holiday); }}>Edit</Button>
            <Button size="sm" variant="ghost" aria-label={`Remove holiday ${holiday.name}`} onClick={() => { remove.reset(); setRemoving(holiday); }}>Remove</Button>
          </td>}</tr>)}</tbody>
        </table>}
      </>}
    </div>
    {editing && <Modal title={editing.id === null ? "Add holiday" : "Edit holiday"} onClose={() => { if (!save.isPending) setEditing(null); }} footer={<>
      <Button disabled={save.isPending} onClick={() => setEditing(null)}>Cancel</Button>
      <Button variant="primary" disabled={save.isPending || !editing.date || !editing.name.trim()} onClick={() => save.mutate(editing)}>Save holiday</Button>
    </>}>
      <TextField label="Holiday date" type="date" value={editing.date} onChange={(e) => setEditing({ ...editing, date: e.target.value })} />
      <TextField label="Holiday name" maxLength={120} value={editing.name} onChange={(e) => setEditing({ ...editing, name: e.target.value })} />
      {save.isError && <p role="alert" className="text-danger">{toApiError(save.error).message}</p>}
    </Modal>}
    {removing && <Modal title="Remove holiday" onClose={() => { if (!remove.isPending) setRemoving(null); }} footer={<>
      <Button disabled={remove.isPending} onClick={() => setRemoving(null)}>Cancel</Button>
      <Button variant="primary" disabled={remove.isPending} onClick={() => remove.mutate(removing.id)}>Remove holiday</Button>
    </>}><p>Remove {removing.name} ({removing.date}) from the calendar?</p>{remove.isError && <p role="alert" className="text-danger">{toApiError(remove.error).message}</p>}</Modal>}
  </Panel>;
}
