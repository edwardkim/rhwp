# 20 — 산출물 카탈로그

`--out-dir` (또는 `output/fidelity/<키>/`) 아래 파일이다. 모두 기존
하네스가 쓴다. 에이전트가 새 파일 이름을 발명하지 않는다.

## 항상 또는 거의 항상

| 파일 | 모드 | 권위 |
| --- | --- | --- |
| `provenance.tsv` | 항상 | 재현 최소 |
| `run-state.tsv` | 항상 | 완전성. missing 이면 exit ≠ 0 |
| `text-report.tsv` | 항상 | 소실/과잉/치환 후보 |
| `report.tsv` | 항상 | 시트면 랭킹, text-only 면 `not-run` |
| `svg-glyph-risk-report.tsv` | 항상 | PUA/FFFD |
| `text-owner-shift-candidates.tsv` | 항상 | 인접 owner |
| `text-owner-sequence-candidates.tsv` | 항상 | 순서 보존 이동 |
| `page-boundary-fidelity-candidates.tsv` | 항상 | 경계 큐 |
| `visible-text-excess-candidates.tsv` | 항상 | clip 안 과잉 |
| `page-count-ledger.tsv` | 항상 | 쪽수 후보 |
| `svg/` | export 후 | 캐시 |
| `svg/export-svg-manifest.json` | `--export-all-svg` | 매니페스트 |

## 시트 모드만

| 파일 | 권위 |
| --- | --- |
| `cmp-pNNN.png` | 사람 눈 |
| (내부) PDF/SVG 래스터 | 도구 임시 |

## `--layout-ledger` 만

| 파일 | 의미 |
| --- | --- |
| `layout-candidates.tsv` | body↔각주, 표↔footer, frame 밖, square wrap |
| `table-fragment-candidates.tsv` | 같은 `(pi,ci)` 인접 쪽 |
| `table-cell-text-overlap-candidates.tsv` | 셀 안 중복 paint |
| `table-cell-text-boundary-candidates.tsv` | 셀 경계 침범 |
| `svg-text-band-clip-candidates.tsv` | glyph band 부분 절단 |
| `svg-table-border-clip-candidates.tsv` | 세로 외곽선 clip |
| `svg-table-horizontal-border-clip-candidates.tsv` | 가로 외곽선 clip |
| `float-owner-shift-candidates.tsv` | owner + 다음 쪽 상단 float |
| `render_tree/` | `export-render-tree` 캐시 |

모든 ledger 행은 **candidate** 다. PDF 시각 대조 전에 결함 확정
금지. README 가 파일마다 같은 문장을 반복한다.

## run-state

```
field	value
mode	text-only
requested_pages_1based	1,2,3
completed_pages_1based	1,2,3
missing_pages_1based	-
run_state	complete
```

`incomplete` 이면 종료 코드 1. 부분 랭킹을 전수로 포장하지 않는다 (F12).

## 읽는 순서 (권장)

1. provenance — 무엇을 비교했는지
2. run-state — 다 돌았는지
3. page-count-ledger — 쪽수 후보
4. text-report + glyph-risk — 글자 후보
5. boundary / owner / layout — 쪽 경계 후보
6. report.tsv + cmp-pNNN — 최악 쪽 눈

질문이 1에서 이미 답(오라클이 참고 등급) 이면 6까지 가지 않는다 (F16).

## 에이전트 금지

- 원장을 합친 `summary.json` 을 이 스킬이 새로 쓰기
- `report.tsv` 헤더를 바꾸기
- 산출을 원본 `samples/` 에 되돌리기
- gym scorecard 형식으로 변환
