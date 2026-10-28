# 단계 완료 보고 — Task M100 #2079 2단계: predict_endnote_render_y (4곳 dedup)

- 작성일: 2026-07-09 / 브랜치: `local/task2079` / goal 방식 (자체 검증)

## 수행 내용

104줄 렌더-시뮬 예측부(local_paras 재구성 → HeightCursor 재주행 → predicted_y)를
`predict_endnote_render_y`(&self, 7파라미터, 읽기 전용 muts 0)로 승격하고 **4곳 치환**:
- 문자 그대로 중복 3곳(judge 계열: title_tail_render_overflows / last_column_visual_split /
  title_tail_body_advances_page) → `let predicted_y = self.predict_…(…);`
- `?` 문체 변형 1곳(split_head_render_overflows) → `…(…)?;` — first_vpos/lookup_local 의
  None 조기 종료가 helper 의 and_then 경로와 의미 동일.
- 본문 무변경 이동 (`&styles` 는 && deref 강제변환으로 그대로 성립). 배치는
  `simulate_endnote_column_bottom_y`(기존 유사 시뮬 helper) 직후 — 도메인 응집.

## 게이트 (전수 통과, 자체 검증)

fmt ✓ / clippy **0** / `--tests` **2,945/0** / issue_1116 **13/13** /
OVR 5샘플 회귀 **0건**.

## 계측

| 함수 | 시작 (r8) | 현재 |
|---|---|---|
| `typeset_endnote_paragraphs` | 4,227줄 · 분기 1,717 | **3,844줄 · 분기 1,669** |
| 신규 `predict_endnote_render_y` | — | 107줄 · 분기 12 |

파일 순감 −272줄 (중복 3본 소거).

## 다음 단계

3단계 — 시뮬형 판정 3건(`EnTailJudgeVars` 공유) 추출.
