---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3668.md
last_verified: 2026-08-01
---

# Task #3668 최종 보고 — LAYOUT_OVERFLOW_CELL 집계 표면과 원장 게이트

- Issue: [#3668](https://github.com/edwardkim/rhwp/issues/3668) (M100) —
  [#3236](https://github.com/edwardkim/rhwp/issues/3236) 의 sub-issue, 독립 축
- 브랜치 `local/task3668` / 2026-08-01 당일 완결
- 단계 기록: `mydocs/working/task_m100_3668_stage{1,2,3}.md`

## 문제와 해법

`LAYOUT_OVERFLOW_CELL`(셀 안 줄의 윗변이 쪽 하단 밖 = 그 줄 확정 소실, MATCH 80건
오탐 0 의 검증된 판정)은 stderr 로만 나가 아무도 보지 않았다 — #3236 에서 리포터도
내부도 7일간 놓쳤다. 이번 작업으로 세 표면을 만들었다:

1. **집계 코어** — layout 엔진 `Cell<u32>` 카운터. stderr 진단과 같은 조건에서만 증가
   (기존 출력 불변), `take_overflow_cell_lines()` 로 페이지 렌더 경계 귀속.
2. **CLI 표면** — `export-svg --json` 봉투에 페이지별 `overflowCellLines` + 문서 합계.
   capabilities 매니페스트 광고, 계약 테스트 확장. MCP `hwp_export_svg` 를 쓰는
   에이전트도 즉시 조회 가능.
3. **원장 게이트** — `tests/overflow_cell_baseline.rs` (기본 스위트, 작업지시자 (a)
   결정): samples 전수 렌더 → 신규 발생·증가만 실패, 감소는 래칫 조임, 원장 부패
   감지 포함. `RHWP_OVERFLOW_CELL_DUMP` 로 4.3.1 절차 준용.

## 원장 초기 동결 (662건 실측, 결정성 2회)

**0 이 아닌 문서 22종, 총 4,896줄** — `tests/fixtures/overflow_cell_baseline.tsv`.

| 상위 | 줄수 | 의미 |
|---|---:|---|
| issue2007_nested_cell_pagination | 2,980 | 작업지시자 지목 샘플 — 최대 발현처, 봉투·stderr 교차 일치 |
| table_giant_cell_overfill ×2 | 649 | 파일명 그대로의 현상 |
| 2025 행정업무운영 편람 .hwpx/.hwp | 96/53 | **#3674(쪽수 386 vs 한컴 383) 유력 단서** |
| 86712_regulatory_analysis ×2 | 66 | #1891/#2105 계열 |

이 원장은 "셀 내용이 조용히 사라지는 문서"의 첫 내부 지도다 — #3236 계열을 외부
리포트 없이 선제 검출·추적하는 기반.

## 검증

| 게이트 | 결과 |
|---|---|
| red-check (카운트 정확성) | #3236 상한 임시 제거 시 **정확히 23**(조사 실측 일치), page 0 귀속 정확 |
| 렌더 산출물 불변 | 카운터 전후 SVG 바이트 동일 (시각 판정 N/A 근거) |
| 계약 테스트 (render_manifest·cli_json·mcp drift·3236) | 33 passed |
| release-test 전체 (신설 2분 게이트 포함) | exit 0 |
| clippy `-D warnings` · fmt · Skia 3종 (58+2+4) · wasm 재빌드 | 전부 통과 |
| 게이트 결정성 | 전수 2회 dump 동일 |

## 남긴 것 (범위 밖 후속 후보)

- 기존 `take_overflows()`(item 수준 LayoutOverflow) 채널이 6개소에서 전부 버려짐 —
  두 채널 통합·소비 표면화는 별도 건.
- export-pdf 등 다른 방출 명령의 봉투 확장, wasm_api/스튜디오 노출.
- 10k 서베이 하니스 편입 — r24 템플릿 계보(원저자 축)라 제안만: 이슈 코멘트로 기록.
- 원장 22종 각각의 원인 조사 — 편람 2건은 #3674 에서 우선 소비 예상.

## 교훈

- **지목 샘플 즉답의 가치** — 작업지시자가 세션 중 지목한 issue2007 이 최대 발현처
  (2,980줄)였고, 새 표면이 완성 직후 바로 실전 질의에 쓰였다. 관측 표면은 만들자마자
  쓰인다.
- 진단과 카운터를 **같은 조건식 안에** 두면 계측 드리프트가 원천 차단된다.
