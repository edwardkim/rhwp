---
kind: pr_review
status: review-only-fast-pass-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4084 검토 — #4054 이른 분할 보고서와 델타-PI 측정 도구

## 결론

**수용.** contributor의 원인 분해 보고서와 델타-PI 측정 방식은 보존할 가치가 있다. 다만 최초
도구는 `dump-pages` 실패를 빈 지도 행으로 기록하고 basename만 TSV key로 써, 재측정 대상이 누락될
수 있었다. 메인터너 보정 `607e2fb73`이 그 두 실패 계약을 차단했고, 해당 code head의 전체 CI와
CodeQL이 통과했다.

동일 날짜 오늘할일 충돌을 보존하기 위해 최신 `devel`을 정확히 한 부모로 둔 current-base bridge
`90c76c10e`을 추가했다. 이 commit은 `607e2fb73`의 full-CI 결과를 재사용할 수 있는 허용된
review-only bridge이고, 이 문서는 그 뒤의 single-parent trailing commit이다. 최신 head의 preflight
fast-pass와 `Build & Test` aggregate가 성공하고 `MERGEABLE/CLEAN`을 다시 확인한 뒤 merge한다.

## 검토 경로

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md,
           post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_external_pr.md, intake_and_review.md,
                  local_validation.md, review_only_fast_pass.md, post_merge.md
source head: ae7e4db3eab9c6d93a381eb823446be8129098ee
maintainer correction: 607e2fb734f4d2151acaeceff94bb792b5833db6
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4084](https://github.com/edwardkim/rhwp/pull/4084) |
| 작성자 | `planet6897` (Jaeuk Ryu, external contributor) |
| 대상 / source | `devel` / `planet6897:docs/4054-early-split-report` |
| 관련 issue | [#4054](https://github.com/edwardkim/rhwp/issues/4054) — 이미 closed, 이 PR은 보고서·측정 도구 기록 |
| 원 contributor commit | `ae7e4db3e` — 보고서와 초기 도구 |
| 메인터너 보정 | `607e2fb73` — 실패 은닉 방지와 회귀 시험 |
| 기준선 | 원 source 공통 조상 `570fa6e4f`; 검토 시점 최신 `upstream/devel` `3f7d87542` |
| current-base bridge | `90c76c10e` — 부모 `upstream/devel` `3f7d87542`와 #4084 code head를 자동 3-way로 합침 |
| 작성 시점 상태 | code head의 CI·CodeQL 성공, `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

## 변경 범위

- `mydocs/report/task_4054_early_split_20260806.md` — 이른 분할 원인 분해, 10k 측정 결과와
  델타-PI 재측정 절차를 기록한다.
- `tools/pi_map_hash.py` — `rhwp dump-pages`의 `(section, PI) -> 첫 페이지` 지도를 해시와
  페이지 수로 TSV에 기록한다.
- `tools/test_pi_map_hash.py` — 메인터너 보정의 성공·실패·입력 계약 회귀 시험이다.

renderer·layout·HWP/HWPX fixture·golden은 변경하지 않는다. 보고서에 인용한 10k 수치를 이 review의
새로운 시각 정합 판정 근거로 재사용하지 않았으므로 visual fixture 보조 경로는 적용하지 않았다.

## 메인터너 보정

최초 도구에서 확인한 문제는 다음과 같다.

1. `dump-pages`가 시간 초과·실행 실패·비정상 종료하면 빈 hash/pages 행을 써서, 두 실행의 실패 행이
   같을 경우 "지도 미변경"으로 오판될 수 있었다.
2. TSV `doc` 열이 `Path.name`만 저장해 서로 다른 경로의 동명 HWP/HWPX를 구분하지 못했다.
3. 비어 있는 chunk 목록과 페이지 헤더가 없는 성공 출력도 유효한 측정처럼 끝날 수 있었다.

`607e2fb73`은 실패 사유를 수집해 하나라도 실패하면 기존 TSV를 원자적으로 유지한 채 exit 1로
종료한다. chunk 입력은 하나 이상의 문서를 요구하고, 페이지 지도가 없는 출력도 실패로 분류한다.
성공 TSV는 원래 입력 경로를 기록하며, `--jobs`와 `--timeout`은 0 초과 정수만 받는다.

## 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest tools/test_pi_map_hash.py` | 4건 통과 — 동명 경로 보존, 실패 시 기존 TSV 유지, 페이지 지도 누락 거부, 빈 chunk 거부 |
| `python3 -m py_compile tools/pi_map_hash.py tools/test_pi_map_hash.py` | 통과 |
| `git diff --check` | 통과 |
| 실제 `target/release-test/rhwp dump-pages` 측정 | `samples/k-water-rfp.hwp`에서 27쪽, PI hash `7669fe076622b9b5` TSV 생성 |
| 최신 `upstream/devel` merge simulation | `3f7d87542` 위 충돌 없이 적용, 같은 Python 회귀 4건 통과 |
| current-base bridge | `607e2fb73`과 최신 `devel`의 자동 3-way tree와 일치하는 정확한 2-parent merge `90c76c10e` |
| GitHub CI | [CI 31098025096](https://github.com/edwardkim/rhwp/actions/runs/31098025096?pr=4084) 전체 성공 — Native Skia, archive 3개, slow·regular shard, aggregate 포함 |
| GitHub CodeQL | [CodeQL 31098025589](https://github.com/edwardkim/rhwp/actions/runs/31098025589?pr=4084) JavaScript/TypeScript·Python·Rust 분석 성공 |

## 위험과 한계

- 이 도구는 PI의 첫 등장 페이지와 총 쪽 수의 변화를 선별한다. 같은 시작 쪽 안에서 발생하는 줄 단위
  layout 차이는 별도 진단 또는 시각 검증이 필요하다.
- 해시 TSV는 성공한 전체 측정일 때만 갱신된다. 실패 문서는 재측정·원인 해결 뒤 다시 실행해야 한다.
- #4054는 이미 closed 상태다. 이 PR merge 뒤에도 별도 issue close 동작은 하지 않고 기존 close 상태만
  확인한다.

## 최종 권고

이 review·오늘할일 trailing commit의 latest-head fast-pass가 성공하면 PR을 merge한다. merge 뒤에는
`post_merge.md` 순서대로 merge SHA, #4054 closed 상태, contributor 감사 comment, `devel` sync와 이번
review/simulation branch 정리를 수행한다.
