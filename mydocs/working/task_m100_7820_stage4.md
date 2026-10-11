---
kind: working
status: active
last_verified: 2026-10-11
---

# #7820 4단계 — PR 제출과 self-review

사용자가 로컬 검증 완료 뒤 PR 생성·CI 확인·self-review와 병합 순서의 진행을 승인했다.
구현·검증 결과는 [3단계](task_m100_7820_stage3.md)에 연결한다.

## 적용 경로

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md` (1,000줄 초과), 이후 `review_only_fast_pass.md`, `post_merge.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 제출 단계 자식 문서
- current head: 제출 증적 추가 전 `2d975b136`; Rust 보정 `11076741a`, 전체 회귀 `c95269728`
- current base: fetch로 확인한 `790a6f8f388445e7bff2bcb165aa7b384f9ece80`

필수 lint·전체 회귀·Native Skia·Docker fresh WASM·전쪽 비교는 완료했다.
이후 추가는 최종 대표 PNG와 제출 기록이며 Rust source/test/manifest를 바꾸지 않는다.
최신 base의 merge simulation은 충돌 없이 통과했다. 제출은 `devel` 대상 Open PR이며
PR 채번과 정확한 code head CI 성공 전에는 번호 기반 review·오늘할일을 만들지 않는다.

## 대표 증적

같은 `footnote-01.hwp` 첫 쪽과 독립 `pdf/footnote-01-hwp-2020.pdf`의 Print 출력을 비교한다.
최종 Native Skia PNG와 Docker WASM SVG raster를 각각 입력으로
`scripts/visual_sweep.py`의 `make_compares`, `make_overlay_page`, `make_review_panels`를
호출했다. overlay의 픽셀 차이 문턱은 24/255이며 실루엣 관용 반경은 2px다.
전쪽 원시 TSV는 ignored output에 보존하고 PR 본문의 접힌 절에도 원문을 싣는다.
중간 JSON·로그·SVG는 커밋하지 않는다.

| 출력 | 첫 쪽 pixel match | strict content proxy | tolerant silhouette |
| --- | ---: | ---: | ---: |
| Native Skia | 92.98086% | 48.01787% | 97.91613% |
| Docker WASM | 92.49256% | 45.14831% | 98.60456% |

대표 파일은 `mydocs/pr/assets/issue_7820_sixel/`에 보존한다.
`footnote-p001-{native,wasm}-{review,overlay}.png` 네 장과
`footnote-p001-png-sixel-review.png` 한 장이다. 후자는 너비 720의 기준 PNG와
libsixel 독립 복원 결과를 좌우로 비교한다. 한국어 라벨·수치·범례와 본문을 직접 열어 확인했다.
앞의 strict proxy를 전체 fidelity 판정으로 쓰지 않는다. 남은 글꼴 형태·굵기와
각주선 차이는 base에서 유지되며 이번 PR은 해당 조판 개선을 주장하지 않는다.
Native PNG 전체 11쪽의 base와 decoded 픽셀 동일성, Sixel 22개 독립 복원 검사와
사용자의 실제 터미널 출력 성공을 이번 출력 계약의 근거로 삼는다.

WezTerm·Windows Terminal의 SSH 수동 판정은 통과이며 Foot와 정확한 터미널 버전은
미검증이다. 따라서 관련 이슈는 `Refs #7820`으로 연결하고 자동 종료 표현을 사용하지 않는다.
최종 PR head SHA 고정 raw URL로 이미지를 표시하고 게시 뒤 본문·asset을 API로 재조회한다.
code head CI 성공 뒤 같은 PR에 self-review 기록을 추가한다. 최신 trailing head의 check와
mergeability 재확인 전에는 병합하지 않으며 branch protection을 우회하지 않는다.

## PR 생성과 최초 CI 보정

`227438c64731bcca78255c58af73d825445e9247`로 [PR #7829](https://github.com/edwardkim/rhwp/pull/7829)를
생성했다. 한국어 본문·고정 head의 asset 5개가 로컬 파일과 동일하고,
실제 CDP Chrome PR 화면에서 모든 이미지가 로드됨을 확인했다.

최초 [CI run](https://github.com/edwardkim/rhwp/actions/runs/38136989011)의 lint job은
Rust lint 실행 전 `Validate CI impact classifier`에서 실패했다. 새 CLI 출력 어댑터가
명시적 영향 bucket에 없다는 계약 검사이며 `src/cli/outputs/sixel.rs` 등록 누락이었다.
출력 어댑터와 `tests/cases/issue_7820_sixel_contract.rs`를 Native Skia 소유 목록에 추가했다.
Canvas 소비자는 아니므로 이 두 파일만 변경할 때는 Native/Rust 검사만 요구한다.
이번 PR 전체는 공통 renderer·Cargo 변경을 포함해 Full CI가 필요하다.

동시에 Native Skia job의 release-test·release 두 경로에 Sixel 계약 실행을 추가했다.
job target의 classifier 소유 검사도 실제 `tests/cases/` 원본을 확인하도록 보완했으며
기존 root test 경로는 그대로 확인한다. 단순히 파일 목록 assertion만 늘리지 않았다.
Rust source·Rust test·Cargo와 실제 PNG/WASM/Sixel 출력은 이 보정에서 바꾸지 않았다.

- CI impact·policy·trusted reuse Node 검사: 327건 통과 (`ci-wiring-node.log`).
- `test_*workflow.py` Python 검사: 186건 통과 (`ci-wiring-python.log`).
- CI와 같은 `run-rust-test.mjs --cargo-test issue_7820_sixel_contract`의
  release-test/Native 실행: 7건 통과 (`ci-wiring-sixel-cargo-test.log`).
- 원 실패 원문: ignored `output/pr-review/issue7820-20261011/github-lint-first.log`.

CI 보정은 새 code candidate로 push하고 새 head CI를 확인한다. 최초 실패 head의 결과로
self-review 기록이나 병합을 진행하지 않는다.
