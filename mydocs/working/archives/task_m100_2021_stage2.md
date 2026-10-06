# 단계 완료 보고 — Task M100 #2021 2단계: 수정 A (캐럿 페이지 힌트 탐색)

- 작성일: 2026-07-07 / 브랜치: `local/task2021`

## 수행 내용

- **Rust**: `order_pages_by_hint`(hint, hint±1 우선 → 잔여 종전 순서) +
  `get_cursor_rect_by_path_with_hint` — 기존 `_native`는 힌트 None 위임. 좌표 값 불변
  (탐색 순서만).
- **wasm**: 신규 `getCursorRectByPathNear(…, hint_page)` — 기존 시그니처 무변경
  (계약 추가만, #2023 프론트 계약 논의와 정합).
- **Studio**: `CursorState.updateRect()` 경로 API 분기에서 직전 rect의 `pageIndex`를
  힌트로 전달 (`getCursorRectByPathNear`), 구버전 wasm 폴백 포함. WASM 재빌드로
  `pkg/rhwp.d.ts` 갱신 (tsc 0 에러).
- **동등성 핀**: `issue_2021_hint_search_equivalence` — 셀 3종 × (무힌트/정힌트/
  오힌트 2종) 좌표 완전 일치.

## 계측 (CDP, 115쪽 거대 표, 입력 1회)

| 시나리오 | 수정 전 | 수정 후 |
|---|---|---|
| 페이지 0 셀 | 32.9ms | 32.9ms (불변) |
| **페이지 ~92 셀** | **3,064.4ms (long task 3,067)** | **32.9ms (long task 0)** — **93×** |

## 게이트 (전수 통과)

Rust: fmt ✓ / clippy 0 / `--tests` **2,925/0** (핀 포함) / lib 2,144/0 / OVR **추가 변동 0**.
Studio: tsc 0 / `npm run build` ✓ / `npm test` **171/0**.

## 축소 제안 (v2 §0 규칙 3)

계획했던 **3단계(수정 B: 낙관적 caret + idle 재계산)는 생략**을 제안한다 — 수용 기준
(long task 유의미 감소)이 A만으로 충족(50ms 임계 미만)됐고, 낙관적 갱신은 시각 글리치
위험 대비 실익이 작아졌다. 필요 시 후속 이슈로. → 승인 시 4단계(보고)로 직행.
