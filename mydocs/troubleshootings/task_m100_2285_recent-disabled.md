# 트러블슈팅 — 최근 문서 메뉴가 계속 비활성화 (#2285)

- **이슈**: edwardkim/rhwp#2285 / PR #2286
- **작성일**: 2026-07-15

## 증상

파일 메뉴의 "최근 문서" 서브메뉴가 문서를 열어도 **계속 비활성화(disabled)** 상태로 남아 열리지 않음.

## 원인

최근 문서 기록 훅(`addRecentDoc`)이 `loadBytes()`에서 **`await initializeDocument()` 뒤**에 위치했다.
`initializeDocument()`는 내부 마지막에 `promptLocalFontsIfNeeded()`(로컬 글꼴 안내 **모달**)를 `await`한다.

- 문서가 미설치 글꼴을 사용해 모달이 뜨는 경우(또는 자동화/헤드리스처럼 모달이 즉시 닫히지 않는 경우), `initializeDocument()` 프라미스가 그 지점에서 대기 → **그 뒤의 기록 훅이 실행되지 않음**.
- 결과적으로 IndexedDB에 최근 문서가 저장되지 않아 목록이 비고, `renderRecentSubmenu()`가 서브메뉴를 계속 disabled로 유지.

## 해결

기록 훅을 **문서 로드 성공 직후·블로킹 UI(폰트/모달) 이전**으로 이동.
`wasm.loadDocument()` 성공 + `wasm.currentFileHandle = fileHandle` 설정 직후에 `addRecentDoc`를 호출한다(파일명/형식은 이 시점에 이미 유효).

```ts
wasm.currentFileHandle = fileHandle;
if (fileHandle) {
  void addRecentDoc({ fileName: wasm.fileName, sourceFormat: wasm.getSourceFormat(), handle: fileHandle })
    .catch((err) => console.warn('[recent] 최근 문서 기록 실패:', err));
}
await autosaveManager.beginDocument(...);
await initializeDocument(...);
```

## 재발 방지

- "부수 효과 기록"은 **가능한 한 이른 시점**(핵심 상태 확정 직후)에 배치하고, 사용자 상호작용을 동반하는 `await`(모달/프롬프트) **뒤에 두지 않는다**.
- puppeteer headless 진단으로 `open-document-bytes` → `loadBytes` → `addRecentDoc` → 렌더 체인을 재현하여 확인.

---

## 2차 증상 — "열기를 해도 등록되지 않음" (오인 → 원복)

### 관찰

1차 수정 후에도, File System Access API 미지원 경로(브라우저 미지원·비보안 컨텍스트·
임베디드 웹뷰의 `input[type=file]` 폴백, 드래그드롭)로 연 파일은 핸들이 `null`이라
기록되지 않았다.

### 오인 경위와 원복 (PR #2286 리뷰)

이를 결함으로 오인해 저장 모델을 **바이트 스냅샷 기반**(모든 열기 등록, 핸들 선택)으로
재설계했으나, 이는 Issue #2285 가 명시한 범위("드롭/파일선택으로 연 핸들 없는 파일 —
재열기 불가 → 목록 제외")와 문서화된 보존 정책(핸들+메타만 저장)을 벗어나는 변경이었다.
리뷰(jangster77)에서 범위 이탈로 반려되어 **핸들-only 로 원복**했다.

- 핸들 없는 열기 경로의 미등록은 **의도된 동작**이다 — 브라우저가 절대 경로를 제공하지
  않아, 위치 참조(재열기 능력)는 `FileSystemFileHandle` 로만 영속화할 수 있다. 바이트
  스냅샷은 "그 위치의 문서"가 아니라 내용 사본이라 최근-문서 의미(이전에 열었던 파일
  위치)와 다르며, 대용량 문서의 IndexedDB 용량·실패 UX 부담도 있다.
- 재열기 UX 확정: 권한 거부 → 항목 유지 + 안내 / 파일 이동·삭제(read 실패) → 항목
  제거 + 안내 (`src/recent/recent-open.ts`, 규칙은 `tests/recent-open.test.ts` 로 고정).
- 핸들이 structured clone 불가한 환경이면 저장을 포기한다(핸들 없는 행을 남기지 않음).

### 재발 방지

관찰된 "미동작"이 이슈의 **명시된 제외 범위**인지 먼저 대조한다 — 범위 밖 확장(저장
모델 변경)은 별도 이슈로 분리해 보존 정책·식별자·용량 UX 와 함께 승인받는다.
회귀 고정: `tests/recent-store.test.ts`(핸들-only·동명 공존·상한 8), `tests/recent-open.test.ts`.
