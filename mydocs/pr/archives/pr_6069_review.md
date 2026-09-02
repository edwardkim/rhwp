---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-25
---

# PR #6069 self-review — W8 rank 8 face 교정 qualification

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `a637c9c8e7348bf836cfae1658f41244d392e00a`

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6069](https://github.com/edwardkim/rhwp/pull/6069) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4967](https://github.com/edwardkim/rhwp/issues/4967), parent [#4960](https://github.com/edwardkim/rhwp/issues/4960) |
| base / head | `devel` / `task_m100_4967` |
| 규모 | 31 files, +6,657 / -11, 8 commits |
| 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=CLEAN` |

1,000줄을 넘지만 제품 교정 하나를 큰 덩어리로 넣은 변경은 아니다. 단계 계획·보고서·결정적 JSON과 세
qualification projector가 대부분이며, 제품 Rust는 읽기 전용 same-snapshot query와 CLI, stored-row probe,
진단 계수기 격리로 한정된다. 대형 PR 규칙에 따라 즉시 admin merge하지 않고 code review, 전체 CI,
review-only tail과 작업지시자 merge 판단을 독립 gate로 유지한다.

별도 `pr_6069_review_impl.md`는 만들지 않는다. 외부 PR 보정이나 복수 PR 통합이 아니고, self-review에서
발견한 정정은 아래의 review 시점 문서 현행화뿐이다.

## 목적과 변경 범위 정합성

#4967 rank 8 lane의 목적은 exact metric을 적용하는 것이 아니라 실제 장평·자간·fixed frame cohort에서
교정 적격성을 판정하는 것이다. PR은 W3·W4·W5·W7.5 증거를 재사용하고 Q0의 동일 6문서만 제한 재판정했다.

- exact TTF와 pinned CDN OTF·WOFF2의 공통 cmap 25,970개 advance mismatch 0을 확인했다.
- 하나의 `PageRenderTree`에서 Font Decision Trace, `TextLine` frame/context, run membership와 production
  stored-row disposition을 함께 읽어 Q3의 snapshot 미조인 4,397자를 0으로 해소했다.
- modelled 회귀 5줄 가운데 현행 cache가 수용한 `admitted` 4줄과 표 셀 `+1.92px` 신규 overflow를
  평균 개선과 상쇄하지 않고 rank 8 일괄 exact metric 후보를 `no-change`로 기각했다.
- metric DB·canonical registry·fallback·paint·font supply와 제품 렌더 출력은 변경하지 않았다.
- #4967은 rank 1·7과 evidence-reopen lane이 남은 tracker이므로 이 PR이 merge돼도 닫지 않는다.

## self-review findings

### [P2][해결] 최종 보고서에 PR 생성 전의 volatile 상태가 현재 사실처럼 남았다

code candidate의 최종 보고서는 작성 당시 사실인 “연결된 열린 PR 없음”, “원격 push·PR 미생성”을 담고
있었다. PR #6069가 생성된 뒤에도 이 문구를 그대로 merge하면 완료 보고서가 현재 통합 상태와 모순된다.

review-only tail에서 보고서의 기술 판정과 수치는 바꾸지 않고, 착수 시점에는 PR이 없었으나 현재
code candidate가 #6069로 제출됐다는 사실과 self-review·merge가 별도 gate라는 상태만 현행화했다.
오늘할일의 완료된 계수기 정정 문장도 미래형에서 과거형으로 바로잡았다.

### 추가 blocker 없음

- same-snapshot API는 이미 만든 tree를 기존 trace 계산과 line/frame probe가 함께 소비하며 문서 IR이나
  layout cache에 행을 publish하지 않는다. 독립 probe의 `admitted`·`rejected`·`unmodelled`·`notApplicable`
  구분은 HWP/HWPX 버전이 아니라 현재 frame capability를 기능 탐지한다.
- `maxCharacters`는 기본 1,024·상한 4,096이고, private projector는 page trace 완전성·child timeout·출력
  크기·문서 크기·regular-file·corpus-root 이탈을 fail-closed로 검사한다.
- 공개 aggregate에는 private 문서명·경로·본문·식별 hash와 font bytes가 없으며, private detail은 owner-only
  local output으로 유지된다.
- current-thread page-tree counter는 기존 process-global 성능 counter를 대체하지 않는다. 모든 uncached
  build는 전역과 현재 스레드를 함께 기록하므로 기존 전용 가드와 generated suite 격리를 동시에 보존한다.
- `tests/cases/`의 기존 두 integration source만 수정했고 generated suite·manifest·Cargo target 파생물은
  PR diff에 없다. `src/**`의 `#[cfg(test)]`도 늘리지 않았다.

## 렌더·시각 증적 판정

HWP/HWPX fixed-frame과 overflow 판정을 다루므로 `visual_fixture_evidence.md`를 적용했다. 그러나 이 PR의
결론은 제품 renderer를 교정하지 않는 `no-change`이며, 제품 layout·paint·WASM 출력 경로의 결과를 바꾸지
않는다. 실제 6문서 관측은 비공개 문서 identity를 노출하지 않는 aggregate qualification 근거이고, 제품
fidelity 통과를 주장하는 visual sweep이 아니다.

공개 fixture에서 same-snapshot frame/cache 경계를 검증했고, GitHub Render Diff의 Canvas visual diff도
같은 code candidate에서 성공했다. 별도 golden·기준 PDF 변경이 없고 사람이 판정할 제품-visible delta가
없으므로 대표 review PNG는 추가하지 않는다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Python qualification 계약 | 17/17 통과 |
| Node trace 계약 | 4/4 통과 |
| fresh generated #4967 suite | 기본 병렬 3회 420/420, 직렬 140/140 |
| 기존 process-global 성능 가드 | 4/4 통과 |
| release-test nextest | 8,313/8,313 통과, 정책 skip 42 |
| Native Skia 공식 3종 | root 3,946 pass / 13 ignore + 내부 crate 182 pass, 2/2, 4/4 |
| Clippy·fmt·diff·Markdown·privacy | 모두 통과 |
| Docker WASM | optimized build 5분 56초, tracked `pkg` delta 0 |

nextest는 현재 host의 0.9.137로 실행되어 저장소 권장 0.9.140 경고가 있었지만 8,313건은 모두 통과했다.
제품 조판 절대 성능은 측정하지 않았다. 결합 query는 쪽 tree를 한 번 만들며 계수기 정정의 제품 hot path
추가 비용은 uncached build당 current-thread `Cell` 갱신 1회다.

## GitHub Actions

code candidate `a637c9c8e`의 [CI run 32855430013](https://github.com/edwardkim/rhwp/actions/runs/32855430013)은
Lint, Native Skia, Frontend package, archive A/B/C와 모든 test worker, Build & Test aggregate가 성공했다.
[CodeQL 32855429643](https://github.com/edwardkim/rhwp/actions/runs/32855429643),
[Render Diff 32855429664](https://github.com/edwardkim/rhwp/actions/runs/32855429664),
[Proptest 32855429488](https://github.com/edwardkim/rhwp/actions/runs/32855429488),
[Adapter inter-diff 32855429551](https://github.com/edwardkim/rhwp/actions/runs/32855429551)도 같은 SHA에서
성공했다. 정책상 WASM Build와 Frontend unit gates의 skip, GHAS 집계의 neutral 외에 실패·대기 check는 없다.

현재 self-review·오늘할일·보고서 상태 정정은 이 녹색 code candidate 뒤의 `mydocs/` 한정 single-parent
trailing commit이다. push 뒤 review-only fast-pass가 위 candidate를 재사용하고 최신 required aggregate가
성공하는지 다시 확인해야 한다.

## 최종 권고

일괄 exact metric은 개선 331줄보다 적은 회귀 5줄도 허용하지 않는 fixed-frame 보호 불변식에 따라
기각됐고, 그 결론을 registry 또는 fallback 변경으로 우회하지 않았다. same-snapshot 증거의 정합성,
비공개 자료 경계, diagnostic counter 격리와 최신 Full CI에서 추가 blocker는 발견하지 않았다.

self-review는 **완료 / 조건부 merge 권고**다. review-only trailing head의 fast-pass, 최신
`MERGEABLE/CLEAN`과 작업지시자의 별도 merge 승인을 확인하기 전에는 merge하지 않는다. #4967 tracker는
merge 후에도 열린 상태로 유지한다.
