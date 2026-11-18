import { useState } from "react";
import { TextField } from "@/components/ui/Field";
import { useConfig } from "./shared";

export function useLabelHeading() {
  const config = useConfig();
  const [custom, setCustom] = useState<string | null>(null);
  const heading = custom ?? config.data?.library?.name ?? "";
  const field = <TextField label="Label heading" value={heading} maxLength={120}
    onChange={(event) => setCustom(event.target.value)}
    hint="Use your library name, enter custom text, or leave blank. Applies to this print job." />;
  return { heading, field };
}
