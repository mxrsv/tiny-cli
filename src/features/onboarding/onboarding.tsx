import { useMutation } from "@tanstack/react-query";
import { ArrowRight, LockKeyhole, RefreshCw } from "lucide-react";
import { ipc } from "../../lib/ipc";
import { usePermissions } from "../../hooks/use-engine";
import { ErrorNotice } from "../../components/error-notice";
export function Onboarding() {
  const permission = usePermissions();
  const open = useMutation({ mutationFn: ipc.openPermissions });
  if (permission.data?.status !== "required") return null;
  return (
    <div className="onboarding-overlay">
      <section className="onboarding-card">
        <div className="permission-icon">
          <LockKeyhole size={32} />
        </div>
        <div className="eyebrow">A SMALL FIRST STEP</div>
        <h1>A clearer view of your Mac.</h1>
        <p>
          macOS keeps some folders private. Full Disk Access lets Tiny measure
          them and show a complete cleanup plan.
        </p>
        <ol>
          <li>Open System Settings → Privacy & Security.</li>
          <li>Choose Full Disk Access and add Tiny.</li>
          <li>Turn access on, then quit and reopen Tiny.</li>
        </ol>
        <p className="caption">
          Tiny cannot grant this permission itself. Your files stay on your Mac,
          and cleanup still requires your confirmation.
        </p>
        <ErrorNotice error={open.error} />
        <button className="button primary" onClick={() => open.mutate()}>
          Open System Settings <ArrowRight size={17} />
        </button>
        <button
          className="text-button"
          onClick={() => {
            void permission.refetch();
          }}
        >
          <RefreshCw size={14} />
          Check access again
        </button>
      </section>
    </div>
  );
}
