# PR #8564 리뷰 — fix(edit): 날짜·파일 경로 코드 바로 앞뒤에 친 글자를 코드 밖에 둔다

## 최종 판정

**메인터너 보정 후 수용 가능 — 원 PR 주장 범위의 로컬 검증 완료.**

원본별 Native/fresh WASM 전쪽 TSV와 같은 주장 페이지의 review·overlay를 아래에 연결했다. [통합 PR #8659](https://github.com/edwardkim/rhwp/pull/8659)의 정확한 code head CI가 성공했다. [통합 리뷰](pr_8659_review.md)에 실행 URL을 기록했으며, trailing head 확인·merge·원 PR close는 후속 조건이다.

## 접수 정보

- 원 PR: [PR #8564](https://github.com/edwardkim/rhwp/pull/8564), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `5477f2cc6c4daf93ea315828650ae94eaf15b1b7`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: Rust `86709ac16711820e61baa3b87e2c4ddaac23573e` · Studio `5feccb130e0f695de0046d994c060130fa3e928e`. 아래 표는 R6 새 Native·fresh WASM 실제 측정이다. R5 과거 증거는 출처를 구분해 보존한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 날짜·경로 코드 앞뒤에 글을 넣고 저장·재열기.

독립 기대 동작: 글은 코드 밖에 남고 코드 값·범위는 보호됨.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `9cd75cef7f52fe1a9b040a61632d4acc8d5d0212` | `07b7178a9984e69698f5433b8ac53894aa563700` |
| `5477f2cc6c4daf93ea315828650ae94eaf15b1b7` | `c60021e63fa62bc5ad4b740e00d5714f599d9dae` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](../assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

제품 R6의 실제 전체 페이지 측정이다. 두 출력 경로와 독립 Print PDF의 전체 쪽수가 일치하며,
모든 페이지가 90% 이상이다. 96 DPI·Print 프로필·색상 경계 보정 전 2px raw를 사용했다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [date-path-context-before.hwp](../../../tests/fixtures/semanticist21_integration_review/pr8564/date-path-context-before.hwp) — 한컴이 갱신한 날짜·파일명 코드 표시 상태, 원본 경계 입력 검사의 보충 대조 | 1 | [91.81401%](../assets/semanticist21_20261105/validation-r6/tsv/8564-.result.9d14e579-4810-4dc8-b9db-a0cf0bb4bd23.partial-hwp-2022-native.tsv) | [91.17272%](../assets/semanticist21_20261105/validation-r6/tsv/8564-.result.9d14e579-4810-4dc8-b9db-a0cf0bb4bd23.partial-hwp-2022-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8564/date-path-context-before-2020.pdf) |
| [date-path-context-edited.hwp](../../../tests/fixtures/semanticist21_integration_review/pr8564/date-path-context-edited.hwp) — 코드 밖 경계 입력 후 독립 Print | 1 | [92.01038%](../assets/semanticist21_20261105/validation-r6/tsv/api-8564-context-after-native.tsv) | [92.28612%](../assets/semanticist21_20261105/validation-r6/tsv/api-8564-context-after-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8564/date-path-context-after-2020.pdf) |

[원본·PDF SHA·producer·정밀 raw](../assets/semanticist21_20261105/validation-r6/measurements.md), [측정 명령·환경](../assets/semanticist21_20261105/validation-r6/README.md)를 연결한다. 원 PR 원본·실제 API 저장본·보충 대조군의 용도를 구분한다.

## 주장에 대응하는 페이지 증적

같은 원본·독립 PDF·페이지의 실제 PNG다. 직접 판독과 실제 API 검증을 구분한다. 일부 Native 패널은 현재 RHWP/PDF PNG 쌍의 SHA가 이전 판독 쌍과 각각 같음을 확인해 재사용했다.

### api-8564-context-after · 1쪽

실제 날짜·파일 경로 코드 앞뒤 입력이 필드 밖에 남는 저장 결과

**native raw 92.01038%** — 날짜 앞뒤 A앞·뒤B와 파일명 앞의 앞·뒤C를 직접 대조했다. 인쇄 때 파일명 코드가 임시 이름으로 재평가되어 문자열 길이가 다르다. api/pr8564-print-context.json에 독립 영수증·빈 기본 Scripts·실제 PDF 문자열을 기록했고 위치 오차나 동일 문자열의 일치로 주장하지 않는다.

![native 같은 원본 1쪽 review](../assets/semanticist21_20261105/validation-r6/png/pr8564/native/api-8564-context-after/review_001.png)

![native 같은 원본 1쪽 overlay](../assets/semanticist21_20261105/validation-r6/png/pr8564/native/api-8564-context-after/overlay_001.png)

**wasm raw 92.28612%** — 날짜 필드 전후 A앞·뒤B, 경로 필드 전후 앞·뒤C가 보존된다. 파일명 필드는 MCP scripts_disabled 인쇄의 임시 서버 파일명과 RHWP 저장 파일명이 달라 내용이 중첩되지 않는다. 기존 독립 변환 영수증으로 확인한 동적 필드 재평가 차이이며 동일 문자열 일치로 주장하지 않는다.

![wasm 같은 원본 1쪽 review](../assets/semanticist21_20261105/validation-r6/png/pr8564/wasm/api-8564-context-after/review_001.png)

![wasm 같은 원본 1쪽 overlay](../assets/semanticist21_20261105/validation-r6/png/pr8564/wasm/api-8564-context-after/overlay_001.png)

## 파일명 코드의 독립 Print 평가 문맥

[Print 영수증·원시 Scripts 스트림·추출 문자열](../assets/semanticist21_20261105/validation-r6/runtime-checks.md)을 대조했다.
한컴 2020 MCP의 Print는 `input_preprocess=scripts_disabled`이며, 두 HWP의 Scripts는
실행 코드가 없는 동일한 빈 기본 스텁이다. 파일명 코드는 인쇄 시 서버의 `rhwp-no-scripts-*.hwp`로
재평가되어 RHWP가 보존한 `.result.*.partial.hwp` 캐시와 다르다. 이 문자열 길이 차이를
조판 개선으로 주장하거나 가리지 않는다. 날짜·파일명 양끝의 `앞`·`뒤`와 다음 본문 `C`는 보존된다.

따라서 위 보충 snapshot은 원본 파일명이 완전히 같은 출력의 증거가 아니다. 원 주장인 필드 경계는
[같은 원본의 실제 공개 API 전후 대조](../assets/semanticist21_20261105/validation-r6/runtime-checks.md)에서
base가 앞뒤 글자를 필드 값에 포함해 실패하고 후보는 원래 필드 값을 보존해 통과함으로 확인했다.

## 회귀와 조판 원칙 준수

- [tests/cases/date_path_code_boundary_typing.rs](../../../tests/cases/date_path_code_boundary_typing.rs)

- 현재 전쪽 검증은 Native 128입력·301쪽 최저 91.81401%, fresh WASM 128입력·301쪽 최저 91.17272%다. 같은 새 base의 각 301쪽 대비 하락 0쪽이다. [현재 증거](../assets/semanticist21_20261105/validation-r6/README.md)를 연결한다.
- 실제 API 12그룹·브라우저 동작·TypeScript/production build·생성 타입 export와 소비 검사는 [API](../assets/semanticist21_20261105/validation-r6/api/README.md)·[UI](../assets/semanticist21_20261105/validation-r6/ui/README.md)에 연결했다. 원래 R5 전체 Rust 12,735건을 R6 새 실행으로 표현하지 않는다.
- 최소 공개 글꼴 5개에서 현재 전체 Rust 12,740건 PASS·기존 ignored 45건, 실제 목록 누락·중복 0이다. 최종 편람·lint·Skia·release·doctest도 완료했다.
- 실제 assertion을 확인해 관계형·고정형을 구분하고 27개 파일 첫머리를 실제 측정값으로 갱신했다. 원본 단위·비율 계약은 유지하며, [17개 완료 페이지](../assets/semanticist21_20261105/validation-r6/qualification.md)는 양쪽 98% 이상과 독립 확대 판독 근거로 기존 고정 좌표를 유지한다.
- 원래 웹폰트 수명 검사는 관련 충돌이 없어 재사용했으나, 최종 Studio 기호 공급 보정은 로딩 경로를 바꾸므로 로더 수명 14건과 실제 브라우저 7시나리오를 추가 실행해 모두 통과했다.

## Merge 후 contributor PR comment 계획

통합 PR의 실제 CI URL·code/trailing head와 확정 merge SHA를 기록한 뒤 원 PR에 한국어 존댓말로 결과를 게시한다. 기존 comment가 있으면 해당 comment를 수정하고 API로 다시 확인한다. 원 기여의 해결 범위와 위 공통 메인터너 보정을 구분한다.

- 실제 확인 페이지: api-8564-context-after p1.
- Native/fresh WASM의 위 같은 원본·쪽 review와 overlay를 사용한다. 수치와 직접 판독 결론·한계는 본문 관측을 그대로 반영한다.
- 대표 이미지 안정 경로: `mydocs/pr/assets/semanticist21_20261105/validation-r6/png/pr8564/<backend>/<key>/<review 또는 overlay>_NNN.png`. 같은 PR의 추가 완료 페이지는 [패널 목록](../assets/semanticist21_20261105/validation-r6/panels.md)을 따른다.
- URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/` 뒤에 위 실제 경로를 붙인다. devel 포함 확인 후 `--body-file` 게시·API 재조회한다.
- 문서 비교는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다. 원 PR·관련 이슈 close는 같은 해결 범위의 완료와 포함 관계를 확인한 뒤 처리한다.
