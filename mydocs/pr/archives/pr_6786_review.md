---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-06
pr: 6786
issue: 6635
author: baba9811
reviewer: postmelee
---

# PR #6786 리뷰 — 색상 버튼 키보드 활성화 복원

## 최종 판정

**승인.** 원 기여의 표준 click 활성화와 선택 보존 구현을 수용한다. 같은 색상 도구에 한정해
collaborator가 숨겨진 글자색 input의 Tab 포커스를 제외하고, 형광펜 팔레트의 Escape 닫기와
트리거 포커스 복귀를 추가했다. 보정 code candidate는
`cc21183ab622d13071acfa7be3224903a048526b`이며 범위 안 구현 blocker는 없다.

이 문서는 기술 검토 판정이다. GitHub Approve 리뷰는 작업지시자에게 초안을 보여주고 승인받은 뒤에만
등록한다. 문서 trailing head의 최신 checks와 merge 가능 상태를 다시 확인해야 하며, merge·issue close·
contributor comment는 아직 수행하지 않았다.

## 라우팅과 metadata

| 항목 | 검토 기준 |
| --- | --- |
| PR / 이슈 | [#6786](https://github.com/edwardkim/rhwp/pull/6786) / [#6635](https://github.com/edwardkim/rhwp/issues/6635) |
| 작성자 / 리뷰 요청 대상 | `baba9811` / `postmelee` |
| base | `devel@51ad998e33ef7f5191b0e1b0b656dc44cef33a1c` |
| 원 contributor head | `d0179dd0410757aea8d116ba4be7b38438e406e9` |
| 보정 code candidate | `cc21183ab622d13071acfa7be3224903a048526b` |
| source | `baba9811/rhwp:fix/6635-color-keyboard` |
| 문서 추가 전 규모 | 6 files, +214/-3, 원 기여 2 commits와 보정 1 commit |
| 작성 시점 참고값 | Open, non-draft, `maintainerCanModify=true`, `MERGEABLE` |
| assignee / labels / milestone | `baba9811` / `bug`, `rhwp-studio`, `test`, `typescript` / `v1.0.0` |

[PR review workflow](../../manual/pr_review_workflow.md)와 [선택표](../../manual/pr_review/README.md)에 따라
기본 [collaborator 외부 PR 경로](../../manual/pr_review/collaborator_external_pr.md)를 적용했다.
보조 문서는 [접수·검토](../../manual/pr_review/intake_and_review.md),
[로컬 검증](../../manual/pr_review/local_validation.md),
[첫 기여자](../../manual/pr_review/first_time_contributor.md),
[review-only fast-pass](../../manual/pr_review/review_only_fast_pass.md)다.
원 PR source head에 보정을 직접 추가하는 경로이며 별도 integration PR은 만들지 않았다.

## 원 기여와 collaborator 보정

| commit | 작성자와 역할 |
| --- | --- |
| `67a5d00b832589fd14ec60e7b10f577c8b949d5c` | baba9811: 버튼 click 활성화 복원, 숨긴 형광펜 input을 button 밖으로 이동, focused E2E와 반응형 검사 갱신 |
| `d0179dd0410757aea8d116ba4be7b38438e406e9` | baba9811: 기존 #6202 E2E의 MANIFEST 등록 누락 보완 |
| `cc21183ab622d13071acfa7be3224903a048526b` | postmelee: 글자색 input `tabindex=-1`, 팔레트 Escape 닫기·포커스 복귀, 회귀 추가 |

원 contributor 두 commit의 author와 SHA를 유지했다. rebase, amend, squash 또는 force-push하지 않았다.
보정과 이 리뷰 문서는 별도 commit으로 나눴다. 실행 순서는 [보정·통합 기록](pr_6786_review_impl.md)에 적었다.

## 구현 검토

1. `mousedown`의 `preventDefault()`는 편집기의 선택 영역 보존을 담당한다. 실제 명령을 표준 `click`으로
   옮겨 Enter/Space가 만드는 버튼 활성화와 마우스 입력을 같은 경로로 처리한다.
2. `다른 색`의 color input을 button 밖으로 이동해 중첩 interactive element와 input click의 부모 button
   재진입을 없앤다. `색 없음`도 같은 원인에 대한 click 보정을 적용한다.
3. 기존 글자색 input은 크기·투명도로만 숨겨져 Tab 순서에 남아 있었다. `tabindex=-1`로 제외해 글자색과
   형광펜 버튼 사이를 Tab/Shift+Tab 한 번으로 이동한다. 버튼에서 호출하는 color picker는 유지된다.
4. Escape 핸들러는 열린 형광펜 dropdown 안에서만 처리한다. 형광펜 버튼, `색 없음`, `다른 색`에
   포커스가 있을 때 팔레트를 닫고 형광펜 버튼으로 돌려준다. 전역 Escape 처리나 서식 명령을 추가하지 않는다.
5. 보정 E2E는 실제 키 입력으로 닫기, 포커스, 선택 영역, 서식, undo 깊이를 함께 확인한다. 기존 색 적용,
   취소, 마우스 활성화 검증도 유지한다. #6202 manifest 한 줄은 실행 파일 등록이며 제품 변경이 아니다.

## 외부 기여자 검증 절차 확인

PR의 해당 목차는 **Studio 단독 변경에 적용할 Rust 검증 범위와 별도 review 환경의 준비 순서에 대한 질문**이다.
기여자는 `cargo fmt --all -- --check`가 파생 suite 28개 누락으로 실패한 사실을 공개했고, 이를 성공으로
집계하거나 파생 파일을 PR에 추가하지 않았다.

기준 SHA의 [CONTRIBUTING.md](https://github.com/edwardkim/rhwp/blob/51ad998e33ef7f5191b0e1b0b656dc44cef33a1c/CONTRIBUTING.md)는
fmt를 제출 전 필수로 표현하면서 일반 contributor source checkout에서 파생 suite를 준비하지 않도록 안내한다.
파생 파일을 source PR에 넣지 않는 목적은 독립 PR 사이의 harness·manifest 충돌 방지다. Rust 검증이 필요할 때
별도 review worktree에서 `--prepare` 후 검사한다는 연결과 Studio 단독 변경의 적용 범위가 공개 체크리스트에서
충분히 명확하지 않았다.

원 head의 review worktree에서 같은 누락을 재현했고, `node scripts/rust-test-suite-manifest.mjs --prepare` 후
`cargo fmt --all -- --check`와 manifest `--check`가 통과했다. Rust source·Cargo.toml 수정은 없었다.
이는 준비된 review 환경의 결과이며 기여자의 source checkout 실패를 성공으로 바꿔 기록하지 않는다.
생성된 suite·manifest는 검증 산출물로만 두고 stage하지 않았다.

이번 PR과 보정은 Studio HTML/TypeScript/E2E 변경이다. CI 분류도 `rust_required=false`,
`frontend_mode=package`다. 변경 범위별 로컬 검증 정책에 따라 Rust 전체 workspace Clippy·nextest를 추가로
실행하지 않았다. 공개 가이드 보완은 작업지시자가 요청한 별도 작업으로 분리하며 이 PR의 제출자 결함으로
판정하지 않는다.

## 실제 로컬 검증

검토 worktree는 `/private/tmp/rhwp-pr6786-review`이며 아래 결과는 collaborator가 직접 실행했다.

| 검증 | 결과 |
| --- | --- |
| fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` | 성공; native 진단 빌드 |
| 보정 전 production 코드 + 신규 E2E | 7건 예상 실패: 양방향 Tab 2, Escape 닫기 3, 포커스 복귀 2 |
| `npm --prefix rhwp-studio run e2e:issue-6635` | 기본 구성 64/64 통과 |
| 위 E2E, `RHWP_WITHOUT_HWPCTRL=1` | 64/64 통과 |
| `npm --prefix rhwp-studio test` | 1,425 pass, 기존 skip 1, fail 0 |
| `npm --prefix rhwp-studio run build` / `build:no-hwpctrl` | 두 구성의 TypeScript·production build 성공 |
| `npm --prefix rhwp-studio run e2e:responsive` | 1,082 pass, fail 0 |
| `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs` | 3/3 통과 |
| `python3 scripts/check_e2e_manifest.py` | 128 files / 128 rows, 오류 0 |

WASM은 원 PR Rust 입력으로 새로 빌드했고, 후속 보정은 Rust 입력을 바꾸지 않아 재사용했다.
`CARGO_TARGET_DIR`는 공용 review cache `target/pr-review`를 사용했다.
WASM SHA-256: `540dcfabf60dc725e2a6ae05b794cf5d02a7db323816ad444911e7a5162f80e9`.
Docker daemon이 실행되지 않아 native 진단 경로를 사용했으며 Docker 최적화 배포 빌드를 검증했다고 주장하지 않는다.
E2E는 로컬 Chrome을 지정하고 `VITE_PORT=7787`로 실행했다.

### 사람의 확인과 자동화의 범위

- 작업지시자는 보정 전 원 PR에서 글자색·`다른 색`의 네이티브 색상 창 Esc 닫기, 형광펜 버튼의
  Enter/Space 재활성화로 닫기, 영역 밖 마우스 클릭으로 닫기를 통과했다고 확인했다.
- 새로 추가한 형광펜 팔레트 Escape 및 한 번의 Tab 이동은 위 자동화로 확인했다. 작업지시자가 이 보정까지
  직접 재확인했다고 기록하지 않는다.
- 자동화는 실제 키보드·마우스가 color input을 한 번 활성화하는지 검사한다. 색 적용 부분에서는 input
  이벤트를 대신 전달하므로 OS 네이티브 색상 창 내부의 색 선택까지 자동 검증한 것은 아니다.
- renderer/layout/paint를 바꾸지 않는 도구 모음 입력 변경이므로 로컬 PDF/SVG Visual Sweep은 수행하지 않았다.
  브라우저 상호작용 결과를 한컴 기준 출력과 대조한 시각 검증으로 표현하지 않는다.

## 보정 code candidate GitHub CI

아래 run은 모두 `cc21183ab622d13071acfa7be3224903a048526b`에 속한다. 원 head의 과거 성공을
보정 head의 결과로 재사용하지 않는다. 외부 fork workflow 실행 승인 대기를 해제한 후 모든 관련 workflow와 required `Build & Test` 성공을 확인했다.

| workflow | 결과 / 증적 |
| --- | --- |
| CI | 성공 — [34021282412](https://github.com/edwardkim/rhwp/actions/runs/34021282412) |
| CodeQL | 성공 — [34021282542](https://github.com/edwardkim/rhwp/actions/runs/34021282542) |
| Render Diff | 성공 — [34021282182](https://github.com/edwardkim/rhwp/actions/runs/34021282182) |
| Adapter inter-diff | 성공 — [34021282384](https://github.com/edwardkim/rhwp/actions/runs/34021282384) |
| Proptest roundtrip | 성공 — [34021282306](https://github.com/edwardkim/rhwp/actions/runs/34021282306) |

CI 영향축은 frontend package, render, JavaScript CodeQL이다. Rust lint·native Skia·Rust shard는
정책상 미해당 skipped이며 실행 성공으로 집계하지 않는다. 별도 `WASM Build` skip과 frontend package
내부의 fresh WASM build도 구분한다.

## 후속 범위와 merge 전 조건

- 다른 도구 모음·팝업의 숨긴 input, 중첩 interactive element, Tab 순서, Escape·포커스 복귀는 별도 접근성
  조사 대상으로 남긴다. 이번 PR에서 전역 변경하지 않았고 별도 이슈가 등록됐다고 주장하지 않는다.
- 오늘할일은 source branch에 `mydocs/orders/20260906.md`가 없고 최신 devel에 다른 PR 기록으로 존재한다.
  변경되지 않은 추가 경계가 없어 원본에 다른 PR 기록을 복사하거나 문서만을 위해 base를 병합하지 않았다.
  선택적 오늘할일 갱신 대신 이번 처리 이력은 이 archive review와 implementation 기록에 남긴다.
- 최신 devel `016fe3ceed904633e74e70127a4cceaa1f18a756`과 code candidate의 merge simulation은 충돌 없이
  통과했다. 문서 commit 뒤에도 최신 base의 merge tree·diff·Markdown 링크를 확인한다.
- code candidate CI 성공 후 archive 두 문서만 trailing commit으로 push한다. 최신 head의 실제 preflight가
  review-only fast-pass를 허용하는지 확인하고, 허용하지 않으면 전체 해당 CI 완료를 기다린다.
- 마지막 head SHA·required checks·`MERGEABLE` 상태를 다시 확인하고 GitHub Approve 초안 승인을 받는다.
  merge는 별도 승인 대상이다. merge 후 원 기여자에게 첫 기여 감사, 기여/보정 구분과 실제 검증을 전달하고,
  #6635 자동 close 여부를 확인한다. 이 문서 작성 시 이러한 원격 후속 조치는 미실행이다.
