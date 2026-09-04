---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 18 — 잔여 visual defect 재분석

## 출발 상태

Stage 17 commit `7b27b955e`는 표 25의 cell URL 각주를 실제 compose 높이로 예약하고, 고정-height native HWP5
RowBreak 표의 terminal footnote budget을 기준 PDF 경계에 맞췄다. 선택 검증에서 p78=105–106, p79=107–111,
p80=112–124가 확인됐고 p80 본문과 각주 112의 겹침은 해소됐다.

이 완료는 표 25/각주 105–124 범위에만 적용한다. native HWP는 220쪽, 한컴 기준 PDF는 215쪽이므로 전체
pagination fidelity가 완료됐다고 볼 수 없다.

## 이월 결함 목록

| rhwp 관측 페이지 | 결함 또는 검증 대상 | Stage 18 시작 상태 |
| --- | --- | --- |
| 37 | 그림 둘이 셋처럼 보이는 중복 paint 또는 fragment duplication | 해결 — 같은 text-start의 빈 guide 줄 둘에 TAC 그림 하나를 중복 귀속하던 결함 |
| 43 | 본문과 각주 영역 overlap | 미분석 |
| 54 | 본문 하단과 각주 영역 overlap | 미분석 |
| 66 | 표와 각주 영역 overlap | 미분석 |
| 76 | 표 24가 기준의 다섯 줄이 아니라 네 줄로 보이는 UI 관측 | native/WASM semantic owner 재확인 필요 |
| 77 | 그림 51/caption이 기준과 다른 physical owner에 놓이는 관측 | native/WASM semantic owner 재확인 필요 |
| 83 | `para=897` FullParagraph overflow 후보 | 미분석 |
| 87 | 기준 PDF와 semantic 흐름 차이 | 미분석 |
| 90 | 기준 PDF와 semantic 흐름 차이 | 미분석 |
| 99–100 | 기준 PDF와 semantic 흐름 차이 | 미분석 |
| 전체 | native HWP 220쪽 vs PDF 215쪽 | 개별 owner 정리 뒤에만 원인 군집화 |

## 우선순위와 방법

실사용 기준 PDF와 rhwp 출력 차이를 원인 미확정 상태에서 다루므로 bug-hunter playbook이 지배 절차다. visual sweep은
후보 page/overlay/render-tree를 좁히는 도구로만 쓴다.

1. p37의 그림 anchor·fragment owner를 raw HWP control, render tree, PDF review PNG로 먼저 확정한다.
2. 그 다음 p43/p54/p66 중 같은 footnote reservation 원인을 공유하는지 비교한다. page 번호만 같다는 이유로 하나의
   수정으로 묶지 않는다.
3. 각 원인군마다 분석 문서 → 최소 regression → 코드 수정 → selected visual sweep/대표 PNG → 결과 문서 순서로
   Stage를 분리한다.
4. 모든 이월 항목을 완료 판정하기 전에는 전체 220/215 차이를 단일 숫자 목표로 축소하지 않는다. 각 페이지의 semantic
   owner와 기준 PDF 대응을 먼저 기록한다.

Stage 17의 HWP/HWPX/PDF SHA-256과 p78–p80 증적은 [Stage 17 작업 기록](task_m100_3738_stage17.md)과
[visual sweep](task_m100_3738_stage17_visual_sweep.md)을 참조한다.

## p37 원인과 수정

기준 PDF 37쪽에는 그림 37과 그림 38 두 개만 있다. 수정 전 rhwp render tree에는 같은 `bin_id=54`의 그림 37이
`(94.5, 706.9, 254.0×221.0)`와 `(661.4, 987.2, 254.0×221.0)`에 두 번, 그림 38이
`(413.6, 728.0, 247.8×211.0)`에 한 번 있었다. 뒤의 그림 37은 페이지 하단에서 잘린 세 번째 그림으로 보였다.

원본 native HWP의 `pi=463`, `control=1`은 TAC 그림 하나인데 composed paragraph에는 `char_start=0`인 빈
guide 줄이 둘이다. `repeated_empty_tac_line_offset`은 TAC 수와 빈 줄 수가 정확히 같을 때만 줄별 귀속을 반환했다.
따라서 TAC가 하나인 이 경우 두 번째 줄이 `None`으로 기본 줄 범위 집합으로 되돌아가 같은 그림을 다시 방출했다.

`src/renderer/layout/paragraph_layout.rs`는 이제 TAC 수가 반복 빈 줄보다 적어도 앞 줄부터 한 번씩 귀속하고,
후속 줄에는 명시적인 빈 집합 `Some(vec![])`을 반환한다. 그림 수와 줄 수가 같은 기존 순차 배정은 유지한다.

## Stage 18 결과

- focused regression `native_hwp5_repeated_empty_guide_lines_emit_tac_picture_once`가 그림 37 하나와 PDF band 위치를
  고정한다.
- 기존 #3738 integration 파일은 8/8 통과했다. p78–p80 각주 경계 회귀도 함께 유지됐다.
- 수정 후 37쪽 render tree의 Image는 두 개뿐이며 bbox는 `(94.5,706.9,254.0×221.0)`와
  `(413.6,728.0,247.8×211.0)`이다.
- selected visual sweep와 3-way review는 [Stage 18 visual sweep](task_m100_3738_stage18_visual_sweep.md)에 기록했다.

이 Stage는 p37 그림 중복만 해결한다. 43·54·66·76·77·83·87·90·99–100쪽과 전체 220/215 pagination 차이는
해결하지 않았으며, 다음 Stage로 이월한다.
