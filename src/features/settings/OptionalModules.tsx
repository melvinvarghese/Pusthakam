import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { moduleVisibilityOptions, OPTIONAL_MODULES } from "@/app/nav";
import { Panel } from "@/components/ui/Panel";
import { appApi, toApiError } from "@/lib/api";

function ModuleVisibility({ path, label, canEdit }: { path: string; label: string; canEdit: boolean }) {
  const client = useQueryClient();
  const options = moduleVisibilityOptions(path);
  const visibility = useQuery(options);
  const save = useMutation({
    mutationFn: (visible: boolean) => appApi.setSetting(`modules.${path.slice(1)}`, String(visible)),
    onSuccess: (_, visible) => client.setQueryData(options.queryKey, visible),
  });
  const error = save.error ?? visibility.error;
  return (
    <div>
      <label className="flex items-center gap-2">
        <input type="checkbox" checked={visibility.data === true}
          disabled={!canEdit || !visibility.isSuccess || save.isPending}
          onChange={(event) => save.mutate(event.target.checked)} />
        Show {label}
      </label>
      {error && <p role="alert" className="text-sm text-danger">{toApiError(error).message}</p>}
    </div>
  );
}

export function OptionalModules({ canEdit }: { canEdit: boolean }) {
  return (
    <Panel aria-labelledby="optional-modules" className="space-y-3 p-4">
      <h2 id="optional-modules" className="font-semibold">Optional modules</h2>
      <p className="text-sm text-muted">Choose which optional modules appear in navigation. Acquisitions and Periodicals are planned features.</p>
      {OPTIONAL_MODULES.map((module) => <ModuleVisibility key={module.path} {...module} canEdit={canEdit} />)}
    </Panel>
  );
}
