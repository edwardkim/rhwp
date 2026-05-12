/** 메타만 목록용 (페이로드는 별도 store) */
export interface DocHistoryEntryMeta {
  id: string;
  label: string;
  createdAt: number;
  sourceFileName: string;
  /** IR JSON UTF-8 길이 또는 구버전 HWP 바이트 길이 */
  byteLength: number;
  /** `ir`: stable_id 보존 스냅샷 JSON. `legacy`: 구 HWP 바이트만 저장된 항목 */
  storageKind?: 'ir' | 'legacy';
}
