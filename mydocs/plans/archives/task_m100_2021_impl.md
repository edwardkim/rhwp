# 구현계획서 — Task M100 #2021: caret rect 선형 페이지 탐색 해소

- 이슈: #2021 / 수행계획서: `task_m100_2021.md` / 작성일: 2026-07-07
- 1단계 계측 결과 반영 (네이티브 프로브 + CDP 실계측, 스크립트 커밋 포함).

## 1. 가설 검증 결과 (확정)

| 계측 | 값 |
|---|---|
| 호스트 문단이 걸친 페이지 (115쪽 거대 표) | **115개 전부** |
| 네이티브: 페이지 트리 1장 빌드 | ~45ms |
| CDP: 페이지 0 셀 입력 1회 rect | 32.9ms |
| **CDP: 페이지 ~92 셀 입력 1회 rect** | **3,064.4ms (long task 3,067ms)** |

원인 확정: `get_cursor_rect_by_path_native`가 후보 페이지 목록(=거대 표는 전 페이지)을
**page 0부터** `build_page_tree_cached`로 재빌드하며 캐럿을 탐색 — 입력 직후 캐시 전체
무효화 상태라 캐럿 페이지 인덱스에 선형 비례. 이슈의 621ms는 중간 페이지 케이스이며
후반 페이지는 3초까지 악화. (#2012의 완화와 별개 축.)

## 2. 수정 설계

### 수정 A (Rust) — 캐럿 페이지 힌트 우선 탐색
- `get_cursor_rect_by_path_native`에 `hint_page: Option<u32>` 추가(내부 시그니처).
  탐색 순서: `hint, hint±1` → 잔여 페이지 종전 순서. 좌표 값 불변(순서만).
- wasm 표면: 기존 `getCursorRectByPath` 시그니처 유지(힌트 None) + 신규
  `getCursorRectByPathNear(section, para, path_json, offset, hint_page)` 추가 — 계약
  호환(추가만, #2023 논의와 충돌 없음).
- Studio `CursorState.updateRect()`: `this.position.pageIndex`(직전 rect의 페이지)를
  힌트로 전달. 셀 이동/클릭 직후엔 클릭 페이지가 힌트.
- 동등성 핀: 거대 샘플에서 힌트 유/무 좌표 완전 일치 (+ 힌트가 틀린 페이지여도 fallback
  으로 동일 결과).

### 수정 B (Studio) — 입력 직후 낙관적 caret + idle 정밀 재계산
- 입력 fast path: 이전 rect + 삽입 폭(`measureText` 근사)으로 caret x 즉시 갱신,
  `updateRect()`(A 적용으로 ~50ms 수준)는 `requestAnimationFrame` 후행으로 이연 —
  input handler 동기 구간에서 rect 호출 제거.
- 퇴행 가드: e2e 프로브(`issue-2021-probe.mjs`)를 검증 스크립트로 정리해 두고, node
  단위 가드로 fast path 가 입력 핸들러 내 동기 rect 호출을 하지 않음을 계측 wrapper 로
  고정(기존 input-edit-invalidation 테스트 패턴).

### 예상 효과
후반 페이지 셀 입력: **3,064ms → 힌트 1~2장 빌드(~50-100ms, wasm) → B 적용 시 input
handler 에서는 ~2ms**(삽입 호출만).

## 3. 단계 매핑

- 2단계 = 수정 A + 동등성 핀 + CDP 재계측 (중간 확인)
- 3단계 = 수정 B + 퇴행 가드 + CDP 최종 계측 (전/후 대비)
- 4단계 = 전체 게이트(Rust/Studio) + 보고

## 4. 게이트

Rust: fmt/clippy 0 · `--tests` 0 실패 · OVR 추가 변동 0 · 동등성 핀.
Studio: `npm run build` · `node --test` · CDP 계측 전/후 대비(수용 기준: long task 유의미 감소).
