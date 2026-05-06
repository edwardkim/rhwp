# Task #639 Stage 1 — TDD RED 통합 테스트

**상태**: Stage 1 완료, 작업지시자 승인 대기
**작성일**: 2026-05-06
**브랜치**: `local/task639`

---

## 1. 목표

Issue #639 의 결정적 룰을 검증하는 통합 테스트 5건 추가. fix 미적용 상태에서
페이지 2, 3 테스트 RED (FAIL), 페이지 1, 6, 74 테스트 GREEN (PASS) 시나리오 확인.

## 2. 검출 패턴 결정

`render_page_svg_native` 의 footer 쪽번호 출력 형식 사전 조사:

```
=== render_page_svg_native(P) ===
  <text y=1079.16 ... font-size="10" ...> body="-"
  <text y=1079.16 ... font-size="10" ...> body="N"   (N = page_num 자릿수만큼)
  <text y=1079.16 ... font-size="10" ...> body="-"
```

페이지 번호 텍스트 "- N -" 는 **각 글자별 분리된 `<text>` 요소** 로 출력됨.
- y="1079.16": footer 영역 고정 좌표 (aift.hwp 페이지 크기 기준)
- font-size="10": 페이지 번호 전용 폰트 크기 (본문은 13.33+ 사용)

검출 함수 `count_footer_page_number_glyphs(svg)`:
- y="1079.16" + font-size="10" 매칭 `<text>` 요소 카운트
- 0 → footer 쪽번호 미표시
- 3+ → 표시 (단자리 page_num=3, 두자리 page_num=4, "- 4 -" or "- 6 8 -")

이 마커 조합은 본문 텍스트와 결정적으로 구분 (본문은 다른 y, 다른 font-size).

## 3. 테스트 5건 추가

위치: `src/renderer/layout/integration_tests.rs:1222` 직후 (test_574 다음).

| 테스트 | 예상 (fix 미적용) | 예상 (fix 적용 후) |
|--------|----------------|-----------------|
| `test_639_aift_page2_cover_style_no_page_number` | **FAIL** (글리프 3) | PASS (글리프 0) |
| `test_639_aift_page3_cover_style_no_page_number` | **FAIL** (글리프 3) | PASS (글리프 0) |
| `test_639_aift_page1_shows_page_number` | PASS (글리프 3, items=2) | PASS |
| `test_639_aift_page6_shows_page_number` | PASS (글리프 3, items=18) | PASS |
| `test_639_aift_page74_tac_true_table_shows_page_number` | PASS (글리프 4, tac=true) | PASS |

## 4. RED 검증 결과

```
$ cargo test --release --lib test_639

running 5 tests
test renderer::layout::integration_tests::tests::test_639_aift_page2_cover_style_no_page_number ... FAILED
test renderer::layout::integration_tests::tests::test_639_aift_page3_cover_style_no_page_number ... FAILED
test renderer::layout::integration_tests::tests::test_639_aift_page1_shows_page_number ... ok
test renderer::layout::integration_tests::tests::test_639_aift_page6_shows_page_number ... ok
test renderer::layout::integration_tests::tests::test_639_aift_page74_tac_true_table_shows_page_number ... ok

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 1136 filtered out
```

### 4.1 FAIL 메시지 (RED 검증)

```
---- test_639_aift_page2_cover_style_no_page_number ----
assertion `left == right` failed: Issue #639: aift.hwp 페이지 2 (cover-style:
  items=1 + Table 35×27 tac=false) 의 쪽번호 footer 가 SVG 에 표시됨
  (font-size=10 + y=1079.16 글리프 3개). 한컴 PDF 미표시와 불일치.
  left: 3   right: 0

---- test_639_aift_page3_cover_style_no_page_number ----
assertion `left == right` failed: Issue #639: aift.hwp 페이지 3 (cover-style:
  items=1 + Table 14×17 tac=false) 의 쪽번호 footer 가 SVG 에 표시됨
  (font-size=10 + y=1079.16 글리프 3개). 한컴 PDF 미표시와 불일치.
  left: 3   right: 0
```

**의도한 RED 시나리오 정확히 재현**. fix 적용 시 left=0 으로 전환되어 PASS 예상.

### 4.2 회귀 가드 PASS 확인

페이지 1 (items=2), 페이지 6 (items=18), 페이지 74 (tac=true) 모두 정상 footer 쪽번호
표시 → 글리프 ≥ 3 PASS. fix 적용 후에도 동일 PASS 유지되어야 함 (회귀 가드).

## 5. 코드 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | +99 LOC (5 테스트 + count helper + 주석) |
| `examples/probe_637.rs` | +21 LOC (SVG footer 검출 패턴 사전 조사용 — 일회성 도구) |

## 6. Stage 2 진입 준비

Stage 2 에서 `src/renderer/pagination/engine.rs:finalize_pages` 에 cover-style 룰 추가.
적용 후 본 5건 테스트 모두 PASS 전환 + 전체 cargo test sweep 회귀 0 확인.

---

**Stage 1 결과**: TDD RED 시나리오 완벽 재현. 페이지 2, 3 FAIL (글리프 3 ≠ 0) +
페이지 1, 6, 74 PASS (회귀 가드). Stage 2 fix 진입 준비 완료.

승인 후 Stage 2 진입.
