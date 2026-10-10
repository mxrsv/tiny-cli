import styles from "./widget-tile.module.css";

type WidgetTileProps = {
  label: string;
  value: string;
  /** Render the value with tabular figures. */
  numeric?: boolean;
  /** Disk-style meter, 0 to 100, with its accessible description. */
  meter?: { percent: number; description: string };
  note?: string;
  /** Names the tile for the hero story, which lights it with its step. */
  tile?: string;
  className?: string;
};

/** Apple-widget style glass tile. Position is the caller's job (className). */
export function WidgetTile({ label, value, numeric, meter, note, tile, className = "" }: WidgetTileProps) {
  return (
    <div className={`${styles.tile} glass ${className}`} data-tile={tile}>
      <p className={`${styles.label} t-callout`}>{label}</p>
      <p className={`${styles.value} t-title${numeric ? " t-num" : ""}`}>{value}</p>
      {meter && (
        <div className={styles.meter} role="img" aria-label={meter.description}>
          <i style={{ width: `${meter.percent}%` }} />
        </div>
      )}
      {note && <p className={`${styles.state} t-footnote`}>{note}</p>}
    </div>
  );
}
