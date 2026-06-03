# Task #634 Stage 1 — TDD RED 통합 테스트

**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/rhwp/issues/634
**구현계획서**: `mydocs/plans/task_m100_634_impl.md`
**진행 시점**: 2026-05-06

---

## 1. 작업 내용

`src/renderer/layout/integration_tests.rs` 에 Task #634 통합 테스트 5건 추가.

### 1.1 헬퍼 함수

```rust
fn count_text_at_y(svg: &str, target_y: f64) -> usize { ... }
```

SVG 에서 특정 y 좌표 (±0.5px) 의 `<text>` 요소 개수를 센다.
페이지 footer 영역의 쪽번호 텍스트 (`"- N -"`) 검출에 사용.

### 1.2 테스트 5건

| 테스트 | 샘플 | 페이지 | 목적 | 결과 |
|--------|------|--------|------|------|
| `test_634_aift_page2_no_page_number_before_new_number` | aift.hwp | 2 (표지) | NewNumber 전 미표시 | **FAIL (RED)** |
| `test_634_aift_page7_shows_page_number_after_new_number` | aift.hwp | 7 (□ 배경) | NewNumber 후 표시 | PASS |
| `test_634_gukrip_page2_no_page_number_before_new_number` | 국립국어원.hwp | 2 (목차) | NewNumber 전 미표시 | **FAIL (RED)** |
| `test_634_gukrip_page3_shows_page_number_after_new_number` | 국립국어원.hwp | 3 (본문) | NewNumber 후 표시 | PASS |
| `test_634_no_newnumber_doc_shows_page_numbers_from_page1` | hwp3-sample.hwp | 1 | A안 (관대) 회귀 방지 | PASS |

### 1.3 RED 확인 출력

```
test test_634_gukrip_page3_shows_page_number_after_new_number ... ok
test test_634_gukrip_page2_no_page_number_before_new_number ... FAILED
test test_634_aift_page2_no_page_number_before_new_number ... FAILED
test test_634_aift_page7_shows_page_number_after_new_number ... ok

failures:
    renderer::layout::integration_tests::tests::test_634_aift_page2_no_page_number_before_new_number
    renderer::layout::integration_tests::tests::test_634_gukrip_page2_no_page_number_before_new_number

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 1120 filtered out
```

(test_634_no_newnumber_doc_shows_page_numbers_from_page1 도 PASS — 출력에서 누락된 것은
정렬 순서 문제일 뿐, 결과 카운트는 5건 중 3 passed 일치)

### 1.4 RED 의 의미

- **aift.hwp 페이지 2**: 현재 y=1079.16 에 3개 (`"- 2 -"` 글자) — fix 후 0 이어야 함
- **국립국어원 페이지 2**: 현재 y=1069.71 에 3개 (`"- 2 -"` 글자) — fix 후 0 이어야 함

두 RED 는 모두 **수행계획서 §1.1 의 핵심 회귀** 와 정확히 일치.

### 1.5 PASS 의 의미

- aift 페이지 7 / 국립국어원 페이지 3: NewNumber 발화 페이지 — 이미 표시 중, fix 후에도 유지
- hwp3-sample 페이지 1: NewNumber 가 없는 문서 — A안 (관대) 으로 즉시 표시. fix 후에도 유지

---

## 2. 산출물

| 파일 | LOC | 설명 |
|------|-----|------|
| `src/renderer/layout/integration_tests.rs` | +118 | 헬퍼 + 5건 테스트 추가 |

---

## 3. 다음 단계

Stage 2 (Fix 적용):
1. `src/renderer/page_number.rs` `PageNumberAssigner.numbering_started`
2. `src/renderer/pagination.rs` `PageContent.show_page_number`
3. `src/renderer/pagination/{state.rs, engine.rs}` 초기화 + finalize 정합
4. `src/renderer/typeset.rs` finalize 정합
5. `src/document_core/queries/rendering.rs` 구역간 carry
6. `src/renderer/layout.rs` `build_page_number` 가드
7. 기존 mk_page 헬퍼 6+1건 갱신
8. RED → GREEN 확인

승인 후 진행.
