# Stage 9 완료 보고 — Task M100 #3789: PR 리뷰 보정과 최신 `devel` 재검증

- **일자**: 2026-08-29 KST
- **브랜치**: `task_m100_3789-render-boundary`
- **원격 PR head**: `3e439a534`
- **리뷰 보정 commit**: `eeffb3e8f`
- **로컬 source candidate**: `16ea38cd2`
- **현재 기준**: `upstream/devel@f6a6bee8f`
- **이슈 / PR**: [#3789](https://github.com/edwardkim/rhwp/issues/3789) /
  [#6276](https://github.com/edwardkim/rhwp/pull/6276)
- **문서 성격**: Stage 9 종료 시점에 작성한 contemporaneous 보고

## 입력 리뷰와 판단

[PR 리뷰 comment](https://github.com/edwardkim/rhwp/pull/6276#issuecomment-5452147207)의 1~7번을
소스와 실제 workflow consumer 기준으로 다시 대사했다. 다음처럼 판정했다.

| 리뷰 항목 | 판정 | 조치 |
| --- | --- | --- |
| 1. `main.rs`에 공유 렌더 입력이 남음 | merge blocker | 문서 로더·인증 입력을 `src/cli/document_io.rs`로 이동 |
| 2. caption은 Render Diff가 직접 소비하지 않음 | merge blocker | caption trigger를 제거하고 실제 공유 입력 경계를 등록 |
| 3. root 렌더 가드가 한 함수명만 검사 | merge blocker | `.render_page_` family 음성 가드로 확대 |
| 4. direct caller 전수 강제가 outputs에만 한정 | 구조 보정 필요 | `src/cli/**/*.rs` direct page renderer caller 전수 검사 추가 |
| 5. `test-caption` all-fail도 exit 0 | 기존 동작·별도 결함 | 이번 move-only 범위에서 바꾸지 않고 별도 이슈 후보로 유지 |
| 6. dispatch assertion이 rustfmt 형태에 결합 | 보정 | arm과 호출 경로를 분리해 검사 |
| 7. `vector.rs` module doc이 구조 출력을 광고 | 보정 | SVG·render tree 출력 어댑터로 정정 |

## 구현 보정

- `load_document`, `load_document_core`, `classify_hwp_error`, 전역 입력·출력 비밀번호 상태와 pre-scan을
  `src/cli/document_io.rs`로 이동했다. 공개 CLI 문구·분기·exit code는 바꾸지 않았다.
- `hu_to_mm`, `hu_to_mm_i`를 `src/cli/units.rs`로 이동해 `main.rs`의 renderer helper 소유를 끝냈다.
- Render Diff와 trusted classifier·policy mirror는 `src/cli/document_io.rs`를 Render Diff와 Native
  Skia의 공유 입력으로 분류한다.
- `caption_validation.rs`와 `vector.rs`는 direct page renderer caller이지만 현재 Render Diff workflow가
  실행하지 않으므로 일반 Rust로 유지한다. `raster.rs`는 Native Skia만 활성화한다.
- 새 전수 계약은 `src/cli/**/*.rs`에서 `.render_page_svg*`, `.render_page_html*`,
  `.render_page_canvas*`, `.render_page_to_canvas*` 호출을 검색하고 모든 caller가 위 소비자 bucket 중 하나에
  명시적으로 들어가도록 강제한다.
- `src/main.rs`는 최초 2,101줄에서 1,716줄로 줄었으며 문서 입력·인증, 단위 변환, 직접 page renderer,
  structure JSON 구현을 소유하지 않는다.

## 최신화

리뷰 당시 원격 head `3e439a534`에서 `upstream/devel@96da78a9c`를 `2357800d2`로 먼저 병합한 뒤
`eeffb3e8f`에 보정을 고정했다. 안전 중지에서 재개했을 때 `devel`이 다시 15커밋 진전해
`upstream/devel@f6a6bee8f`를 `16ea38cd2`로 current-base merge했다. 두 병합 모두 충돌 없이 완료됐고,
source candidate 시점 branch 관계는 `ahead 17 / behind 0`이다. 최신 15커밋은 renderer 회귀와 review 문서
중심이며 #3789의 CLI·CI 경계와 직접 겹치지 않았다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| #3789 focused Rust 8개 selector | 113/113 PASS |
| classifier·policy Node | 69/69 PASS |
| Render Diff·CI impact workflow Python | 37/37 PASS |
| 전체 release-test nextest | runnable 8,553/8,553 PASS, 43 ignored, 실패 0 |
| 필수 clippy `-D warnings` | PASS, 경고 0 |
| Cargo format / diff check | PASS |
| actionlint | PASS |
| integration suite manifest | 1,017 sources / 4,503 attrs / 48 targets, PASS |
| source unit-tier | 4,221 tests / 299 modules, PASS |

전체 nextest는 `--no-fail-fast`로 실행해 종료 코드 0을 확인했다. TTY 진행 표시가 최종 숫자 행을 보존하지
않아 `cargo nextest list --message-format json`의 8,596개 inventory와 ignored 43개를 대사해 runnable
8,553개를 확인했다. `--prepare`가 최신 devel 통합 테스트에 맞게 갱신한 generated suite는 ignored이며
제출 대상에 포함하지 않는다.

## 종료 판단과 다음 게이트

리뷰 1·2·3번 blocker와 4·6·7번 구조·문서 보정을 완료했고 최신 `devel` 기준 필수 로컬 게이트가 모두
통과했다. 5번은 이동 전부터 있던 독립 false-pass 문제이므로 이번 PR의 동작 보존과 섞지 않는다.

현재 원격 PR은 여전히 이전 `3e439a534`의 Draft이며 그 head의 CI만 성공 상태다. 로컬 candidate
`16ea38cd2`와 이 보고를 포함한 trailing 문서 commit은 아직 push하지 않는다. 작업지시자의 별도 원격 반영
승인 뒤 push하고, 최신 원격 head의 Full CI와 `MERGEABLE / CLEAN`을 새로 확인한 다음 보정 완료 comment를
게시한다.
