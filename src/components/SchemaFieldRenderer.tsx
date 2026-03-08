/* ============================================================
   SchemaFieldRenderer - configSchema ベースの動的フォーム
   WidgetSettingsWindow 等で共有する汎用コンポーネント群
   ============================================================ */
import type { ConfigSchemaField } from "../types";
import { invoke } from "@tauri-apps/api/core";

interface FieldProps {
  field: ConfigSchemaField;
  config: Record<string, unknown>;
  update: (key: string, val: unknown) => void;
}

/** configSchema の単一フィールドをレンダリング */
export function SchemaField({ field, config, update }: FieldProps) {
  switch (field.type) {
    case "color":
      return <ColorField label={field.label} k={field.key} config={config} update={update} />;
    case "checkbox":
      return <CheckboxField label={field.label} k={field.key} config={config} update={update} />;
    case "select":
      return (
        <SelectField
          label={field.label}
          k={field.key}
          config={config}
          update={update}
          options={(field.options ?? []).map((o) => ({ value: o.value, label: o.label }))}
        />
      );
    case "text":
      return (
        <div className="ws-field">
          <label>{field.label}</label>
          <input
            type="text"
            value={(config[field.key] as string) ?? ""}
            onChange={(e) => update(field.key, e.target.value)}
          />
        </div>
      );
    case "number":
      return (
        <div className="ws-field">
          <label>{field.label}</label>
          <input
            type="number"
            value={(config[field.key] as number) ?? field.default ?? 0}
            min={field.min}
            max={field.max}
            step={field.step}
            onChange={(e) => update(field.key, Number(e.target.value))}
          />
        </div>
      );
    case "datetime":
      return (
        <div className="ws-field">
          <label>{field.label}</label>
          <input
            type="datetime-local"
            value={toLocalDatetime((config[field.key] as string) ?? "")}
            onChange={(e) => update(field.key, new Date(e.target.value).toISOString())}
          />
        </div>
      );
    case "file":
      return <FileField label={field.label} k={field.key} config={config} update={update} />;
    default:
      return null;
  }
}

/* ── ヘルパーコンポーネント ── */

interface BasicFieldProps {
  label: string;
  k: string;
  config: Record<string, unknown>;
  update: (key: string, val: unknown) => void;
}

function ColorField({ label, k, config, update }: BasicFieldProps) {
  const val = (config[k] as string) ?? "#000000";
  const colorVal = val === "transparent" ? "#1e1e2e" : val;
  return (
    <div className="ws-field">
      <label>{label}</label>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <input
          type="color"
          value={colorVal}
          onChange={(e) => update(k, e.target.value)}
        />
        <span style={{ fontSize: 10, color: "var(--text-muted)" }}>{val}</span>
      </div>
    </div>
  );
}

function CheckboxField({ label, k, config, update }: BasicFieldProps) {
  return (
    <div className="ws-field" style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
      <input
        type="checkbox"
        checked={!!config[k]}
        onChange={(e) => update(k, e.target.checked)}
        style={{ accentColor: "var(--accent-color)" }}
      />
      <label>{label}</label>
    </div>
  );
}

interface SelectFieldProps extends BasicFieldProps {
  options: { value: string; label: string }[];
}

function SelectField({ label, k, config, update, options }: SelectFieldProps) {
  return (
    <div className="ws-field">
      <label>{label}</label>
      <select
        value={(config[k] as string) ?? options[0]?.value}
        onChange={(e) => update(k, e.target.value)}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    </div>
  );
}

function FileField({ label, k, config, update }: BasicFieldProps) {
  const val = (config[k] as string) ?? "";
  const handleBrowse = async () => {
    try {
      const selected = await invoke<string>("pick_sound_file");
      if (selected) {
        update(k, selected);
      }
    } catch (err) {
      console.error("File picker error:", err);
    }
  };
  // ファイル名だけ表示（パスが長い場合）
  const displayName = val ? val.replace(/^.*[\\/]/, "") : "";
  return (
    <div className="ws-field">
      <label>{label}</label>
      <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
        <input
          type="text"
          value={val}
          placeholder="ファイルを選択..."
          onChange={(e) => update(k, e.target.value)}
          style={{ flex: 1, minWidth: 0 }}
          title={val}
        />
        <button
          type="button"
          onClick={handleBrowse}
          style={{
            padding: "4px 10px",
            fontSize: 11,
            cursor: "pointer",
            background: "var(--bg-secondary)",
            border: "1px solid var(--border-color)",
            borderRadius: "var(--border-radius-sm)",
            color: "var(--text-primary)",
            whiteSpace: "nowrap",
          }}
        >
          📂 参照
        </button>
      </div>
      {displayName && (
        <span style={{ fontSize: 10, color: "var(--text-muted)", marginTop: 2 }}>
          {displayName}
        </span>
      )}
    </div>
  );
}

/** ISO 日時文字列 → datetime-local 入力用フォーマット */
function toLocalDatetime(isoString: string): string {
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return "";
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  } catch {
    return "";
  }
}
