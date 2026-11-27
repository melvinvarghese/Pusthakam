import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Panel } from "@/components/ui/Panel";
import { appApi, toApiError } from "@/lib/api";
import { staffHttpConfig } from "@/lib/api/staffHttp";

/** Start Pusthakam in the system tray when this computer's user signs in. Desktop on Windows only. */
export function StartupPanel({ canEdit }: { canEdit: boolean }) {
  const client = useQueryClient();
  const browserStaff = staffHttpConfig() !== null;
  const startup = useQuery({ queryKey: ["launch-at-startup"], queryFn: appApi.startup, enabled: !browserStaff, retry: false });
  const save = useMutation({
    mutationFn: appApi.setStartup,
    onSuccess: (setting) => client.setQueryData(["launch-at-startup"], setting),
  });
  if (browserStaff || !startup.data?.supported) return null;
  return (
    <Panel aria-labelledby="startup-h" className="space-y-2 p-4">
      <h2 id="startup-h" className="font-semibold">Start with Windows</h2>
      <label className="flex items-center gap-2">
        <input type="checkbox" checked={startup.data.enabled} disabled={!canEdit || save.isPending}
          onChange={(event) => save.mutate(event.target.checked)} />
        Open Pusthakam in the system tray when this computer starts
      </label>
      <p className="text-sm text-muted">The daily backup runs when Pusthakam first opens each day, so starting with Windows keeps backups regular. Find Pusthakam under the taskbar&apos;s hidden icons. Closing the window keeps it running there; right-click the icon and choose Quit to close it.</p>
      {save.isError && <p role="alert" className="text-sm text-danger">{toApiError(save.error).message}</p>}
    </Panel>
  );
}
