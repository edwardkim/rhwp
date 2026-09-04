---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3236.md
last_verified: 2026-08-01
---

# Task #3236 최종 보고 — 쪽 높이를 넘는 1×1 표 셀 내용의 분할 복원

- Issue: [#3236](https://github.com/edwardkim/rhwp/issues/3236) (M100) — 외부 리포터
  `hsKim25` (2026-07-24 등록, rhwp@core 0.7.19)
- 브랜치 `local/task3236` / 기간: 2026-07-31 ~ 08-01
- 단계 기록: `mydocs/working/task_m100_3236_stage{1,2,3}.md`
- 파생 이슈: [#3668](https://github.com/edwardkim/rhwp/issues/3668)
  (`LAYOUT_OVERFLOW_CELL` 집계·게이트 편입 — sub-issue, 독립 진행)

## 증상과 근인

첫 쪽에서 시작한 표가 쪽 높이에 도달해도 다음 쪽으로 분할되지 않고, 셀 내용이 쪽
하단 밖에서 clip 되어 소실됐다(쪽 밖 23줄, 최대 초과 468px). 한컴 2020 정답지는 같은
문서에서 셀을 쪽 경계에서 분할해 p2 로 잇는다.

근인은 [#1891](https://github.com/edwardkim/rhwp/issues/1891)(2026-07-05,
`193cd714d`)이 도입한 **단일행 선언-신뢰 특례**다. 1×1 RowBreak 표에서 저장 선언
높이가 현재 쪽 잔여에 들어가면 측정 높이를 무시하고 통짜 배치하는데, **측정 초과의
상한이 없어** 선언 322.6px vs 측정 910.8px(2.82배)인 리포터 문서까지 포섭했다.
인트라-로우 분할 인프라 자체는 정상이었다 — 특례가 그 경로를 우회시킨 것이다
(작업지시자의 "예전에 처리했던 것의 회귀" 기억과 정합. 리포터의 0.7.19 는 7/5 이후
버전으로 시점도 일치).

## 수정

`src/renderer/typeset.rs` +11줄 (로직은 조건 1줄):

```rust
const SINGLE_ROW_DECLARED_TRUST_MAX_RATIO: f64 = 1.5;
// single_row_object_declared_fits_current 요건에 추가:
&& table_total <= declared_object_total * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
```

폰트 대체 팽창은 인접 가드들 기준 10~20% 수준이므로 1.5배 초과는 셀 내용이 진짜로
큰 것이다 — 이때는 특례를 접고 인트라-로우 분할 경로에 맡긴다. 64px~1.5배 구간의
특례는 보존되어 #1891 의도(쪽수 보정)가 유지된다.

## 검증 (전부 통과)

| 게이트 | 결과 |
|---|---|
| 신설 회귀 테스트 + red-check | `issue_3236_split_single_cell_table.rs` — 상한 제거 시 정확히 FAILED |
| 분할점 정합 | p2 시작 "경과되지 않은 외국인투자기업인…" — **한컴과 글자 단위 일치**, p1 절단점("…5년이")도 동일 |
| 쪽별 분포 | 84%/16% → **46%/54%** (한컴 45%/55%) · 쪽수 2 불변 · `LAYOUT_OVERFLOW_CELL` 23→**0** |
| release-test 전체·clippy·Skia 3종·wasm 재빌드 | exit 0 |
| issue_1891 쪽수 fixture · table/split/rowbreak 계열 | 3건 + 118건 통과 |
| samples 664건 쪽수 A/B (수정 전/후) | **차이 0건** — 행동 변화는 본 fixture 국한 |
| IR field sweep (신규 fixture) | 발산 0건 (+ #3507 개선 미반영 래칫 2행 조임) |
| studio CDP 잉크 | p2 하단 0.02% → **14.01%** |
| **작업지시자 시각 판정** | **통과** (2026-08-01, `/mnt/e/hwp/swap3236/` 스왑 세트) |

## 교훈

1. **정답지 우선** — 초기 두 차례 오판("재현 안 됨" → "Canvas 백엔드 결함")은 정답지
   없이 자기 산출물끼리 비교한 결과였다. 정답지 PDF 를 먼저 만들었다면 첫 측정에서
   잡혔다.
2. **죽은 경로 계측 주의** — `pagination/engine.rs` 5지점 계측이 전부 미도달이었던
   이유는 그 코드가 실사용 경로가 아니어서였다(실경로는 `typeset.rs`). 계측이 안
   찍히면 "조건이 거짓"이 아니라 "경로 자체가 아님"을 먼저 의심할 것.
3. **측정법도 검증 대상** — "63% 소실"의 자릿수는 SVG 노드 추출 vs pdftotext 집계
   차이였다(clip 된 텍스트도 SVG 에는 방출된다). 분포·경계 텍스트가 올바른 지표였다.
4. **병렬 스윕 상이 건은 단독 재실행으로 재확인** — 664건 A/B 의 유일 상이 건은
   하니스 아티팩트였다(5회 재실행·SVG 바이트 대조로 판명).
5. **특례에는 상한을** — 선언-신뢰류 휴리스틱이 상한 없이 도입되면 극단 문서가
   조용히 포섭된다. 인접 가드들(10%/64px, 20px)처럼 신뢰 범위를 명시해야 한다.

## 후속

- PR 준비 완료 — 생성은 별도 승인.
- merge 후 리포터 `hsKim25` 에게 해결 사실·적용 버전·확인 방법 회신
  (docs_and_git_workflow 외부 회신 의무). 리포터가 언급한 "확장에서는 정상"은
  버전 차이(확장이 더 새 코어를 탑재했을 가능성)로 설명되나, 확정 진단은 회신에
  포함하지 않고 재현 확인 사실만 전한다.
- #3668 (진단 집계 인프라) — 독립 진행.
