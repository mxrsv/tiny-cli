import type { ReactNode } from "react";
import styles from "./app-screen.module.css";

function Icon({ children }: { children: ReactNode }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      {children}
    </svg>
  );
}

export type AppScreenStep = "processes" | "scan" | "clean";

type RowProps = { checked: boolean; title: string; detail: string; risk?: "Safe" | "Review"; size: string; gone?: boolean };

function Row({ checked, title, detail, risk = "Safe", size, gone }: RowProps) {
  return (
    <div className={`${styles.row}${gone ? ` ${styles.gone}` : ""}`}>
      <span className={`${styles.box}${checked ? ` ${styles.on}` : ""}`} />
      <span>{title}<small>{detail}</small></span>
      <span className={`${styles.risk}${risk === "Review" ? ` ${styles.review}` : ""}`}>{risk}</span>
      <span className={styles.size}>{size}</span>
    </div>
  );
}

type ProcRowProps = { title: string; cpu: string; mem: string; share: number };

function ProcRow({ title, cpu, mem, share }: ProcRowProps) {
  return (
    <div className={styles.prow}>
      <span>{title}</span>
      <span className={styles.num}>{cpu}</span>
      <span className={styles.num}>{mem}</span>
      <span className={styles.share}><i style={{ width: `${share}%` }} /></span>
    </div>
  );
}

function ProcessesPane() {
  return (
    <>
      <div>
        <h4>Processes</h4>
        <div className={styles.sub}>CPU and memory use right now.</div>
      </div>
      <div className={styles.sum}>
        <div>CPU<b>62%</b></div>
        <div>Memory<b>14.2 GB of 24 GB</b></div>
        <div>Showing<b>6 processes</b></div>
      </div>
      <div className={styles.rows}>
        <div className={`${styles.prow} ${styles.phead}`}><span>Process</span><span className={styles.num}>CPU</span><span className={styles.num}>Memory</span><span /></div>
        <ProcRow title="Docker Desktop" cpu="4%" mem="3.1 GB" share={100} />
        <ProcRow title="Xcode" cpu="38%" mem="2.4 GB" share={77} />
        <ProcRow title="Safari" cpu="6%" mem="1.1 GB" share={35} />
        <ProcRow title="WindowServer" cpu="5%" mem="0.7 GB" share={23} />
        <ProcRow title="Slack" cpu="1%" mem="0.6 GB" share={19} />
        <ProcRow title="Finder" cpu="2%" mem="0.3 GB" share={10} />
      </div>
    </>
  );
}

function ScanFile({ title, detail, size }: { title: string; detail: string; size: string }) {
  return (
    <div className={styles.srow}>
      <Icon><path d="M7 3h7l4 4v14H7z" /><path d="M14 3v4h4" /></Icon>
      <span>{title}<small>{detail}</small></span>
      <span className={styles.size}>{size}</span>
    </div>
  );
}

function ScanPane() {
  return (
    <>
      <div>
        <h4>Scan<span className={styles.soon}>Available in the CLI</span></h4>
        <div className={styles.sub}>A read-only look at Downloads, Desktop and Documents. tiny never changes these files. Not in the first app release.</div>
      </div>
      <div className={styles.sum}>
        <div>Files measured<b>48,213</b></div>
        <div>Large files, 100 MB or more<b>37</b></div>
        <div>Not modified in 90 days<b>212</b></div>
      </div>
      <div className={styles.rows}>
        <ScanFile title="macOS-installer.dmg" detail="~/Downloads, not modified in 13 months" size="4.7 GB" />
        <ScanFile title="Project-recording.mov" detail="~/Documents, not modified in 8 months" size="3.9 GB" />
        <ScanFile title="Ubuntu-24.04.iso" detail="~/Downloads, not modified in 11 months" size="2.6 GB" />
        <ScanFile title="Screen Recording.mov" detail="~/Desktop, not modified in 5 months" size="1.8 GB" />
        <ScanFile title="Archive.zip" detail="~/Documents, not modified in 2 years" size="1.2 GB" />
      </div>
      <div className={styles.foot}>
        <span className={styles.readonly}>Read-only</span>
      </div>
    </>
  );
}

function CleanPane({ moved }: { moved: boolean }) {
  return (
    <>
      <div>
        <h4>Clean</h4>
        <div className={styles.sub}>Everything below is moved to the Trash, so Put Back works until the Trash is emptied.</div>
      </div>
      <div className={styles.sum}>
        <div>Selected<b>{moved ? "0 GB" : "9.8 GB"}</b></div>
        <div>Found<b>{moved ? "8.8 GB" : "18.6 GB"}</b></div>
        <div>Skipped while app is open<b>Docker</b></div>
      </div>
      <div className={styles.rows}>
        <Row gone={moved} checked title="Xcode DerivedData" detail="~/Library/Developer/Xcode/DerivedData" size="9.8 GB" />
        <Row checked={false} title="node_modules idle 30+ days" detail="14 projects under ~/Code" risk="Review" size="5.1 GB" />
        <Row checked={false} title="Browser caches" detail="Safari, Chrome, Arc" risk="Review" size="2.3 GB" />
        <Row checked={false} title="iOS Simulators" detail="3 unavailable runtimes" risk="Review" size="1.4 GB" />
      </div>
      <div className={styles.foot}>
        <span className={styles.go}>{moved ? "Moved to Trash" : "Move 9.8 GB to Trash"}</span>
      </div>
      {moved && <div className={styles.toast} role="status">9.8 GB moved to the Trash</div>}
    </>
  );
}

const PANES: Record<AppScreenStep, (moved: boolean) => ReactNode> = {
  processes: () => <ProcessesPane />,
  scan: () => <ScanPane />,
  clean: (moved) => <CleanPane moved={moved} />,
};

type AppScreenProps = {
  /** Which app screen to draw. */
  step?: AppScreenStep;
  /** Clean only: the picked items have moved to the Trash. */
  moved?: boolean;
};

/**
 * Stand-in for the app's screens, laid out at 1440 x 900. CSS scales it to the
 * width of its parent frame, so the server HTML already shows the picture. The
 * frame must be an inline-size container and may crop to a region of the
 * screen by setting --crop-x, --crop-y and --crop-w (design pixels); without
 * them the whole screen fits.
 */
export function AppScreen({ step = "clean", moved = false }: AppScreenProps) {
  const navClass = (id: AppScreenStep) => `${styles.nav}${id === step ? ` ${styles.on}` : ""}`;

  return (
    <div className={styles.screen} aria-hidden="true">
      <div className={styles.menubar}>
        <span>{""}</span><b>tiny</b><span>File</span><span>Edit</span><span>View</span><span>Window</span>
        <span className={styles.right}><span>100%</span><span>Fri 9 Oct 19:40</span></span>
      </div>
      <div className={styles.win}>
        <div className={styles.side}>
          <div className={styles.lights}><i /><i /><i /></div>
          <div className={navClass("processes")}><Icon><path d="M3 12h4l3-8 4 16 3-8h4" /></Icon>Processes</div>
          <div className={navClass("clean")}><Icon><path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13" /></Icon>Clean</div>
          <div className={navClass("scan")}>
            <Icon><circle cx="11" cy="11" r="6" /><path d="m20 20-4.5-4.5" /></Icon>Scan<span className={styles.soon}>Available in the CLI</span>
          </div>
          <div className={styles.free}>
            Free on Macintosh HD<b>61.4 GB</b>of 494.4 GB<div className={styles.bar}><i /></div>
          </div>
        </div>
        <div className={styles.main}>{PANES[step](moved)}</div>
      </div>
    </div>
  );
}
