---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6081 self-review — W8 rank 1 문체부 바탕체 교정 qualification

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `b83d9d08e3e3c644337d9a21ee547c1f9136183c`

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6081](https://github.com/edwardkim/rhwp/pull/6081) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4967](https://github.com/edwardkim/rhwp/issues/4967), parent [#4960](https://github.com/edwardkim/rhwp/issues/4960) |
| base / head | `devel` / `task_m100_4967_v2` |
| 규모 | 22 files, +3,664 / -28, 8 commits |
| 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=CLEAN` |
| 최신 base | `upstream/devel@35c270f47`, code candidate보다 2커밋 전진 |

GitHub 상태는 변할 수 있으므로 merge 직전에 다시 확인한다. 최신 base의 2커밋은 시각 검증 절차 문서 3개만
바꿨고, `git merge-tree --write-tree b83d9d08e upstream/devel`은 충돌 없이 tree
`55c6b3d48cbf2da2dace6cd5ee8ca48063ef334f`를 생성했다.

1,000줄을 넘는 변경의 대부분은 단계별 판정을 재현하는 projector·계약 테스트·canonical JSON과 계획·보고서다.
Q1의 runtime boundary 출력은 Q2의 metric hypothesis 입력이므로 임의 분할하면 판정 계보와 hash 계약을
끊는다. 대형 PR 규칙에 따라 즉시 admin merge하지 않고 self-review, 전체 CI, review-only tail과
메인테이너 merge 판단을 독립 gate로 유지한다.

별도 `pr_6081_review_impl.md`는 만들지 않는다. 외부 PR 보정, 복수 PR 통합 또는 source 충돌 해결이 아니며,
self-review에서 발견한 정정은 아래의 review 시점 문서 현행화뿐이다.

## 목적과 변경 범위 정합성

#4967 rank 1 lane의 목적은 높은 위험량만으로 제품 alias를 추가하는 것이 아니라 `문체부 바탕체`의 runtime
miss가 실제인지 판정하고, 정확히 한 decision plane의 교정이 조판 이득을 만드는지 검증하는 것이다.

- HWPX와 결정적으로 변환한 HWP5 fixture에서 각각 1,556건을 관찰해 runtime miss와 첫 divergence
  `layout-name`을 고정했다. W4 face-miss는 projection 오탐이 아니다.
- 가상 `문체부 바탕체 -> MBatang` relation과 exact `MT.TTF hmtx`를 비교한 결과 전체 layout-bearing
  2,351 codepoint, transform 13축, fixed-frame 6축의 advance·crossing delta가 모두 0이었다.
- layout 이득이 없는 metadata-only alias를 제품 규칙으로 만들지 않고 최종 disposition을 `no-change`로
  확정했다.
- 제품 metric DB·registry·fallback·renderer·paint·font supply source는 변경하지 않았다.
- #4967은 rank 7과 evidence-reopen lane이 남은 tracker이므로 이 PR이 merge돼도 닫지 않는다.

## self-review findings

### [P2][해결] 최종 보고서와 오늘할일에 PR 생성 전 상태가 현재 사실처럼 남았다

code candidate의 계획·보고서는 작성 당시 사실인 “rank 1 연결 PR 없음”, “원격 repository 미변경”과
“PR 생성 전 재검증” 상태를 담고 있었다. PR #6081 생성과 전체 CI 성공 뒤에도 그대로 merge하면 현재
통합 상태와 모순된다.

review-only tail에서 기술 판정과 수치는 바꾸지 않고, 착수 시점과 현재 상태를 구분했다. PR 번호, 정확한
code candidate, 최신 base의 문서-only 전진, merge-tree 결과와 남은 fast-pass·merge gate를 계획·보고서·
오늘할일에 현행화했다.

### 추가 blocker 없음

- Q0 projector는 bounded streaming journal만 읽고 regular non-symlink 입력, exact digest와 owner-only
  private output mode `0600`을 fail-closed로 검사한다. 공개 결과에는 private 문서명·경로·본문·식별 hash와
  font bytes가 없다.
- Q1은 `maxCharacters=4096` 상한, native/WASM canonical byte parity, regular-file·size guard와 output
  symlink 거부를 유지한다.
- Q2는 exact font·fixture hash, canonical input, exhaustive character/style domain guard와 child process
  60초·64MB 한계를 검사한다.
- rank 8 공용 helper는 기존 동작을 계약 테스트로 고정한 채 일반화·export했으며 같은 계산을 복제하지 않았다.
- 새 integration source, generated suite·manifest 또는 Cargo target 파생물은 PR diff에 없다.

## 렌더·시각 증적 판정

PR이 공개 HWP fixture를 추가하므로 최신 `visual_fixture_evidence.md`를 적용해 필수 여부를 판정했다. 다만
변경된 실행 코드는 qualification projector뿐이고 제품 `src/renderer`, layout, typeset, paint, WASM 출력
경로를 바꾸지 않는다. 특정 페이지·표·줄바꿈·clipping·배치 개선도 주장하지 않는다.

HWP는 제품 시각 회귀 fixture가 아니라 기존 공개 HWPX의 runtime name trace를 독립 형식에서 확인하기 위한
결정적 변환본이다. `rhwp convert --verify --verify-pages`에서 반복 byte digest가 같고 IR difference 0,
page count 1→1을 확인했다. 따라서 “사용자-visible 경로 변경과 HWP/HWPX/PDF fixture의 결합” 또는
“특정 시각 개선 주장”이라는 직접 visual sweep 필수 조건에 해당하지 않는다. 별도 PDF·PNG를 추가하지
않았으며, 이 review는 “시각 검증 통과”를 주장하지 않는다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Q0 Python 계약 | 9/9 통과 |
| Q1·shared Node 계약 | 8/8 통과 |
| Q2·shared Python 계약 | 12/12 통과 |
| native `rhwp-q-font-trace` | build 통과 |
| Docker WASM | optimized build 6분 02초, tracked `pkg` delta 0 |
| native/WASM trace | HWPX·HWP5 각각 1,556건 byte-exact |
| fmt·diff·changed Markdown link·privacy | 모두 통과 |

전체 문서 metadata 검사는 변경하지 않은 기존 4문서의 16건을 보고했으며, 이번 #4967 계획·보고서·
working 문서는 목록에 없다. 기존 repository debt를 이 PR의 성공으로 오인하거나 범위를 넓혀 수정하지 않는다.

## GitHub Actions

code candidate `b83d9d08e`의 [CI run 32874169851](https://github.com/edwardkim/rhwp/actions/runs/32874169851)은
preflight, Lint, Native Skia, Frontend package, archive A/B/C와 모든 test worker, Build & Test aggregate가
성공했다. [CodeQL 32874169563](https://github.com/edwardkim/rhwp/actions/runs/32874169563),
[Proptest 32874169523](https://github.com/edwardkim/rhwp/actions/runs/32874169523),
[Adapter inter-diff 32874169509](https://github.com/edwardkim/rhwp/actions/runs/32874169509)도 같은 SHA에서
성공했다. 정책상 WASM Build와 Frontend unit gates의 skip 외에 실패·대기 check는 없다.

최신 base 전진은 review 절차 문서만 바꿨고 merge-tree도 clean이므로 review 기록을 위해 source branch에
devel을 merge·rebase하지 않는다. 현재 self-review·오늘할일·계획·보고서 상태 정정은 이 녹색 code
candidate 뒤의 `mydocs/` 한정 single-parent trailing commit이다. push 뒤 review-only fast-pass가 정확한
candidate를 재사용하고 최신 required aggregate가 성공하는지 확인해야 한다.

## 최종 권고

runtime miss는 실제지만 가상 name relation과 exact metric이 조판 이득을 만들지 않으므로 제품 alias를
추가하지 않은 `no-change` 결론은 증거와 일치한다. 비공개 자료 경계, resource limit, native/WASM parity,
최신 base 호환성과 Full CI에서 추가 blocker는 발견하지 않았다.

self-review는 **완료 / 조건부 merge 권고**다. review-only trailing head의 fast-pass, 최신
`MERGEABLE/CLEAN`과 메인테이너의 별도 merge 승인을 확인하기 전에는 merge하지 않는다. #4967 tracker는
merge 후에도 열린 상태로 유지한다.
