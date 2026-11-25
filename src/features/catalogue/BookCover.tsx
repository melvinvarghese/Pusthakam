import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { catalogueApi, toApiError } from "@/lib/api";
import { fetchIsbnCover, prepareImage } from "@/lib/images";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Panel } from "@/components/ui/Panel";

export function BookCover({ id, title, isbn, editable, compact = false }: { id: number; title: string; isbn: string | null; editable: boolean; compact?: boolean }) {
  const client = useQueryClient();
  const attempted = useRef<string | null>(null);
  const [failedIsbn, setFailedIsbn] = useState<string | null>(null);
  const [loadedIsbn, setLoadedIsbn] = useState<string | null>(null);
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
  const code = isbn?.replace(/[-\s]/g, "") ?? "";
  const direct = lookup.isError && /^(?:\d{13}|\d{9}[\dXx])$/.test(code) && failedIsbn !== code
    ? `https://covers.openlibrary.org/b/isbn/${code}-M.jpg?default=false` : null;
  const imageSrc = image ? `data:image/png;base64,${image}` : direct;
  const error = save.error ?? cover.error ?? (loadedIsbn !== code ? lookup.error : null);
  const picture = imageSrc
    ? <img src={imageSrc} alt={`Cover of ${title}`} loading="lazy" referrerPolicy="no-referrer"
        onError={() => { if (direct) setFailedIsbn(code); }}
        onLoad={() => { if (direct) setLoadedIsbn(code); }}
        className={compact ? "h-20 w-14 shrink-0 object-contain" : "max-h-64 max-w-full object-contain"} />
    : <span className={compact ? "grid h-20 w-14 shrink-0 place-items-center rounded border border-line bg-band text-center text-[10px] text-muted" : "block text-sm text-muted"}>
        {busy || cover.isPending ? "Loading cover…" : "No cover available"}
      </span>;
  if (compact) return <div aria-label={`Book cover for ${title}`} title={error ? toApiError(error).message : undefined}>{picture}</div>;
  return <Panel aria-label="Book cover" className="space-y-3 p-4">
    <h2 className="font-semibold">Book cover</h2>
    {picture}
    {!imageSrc && isbn && !busy && <Button variant="secondary" onClick={() => { setFailedIsbn(null); loadCover(); }}>Retry ISBN cover</Button>}
    {!isbn && !image && <p className="text-sm text-muted">Add an ISBN to find a cover automatically, or choose an image.</p>}
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
