# Task #595 Stage 1 완료 보고서

**Issue**: #595 — exam_math.hwp 2페이지부터 수식 더블클릭 hitTest 오동작
**브랜치**: `local/task595`
**Stage**: 1 — 본질 진단 + 재현 단위 테스트 + 광범위 sweep
**날짜**: 2026-05-07

---

## 1. Stage 1 산출물

| 항목 | 위치 | 상태 |
|------|------|------|
| 정적 분석 진단 도구 | `examples/inspect_595.rs` | 작성 ✓ |
| e2e 진단 테스트 | `rhwp-studio/e2e/issue-595.test.mjs` | 작성 + 실행 ✓ |
| 재현 단위 테스트 | `tests/issue_595.rs` (5 케이스) | 작성 + 실행 ✓ (3 fail / 2 pass) |
| 임시 자료 | `rhwp-studio/public/samples/exam_math.hwp` | 진단용 임시 (Stage 종료 시 정리) |

## 2. 본질 결함 위치 정확 식별

**함수**: `src/renderer/layout.rs::build_header` ([line 888-931](../../src/renderer/layout.rs#L888))

**결함 라인**: [layout.rs:928](../../src/renderer/layout.rs#L928)
```rust
// Header bbox를 자식 노드 범위까지 확장 + 셀 클리핑 해제
// (머리말 표 셀 내 Shape가 header_area 밖에 배치될 수 있음)
Self::expand_bbox_to_children(&mut header_node);
```

**메커니즘**:
1. Header 노드 초기 bbox = `layout_rect_to_bbox(&layout.header_area)` — 정상 머리말 영역 (예: y=0~147)
2. 머리말 내부 컨텐츠 (문단, 셀, Shape, line 등) 추가
3. **`expand_bbox_to_children`** 가 자식 모두의 bbox 합집합으로 Header bbox 확장
4. 자식 중 본문 끝까지 늘어진 객체 (단 구분선 line `paraIdx=0 ci=2`, h≈1227px) 가 있으면 Header bbox 가 본문 영역까지 침범

**의도 (주석)**: 머리말 표 셀 내 Shape 가 머리말 영역 외부 배치 시 클리핑 방지.
**부작용**: 페이지 전체 길이로 늘어진 자식 객체에 대해서는 hit test 영역까지 본문 침범.

## 3. 페이지별 발현

`examples/inspect_595.rs` 결과 (5px sweep, x=514):

| 페이지 | Header hit y 범위 | 실제 머리말 영역 | 판정 |
|--------|------------------|------------------|------|
| page 0 (1p) | **60.0 ~ 145.0** | 0 ~ 147 | **정상** ✓ |
| page 1 (2p) | **60.0 ~ 1355.0** | 0 ~ 147 | **결함** ✗ |
| page 2 (3p) | 60.0 ~ 1355.0 | 0 ~ 147 | 결함 ✗ |
| page 3 (4p) | 60.0 ~ 1355.0 | 0 ~ 147 | 결함 ✗ |

**page 0 vs page 1+ 차이**:
- page 0: 단 구분선 line `paraIdx=0 ci=5 y=224.6 h=1133.9` — Body 자식
- page 1+: 단 구분선 line `paraIdx=0 ci=2 y=132.3 h=1226.8` — Header 자식 (controlIdx 차이)

## 4. 재현 단위 테스트 (`tests/issue_595.rs`)

```
running 5 tests
test issue_595_page1_equation_coord_not_header ... FAILED            ← 정정 전 fail
test issue_595_page1_header_area_still_hits ... ok                    ← 정상 가드
test issue_595_page1_body_coord_not_header_regression_guard ... FAILED ← 정정 전 fail
test issue_595_page0_body_coord_not_header ... ok                     ← 정상 baseline
test issue_595_page1_body_center_not_header ... FAILED               ← 정정 전 fail

test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured
```

**Fail 메시지 정합**:
- `(514, 200)` → `{"hit":true,"isHeader":true,"sectionIndex":0,"applyTo":1}` ← 본문 좌표 머리말 hit
- `(654.5, 209.7)` → 동일 (이슈 명세 수식 좌표)
- `(514, 800)` → 동일 (본문 중앙)

→ 결함 재현 정합. 정정 후 5 케이스 모두 pass 가 stage 2 검증 기준.

## 5. 광범위 sweep — 영향 영역 정량

`samples/` 전체 164 fixture / 1684 페이지 점검 (x=514, y=800 본문 좌표 기준):

| 항목 | 측정값 |
|------|--------|
| 머리말 hit 본문 침범 fixture | **2 / 164 (1.2%)** |
| 머리말 hit 본문 침범 페이지 | **32 / 1684 (1.9%)** |

**발현 fixture**:
- `exam_math.hwp` — 20p 중 16p (page 0 제외 모든 페이지)
- `exam_math_no.hwp` — 동일 양식 (16/20p)

**해석**: 매우 특수한 양식 (머리말에 단 구분선 line 이 자식으로 포함된 경우) — 회귀 위험 매우 낮음.

## 6. 이슈 본문 정오표

이슈에서 "Rust 측 점검 결과 — 정합" 으로 결론 지었으나 **점검 범위가 부분적**:

| 점검 영역 | 이슈에서 점검 | 본 stage 점검 |
|-----------|---------------|----------------|
| `getPageControlLayout` controls 배열 (수식 bbox) | ✓ 정합 확인 | ✓ 정합 재확인 |
| **`hitTestHeaderFooter`** | ✗ 점검 안 함 | **✗ page 1+ 결함 발견** |
| `build_page_tree` 의 Header 노드 bbox | ✗ 점검 안 함 | **✗ build_header 의 expand_bbox_to_children 결함** |

이슈에서 후속 진단 후보로 제시한 (a)/(b)/(c) 가 모두 TS 측 좌표 변환 영역 — Rust 측 hit test 영역은 의심 후보 밖.

**놓친 단서 (사용자 추가 보고 시 명확화)**:
- "hover 시 손바닥 표시는 뜨는데 클릭 반응 없음" → `findPictureAtClick` 자체는 정상 hit, dblclick 흐름의 별도 분기 (머리말 검사) 가 가로채는 중

## 7. e2e 진단 결과 (참고)

`rhwp-studio/e2e/issue-595.test.mjs` (1365×1018 사용자 환경 모사):

| 시나리오 | findPictureAtClick | hfHit | dblclick 결과 |
|----------|-------------------|--------|---------------|
| zoom=1 page 0 paraIdx=18 ci=0 | hit ✓ | false | picSel=true ✓ |
| zoom=1 page 1 paraIdx=65 ci=0 | hit ✓ | **true** | picSel=true (e2e mock 한계) |
| zoom=0.5 그리드 모드 | null ✗ | false | picSel=false |

**별도 결함 발견**: zoom ≤ 0.5 그리드 모드에서 `pageLeft = (sc.clientWidth - pageDisplayWidth) / 2` 단일 컬럼 가정으로 hit 좌표 계산 결함. 본 이슈 (#595) 와 별개 영역 — 별도 task 분리 검토.

## 8. Stage 2 정정 영역 후보

| 옵션 | 영향 영역 | 안전도 | 비고 |
|------|----------|--------|------|
| **A. `hit_test_header_footer_native` 에서 `layout.header_area` 로 hit 판정** | 단일 함수 | 높음 | bbox 확장과 무관하게 정확한 머리말 영역만 hit. 렌더링 동작 무영향. |
| **B. `expand_bbox_to_children` 후 `layout.header_area` 로 max-clip** | `build_header` | 중 | bbox 자체를 보정. 단 구분선 line 의 클리핑 동작에 영향 가능. |
| **C. 단 구분선 line 을 Header 자식이 아니라 별도 분류** | layout 분류 영역 | 낮음 | 영향 영역 광범위 — 회귀 위험 ↑ |

**권장**: **옵션 A** — 영향 영역 좁고 본 결함 정정에 충분. 회귀 위험 최소.

## 9. 다음 단계 (Stage 2)

1. 정정 옵션 (A 권장) 작업지시자 승인
2. `cursor_rect.rs::hit_test_header_footer_native` 에서 `layout.header_area` 사용으로 정정
3. 단위 테스트 5 케이스 모두 pass 확인
4. 광범위 sweep 재실행 — 회귀 0 확인
5. e2e 정정 후 재시도 — 사용자 환경 정합 확인
6. Stage 2 보고서 작성

## 10. Stage 1 정리 항목

- `examples/inspect_595.rs` — 보존 (정정 후 회귀 검증용으로 재사용 가능)
- `rhwp-studio/public/samples/exam_math.hwp` — Stage 2 e2e 검증 후 제거
- `tests/issue_595.rs` — 영구 보존 (회귀 차단 가드)

---

**Stage 1 완료 — Stage 2 진입 승인 요청**
