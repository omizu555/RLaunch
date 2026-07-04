//! 実ユーザーデータ（%APPDATA%/com.rlaunch.app/launcher-data.json）との互換テスト。
//! ファイルが無い環境では自動スキップ。
//! 「元ファイルにあるキー/値は、パース→再シリアライズ後もすべて保持される」ことを検証する
//! （こちらが新たに付与するデフォルト値の追加は許容）。

use rlaunch::model::data::LauncherData;
use serde_json::Value;

fn assert_subset(original: &Value, output: &Value, path: &str, problems: &mut Vec<String>) {
    match (original, output) {
        (Value::Object(o), Value::Object(n)) => {
            for (k, v) in o {
                match n.get(k) {
                    Some(nv) => assert_subset(v, nv, &format!("{}.{}", path, k), problems),
                    None => problems.push(format!("{}.{} が出力に存在しない", path, k)),
                }
            }
        }
        (Value::Array(o), Value::Array(n)) => {
            if o.len() != n.len() {
                problems.push(format!(
                    "{} の配列長が変化: {} → {}",
                    path,
                    o.len(),
                    n.len()
                ));
                return;
            }
            for (i, (ov, nv)) in o.iter().zip(n.iter()).enumerate() {
                assert_subset(ov, nv, &format!("{}[{}]", path, i), problems);
            }
        }
        _ => {
            if original != output {
                problems.push(format!("{} の値が変化: {} → {}", path, original, output));
            }
        }
    }
}

#[test]
fn real_user_data_roundtrip_preserves_everything() {
    let appdata = match std::env::var_os("APPDATA") {
        Some(v) => std::path::PathBuf::from(v),
        None => return,
    };
    let path = appdata.join("com.rlaunch.app").join("launcher-data.json");
    if !path.exists() {
        eprintln!("実データ無し、スキップ: {}", path.display());
        return;
    }
    let text = std::fs::read_to_string(&path).expect("read real data");
    let data: LauncherData =
        serde_json::from_str(&text).expect("実ユーザーデータがパースできること");
    let original: Value = serde_json::from_str(&text).unwrap();
    let output = serde_json::to_value(&data).unwrap();

    let mut problems = Vec::new();
    assert_subset(&original, &output, "$", &mut problems);
    assert!(
        problems.is_empty(),
        "ラウンドトリップで情報が失われた:\n{}",
        problems.join("\n")
    );
}
