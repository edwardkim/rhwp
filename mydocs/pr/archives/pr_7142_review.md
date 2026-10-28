---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7142 언어팩 골격 — 최종 검토

**판정: 메인터너 보정 후 수용 가능.** 원 contributor head의 확장 자산 누락과 DOM 자식 삭제를 보정했고, 실제 원 PR에 반영한 code head의 CI까지 완료했다. 동일 source branch의 검토 기록 trailing head CI·merge 가능 상태 확인이 남아 있다.

## 출처와 첫 기여자 처리

- [PR #7142](https://github.com/edwardkim/rhwp/pull/7142), rubidus-api의 rhwp 첫 기여. 기존 merged PR 0건을 확인했고 reviewer jangster77이 할당돼 있었다.
- contributor 원 commit: `7e57741eb6df800921de836d4e0df7286dd71864`, fork `rubidus-api/rhwp`, branch `i18n/1-skeleton`, base devel.
- 메인터너 보정: `0308b476baed1e1c57f93af27a27edf47d18109c`. 원 commit 바로 위에 추가했으며 contributor 이력·저자·원격 branch를 rewrite하지 않았다. 최초 검토는 `upstream/devel@751c315a8` 위 체리픽 `06a168c5b`에서 수행했고, 직접 보정 경로로 전환할 때 같은 가시성 branch `codex/pr7142-review-20260914`의 미공개 메인터너 기록만 replay했다.
- 원 PR 19파일 +1110/−17을 새 기반 모듈, 기존 표시 호출부, 테스트, 확장 자산 경로로 나눠 검토했다. 보정은 5파일 +28/−1이며 Rust·조판·CI workflow 변경이 없다.
- [이슈 #5852](https://github.com/edwardkim/rhwp/issues/5852)의 4단계 중 1단계다. 메뉴·대화상자 전체 영어 전환, 탭 ID 분리, 언어 선택 UI는 이후 단계이며 이슈는 OPEN으로 유지한다. contributor가 후속 2~4단계 트리에서 측정한 전체 UI·문서 parity를 이 PR의 직접 검증으로 취급하지 않는다.

## 메인터너 보정과 보류 해소

1. **Chrome·Firefox 확장 초기화 파일 누락:** 원 HTML은 동기 `/locale-init.js`를 요청하지만 `publicDir:false`인 두 Vite 산출물과 수동 패키징에 파일이 없었다. 원 산출물의 [누락 JSON](../assets/pr7142_extension-builds.json)·[빌드 로그](../assets/pr7142_extension-builds.log.txt)에 남겼다. 두 build.mjs에 원본 파일 복사와 REQUIRED_DIST_FILES gate를 추가하고, 배포 계약 테스트에서 HTML의 동기 참조와 원본/산출물 내용 일치를 확인하도록 보정했다.
2. **DOM 아이콘 삭제:** 의미 있는 직계 텍스트가 없는 요소에 `textContent`를 대입해 자식을 삭제했다. `appendChild(createTextNode(value))`로 기존 자식을 보존하고 반복 적용 테스트를 추가했다. 원 동작은 [실제 Chrome DOM 결과](../assets/pr7142_browser.json)의 `iconPreserved:false`로 재현했다. 이번 1단계의 메뉴에서 이미 아이콘이 사라졌다는 주장이 아니라 새 helper의 경계 결함이다.

두 항목 모두 아래 로컬 실행과 새 code CI를 통과했다. 전역 baseline·golden·허용치 변경은 없다.

## 완료한 검증

- 원 head CI: [34838074070](https://github.com/edwardkim/rhwp/actions/runs/34838074070). Frontend package 실제 로그는 1733 tests / 1731 pass / 0 fail / 2 skipped였다.
- **보정 head CI:** [0308b476b / 34848683935](https://github.com/edwardkim/rhwp/actions/runs/34848683935). 해당 head의 Build & Test·Frontend package gates, CodeQL·Render Diff·Adapter·Proptest와 정책 검사를 완료했다. [최종 checks 원문](../assets/pr7142_code_ci_final.json). Rust lint·Native Skia·archive job의 SKIPPED는 실제 Rust 검사 성공으로 세지 않는다.
- 첫 기여자 fork의 새 CI가 action_required여서 실행 승인 뒤 완료를 기다렸다. 이전 head의 녹색 상태를 새 보정 검증으로 대체하지 않았다.
- 로컬 i18n 집중 테스트 **40/40 PASS**, ci-unit 및 전체 TypeScript 검사 exit 0. [집중 로그](../assets/pr7142_fixed-focused.log.txt), [전체 타입검사](../assets/pr7142_fixed-tsc.log.txt). 사용자 지시에 따라 전체 회귀를 로컬에서 반복하지 않았다.
- 처음의 공유 pkg는 오래돼 hyperlink API 타입 오류 5건이 났다. 기존 통합에서 생성한 호환 WASM 선언을 scratch @wasm 경로에 주입해 타입검사를 완료했다. 공유 pkg·원본 tsconfig를 변경하지 않았으며, scratch typeRoots 누락도 바로잡았다. 초기 UI 검토의 Rust/Cargo 동일성·빌드 hash는 [증적](../assets/pr7142_evidence.json)에 있다. 보정 검증은 순수 UI/DOM·패키징 범위로, 새 source head의 Rust 재컴파일이나 전체 renderer 검증을 주장하지 않는다.
- **두 확장 전체 build.mjs 패키징 성공 및 배포 계약 3/3 PASS.** 격리 복사본에서 실행해 공유 dist를 보존했다. [실행 내역](../assets/pr7142_fixed-builds.json), [Chrome](../assets/pr7142_fixed-chrome-package.log.txt), [Firefox](../assets/pr7142_fixed-firefox-package.log.txt), [계약 검사](../assets/pr7142_fixed-dist-contract.log.txt).
- 실제 Chrome 152.0.7977.83에서 각 최종 확장 패키지의 locale-init.js를 최소 HTML에 동기 로드했다. 앱 번들 없이 `?lang=en`의 html.lang=en을 확인했다. 실제 DOM helper 두 번 적용 후 icon 객체 identity·자식 1개·클릭 listener 1회·`★Open` 텍스트를 확인했다. [실행 결과](../assets/pr7142_fixed-browser.json), [실행기](../assets/pr7142_fixed-browser.mjs). Firefox 패키지의 JS도 Chrome에서 확인한 것으로 Firefox 설치형 확장 전체 실행을 뜻하지 않는다.
- 초기 실제 Studio ko/en 세션에서 상태 페이지·구역·수정 모드·머리말/꼬리말 문구를 확인했다. 언어 선택 즉시 현재 로케일·수정 상태는 유지되고 localStorage만 변경됐으며, 새 Studio 페이지에서 선택 언어가 적용됐다. pageerror 0. 상태 eventBus 주입과 실제 footer 문서 편집 E2E는 구분한다. [결과](../assets/pr7142_browser.json), [한국어 화면](../assets/pr7142_ko-studio.png), [영어 화면](../assets/pr7142_en-studio.png).
- 화면을 직접 판독했다. 이번에 옮긴 상태/꼬리말 문구는 표시되며 나머지 한국어 메뉴는 후속 단계 범위다. 모든 화면 폭·스킨·전체 영어 UI parity를 보증하지 않는다.

## 공통 조판 원칙·입력 커밋 확인

줄 소속·측정/배치·좌표·golden·허용치 변경은 없다. canvas-view는 HTML 머리말 배지 표시 문자열만 변경한다. 문서 ID 예외나 좌표 clamp도 없다. 직접 검증 입력은 앱에서 만든 빈 문서와 DOM/locale 계약이며 HWP/HWPX/PDF를 입력으로 사용하지 않았다. 따라서 기준 PDF 추가·한컴 변환·Visual Sweep은 비해당이다. UI 화면을 한컴 출력 일치 증거로 사용하지 않는다.

## 원격 반영 및 trailing

fork 일반 API 권한은 push:false, PR maintainer_can_modify:true였다. [LFS 판독·dry-run](../assets/pr7142_push_preflight.json) 후 사용자 승인으로 보정 code-only SHA를 실제 push했고 PR head 일치를 확인했다. code CI 완료 후 이 review·증적·오늘할일을 동일 branch의 단일 trailing commit으로 준비한다. 최신 devel 오늘할일을 source 전체에 복사하거나 devel을 merge/rebase하지 않는다. 변경되지 않은 section 경계에 이번 기록만 추가하고, push 전에 실제 merge tree의 충돌·공백·링크·기존 기록 보존을 확인한다.

최종 조건은 trailing head CI, MERGEABLE/CLEAN 및 사용자 merge 승인이다. 사용자는 CI 완료 뒤 trailing·merge·후속처리까지 승인했다. [단계별 실행·정리 계획](pr_7142_review_impl.md).

## Merge 후 contributor PR comment 계획

“rhwp 첫 기여를 보내주셔서 감사합니다”라는 환영과 함께 원 기여의 언어 결정·다음 실행 적용·원문 fallback을 구체적으로 언급한다. 두 메인터너 보정의 사유와 SHA를 분리하고 실제 code/trailing CI 및 로컬 실행 결과만 기록한다. 실제 merge SHA의 위 ko/en UI PNG를 포함하되 PDF fidelity 증적이라고 표현하지 않는다. 이슈 #5852에 1단계 반영을 알리고 OPEN을 유지한다. contributor fork branch와 기본 작업공간·공유 캐시는 보존하고 이번 로컬 branch·검토 fetch ref·전용 임시 자료만 정리한다.
