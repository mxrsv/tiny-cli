import { AlertCircle } from "lucide-react";
import { errorMessage } from "../lib/format";
export function ErrorNotice({ error }: { error: unknown }) {
  if (!error) return null;
  return (
    <div className="error-notice" role="alert">
      <AlertCircle size={17} />
      <span>{errorMessage(error)}</span>
    </div>
  );
}
