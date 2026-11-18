import { useEffect, useRef } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { catalogueApi, toApiError } from "@/lib/api";
import { fetchIsbnCover, prepareImage } from "@/lib/images";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";

export function BookCover({ id, title, isbn, editable }: { id: number; title: string; isbn: string | null; editable: boolean }) {
  const client = useQueryClient();
  const attempted = useRef<string | null>(null);
  const key = ["catalogue", "cover", id];
  const cover = useQuery({ queryKey: key, queryFn: () => catalogueApi.getCover(id), retry: false });
  const save = useMutation({
    mutationFn: async (file: File | null) => catalogueApi.setCover(id, file ? await prepareImage(file) : null),
    onSuccess: () => client.invalidateQueries({ queryKey: key }),
  });
  const lookup = useMutation({
    mutationFn: async () => {
      if (!isbn) return;
      const image = await fetchIsbnCover(isbn);
      if (editable) await catalogueApi.setCover(id, image);
      return image;
    },
    onSuccess: () => client.invalidateQueries({ queryKey: key }),
  });
  const { mutate: loadCover } = lookup;
  useEffect(() => {
    function load() {
      if (!isbn || !cover.isSuccess || cover.data || attempted.current === isbn || !navigator.onLine) return;
      attempted.current = isbn;
      loadCover();
    }
    function reconnect() { attempted.current = null; load(); }
    load();
    window.addEventListener("online", reconnect);
    return () => window.removeEventListener("online", reconnect);
  }, [isbn, cover.isSuccess, cover.data, loadCover]);
  const image = cover.data ?? lookup.data;
  const busy = save.isPending || lookup.isPending;
  const error = save.error ?? cover.error;
  return <Panel aria-label="Book cover" className="space-y-3 p-4">
    <h2 className="font-semibold">Book cover</h2>
    {image && <img src={`data:image/png;base64,${image}`} alt={`Cover of ${title}`} className="max-h-64 max-w-full object-contain" />}
    {editable && <TextField label="Choose book cover" type="file" accept=".jpg,.jpeg,.png" disabled={busy}
      onChange={(event) => {
        const file = event.target.files?.[0];
        event.target.value = "";
        if (file) { lookup.reset(); save.mutate(file); }
      }} />}
    {lookup.isPending && <p role="status" className="text-sm text-muted">Loading book cover.</p>}
    {editable && cover.data && <Button disabled={busy} onClick={() => { lookup.reset(); save.mutate(null); }}>Remove cover</Button>}
    {error && <p role="alert" className="text-danger">{toApiError(error).message}</p>}
  </Panel>;
}
