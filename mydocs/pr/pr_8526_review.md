# PR #8526 리뷰 — feat(style): 새 스타일을 한/글처럼 고른 스타일 바로 뒤에 넣는다

## 최종 판정

**머지 보류 — 최종 통합 후보의 행 조각 추가 보정·전체 회귀·fresh WASM과 필수 품질 보존 검증 진행 중.**

원 PR head와 메인터너 보정 제품을 구분한다. 원격 comment·approve·push·merge·close는 실행하지 않았다. 검증 완료 뒤 판정과 증적을 갱신한다.

## 접수 정보

- 원 PR: [PR #8526](https://github.com/edwardkim/rhwp/pull/8526), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `a4ce64e250512153d3db6e27d88bb9448f120ee2`. 2026-11-05 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `ece9c3383bb2c73e0f055459ede727a9d3101ac6`.
- 현재 보정 후보: `a314d7a24d3234305abff43149f575853db31105`. 아래 완료된 R2 측정 제품은 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`이며 새 후보 측정으로 바꾸어 쓰지 않는다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 선택한 스타일 뒤에 새 스타일 삽입 후 HWP/HWPX 재열기.

독립 기대 동작: 새 ID 1, 기존 본문·각주·머리말·꼬리말·덧말·메모·복사본의 스타일 참조와 다음 스타일 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | ECE9 리베이스 후 적용 commit |
| --- | --- |
| `e1fc9e35fb9a923feb0a138627175f1d11d1ab3f` | `ef2e7f0e4b968d57d092a86fe9b1ef6016859878` |
| `a4ce64e250512153d3db6e27d88bb9448f120ee2` | `80398d7db157516ceed8437d8a3a1445a3c7a0eb` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래 Native·fresh WASM은 R2 제품 측정이다. 수용 판정은 최종 보정 후보 검증까지 완료한 뒤 내린다.

| 입력·용도 | 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [inserted-style.hwp](../../tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/api-8526-inserted-style-hwp.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [inserted-style-hwp-2020.pdf](../../pdf/semanticist21-20261105/pr8526/inserted-style-hwp-2020.pdf) |
| [inserted-style.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/api-8526-inserted-style-hwpx.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [inserted-style-hwpx-2020.pdf](../../pdf/semanticist21-20261105/pr8526/inserted-style-hwpx-2020.pdf) |

입력의 실행 경로·정식 보존 경로·SHA-256·크기·역할은 [입력 대응표](assets/semanticist21_20261105/validation-inputs.json)에 있다. 새 정식 보존 파일은 최종 제출 전에 실제 commit 포함과 실행 바이트 동일성을 확인한다.

## 주장에 대응하는 페이지 증적

아래 PNG는 같은 R2 측정에서 나온 원본 PNG를 합성했다. 점수와 직접 확대 판독을 구분하며, 최종 후보에서 달라진 영역은 새 증적으로 교체한다.

- `api-8526-inserted-style-hwpx` 1쪽: [review](assets/semanticist21_20261105/validation-r2/png/api-8526-inserted-style-hwpx/review_001.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/api-8526-inserted-style-hwpx/overlay_001.png).

## 회귀와 조판 원칙 준수

- [tests/cases/inserted_style_keeps_references.rs](../../tests/cases/inserted_style_keeps_references.rs)

- R2 제품의 최소 글꼴 집중 검사는 35모듈·169건 PASS이며 새 각주 관계 검사 두 건도 PASS다. 전체 nextest 통과와 구분한다.
- R2 전체 nextest는 행 조각 제품 보정이 필요해 컴파일 중 종료 코드 130으로 중단했으며 최종 후보에서 다시 실행한다.
- 새 base PR #8640의 실제 46상태·124쪽을 포함한 R2 Native 104사례·262쪽은 모두 90% 이상이다. 최저 91.36%, 새 base 대비 2%p 초과 하락 0쪽, 리베이스 전 136쪽 표시값 하락 0쪽이다.
- 현재 진행 중인 fresh WASM·기존 PR #8389/#8397 보호 문서·편람 필수 gate·Rust lint·Native Skia·필요한 브라우저 검증을 완료한 뒤 최종 수용 여부를 갱신한다. 웹폰트 시간 제한 검사는 사용자 지시에 따라 충돌이 없으므로 반복하지 않는다.
- 원본 단위·참조 보존 계약과 실물 배치 좌표 고정을 구분한다. 새 회귀·기존 좌표 핀은 실제 Native/fresh WASM·확대 판독 조건에 맞춰 점검하고 첫머리 근거를 갱신한다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.

## 스타일 생성 실패 반환의 UI 소비 보정

PR #8526이 잘못된 삽입 위치를 -1로 거절하면서, 이전 Node 소스 검사의 ‘항상 push·항상 성공’ 가정이 맞지 않게 됐다. 원 해결 PR #3400·issue #3438의 보정 이력을 대조했다. 실제 생성·삽입·거절의 Rust 보호는 유지하고, UI에서 음수 ID로 모양 적용·성공 알림을 진행하지 않도록 보완했다. 실제 확인 메서드를 실행한 보정 전 2 FAIL / 보정 후 관련 14 PASS, 최적화 fresh WASM의 실제 -1·문서 보존·정상 삽입을 확인했다.

[분석·실행 근거](assets/semanticist21_20261105/style-return-contract/README.md). TypeScript·production build도 통과했으며 실제 browser 소비 검증과 전체 통합 게이트를 완료한 뒤 최종 판정을 갱신한다. 이 UI/API 검사에 다른 문서의 시각 수치를 붙이지 않는다.
