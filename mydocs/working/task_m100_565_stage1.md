# Task #565 Stage 1 완료 보고서

## 1. 변경 요약

- `layout_inline_table_paragraph` 가 TAC 표만 순회하던 구조를 `composed.tac_controls` 기반 TAC 개체 순회로 보정했다.
- TAC 표는 기존 표 폭/측정/렌더링 경로를 유지했다.
- TAC 수식은 인라인 `EquationNode` 로 직접 렌더링하고 `inline_shape_position` 을 등록하여 fallback 중복 렌더링을 막았다.
- `inline_items` 는 text position 기준으로 정렬하여 control index 순서와 저장 위치가 어긋난 경우를 방어했다.
- 같은 text position 에 TAC 수식과 TAC 표가 함께 있는 경우, 두 개체를 같은 gap 에서 연속 배치하도록 보정했다.

## 2. 테스트

```bash
cargo test --lib test_565_exam_science_inline_tac_equations_are_not_collapsed
cargo test --lib renderer::layout::integration_tests -- --nocapture
```

결과:

- #565 단일 회귀 테스트 통과
  - 12번 `pi=61`
  - 15번 `pi=79`
- 18번 `pi=110`
- 19번 `pi=118`
- 19번 답문항 `pi=120` 의 `수식 → 분수 표 → 본문 텍스트` 순서 검증 추가
- layout integration tests 22개 통과
- 기존 경고 5건은 기존 코드의 naming / unused Result / unused_parens 경고이며 이번 변경과 무관

## 3. 시각 검증

```bash
cargo run --bin rhwp -- export-svg samples/exam_science.hwp -p 1 -o /private/tmp/rhwp_task565_after
cargo run --bin rhwp -- export-svg samples/exam_science.hwp -p 2 -o /private/tmp/rhwp_task565_after
cargo run --bin rhwp -- export-svg samples/exam_science.hwp -p 3 -o /private/tmp/rhwp_task565_after
```

- SVG: `/private/tmp/rhwp_task565_after/exam_science_002.svg`
- SVG: `/private/tmp/rhwp_task565_after/exam_science_003.svg`
- SVG: `/private/tmp/rhwp_task565_after/exam_science_004.svg`
- AS-IS PNG: `mydocs/working/task_m100_565_stage1/rhwp_task565_as_is_p2.png`
- AS-IS PNG: `mydocs/working/task_m100_565_stage1/rhwp_task565_as_is_p3.png`
- AS-IS PNG: `mydocs/working/task_m100_565_stage1/rhwp_task565_as_is_p4.png`
- PNG: `mydocs/working/task_m100_565_stage1/rhwp_task565_after_p2.png`
- PNG: `mydocs/working/task_m100_565_stage1/rhwp_task565_after_p3.png`
- PNG: `mydocs/working/task_m100_565_stage1/rhwp_task565_after_p4.png`

확인 결과: 2쪽 12번 문장과 그래프 라벨의 `X`, `A`, `B`, `C`, `D`, `m-4`, `m-2`, `m+2`, `m+4` 가 fallback 좌표에 겹치지 않고 인라인 위치에 표시된다.
3쪽 15번, 4쪽 18/19번 TAC 수식도 각 문장 안의 인라인 위치에 표시된다.
초기 캡처에서 보였던 2쪽 8번 본문 겹침은 `main` 기반 캡처에서 확인된 기존 레이아웃 문제였고, `origin/devel` 로 rebase 한 뒤 재생성한 캡처에서는 해소되어 있다.

### AS-IS / TO-BE 판독 포인트

| 페이지 | AS-IS | TO-BE |
|--------|-------|-------|
| 2쪽 12번 | 본문에 `X`, `A`, `B`, `C`, `D`, `m-4`, `m-2`, `m+2`, `m+4` 가 들어갈 자리가 비거나 쉼표만 남고, 일부 수식이 선택지 근처에 겹쳐 찍힘 | 같은 수식들이 문제 본문과 그래프 라벨 위치에 인라인으로 표시됨 |
| 3쪽 15번 | `W`, `X`, `Y`, `Z` 등 TAC 수식이 그래프 왼쪽에 뭉쳐 겹침 | 수식들이 설명 문장과 그래프 라벨의 원래 위치에 표시됨 |
| 4쪽 18/19번 | `x`, `a`, `b`, `Na`, `A(g)` 등 TAC 수식이 선택지/그림 주변에 뭉쳐 겹침 | 수식들이 표 아래 문장과 19번 그림 설명 줄에 인라인으로 표시됨 |

#### 2쪽 12번

AS-IS:

![2쪽 12번 AS-IS](task_m100_565_stage1/rhwp_task565_as_is_p2.png)

TO-BE:

![2쪽 12번 인라인 TAC 수식](task_m100_565_stage1/rhwp_task565_after_p2.png)

#### 3쪽 15번

AS-IS:

![3쪽 15번 AS-IS](task_m100_565_stage1/rhwp_task565_as_is_p3.png)

TO-BE:

![3쪽 15번 인라인 TAC 수식](task_m100_565_stage1/rhwp_task565_after_p3.png)

#### 4쪽 18/19번

AS-IS:

![4쪽 18/19번 AS-IS](task_m100_565_stage1/rhwp_task565_as_is_p4.png)

TO-BE:

![4쪽 18/19번 인라인 TAC 수식](task_m100_565_stage1/rhwp_task565_after_p4.png)

## 4. 리뷰 반영

- 최초 후처리 분기 가설은 코드 확인 후 폐기했다.
- 실제 원인인 `layout_inline_table_paragraph` 의 TAC 표 전용 순회 경로를 수정했다.
- 리뷰 후 #565 이슈 범위에 맞게 회귀 테스트를 12번 단일 문단에서 12/15/18/19번 대상 문단 전체로 확장했다.
- 추가 리뷰에서 같은 text position 의 TAC 수식/표가 하나의 gap 에 함께 들어오는 19번 답문항을 확인하고, 단일 gap 에 여러 TAC 개체를 배치하도록 수정했다.
- `cargo fmt` 로 발생한 광범위 formatting 변경은 모두 되돌리고, #565 관련 파일만 남겼다.

## 5. 다음 단계

- 작업지시자 확인 후 fork 브랜치에서 `devel` 대상으로 draft PR 생성.
