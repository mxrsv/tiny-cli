import type { Metadata } from "next";
import { SiteHeader } from "@/components/site-header";
import { SiteFooter } from "@/components/site-footer";
import { DocsLayout } from "@/components/docs/docs-layout";
import { InstallSection } from "@/components/docs/install-section";
import { FullDiskAccessSection } from "@/components/docs/full-disk-access-section";
import { CleanSection } from "@/components/docs/clean-section";
import { ProcessesSection } from "@/components/docs/processes-section";
import { CliSection } from "@/components/docs/cli-section";
import { StatusTag } from "@/components/docs/status-tag";
import styles from "./page.module.css";

export const metadata: Metadata = {
  title: "Docs: install, permissions, Clean, Processes and the CLI | tiny",
  description:
    "How to install tiny, grant Full Disk Access, clean caches safely, read the process list and use the open-source command line.",
};

export default function DocsPage() {
  return (
    <>
      <SiteHeader />
      <main>
        <DocsLayout>
          <header className={styles.intro}>
            <p className={`${styles.eyebrow} t-lead`}>Docs</p>
            <h1 className="t-headline">Set up tiny and clean safely</h1>
            <p className={`${styles.lead} t-body`}>
              Install it, give it the access it needs, and clean your Mac with a preview before anything
              moves.
            </p>
            <p className={`${styles.status} glass t-callout`}>
              tiny for Mac is not released yet. These pages cover the open-source command-line tool you can
              build today; anything that is not in the CLI yet carries a <StatusTag kind="planned">Planned for the app</StatusTag> label.
            </p>
          </header>
          <InstallSection />
          <FullDiskAccessSection />
          <CleanSection />
          <ProcessesSection />
          <CliSection />
        </DocsLayout>
      </main>
      <SiteFooter />
    </>
  );
}
