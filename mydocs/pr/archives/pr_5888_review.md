---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5888 self-review — Chrome 실제 package 핵심 surface smoke

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 네 자식 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `38c7d2ba53aa926cf6f151a0e70be3ea0304a9ea`

## 작성 시점 metadata

| 항목 | 값 |
|---|---|
| PR | [#5888](https://github.com/edwardkim/rhwp/pull/5888) |
| 작성자 | `postmelee` |
| base / head | `devel` / `postmelee:codex/issue-3514-extension-smoke` |
| 상태 | Open, non-draft |
| 규모 | +2,089 / -2, 19 files, 7 commits |
| mergeability | `MERGEABLE`, `mergeStateStatus=BLOCKED` — 2026-08-22 17:47 KST 참고값 |
| 관련 이슈 | closes #3514, parent #3512, follow-up #3513·#3515 |

1,000줄을 넘지만 하나의 #3514 범위에서 harness·lockfile·Hyper-Waterfall 단계 증적이 함께 증가한 결과다.
핵심 실행 변경은 `rhwp-chrome/package*.json`과 `rhwp-chrome/e2e/` 두 파일에 집중돼 있고, 각 기능·검증
보강은 Stage별 커밋으로 분리돼 있다. 즉시 admin merge하지 않고 code review, 최신-base 정합화, 로컬 검증,
GitHub CI와 작업지시자 판단을 각각 독립 gate로 유지한다.

## 변경 범위 판정

- Puppeteer의 Chrome for Testing에 production `rhwp-chrome/dist`를 unpacked extension으로 설치한다.
- MV3 worker, 실제 HWP3 viewer canvas, 다크 정적 자산, options hydration, same-origin print surface,
  loopback content script를 한 명령으로 검증한다.
- 예상 밖 탭은 진행 중 surface를 즉시 중단하고, 외부망 차단 proxy의 정상 client abort는 프로세스 오류로
  번지지 않게 하면서 예상 밖 socket 오류는 진단 실패로 보존한다.
- 제품 extension 코드·권한·CSP·renderer/layout·sample·golden은 바꾸지 않는다.
- CI workflow는 추가하지 않는다. 자동 CI 연결은 #3515, 같은 profile 다운로드·재시작 수명주기는
  #3513의 후속 범위다.

## 렌더·시각 검증 판정

시각·fixture 보조 경로는 적용하지 않는다. 기존 공개 HWP3 sample을 읽어 canvas 존재와 package 배선만
검증하며 renderer/layout 출력, sample, 기준 PDF, golden을 변경하거나 시각 fidelity 개선을 주장하지 않는다.
제품 UI 변경도 없어 screenshot 증적은 필요하지 않다.

## 코드 self-review

- extension ID를 고정하지 않고 실제 worker URL에서 구해 설치별 ID 변화에 안전하다.
- profile과 download 경로는 반복마다 새 임시 경로를 사용하고 `finally`에서 browser·server·파일을 정리한다.
- 외부 HTTP(S)는 loopback proxy가 연결하지 않고 차단하며 page·worker의 외부 요청도 진단 실패로 잡는다.
- page-budget 실패 promise는 모든 비동기 surface와 race해 추가 탭 발생 시 timeout까지 기다리지 않는다.
- CONNECT socket listener는 차단 응답을 쓰기 전에 설치된다. `ECONNRESET`·`EPIPE`만 정상 client abort로
  분류하고 그 밖의 오류는 `[fixture-proxy]` 오류로 남는다.
- 차단 결함이나 범위 밖 변경은 발견하지 않았다.

## 완료한 로컬 검증

- 최신 `upstream/devel@65f71270f` 조상 관계와 `0 behind / 7 ahead`, 71커밋 rebase `range-diff`를 확인했다.
- PR 검토용 파생 harness를 임시 준비해 manifest 864 source를 확인했고 `cargo fmt --all`과
  `cargo fmt --all -- --check`를 통과했다. 파생 suite·manifest와 추적 Rust diff는 남기지 않았다.
- locked native WASM `--no-opt` fresh build를 1분 56초에 통과했고 Cargo.lock blob hash가 전후
  `e0ad3758affc57b170a03cfbe2f1c8294c89d7aa`로 같았다.
- Studio TypeScript와 unit 1,065 pass·1 skip, Chrome·Firefox extension Node 134/134, Firefox production
  build, Chrome·Firefox·Safari dist 3/3을 통과했다.
- Stage 6의 첫 Chrome 실행은 1/10 전에 fixture proxy `ECONNRESET`을 드러냈다. 이를 성공 재시도로
  덮지 않고 Stage 7 code·계약으로 보강했다.
- 새 code head에서 `RHWP_EXTENSION_SMOKE_REPEAT=10 npm --prefix rhwp-chrome run test:e2e:smoke`를
  한 번 실행해 production build 1회와 새 profile 10/10을 retry 없이 통과했다.
- `git diff --check`를 통과했다.

로컬 Docker daemon이 꺼져 표준 최적화 WASM은 실행하지 못했다. documented native `--no-opt` fallback으로
실제 package와 Chrome을 검증했다. 이 PR의 경로 분류에서는 GitHub `WASM Build`가 skip되므로 표준
최적화 경로는 Docker 사용 가능 환경 또는 release pipeline의 재확인 범위다.

## GitHub Actions와 남은 조건

code candidate `38c7d2ba5`의 CI frontend package gate와 Build & Test aggregate, CodeQL
JavaScript/TypeScript·Rust·Python, Proptest roundtrip, Adapter inter-diff가 모두 성공했다. Rust archive,
Lint, Native Skia, WASM Build는 frontend package 영향 분류에 따라 정상 skip됐다. review-only trailing
commit을 push한 뒤 exact candidate 재사용 여부와 최신 aggregate를 새 head에서 다시 확인한다.

- CI: <https://github.com/edwardkim/rhwp/actions/runs/32563164074>
- CodeQL: <https://github.com/edwardkim/rhwp/actions/runs/32563164022>
- Proptest: <https://github.com/edwardkim/rhwp/actions/runs/32563164054>
- Adapter inter-diff: <https://github.com/edwardkim/rhwp/actions/runs/32563164089>

## 위험과 후속

- 각 반복은 profile을 공유하지 않으므로 같은 profile의 과거 다운로드·Chrome 재시작 결함은 #3513에서
  별도 시나리오로 구현한다.
- CI job 연결과 실패 artifact는 #3515 범위다.
- 실제 OS 인쇄 대화상자와 출력물 pixel/layout fidelity는 이 smoke의 보증 범위가 아니다.

## 권고

코드 self-review와 로컬·GitHub code candidate 검증에서 차단 결함을 발견하지 않아 **조건부 merge를
권고**한다. 최종 조건은 review-only 기록이 포함된 최신 PR head의 required checks 성공, 최신
mergeability 재확인, 작업지시자의 별도 merge 승인이다.
