# Stage 5 보고서 — Task #1062: 잔여/악화 추가 조사 + 누적 floor 보정

- 브랜치: `local/task1062`

## 1. 3-09 2022 잔여 조사 (Stage 4 잔여 27건)

잔여 항목 분석:
- 전부 FullParagraph, **본문(para=366)+미주(497~) 혼재**.
- 패턴 (a) 단일 대형 1건(para=858, 277px), (b) 소규모 적층 런(예: 934→937 = 11→155px).
- trailing_ls 과소(Stage 3 해소분)와 **분리되는 잔류**.

## 2. 악화 4파일 조사 (sungeo·hwpctl_API·hwp3-sample4·hwp3-sample10, +8)

- 모두 다단+미주. 변경 후 신규 overflow 는 소량·소폭(3~23px).
- **검증**: 누적/판정에 안전 하한 `max(.., height_for_fit)`(종전보다 조밀해지거나 늦게 끊기지
  않음 보장)을 적용해도 **4파일 회귀 불변**.
  → 회귀 원인은 누적 과소가 **아님**. typeset 미주 분할점 이동이 렌더러 vpos **base 리셋
  지점**과 미정렬되어, 페이지 첫 미주 변경 시 base 가 바뀌며 경계 항목이 tolerance(2px) 위로
  미세 이동한 결과.
- 즉 **3-09 2022 잔여와 동일 부류**(typeset 분할 ↔ 렌더러 미주 base 리셋 정합) — 본 과제
  핵심(미주 trailing_ls)과 분리되는 별도 잔류. 해소하려면 렌더러(height_cursor) 미주 base
  리셋을 typeset 분할점과 맞추는 별도 작업 필요.

## 3. 누적 floor 보정 (채택)

```rust
let fit = (advance - trailing_ls).max(fmt.height_for_fit);
let acc = advance.max(fmt.height_for_fit);
```
- 효과: **대상 추가 개선** (3-09'22 27→20, 3-09'23 31→20, 3-10 22→7, 3-11 9→4).
  (일부 미주는 vpos 전진 > height_for_fit floor 로 더 정확히 채워져 분할 정밀화.)
- 회귀 안전성: 종전 height_for_fit 미만으로 누적/판정이 내려가지 않음 → packing 종전 이하 보장.

## 4. 최종 결과 (floor 버전, devel 대비)

| | devel | 변경 |
|---|---|---|
| 251 샘플 LAYOUT_OVERFLOW 합 | 1624 | **769 (−855, −53%)** |
| cargo test | — | **1550 passed / 0 failed** |
| 골든 SVG | — | 8종 통과(회귀 0) |

| 대상 | overflow | 쪽수 우리/PDF |
|------|------|------|
| 3-09 2022 | 155→20 | 22/23 |
| 3-09 2023 | 119→20 | 20/20 ✓ |
| 3-10 2022 | 112→7 | 18/18 ✓ |
| 3-11 2022 | 94→4 | 21/21 ✓ |

악화 4파일 +8(3~23px, 골든 무회귀) — 별도 잔류(렌더러 미주 base 정합).

## 5. 결론

- 핵심 결함(다단 미주 누적 trailing_ls 과소) **해소**, floor 보정으로 대상 추가 개선.
- 잔여(3-09 2022 20건 + 악화 4파일 +8)는 **typeset 분할 ↔ 렌더러 미주 base 리셋 정합**이라는
  별도 결함 — 후속 이슈 권장.
