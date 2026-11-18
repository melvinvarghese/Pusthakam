import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { BarcodeSvg } from "@/features/catalogue/shared";
import { membersApi, toApiError } from "@/lib/api";
import type { Member } from "@/lib/api/types";

import { prepareImage as preparePhoto } from "@/lib/images";

export function MemberCard({ member, libraryName, onClose }: {
  member: Member;
  libraryName: string;
  onClose: () => void;
}) {
  const hasBarcode = /^[ -~]+$/.test(member.memberNumber);
  const client = useQueryClient();
  const [confirmRemove, setConfirmRemove] = useState(false);
  const photo = useQuery({
    queryKey: ["member-photo", member.id],
    queryFn: () => membersApi.getPhoto(member.id),
    retry: false,
  });
  const save = useMutation({
    mutationFn: async (file: File | null) => {
      const data = file ? await preparePhoto(file) : null;
      await membersApi.setPhoto(member.id, data);
    },
    onSuccess: async () => {
      setConfirmRemove(false);
      await client.invalidateQueries({ queryKey: ["member-photo", member.id] });
    },
  });
  const busy = photo.isPending || save.isPending;

  return (
    <Modal title="Member card" onClose={() => { if (!save.isPending) onClose(); }} footer={<>
      <Button disabled={save.isPending} onClick={onClose}>Close</Button>
      <Button variant="primary" disabled={busy || photo.isError} onClick={() => window.print()}>Print card</Button>
    </>}>
      <div className="print-area flex min-h-[54mm] w-[85.6mm] max-w-full flex-col justify-between gap-2 rounded-md border border-line bg-white p-4 text-black">
        <header className="break-words border-b border-black/20 pb-2 text-sm font-semibold">{libraryName}</header>
        <div className="flex items-center gap-3">
          <div className="min-w-0 flex-1 break-words">
            <p className="font-semibold">{member.name}</p>
            <p className="text-xs">{member.categoryName} · Valid through {member.expiresOn}</p>
          </div>
          {photo.data && (
            <img className="size-[14mm] shrink-0 object-contain" src={`data:image/png;base64,${photo.data}`} alt="" />
          )}
        </div>
        {hasBarcode ? (
          <div className="[&_svg]:h-auto [&_svg]:max-w-full">
            <BarcodeSvg value={member.memberNumber} height={34} />
          </div>
        ) : <p className="break-all font-mono text-xs">{member.memberNumber}</p>}
      </div>
      {!hasBarcode && <p className="text-sm text-muted">This member number prints as text because it contains characters Code 128 cannot encode.</p>}
      {photo.isError && <p role="alert" className="text-sm text-danger">{toApiError(photo.error).message}</p>}
      {save.error && <p role="alert" className="text-sm text-danger">{toApiError(save.error).message}</p>}
      <TextField label="Member photo" type="file" accept=".png,.jpg,.jpeg,image/png,image/jpeg" disabled={save.isPending}
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          if (file) save.mutate(file);
        }} />
      {photo.data && !confirmRemove && (
        <Button disabled={save.isPending} onClick={() => setConfirmRemove(true)}>Remove photo</Button>
      )}
      {confirmRemove && <div className="flex gap-2">
        <Button disabled={save.isPending} onClick={() => setConfirmRemove(false)}>Keep photo</Button>
        <Button disabled={save.isPending} onClick={() => save.mutate(null)}>Confirm removal</Button>
      </div>}
    </Modal>
  );
}
