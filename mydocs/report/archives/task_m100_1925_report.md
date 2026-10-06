# 최종 보고서 — Task M100 #1925: 1차 리팩토링 라운드 2 (layout_composed_paragraph 해체)

- 이슈: #1925 (라운드 1 #1904, 계획 #1883, umbrella #1582) / 브랜치: `local/task1925`
- 기간: 2026-07-05 / 거버넌스: SOLID + 복잡도, stage-gate 준수

## 1. 결과 요약

`layout_composed_paragraph`에서 분기-비접촉 블록 3건을 추출해 **CC 288 → 226 (−21.5%),
3,771 → 3,071줄 (−18.6%)**. 전 추출 동작 불변(게이트 전수 통과, 행동 회귀 0건).

| 지표 (공식, 대시보드) | 영점 07-04 | r1 07-04 | **r2 07-05** |
|---|---|---|---|
| `layout_composed_paragraph` CC | 288 | 288 (전체 1위) | **226 (2위로 하강)** |
| 전체 최대 CC | 288 | 288 | **234** (`parse_paragraph_list`, HWP3) |
| CC>25 함수 수 | 80 | 81 | 82* |
| 테스트 | 2,820 | 2,858 | **2,875 / 실패 0** |

\* 분할 과도기 +1/라운드 (r1에서 실증된 패턴): 추출 신설 함수 중 `estimate_line_run_widths`
(CC 27)가 경계 초과로 가산 — 상위 함수 감량(−62)과의 교환으로 계획된 효과.

## 2. 추출 내역 (3건, 각 소스분기 0)

| 단계 | 신규 함수 | 줄수 | CC(공식) | 인터페이스 |
|---|---|---|---|---|
| 2 | `layout_click_here_and_bookmark_markers` | 331 | 24 | 15 param, shift 반환 (mut 캐리오버 제거) |
| 3 | `estimate_line_run_widths` | 292 | 27 | 13 param, `LineWidthEst` 2필드 반환 |
| 3 | `layout_empty_runs_line` | 182 | 14 | 참조 9 + `EmptyRunsLineVars`(스칼라 18) + mut 1 |

커밋: `38fdf903`(1단계 분석·구현계획) → `8999c723`(추출 1) → `272bcea8`(추출 2·3).

## 3. 게이트 (매 추출 공통 — baseline manifest 00014ecf)

fmt --check ✓ / clippy 경고 0 / `cargo test --profile release-test --tests` 2,875·실패 0 /
OVR baseline 5샘플 개체 회귀 0건 (2단계·3단계 각 1회, 총 2회 전수 실행).

## 4. 설계 판단 기록

- **의존 실측 기반 인터페이스 축소**: est 누산기 7종 중 하류 소비 3개 실측 →
  `LineWidthEst` 2필드로 축소. 주석 전용 가짜 의존 5개 배제(단어 매칭 한계 보정).
- **C 블록 스칼라 묶음**: 실의존 28개 → `EmptyRunsLineVars` struct (계획서의 >12 임계 조항 적용).
- **B(run 방출 루프, 1,056줄) 이연**: 의존 45/mut 11 — 라운드 1 축소 전례(32/9) 초과.
  `RunEmitState` mut-묶음 struct 설계 선행이 정공법 (다음 라운드 입력).
- 편차 1건: 추출 2·3을 1커밋으로 묶음(같은 단계 연속 작업, 게이트는 결합 1회 전수).

## 5. 다음 라운드 제안 (재평가 기준)

1. **`parse_paragraph_list`(HWP3, CC 234 — 현 1위)**: 파서 격리 영역이라 렌더 게이트 부담 낮음.
2. `layout_composed_paragraph` 잔여(226): B 블록 `RunEmitState` struct 설계 + 루프 중반
   정렬/줄배경 구간(분기 지표 109, brace 단일 블록 아님 — 의존 실측 선행).
3. `typeset_section_endnotes`(179): `EndnoteFlowState` 설계 (라운드 1 이연분).
4. 도구 정비 소품: metrics.sh 요약 줄 버그(CC>25 개수 자리에 전체 함수 수 787 출력),
   같은 날짜 스냅샷 라벨, tarpaulin 장시간 실행 대응(스크립트 차원 skip 옵션).

## 6. 산출물

- 계획: `plans/task_m100_1925.md`, `plans/task_m100_1925_impl.md`
- 단계 보고: `working/task_m100_1925_stage2.md`, `_stage3.md`
- 스냅샷: `mydocs/metrics/2026-07-05/` (dashboard.html + metrics.json + history)
