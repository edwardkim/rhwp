# Task #402: 동일 문단 내 두 번째 line의 inline 그림이 첫 line과 겹침 — 수행계획서

## 목표

같은 paragraph 안에 inline 컨트롤(`treat_as_char=true`) 2개 이상이 서로 다른 line_seg에 배치된 경우, 두 번째 이후의 inline shape가 첫 번째 line과 같은 y 좌표에 그려지는 버그를 수정한다. line_seg별 vpos를 반영하여 정상적으로 줄바꿈/페이지 분할이 일어나도록 한다.

## 재현

샘플: `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`, 7쪽 (0-indexed page=6)

```bash
rhwp export-svg "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx" -p 6
qlmanage -t -s 1200 -o /tmp/ "output/2025년 기부·답례품 실적 지자체 보고서_양식_007.svg"
```

증상: 12행 표(pi=57 ci=0)와 파이 차트 그림(pi=57 ci=1)이 거의 같은 y 좌표(578~581)에서 시작하여 표 위에 그림이 겹쳐 보임. PDF 원본은 표만 7쪽에 두고 그림은 다음 쪽으로 흘려보냄.

## IR 구조

```
--- 문단 0.57 --- cc=17, controls=2
  ls[0]: ts=0, vpos=436603, lh=25223        # Table line
  ls[1]: ts=8, vpos=462486, lh=27707        # Picture line
  [0] 표: 12행×3열 ... tac=true
  [1] 그림: 169.9×97.7mm ... tac=true
```

같은 paragraph에 inline 컨트롤이 2개이고 각자 자신의 line_seg(ls[0], ls[1])에 배치되어야 정상.

## 원인 가설 (사전 코드 조사 결과)

| 후보 | 위치 | 내용 |
|------|------|------|
| **A** (최우선) | `src/renderer/layout.rs` 의 `layout_shape_item()` 부근 | TAC shape의 y 좌표를 `para_start_y[para_index]` (paragraph 시작 y)에 고정 → 같은 paragraph의 모든 inline shape가 같은 y를 공유 |
| B | `src/renderer/render_tree.rs` 의 `set_inline_shape_position()` | inline shape 위치를 paragraph 단위로만 저장 |
| C | `src/renderer/pagination/engine.rs` | line_seg 단위 분할(`PartialParagraph`)은 지원하나, 분할 후 inline shape의 y가 갱신되지 않음 |

**최우선 수정처는 A.** TAC shape를 그릴 때 자기가 속한 line_seg의 `vertical_pos`를 사용하도록 변경.

## 수행 단계 (요약)

1단계 — 가설 검증: 코드 reading + 최소 디버그 로그 추가로, 두 번째 inline shape의 y 좌표가 어떤 변수로부터 오는지 확정. 수정 범위 결정.

2단계 — 구현: TAC shape 배치 시 line_seg(또는 ts/character offset)에 해당하는 vpos를 사용하도록 로직 수정. PartialParagraph 분할에도 일관되게 동작하도록 함.

3단계 — 검증: 회귀 테스트(`cargo test`), 샘플 SVG 재생성 후 PDF와 비교, 기존 페이지네이션 케이스(특히 `re_sample_gen` 자동 재현 검증) 통과 확인.

## 비범위 (Out of scope)

- TAC가 아닌 shape(절대 위치 anchor)의 페이지네이션은 손대지 않음
- 문단 line 생성 자체(line_seg 분할 로직)는 변경하지 않음
- HWP/HWPX 파서는 변경하지 않음 (IR은 정상으로 확인됨)

## 검증 기준

1. 7쪽 SVG에 파이 차트가 더 이상 표시되지 않거나, 표와 겹치지 않음
2. 7쪽 표 영역이 PDF와 동일한 위치에 그려짐
3. 파이 차트가 다음 페이지로 흘러감
4. 기존 회귀 테스트 모두 통과
