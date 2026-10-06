# PR #7289 리뷰 — 레거시 OLE 수식 편집 지원

## 접수 정보

2026-09-21에 확인한 code candidate의 참고값이다. 이 검토 기록을 포함한 최신 head의
상태와 CI는 merge 직전에 다시 확인한다.

| 항목 | 값 |
| --- | --- |
| PR | [#7289](https://github.com/edwardkim/rhwp/pull/7289) |
| 작성자·경로 | `jangster77`, collaborator self-review |
| 관련 이슈 | [#7105](https://github.com/edwardkim/rhwp/issues/7105) |
| base | `devel` / `c01c9f4972c395efa45e38597f2cdfe7f88d7579` |
| 검토한 code candidate | `0f1fa2f72ff9e01d5244e11e9c58aabd8ee9e584` |
| 최초 제출 규모 | 13 files, +239 / -19 |
| GitHub 상태 | code candidate에서 Open, non-draft, MERGEABLE / BLOCKED(CI 대기). 최종 head는 다시 확인 필요 |
| reviewer | self-review이므로 지정하지 않음 |

라우팅: `collaborator_self_merge` + `intake_and_review` + `local_validation`.
읽은 정본: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_self_merge.md`, `intake_and_review.md`, `local_validation.md`.

## 변경과 코드 검토

구 한/글 OLE 수식은 `Control::Shape(ShapeObject::Ole)`로 저장되지만, 렌더 트리에서는
수식처럼 보인다. 따라서 기존 수식 편집 명령은 native `Control::Equation`만 받으며, 선택된
레거시 OLE의 편집이 거부됐다.

`DocumentCore::promote_ole_equation_native`는 선택한 동일 section·paragraph·control slot에서
OLE `Contents`의 수식 script를 읽어 native equation으로 교체한다. 공통 배치 정보는 유지하고,
변환 뒤에는 기존 `set_equation_properties_native` 편집 경로를 사용한다. OLE를 구 방언으로
역직렬화해 덮어쓰지 않으므로, 사용자가 편집하면 저장 결과는 native equation이다.

Studio는 OLE 수식의 문맥 메뉴와 더블 클릭을 전환·편집 명령으로 연결하고 mutation snapshot을
기록한다. Firefox service worker의 `open-hwp`도 `browser.tabs.create` Promise를 기다려
응답 전에 viewer 생성이 완료되게 했다. 특정 문서 ID, 좌표, 폰트에 의존하는 예외나 출력 은폐는
추가하지 않았다.

최초 CI의 Studio unit gate는 `promoteOleEquation`가 `MUTATING_METHODS`에는 있으나
`MUTATING_VERB` 감사 동사에는 없는 누락을 검출했다. `promote`를 감사 동사에 추가한
`0f1fa2f72` 뒤 같은 가드와 #7105 Studio 검사를 다시 실행했다.

## 조판 원칙 준수 검토

| 검토 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | `ShapeObject::Ole`와 실제 CFB `Contents` 수식이라는 구조 조건으로 전환하며 샘플·좌표 조건이 없다. |
| 측정·배치 일관성 | 비해당 | 기존 개체의 `common` geometry를 옮기고 편집 대상 종류만 바꾸며, 줄 측정·배치·backend 좌표 계산을 바꾸지 않는다. |
| 분할·이어받기 계약 | 비해당 | pagination, table fragment, reservation 경로를 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | 저장 LineSeg·줄 구성·점유 높이를 바꾸지 않는다. |
| 사례와 증거의 독립성 | 충족 | 저장소 HWPX의 실제 OLE 요소를 본문 슬롯으로 복제해 전환·일반 편집·HWP 저장·재열기를 검사하고 native equation 대조군도 실행했다. |
| 기준값 변경 | 비해당 | baseline, golden, 허용치를 바꾸지 않았다. |
| 주장과 검증 범위 | 충족 | Core 전환·HWP 재열기, Studio 명령 경계, Firefox service-worker의 viewer 생성과 Windows/macOS 실열기 결과를 각각 확인했다. |

## 검증과 입력 공급

| 검증 | 실제 결과 |
| --- | --- |
| #7105 Core focused 회귀 | Windows 10 `issue_7105_legacy_ole_equation_promotes_to_editable_native_equation_and_survives_hwp_save`: 1 passed / 0 failed |
| Studio 명령 경계 | `node --test rhwp-studio/tests/issue-7105-ole-context-menu-delete.test.ts`: 3 passed / 0 failed |
| mutation routing guard | `node --test rhwp-studio/tests/mutation-routing-guard.test.ts`: 10 passed / 0 failed |
| Studio 패키지 | `npm --prefix rhwp-studio run build` 통과 |
| Firefox service worker | `node --test rhwp-firefox/sw/download-interceptor.test.mjs`: 26 passed / 0 failed |
| Rust 품질 게이트 | Windows 10에서 `CARGO_BUILD_JOBS=8`: fmt, native Clippy, WASM32 Clippy, workspace build, all-target Clippy 통과 |
| suite 정책 | `rust-test-suite-manifest --prepare` 및 base `c01c9f497` 대비 `--check` 통과 |
| Firefox 실제 열기 | Windows Firefox 153.0.4 및 macOS Firefox 155.0.1에서 최신 확장으로 `transistor-mosfet.hwp` 11쪽 로드 완료 |

실행 입력은 모두 검토 대상 commit에 이미 포함되어 있다.

| 입력 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/issue5725/2921145_equation_ole.hwpx` | Core 회귀가 본문 OLE 수식을 복제하는 원본 | `b45cadcc8dbb7070de51bb819fa9d00867e41958cebf9b34eb43d1a8d8f75038` |
| `tests/fixtures/issue_7105/transistor-mosfet.hwp` | Firefox 실제 열기 입력 | `ae2f5de257f9666a2bb0b04aa1548d3165e12d5ae6755358bfc57f7a62e3c0aa` |

이 PR은 수식의 렌더·조판·페이지 외형을 바꾸지 않고, 사용자가 시작한 편집에서 OLE를 native
수식으로 전환한다. 따라서 PDF Visual Sweep은 **비해당**이다. Firefox 자동화는 privileged
extension viewer의 화면 캡처를 지원하지 않았지만, 두 OS에서 실제 viewer 탭 생성과 11쪽 DOM
상태를 확인했다. 전체 release-test는 최신 PR head의 GitHub required CI가 담당한다.

## 최종 판정

- 판정: **승인**
- 근거: 레거시 OLE 수식이 native equation으로 전환된 같은 control slot에서 일반 편집을 받고,
  HWP 저장·재열기 뒤 script가 유지되는 focused 회귀를 확인했다. Studio와 Firefox의 연결 경계도
  별도로 통과했고, 두 Firefox 환경에서 실제 문서 열기를 확인했다. CI가 검출한 mutation audit
  누락도 보정 뒤 가드 전체 통과로 해소했다.
- merge 전 조건: 이 review·오늘할일 trailing commit을 포함한 최신 PR head의 required CI 통과,
  mergeable/CLEAN 재확인, 작업지시자의 merge 승인.
- 이 문서는 self-review 기록이며 GitHub APPROVE, issue close, comment 또는 merge를 실행하지 않는다.
  단일 PR이므로 별도 `review_impl`은 만들지 않는다.
