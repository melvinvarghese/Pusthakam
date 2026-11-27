import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { classificationApi, toApiError } from "@/lib/api";
import type { BrowseCategory } from "@/lib/api/types";

/**
 * Member-facing browse categories ("Malayalam Fiction", "Physics") shown as buttons in the public
 * catalogue instead of DDC numbers. DDC rules place books automatically; staff can adjust any book
 * from its record form.
 */
export function BrowseCategoriesTab({ writable }: { writable: boolean }) {
  const qc = useQueryClient();
  const [showArchived, setShowArchived] = useState(false);
  const [editing, setEditing] = useState<BrowseCategory | "new" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const query = useQuery({
    queryKey: ["browse-categories", showArchived],
    queryFn: () => classificationApi.listBrowseCategories(showArchived),
  });
  const archive = useMutation({
    mutationFn: ({ id, archived }: { id: number; archived: boolean }) =>
      classificationApi.setBrowseCategoryArchived(id, archived),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["browse-categories"] }),
    onError: (err) => setError(toApiError(err).message),
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <p className="max-w-2xl text-sm text-muted">
          Members browse the public catalogue by these categories. DDC numbers fill them in automatically,
          and staff can add or remove a category on any book from its record. Only categories that hold
          books are shown to members.
        </p>
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 text-sm">
            <input type="checkbox" checked={showArchived} onChange={(e) => setShowArchived(e.target.checked)} />
            Show archived
          </label>
          {writable && <Button onClick={() => setEditing("new")}>+ Add category</Button>}
        </div>
      </div>
      {error && <p role="alert" className="text-sm font-medium text-danger">{error}</p>}

      <div className="overflow-x-auto rounded-lg border border-line bg-surface">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-line bg-surface-hover text-xs font-medium text-muted">
            <tr>
              <th className="px-4 py-2.5">Category</th>
              <th className="px-4 py-2.5">DDC numbers that fill it</th>
              <th className="px-4 py-2.5">Books</th>
              <th className="px-4 py-2.5 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-line">
            {query.isLoading ? (
              <tr><td colSpan={4} className="px-4 py-8 text-center text-muted">Loading categories...</td></tr>
            ) : !query.data?.length ? (
              <tr><td colSpan={4} className="px-4 py-8 text-center text-muted">No categories found.</td></tr>
            ) : (
              query.data.map((c) => (
                <tr key={c.id} className="hover:bg-surface-hover">
                  <td className="px-4 py-2.5 font-medium">
                    {c.name}
                    {c.archived && <span className="ml-2 text-xs font-normal text-muted">(archived)</span>}
                  </td>
                  <td className="px-4 py-2.5 font-mono text-xs text-muted">
                    {c.ddcRules ? c.ddcRules.split(",").join(", ") : "Assigned by staff only"}
                  </td>
                  <td className="px-4 py-2.5 text-xs">
                    <span className={c.recordCount ? "" : "text-muted"}>{c.recordCount}</span>
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    {writable && (
                      <div className="flex items-center justify-end gap-2">
                        {!c.archived && (
                          <Button variant="ghost" className="h-7 px-2 text-xs" onClick={() => setEditing(c)}>Edit</Button>
                        )}
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          disabled={archive.isPending}
                          onClick={() => { setError(null); archive.mutate({ id: c.id, archived: !c.archived }); }}
                        >
                          {c.archived ? "Restore" : "Archive"}
                        </Button>
                      </div>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {editing && (
        <CategoryModal category={editing === "new" ? null : editing} onClose={() => setEditing(null)} />
      )}
    </div>
  );
}

function CategoryModal({ category, onClose }: { category: BrowseCategory | null; onClose: () => void }) {
  const qc = useQueryClient();
  const [name, setName] = useState(category?.name ?? "");
  const [rules, setRules] = useState(category?.ddcRules?.split(",").join(", ") ?? "");
  const [error, setError] = useState<string | null>(null);
  const save = useMutation({
    mutationFn: classificationApi.saveBrowseCategory,
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ["browse-categories"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    save.mutate({ id: category?.id ?? null, name: name.trim(), ddcRules: rules.trim() || null });
  };
  return (
    <Modal title={category ? "Edit category" : "Add category"} onClose={onClose}>
      <form onSubmit={submit} className="space-y-4">
        {error && <p role="alert" className="text-sm font-medium text-danger">{error}</p>}
        <TextField label="Category name" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. Kerala History" required maxLength={80} />
        <TextField
          label="DDC numbers (optional)"
          value={rules}
          onChange={(e) => setRules(e.target.value)}
          placeholder="e.g. 954.83"
          hint="Books whose DDC number starts with any of these go into this category automatically. Separate with commas. Leave empty to assign books by hand."
        />
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={save.isPending}>{save.isPending ? "Saving..." : "Save category"}</Button>
        </div>
      </form>
    </Modal>
  );
}
