# Task M100 #6042 Stage 6 — 제출 준비

- Issue: [#6042](https://github.com/edwardkim/rhwp/issues/6042)
- 완료일: 2026-09-02 KST
- 상태: **Draft PR #6637 제출 — stack 재정렬·Full CI 대기**
- 검증 source: `4ea694ff3`
- 직접 base: #6467 `23b5bcf73f6e8659a90b25ebfde1311e1965364f`
- branch: `codex/issue-6042-page-virtualization`
- PR 제목: `perf(studio): 다중 페이지 스크롤 렌더링을 가상화한다`
- 최종 보고: [처리 결과](../report/task_m100_6042_report.md)
- PR 본문: [Draft PR 초안](../report/task_m100_6042_pr_body.md) — `--body-file`에 바로 사용
- 제출 PR: [#6637](https://github.com/edwardkim/rhwp/pull/6637)

## 1. 판정

#6042의 구현, 보정, 실제 문서 A/B, 시각·수명 회귀 검증과 최종 Studio 게이트를 완료했다. 검증 source는
#6467 head의 직접 descendant이고 base 대비 19개 커밋이다. 이 Stage 6 문서 커밋을 더해도 같은 lineage의
20개 커밋만 가지므로 **#6467 branch를 base로 하는 GitHub native Draft PR**을 만들 수 있는 로컬 상태다.

Stage 6 완료 commit 자체는 원격 변경을 하지 않았다. 후속 사용자 승인으로 code candidate를 push하고
Draft PR #6637을 생성했으며, 이 문서와 PR 번호 기반 self-review·오늘할일은 review-only trailing
commit으로 이어진다. Ready 전환, merge, 다른 contributor PR의 close는 여전히 포함하지 않는다.

## 2. 최종 stack·ancestry 감사

2026-09-02 원격 상태를 다시 조회했다.

| 위치 | PR / branch | 상태 | 판정 |
| --- | --- | --- | --- |
| bottom | #6458 `codex/issue-6040-zoom-topology` | OPEN / Draft, 최신 `devel`과 conflict | 전체 stack Ready 전 restack과 재검증 필요 |
| middle | #6467 `codex/issue-6041-budget-first-render-scale` | OPEN / Draft, base #6458, CLEAN | 현재 #6042의 정확한 직접 base |
| top | `codex/issue-6042-page-virtualization` | 검증 source 19 commits, Stage 6 commit 포함 20 | #6467 위 Draft PR 생성 가능 |

검증 source `4ea694ff3`에서 `git merge-base --is-ancestor 23b5bcf73... HEAD`는 통과했고,
`git rev-list --left-right --count 23b5bcf73...HEAD`는 `0 19`였다. #6458의 conflict는 현재 top PR을 Draft로
게시하는 것을 막지는 않지만, 세 PR을 Ready로 바꾸기 전에는 bottom부터 restack하고 descendant를 다시
쌓은 뒤 CI·시각 검증을 재자격화해야 한다. 현재 #6637의 base는 feature branch라 `main`/`devel`만
대상으로 하는 pull-request workflow가 실행되지 않는다. checks 없음은 green CI가 아니다. Stage 6에서
하위 branch는 수정하지 않았다.

## 3. 범위 감사

PR 채번 전 code candidate의 직접 base 대비 diff는 202 files, +489,954/-219다. 대부분은 반복 A/B의
raw JSON이다. 채번 뒤의 self-review·오늘할일은 별도 review-only trailing commit이며 제품 범위를
바꾸지 않는다.

| 범주 | 파일 / 크기 | 설명 |
| --- | ---: | --- |
| Studio source·test·E2E | 24 files, +4,063/-218 | 행/X 인덱스, visibility snapshot, page surface LRU, scheduler, 수명·회귀 테스트 |
| 계획·보고·피드백(원시 증거 제외) | 16 files, +2,378/-1 | Hyper-Waterfall 단계별 계약과 판정 |
| `issue6042*` 증거 | 162 files, 약 16MB | 실문서 raw trace, 정규화 snapshot, 비교 이미지·집계기 |

Rust source, Rust test/fixture, `Cargo.toml`, `Cargo.lock`, GitHub Actions 변경은 없다. 따라서 저장소 정책의
Rust fmt·Clippy·workspace build 묶음은 이 PR의 변경 범위에 해당하지 않는다. 계측은 DEV opt-in이며
production 경로에서는 설치되지 않는다. raw 증거에는 localhost URL, fixture 경로, browser 정보와 bounded
trace만 있고 계정·토큰·개인 문서는 없다.

증거 파일 수가 많아 GitHub diff가 원시 JSON을 접을 수 있다. 리뷰의 시작점은 최종 보고와 각 Stage의
`summary.json`이며, 제품 diff는 `rhwp-studio/src`, 계약 diff는 `rhwp-studio/tests`와 `rhwp-studio/e2e`로
분리해 확인하도록 PR 본문에 안내한다. 원시 증거는 재계산 가능성을 위해 보존한다.

## 4. 최종 검증

`4ea694ff3`에서 다음을 새로 실행했다.

| 게이트 | 결과 |
| --- | --- |
| `npm test` | 1,422 total / 1,421 pass / 1 skip / 0 fail |
| `npx tsc --noEmit --pretty false` | 통과 |
| `npm run build` | 통과 |
| `npm run e2e:manifest-check` | tracked 126 / manifest 126 |
| 핵심 8개 Node suite | 71/71 통과 |
| `ruler-resize.test.mjs --mode=headless` | 실제 DPR 1, 28/28 viewport snapshot 통과 |
| `page-virtualization-image-failure.test.mjs --mode=headless` | decode 실패 3, fallback render 3, 잔여 queue/error 0 |
| Stage 집계기 4종 재실행 | 통과 |
| `issue6042*` JSON 전체 `jq empty` | 통과 |
| ancestry / `git diff --check` | 통과 |

핵심 8개 suite는 scheduler, surface LRU, CanvasView scroll/surface 실행, render surface budget,
visibility snapshot, observation, smooth zoom을 포함한다. build의 CanvasKit `fs`/`path` externalization과
500kB chunk 메시지는 기존 경고이며 실패가 아니다. 첫 image E2E 시도는 기본 host CDP 주소가 닿지 않아
제품 실행 전에 timeout됐고, `--mode=headless`로 로컬 Chrome을 강제한 공식 재실행은 통과했다.

## 5. 제출 시 전달할 핵심 결과

- 178쪽 4열·34% cold jump에서 첫 visible p50/p95가 271.2/278.9ms에서 56.5/59.0ms로 줄고,
  `visibility.update`가 243.8/247.7ms에서 19.1/19.6ms로 줄었다. 같은 12쪽 raster를 callback 밖의 page
  slice로 옮긴 결과이며 모든 장치의 compositor frame 개선으로 일반화하지 않는다.
- 같은 cold 표본의 long task는 40건·합계 6,887ms에서 0건으로 줄었다. 한 쪽 raster 자체는 여전히
  선점할 수 없다.
- 178쪽 warm 왕복은 Stage 3 16.9/17.7ms, 보정 후 16.6/17.7ms이고 양쪽 모두 main raster 0이다.
- Stage 5에서 발견한 `exam_kor` 역방향 cache thrash는 target-state 예약과 선택 prefetch gate로 제거했다.
  역방향 raster는 Stage 3과 보정 모두 3회, retained p50/p95 증가는 4.2/4.6ms로 사전 경보선 안이다.
- scroll hot path는 현재 surface를 유지하고, 마지막 입력 150ms 뒤 충분히 노출된 visible 쪽만 64M hard
  gate 안에서 화질을 회복한다. 두 쪽 100%의 callback p50/p95는 0/0.1ms로 같고, 최종 화질 비용 때문에
  known-work는 33.3/54.1ms 늦어진다. 이 비용을 성능 개선으로 포장하지 않는다.
- 자동 34%의 DPR·배치, 34/50/100/200% zoom, Canvas2D·CanvasKit·auto fallback, 가로·대면·마지막 행,
  문서 교체, 편집·undo·redo, fractional DPR CSS geometry가 before와 같은 계약을 유지했다.

## 6. 의도적으로 포함하지 않은 것

장치 성능별 자동 확장, 짧은 문서 전체 surface 상주, worker/OffscreenCanvas, raster 내부 선점, #6521의
저배율 DPR 1.5 정책, 편집 focus를 viewport focus로 바꾸는 동작은 포함하지 않았다. #6467의 32M/40M
planner·retained 예산도 바꾸지 않았다. scroll 정착의 64M은 visible 품질 승격만 막는 별도 absolute
safety gate다.

## 7. 다음 승인 경계

승인에 따라 이 branch를 push하고, base를 `codex/issue-6041-budget-first-render-scale`로 지정한 Draft
PR #6637을 생성했다. 게시 직후 base/head·Draft·`MERGEABLE/CLEAN`, 한글 본문과 이미지 링크를 재조회해
확인했다. 채번된 self-review·오늘할일의 trailing commit push와 원격 본문 재검증까지 완료했다. 현재
feature branch base에서는 저장소 Actions trigger상 PR CI가 생성되지 않으므로 이를 통과로 간주하지
않는다. Ready 전환은 하지 않는다. 전체 stack Ready 전에는 #6458을 최신 `devel` 위에 restack하고
#6467·#6637을 순서대로 다시 쌓아 `devel` 대상 exact head에서 Full CI와 각 게이트를 재실행한다.
