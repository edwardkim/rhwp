# 구현계획서 — Task M100 #2079: typeset_endnote_paragraphs 2차 회전 (라운드 9)

- 수행계획서: `task_m100_2079.md` / 작성일: 2026-07-09 / goal 방식 (승인 게이트는 최종만)

## 1. 재지도 결과 (1단계, 사각지대 체크리스트 6종 적용)

P6 fit 판정 연쇄의 실체 = **`let <판정명> = <불리언 연쇄/시뮬 블록>;` 20여 건**
(5463~8284). 최종 집계자 `advance_for_fit`(논리 76)·`advance_for_new_endnote`(논리 42)는
상류 판정 지역변수들의 순수 집계라 **추출 부적합**(58/40개 불리언 전달 필요) — 상류
판정을 데이터-레벨에서 절단한다.

**핵심 실측**: 104줄 렌더-시뮬 예측부(local_paras 구성→HeightCursor→predicted_y)가
**4곳 중복** — 6159/6375/7275 문자 그대로(strip 해시 동일), 5610 은 동일 계산의 `?`
문체 변형. 함수 밖 2곳(2134/9318)은 다른 계산(해시 상이) — 범위 외.

## 2. 추출 설계

### 추출 1 — `predict_endnote_render_y` (&self, 4곳 dedup, 예상 −300줄)

```
fn predict_endnote_render_y(&self, st: &TypesetState, paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet, en_para_idx: usize, en_col_w: f64, available: f64,
) -> Option<f64>
```
- 읽기 전용 (muts 0 — hc/local_paras 는 내부 지역). st 경유로 current_items/
  endnote_paragraphs/current_start_height/skip_spacing_before_prededuct/
  current_endnote_flow/endnote_between_notes_hu 취득 — 직접 파라미터 7개로 §6 충족.
- 4곳 호출 치환: 6159/6375/7275 는 `let predicted_y = self.predict_…(…);`,
  5610 변형은 `let predicted_y = self.predict_…(…)?;` + 후속 지역 계산 유지.

### 추출 2 — 시뮬형 판정 3건 (&self 메서드, 추출 1 이후 잔여부)

| 판정 | 위치 | 반환 |
|---|---|---|
| `judge_large_between_title_tail_render_overflows` | 6119~6276 | bool |
| `judge_large_between_last_column_visual_split` | 6356~6520 | Option<usize> |
| `judge_large_between_title_tail_body_advances_page` | 7257~7391 | bool |

- 공통 guard 스칼라를 `EnTailJudgeVars`(Copy: default_between_notes_gap,
  compact_endnote_separator_profile, has_visible_endnote_separator,
  visible_large_between_notes_gap, compact_between_notes_gap,
  zero_endnote_spacing_profile, local_vpos_rewind, internal_vpos_rewind, ep_idx,
  en_fit, available) 로 묶어 3건 공유 (EnMetricsVars 전례).
- 참조는 직접: st/fmt/en_para/paragraphs/styles (+건별: en_ctrl/en_ref/composed).
- 소스분기 없음 확인(§1). 탈출은 클로저 내부 return/블록-지역 continue 뿐 — 외부 무영향.

### 추출 3 (여력 시) — 시뮬형 잔여: `large_between_split_head_render_overflows`(5607~5722),
`large_between_tail_render_overflows`(5889~5952) 등 — 2단계 계측 후 소득/위험으로 결정
(v2 §0 축소 허용).

## 3. 단계 매핑

2단계 = 추출 1 (dedup) → 게이트 / 3단계 = 추출 2 (판정 3건) → 게이트 /
4단계 = 추출 3 (선택) → 게이트 / 5단계 = 재평가 r9 + 최종 보고 → 승인 요청.

## 4. 게이트

수행계획서 §5 (issue_1116 13/13 표적 포함, 매 단계 즉시).
