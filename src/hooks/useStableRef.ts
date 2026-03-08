/* ============================================================
   useStableRef - 常に最新値を保持する ref を返すヘルパーフック
   コールバックを useMemo 依存配列に入れずに最新値を参照するパターンを共通化。

   使用例:
     const onSaveRef = useStableRef(onSave);
     // → onSaveRef.current は常に最新の onSave を指す
   ============================================================ */
import { useRef } from "react";

/**
 * 値が更新されるたびに ref.current を同期する。
 * `useCallback` 不要で安定した参照を取得するのに便利。
 */
export function useStableRef<T>(value: T) {
  const ref = useRef(value);
  ref.current = value;
  return ref;
}
