import { Button } from "@/components/ui/Button";

interface Props {
  page: number;
  pageSize: number;
  total: number;
  onPage: (page: number) => void;
}

/** "1-25 of 312" with previous and next. Pages are served by the backend; nothing is loaded wholesale. */
export function Pager({ page, pageSize, total, onPage }: Props) {
  const first = total === 0 ? 0 : (page - 1) * pageSize + 1;
  const last = Math.min(page * pageSize, total);
  const pages = Math.max(1, Math.ceil(total / pageSize));
  return (
    <div className="flex items-center justify-between border-t border-line px-4 py-2 text-xs text-muted">
      <span aria-live="polite">{total === 0 ? "No results" : `${first}–${last} of ${total}`}</span>
      <div className="flex items-center gap-2">
        <Button size="sm" disabled={page <= 1} onClick={() => onPage(page - 1)}>Previous</Button>
        <span>Page {page} of {pages}</span>
        <Button size="sm" disabled={page >= pages} onClick={() => onPage(page + 1)}>Next</Button>
      </div>
    </div>
  );
}
