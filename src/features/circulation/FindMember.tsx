import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { membersApi, toApiError } from "@/lib/api";

export function FindMember({ onClose, onSelect }: { onClose: () => void; onSelect: (number: string) => void }) {
  const [filter, setFilter] = useState({ name: "", page: 1 });
  const members = useQuery({
    queryKey: ["circulation", "member-lookup", filter],
    queryFn: () => membersApi.list(filter.name, filter.page, false),
    enabled: Boolean(filter.name),
  });
  const result = members.data;
  return (
    <Modal title="Find member by name" onClose={onClose}>
      <form className="flex items-end gap-2" onSubmit={(event) => {
        event.preventDefault();
        const name = String(new FormData(event.currentTarget).get("name") ?? "").trim();
        setFilter({ name, page: 1 });
      }}>
        <TextField label="Member name" name="name" required />
        <Button type="submit">Search</Button>
      </form>
      {members.isFetching && <p role="status">Searching members.</p>}
      {members.isError && <p role="alert" className="text-danger">{toApiError(members.error).message}</p>}
      {result && <>
        {result.rows.length === 0 && <p>No matching members.</p>}
        <ul className="space-y-2 py-2">
          {result.rows.map((member) => <li key={member.id}>
            <Button type="button" onClick={() => onSelect(member.memberNumber)}>
              {member.name} ({member.memberNumber})
            </Button>
          </li>)}
        </ul>
        <div className="flex gap-2">
          <Button type="button" disabled={filter.page <= 1} onClick={() => setFilter({ ...filter, page: filter.page - 1 })}>Previous</Button>
          <Button type="button" disabled={result.page * result.pageSize >= result.total}
            onClick={() => setFilter({ ...filter, page: filter.page + 1 })}>Next</Button>
        </div>
      </>}
    </Modal>
  );
}
