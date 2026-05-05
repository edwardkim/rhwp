# Task #574 Stage 2 — TDD 통합 테스트 + 단위 테스트 갱신 (RED 확인)

**브랜치**: `local/task574`
**이슈**: https://github.com/edwardkim/rhwp/issues/574
**선행 단계**: Stage 0 본질 확정 / Stage 1 구현 계획서 승인

---

## 1. 변경 사항

### 1.1 단위 테스트 갱신 (`src/renderer/layout/tests.rs:938`)

`test_is_heavy_display_face_matches_known_heavy_faces` 갱신:
- "HY견명조" 를 heavy 단언에서 **NOT heavy 단언으로 이동**
- "HY견명조B" 는 heavy 단언에 **유지** (명시 Bold variant)
- Task #574 의도 주석 추가

### 1.2 통합 테스트 추가 (`src/renderer/layout/integration_tests.rs:797`)

`test_574_page_number_not_force_bold_for_hy_kyun_myeongjo` 신규:
- `samples/exam_science.hwp` 페이지 1 SVG 에서 우상단 쪽번호 "1" 식별
- 식별 가드: `font-size="44"` + `HY견명조` + `translate(x≈924, y≈115)` + body="1"
- 단언: 해당 텍스트 요소에 `font-weight="bold"` **미포함**

## 2. RED 확인

### 2.1 단위 테스트 RED

```
$ cargo test --release --lib test_is_heavy_display_face_matches_known_heavy_faces -- --nocapture

thread 'renderer::layout::tests::test_is_heavy_display_face_matches_known_heavy_faces'
panicked at src/renderer/layout/tests.rs:953:9:
HY견명조 should NOT be heavy
test result: FAILED. 0 passed; 1 failed
```

### 2.2 통합 테스트 RED

```
$ cargo test --release --lib test_574_page_number_not_force_bold -- --nocapture

thread 'renderer::layout::integration_tests::tests::test_574_page_number_not_force_bold_for_hy_kyun_myeongjo'
panicked at src/renderer/layout/integration_tests.rs:874:13:
Task #574: 쪽번호 '1' (x=924.4, y=114.9, HY견명조, font-size=44) 가
font-weight="bold" 강제 적용됨. CharShape cs_id=0 의 bold=false 가 무시되는
is_heavy_display_face 결함.
header=[ transform="translate(924.36...) scale(0.9000,1)"
        font-family="HY견명조,..." font-size="44" font-weight="bold" fill="#000000"]
test result: FAILED. 0 passed; 1 failed
```

→ 두 테스트 모두 예상대로 RED. Stage 3 fix 적용 후 GREEN 으로 전환 예상.

## 3. 회귀 영향 (현재 시점)

테스트 갱신/추가만 — 본질 fix 미적용. 다른 테스트 영향 없음.

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/tests.rs:938-955` | 단위 테스트 갱신 (HY견명조 NOT heavy) |
| `src/renderer/layout/integration_tests.rs:797-868` | 통합 테스트 신규 |
| `mydocs/working/task_m100_574_stage2.md` | 본 보고서 |

## 5. 다음 단계

Stage 3 — `is_heavy_display_face` 의 `"HY견명조"` 제거 → RED → GREEN.
