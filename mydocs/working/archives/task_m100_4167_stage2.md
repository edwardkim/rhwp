---
kind: working
status: completed
issue: 4167
last_verified: 2026-08-08
---

# Task #4167 Stage 2 - deferred 편집의 cell_units 지문 보존

## 구현

- Stage 1 이후 잔존 병목: deferred 셀 편집마다 `invalidate_cell_units_after_text_edit`가
  편집 셀의 memoized units 를 전량 evict → 편집 직후 첫 캐럿 질의가 2,507문단 전량
  recompose(native 11.2ms, `compose_paragraph` 지배)를 지불. IME 조합은 매 업데이트가
  편집→질의라 매 키가 이 비용을 문다.
- `LayoutEngine::cell_paragraph_units_fingerprint` — units 산출이 읽는 문단 입력만 해시:
  line_segs (개수, vertical_pos, line_height, tag 의 synthetic 비트(bit 31)만), controls 수,
  공백/빈 문단 클래스. `text_start`·`segment_width`는 제자리 타이핑에도 매 키 변하지만
  units 산출이 읽지 않아 제외(전 구간 grep 근거). 원본 로드 tag 잔여 비트(예: 0x100000)는
  reflow 미재방출로 첫 편집에서 무의미하게 지문을 바꿔 마스킹.
- 편집 관문 2곳(셀 replace/insert impl·delete)에서 reflow 전후 지문 캡처 →
  `invalidate_cell_units_after_text_edit(unit_fingerprint_unchanged)` — 불변이면 eviction
  생략(캐시 벡터가 항등 유효), 변하면 종전대로 셀 단위 제거.

## 검증 결과

- 신규: `issue4167_fingerprint_unchanged_edit_retains_cell_units`(보존/제거 양분기),
  `issue4167_units_fingerprint_sensitivity`(불변: text_start/segment_width/원본 tag 잔여 비트 —
  변화: 줄 수/높이/synthetic/스페이서 클래스), `issue4167_units_fingerprint_doc_contract`
  (실문서: 텍스트 문단 제자리 삽입 불변 — 공백 스페이서에 글자 삽입은 클래스 전이로 변화).
- `issue2214_deferred_insert_uses_scoped_cache_eviction` 기대값을 "무조건 evict"에서
  "지문 변화 시에만 evict"로 갱신(이력 주석 포함) — 실측상 두 phase 의 최종 삽입 모두 지문
  불변이라 보존이 정답. #2214 나머지 8개 테스트 불변 pass.
- 계측: `issue4149_deferred_edit_first_cursor_query_decomposition` — 편집 직후 find_pages
  **11.2ms → 5.8~11.8µs**, 편집 직후 rect 질의 전체 ~0.66ms. 브라우저(wasm 재빌드 후) IME
  조합 업데이트 58~62ms(기준) → 17~24ms(Stage 1까지) → **5.8~11.5ms**(Stage 2 포함),
  일반 삽입 warm 1.6~6.8ms — 전 키스트로크 16ms 프레임 예산 안.
- 전체 lib: **3,308 passed / 0 failed** (당시 트리 기준; 스택 재구성 후 레이어 게이트에서 재검증).
- 진단 부산물: `issue4149_deferred_units_rebuild_profile_loop`(--ignored, sample 용).
