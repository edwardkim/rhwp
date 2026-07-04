# 최종 보고서 — #1637 HWPX roundtrip 페이지네이션 변동 수정 (원인 A+B 완전 해소)

- 마일스톤: M100 / 브랜치: local/task1637 (devel d3627f55 분기)
- 일자: 2026-06-29
- 선행: Task #1636 v3 전수 검증에서 발견(T-PI), 근본원인 조사 `mydocs/tech/task1637_pagination_hidefirstemptyline.md`

## 1. 문제

HWPX parse→serialize→reparse 후 일부 PASS(IR diff=0) 문서의 페이지네이션이 달라져
PI가 다른 페이지로 렌더링됨(0.77% 표본). IR 게이트·lineseg 게이트 모두 미검출(IR-invisible).

## 2. 지배원인 A (19/23 = 83%)

직렬화기가 secPr `<hp:visibility>` 를 정적 템플릿(`empty_section0.xml`, `hideFirstEmptyLine="0"`)으로
방출하고 파싱된 `SectionDef`(hide_empty_line 등 6필드)를 무시 → 원본 `hideFirstEmptyLine="1"`
(첫 빈줄 숨김)이 "0"으로 드롭 → 선두 빈줄 가시화 → 본문 하향 → 페이지 확장/PI 이동.
`diff_documents` 가 visibility 미비교라 IR diff=0(3중 사각).

## 3. 수정 (소스 2파일)

### (1) 직렬화기 — visibility IR 치환 (`src/serializer/hwpx/section.rs`)
- `TEMPLATE_VISIBILITY` 상수 + `render_visibility(&SectionDef)` + `replace_visibility()` 추가.
- `write_section` 에서 page_border_fill 치환 직후 `out = replace_visibility(&out, &section.section_def)`.
- IR 보존 6필드(hideFirstHeader/Footer/MasterPage·border·fill·hideFirstEmptyLine)를 방출.
  IR 미보존 2필드(hideFirstPageNum·showLineNumber)는 템플릿 기본값 "0" 유지. (#1388 secPr 치환 동형)

### (2) 게이트 — visibility IR-visible 화 (`src/serializer/hwpx/roundtrip.rs`)
- `IrDifference::SectionVisibility { section, detail }` 변형 + Display 추가.
- `diff_visibility(&SectionDef, &SectionDef)` (6필드 비교) 추가, `diff_documents` 루프에 동승.
- 재발 가드 단위테스트 3건(`task1637_visibility_in_gate` / `_equal_is_empty` / `_roundtrip_preserves_hide_empty_line`).

## 4. 검증

| 항목 | 결과 |
|------|------|
| 재현 파일 36374808 | page0 8 → **10** (orig 일치), hideFirstEmptyLine="1" 보존, PASS |
| 재현 파일 36373372(한글 EXPAND) | page0 9 → **11** (orig 일치), 보존, PASS |
| 신규 단위테스트 3건 | 전부 ok |
| lib 전체 테스트 | **1983 passed, 0 failed** |
| hwpx_roundtrip_baseline | **4 passed** (samples/hwpx 회귀 0) |
| 전수 IR_DIFF (25,817→26,145) | **5 → 5** (신규 IR 회귀 0; 게이트 추가가 PASS 깨지 않음) |
| **통제 비교(T-PI shift 23건)** | **SAME 19 / 잔여 4** = hideFirstEmptyLine 케이스 **19/19 전부 해소** |
| **회귀(T-PI 3,005 표본)** | 23 → 7. 잔여 7 전부 **선존 cause B**: hfel=0 5건은 old-rt와 **바이트 동일**(미변경), hfel=1 1건은 hfel 보존됨(잔여 partial-split), 1건은 기존 23 소속. **신규 회귀 0** |
| fmt(`cargo fmt --all --check`) | 변경 2파일만, Diff 0 |

**통제 비교 결론(개선−회귀)**: 개선 19, 회귀 0 → 순 +19. 채택 기준 충족.

## 5. 잔여원인 B (4/23 = 17%) — 표 flowWithText 드롭 (규명·수정 완료)

cause A 수정 후 잔여 4건(hfel=0)을 추가 조사 → **section0.xml → table region → table5 →
`<hp:pos@flowWithText>` 0→1** 로 이진탐색 수렴.

**근본원인**: `src/serializer/hwpx/table.rs:147` `write_pos` 가 `flowWithText="1"` **하드코딩**
(shape writer `section.rs:1507` 은 `c.flow_with_text` 사용·정상). 원본 표가 `flowWithText="0"`
(treatAsChar=1)일 때 roundtrip 이 "1"로 드롭 → 표 partial-split 임계 변동 → 페이지네이션 변동.
#1594 holdAnchorAndSO(같은 함수 직상단) 와 **동형 IR-invisible 결함**.

**수정**:
- `table.rs:147`: `("flowWithText", "1")` → `("flowWithText", bool01(c.flow_with_text))`.
- `roundtrip.rs`: `IrDifference::ObjectFlowWithText` + `diff_flow_with_text` 게이트(표 site 동승, #1594 동형) + 단위테스트 `task1637_table_flow_with_text_in_gate`.

**검증**: cause B 4건(36384855·36390819·36400485·36385069) + 복합 1건(36381640, A+B 동반) **전부 SAME**.
lib **1984 passed**, baseline 4 passed, IR_DIFF 5→5(신규 회귀 0), fmt/clippy clean.

## 6. 잔여 (본 타스크 범위 밖)

- 한글 페이지 붕괴 ~0.5%(#1589), char_shape +8(#1591/#1593).
- equation `flowWithText` 도 하드코딩(section.rs:1608)이나 cause B 무관 — 별도.

## 6. 산출물

- 소스: `src/serializer/hwpx/section.rs`, `src/serializer/hwpx/roundtrip.rs`
- 근본원인: `mydocs/tech/task1637_pagination_hidefirstemptyline.md`
- 검증 데이터: `output/poc/fidelity3/pi_page_fixed_shift23.tsv`, `pi_page_fixed_full.tsv`, `hwpx_fixed/`
