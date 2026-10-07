import { Sprout } from "lucide-react";
import type { ReactNode } from "react";
export function EmptyState({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="empty-state">
      <Sprout size={36} />
      <h3>{title}</h3>
      <p>{children}</p>
    </div>
  );
}
