"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

/** Link that marks itself `aria-current="page"` when its path is the current one. */
export function NavLink({ href, className, children }: { href: string; className?: string; children: ReactNode }) {
  const pathname = usePathname();
  return (
    <Link className={className} href={href} aria-current={href === pathname ? "page" : undefined}>
      {children}
    </Link>
  );
}
