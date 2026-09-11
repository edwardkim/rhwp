# PR #7010 검토: Rust 가변 export 기반 변이 메서드 분류 감사

## 판정: 메인터너 보정 후 수용 가능

## 검토 대상과 권한

- 검토일: 2026-09-11, 검토자: jangster77.
- 기본 경로: collaborator 매개 외부 PR. 원 PR의 사전 reviewer 할당은 완료했으며, 통합 PR의 owner 자동 지정과 구분한다.
- 작성자 lpaiu-cs는 기존 기여자다. 첫 기여자 절차를 새로 적용하지 않는다.
- 검토 브랜치: `review/lpaiu-cs-7004-7022-20260911`.
- 통합 기준: `upstream/devel b59323de0448df0a42bcb9cdd12cb80d51fe83d2`.
- 체리픽 통합 코드 head: `b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9`.
- 실제 검증 후보: 위 체리픽 head와 #7010 메인터너 보정 commit `d6b1c25bd584e766dd8121f3678a64919040a37d`의 3개 파일. [통합 검증 기록](pr_7004_review_impl.md)의 SHA-256으로 정확한 내용을 구분한다.
- 사용자가 통합 PR 생성을 승인했다. 검증한 코드와 리뷰·오늘할일·필수 PNG를 제출하며 원 PR/이슈 comment·close·merge는 아직 수행하지 않는다.

## 원본 보존 체리픽

```text
b6bcf361f36d63da73edf01f6d03376a9a7ac982 -> 9fccf9e175f4a3c32adfa81b3073c0d393a3f410
29c9192f9b46d6b6c603ad1e455a997bd7f1ce2d -> 1cef159930b8888ad1c81379b62bafc3d315c272
```

## 메인터너 보정 내용과 검증 상태

**보정 적용 및 로컬 검증은 완료됐다. 보정 commit은 `d6b1c25bd584e766dd8121f3678a64919040a37d`다. 원 contributor head를 무보정으로 승인하는 판정은 아니다.**

- 연결 Task: #7002. 기존 동사 목록만으로는 누락되는 변이 표면을 Rust `&mut self` export와 대조하려는 목적은 타당하다.
- 원 구현은 속성과 함수 사이 400자 제한, impl-level export의 여러 무속성 메서드 누락, 기본 이름의 camelCase 추정, 문자열 `js_name` 미처리, 빈 추출 결과의 조용한 통과 가능성이 있었다.
- `js_name`을 지정하지 않으면 Rust 이름을 보존하도록 바로잡았다. 근거: [wasm-bindgen js_name 문서](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/js_name.html).
- 테스트 전용 lexical helper로 직접 `#[wasm_bindgen]` impl의 공개 가변 receiver 메서드를 열거하고, 장문 주석·중첩 주석·일반/원시 문자열을 구분한다. 이름 명시, 무속성 연속 메서드, async/unsafe, lifetime receiver를 검사한다.
- 빈 결과·잘못된 구분자·지원하지 않는 가변 receiver·wrapped wasm_bindgen 속성·지원하지 않는 공개 impl 항목은 실패시켜 감사 누락이 녹색으로 숨지 않게 했다. 일반 Rust compiler 전체를 구현한 파서라고 주장하지 않으며, 새 export 문법 도입 시 이 helper의 지원 계약을 함께 확장해야 한다.
- 기존 22개 제외 메서드와 사유는 유지했다. 분류 registry, mutation baseline, 제품의 문서 편집 동작을 완화하거나 바꾸지 않았다.
- 최종 집중 테스트 13개와 전체 Studio 테스트 1,649개 통과·2개 건너뜀으로 보정 효과 및 기존 분류 계약을 확인했다.

## 보정 식별과 통합 조건

[통합 기록](pr_7004_review_impl.md)에 기록된 보정 3개 파일의 SHA-256이 실제 검증 기준이다. 해당 보정이 빠진 원 PR을 직접 merge하지 않는다. 보정 commit `d6b1c25bd584e766dd8121f3678a64919040a37d`를 포함한 통합 PR의 최신 head CI를 확인해야 한다.

## 코멘트 계획

향후 원 PR/Task #7002에 원 head와 보정 commit·merge SHA를 분리해 적고, 장문 문서/impl 이름 추출의 사각 해소 및 실제 13개 집중·1,649개 frontend 통과 결과를 설명한다. 시각 동작 변경이 아니므로 관련 없는 PNG/PDF는 첨부하지 않는다. UTF-8 body-file을 사용하고 중복 댓글 대신 기존 코멘트를 수정한다.

## 공통 로컬 검증 결과

| 검증 | 실제 결과 |
| --- | --- |
| Rust 집중 nextest | 11개 통과; 조각 삭제 복원 7개, 캐시 목록 1개, 셀 도형 경로 3개 |
| 전체 nextest, release-test, 8 threads, no-fail-fast | 9,471개 통과, 46개 건너뜀, 실패 0개; 실행 360.994초 |
| Rust export 감사 및 추출기 집중 테스트 | 최종 보정본 13개 통과 |
| Studio 및 npm/editor 통합 Node 테스트 | 최종 보정본 1,649개 통과, 2개 건너뜀 |
| 회전 oracle Python 테스트 | 8개 통과 |
| Undo-depth workflow Python 계약 | 5개 통과 |
| E2E 목록 검사 | 추적 파일 129개와 manifest 129행 일치 |
| cargo fmt --all -- --check | 통과 |
| native Clippy, WASM32 Clippy | 각각 -D warnings 통과 |
| workspace build, workspace all-target Clippy | 각각 통과 |
| Rust suite manifest --check | 1,258 sources, 28 suites + 20 exceptions, 48/48 targets 일치 |
| 잠금 파일 보호 wrapper WASM 빌드 | 통과; 210초, 새 pkg 사용 |
| Studio tsc + Vite production build | 통과 |
| 실제 Vite Undo 깊이 E2E | 110라운드, 이력 255개, 조각 삭제 103개, snapshot 슬롯 0, 키보드 Undo 255/255 통과 |

모든 Cargo 계열 명령은 Mac의 `target/pr-review`에서 순차 실행했다. 전체 회귀에 `cargo test --profile release-test --tests`를 사용하지 않았다. 원 PR의 CI 성공을 로컬 실행 결과로 바꿔 기록하지 않는다. 원시 로그·임시 스크립트·JSON은 커밋 대상에서 제외한다.

## 원 PR의 최신 상태

- 원 head: `29c9192f9b46d6b6c603ad1e455a997bd7f1ce2d`; OPEN, non-draft.
- 마지막 조회에서 원 head 추가 변경 없음. 본문 댓글 0개, 제출된 review 0개.
- 아래는 원 head의 원격 결과다. 아직 생성하지 않은 통합 PR의 CI 결과가 아니다. CodeQL의 NEUTRAL은 SUCCESS로 바꿔 적지 않는다.

| 확인 항목 | 결과 | 증적 |
| --- | --- | --- |
| Canvas visual diff | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34568808100/job/103166570689) |
| adapter inter-diff | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34568808298/job/103166363379) |
| prop roundtrip | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34568808272/job/103166501732) |
| Build & Test | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34568808248/job/103167916367) |
| CodeQL | NEUTRAL | [실제 검사](https://github.com/edwardkim/rhwp/runs/103167019112) |
| CI Impact Policy | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34569379300) |

## 다음 원격 게이트

이 판정은 GitHub approve/merge 실행이 아니다. 보정을 포함한 최종 통합 commit과 문서·필수 PNG를 확정한 뒤, 승인된 통합 PR의 최신 head CI 및 mergeability를 확인해야 한다. 원 PR 직접 merge나 원 이슈 조기 종료를 수행하지 않는다.
