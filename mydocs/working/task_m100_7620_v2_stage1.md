# #7620 후속 1단계 — RowBreak 자리 차지 표의 첫 조각 바닥 예산

2026-11-10 KST. [이슈 #7620](https://github.com/edwardkim/rhwp/issues/7620)의 남은 항목 중
첫째(본문 바닥 경계의 예산 판정)를 처리한다. base는 `upstream/devel`
`b5064fae1eefff92d60464c51844a38debec09f9`, 브랜치는 `task/7620-rowbreak-margin-budget`이다.
선행 작업은 #7644(#7681로 통합)와 planet6897의 #8149(빈 host 첫 조각 바닥 예약)다.

## 확정 범위

- 대상: 쪽 경계 나눔(RowBreak, `TablePageBreak` 2)·비TAC·자리 차지·문단 기준·오프셋 0 표가
  쪽 바닥에 걸릴 때 첫 조각에 실을 행이 있는지 판정하는 예산.
- 비범위: 56345 문서의 7·8·9·17·18쪽 부족분(#7806, jangster77 담당), footer 문서 11쪽
  높이 초과(다음 단계), CellBreak 실제 이월(다음 단계).
- 초기 가설이던 `entry.rs`의 `whole_frame_budget` 실패 fallback(상자 소실)은 생성 대조군과
  기존 회귀에서 재현되지 않았다. 실제 결함은 아래의 분할 판정이었다.

## 독립 기준(입력 생성 → 기준 → 관측값)

입력은 `samples/issue7620/generate-budget.mjs`(devel web WASM API)로 만든 HWP5 대조군이다.
앞 문단 N개 · R×2 표 · 빈 문단 · 뒤 문단이며, 표는 바깥 여백 위·아래 2835HU(37.8px)다.
기준은 같은 입력을 한컴독스 웹 한글에서 인쇄한 PDF(`pdf/issue7620/*-hancomdocs.pdf`)다.

| 입력 | N·R | 한컴 관측(96dpi) |
|---|---|---|
| b | 37·2 | 1쪽: 앞 문단 37 뒤에 표 문단과 뒤 문단이 이어짐. 2쪽: 표 전체가 170.1px(본문 윗변 + 바깥 위 여백)에서 시작 |
| c | 39·2 | 표가 2쪽 170.1px에서 시작, 뒤 문단은 표 아랫변 + 바깥 아래 여백(242px) |
| d | 20·40 | 1쪽 표 596–955px(아래에 37.8px 남김), 2쪽 170–495px, 뒤 문단 554px |

따라서 한/글은 `바깥 위 여백 + 첫 행 + 바깥 아래 여백`이 현재 쪽에 들어갈 때만 행 경계에서
나누고, 첫 행도 들어가지 않으면 표만 다음 쪽 위로 옮긴다. 표 문단과 뒤 글줄은 현재 쪽에 남는다.
보조 탐침(쪽 나눔 None 표, 여러 행 경계)은 기존 출력과 같아 변경하지 않았다.

## 원인 경로

`TypesetEngine` 표 배치(`src/renderer/typeset/table/block/entry.rs:107` → `prepare.rs`)에서
첫 행 이월 판정(`remaining_on_page_for_first_unit`)은 저장 원점 상자 `p`를 쓰는
`fragment_host_placement` 분기에서 바깥 아래 여백만 빼고 #8149의 빈 host 바닥 예약을 빼지 않았다.
b에서 이 판정은 49.7px가 남는다고 보고 첫 행(약 17.1px)이 들어간다고 결론냈다. 반면 실제 행
스캐너(`continuation/fragment/budget.rs`)는 첫 조각에서 같은 예약을 무조건 빼 11.9px 예산을
계산했고, 0행 예산에서 첫 행을 억지로 실어 표를 나눴다. 측정(이월 판정)과 배치(행 스캐너)가
다른 예산을 소비한 것이 원인이다.

## 수정

1. `prepare.rs`: 첫 조각의 바닥 예약을 `fragment_outer_bottom + empty_host_reserve`로 묶어
   `fragment_host_placement` 분기에서도 스캐너와 같은 값을 뺀다. 두 경로가 같은 예약을 소비한다.
2. `deferred_float.rs`: 첫 행의 최소 단위(첫 줄 + 안 여백)와 바깥 위·아래 여백이 현재 쪽에
   들어가지 않고 표 상자 전체는 새 쪽에 들어가는 RowBreak 표를, 기존 None 표의 이월 경로
   (`try_defer`)로 보낸다. 표만 다음 쪽 위로 옮기고 host·뒤 글줄은 현재 쪽에 남긴다.
   쪽 하나보다 큰 표는 여기서 제외되어 기존 분할 경로를 따른다.

경로·컷·종료 대조:

| 분기 | 시작/끝 컷 | 요구 높이 | 예산 실패 시 | 실제 배치 |
|---|---|---|---|---|
| b(첫 행 불가, 통째 가능) | 컷 없음 | 바깥 위 + 표 + 바깥 아래 | 표만 다음 쪽으로 이월 | 2쪽 170.1px, 행 1·2 모두 |
| c(앵커만 가능) | 컷 없음 | 같음 | 같음 | 2쪽 170.1px, 뒤 문단 표 아래 |
| d(여러 행 가능) | 행 경계 | 행 누적 + 바깥 아래 | 남은 행 이월 | 1쪽 1–N행, 2쪽 나머지, 행 1–40 각 1회 |
| None 표 | 기존과 같음 | — | — | 변경 없음 |

## 검증(작업 트리, 커밋 전)

- Visual Sweep: `mydocs/pr/assets/issue_7620_rowbreak_bottom_margin_budget/README.md`.
  b는 수정 전 Native 89.88/59.65%, fresh WASM 90.21/60.49%에서 수정 후 양쪽 100.00/100.00%다.
  c·d는 수정 전후 100.00%다. 1쪽 바닥과 2쪽 표 위치를 review PNG로 직접 판독했다.
- `layout-anomaly`: b·c·d 모두 0건. 래칫 baseline 추가가 필요 없다.
- 회귀: `tests/cases/issue_7620_rowbreak_float_bottom_margin_budget.rs`(관계형 3건).
  수정 전 실패·수정 후 통과와 전체 게이트는 커밋 뒤 review worktree에서 기록한다.
