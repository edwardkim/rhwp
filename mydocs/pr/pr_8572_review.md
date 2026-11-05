# PR #8572 리뷰 — fix(edit): 칸 글로 높이가 바뀐 글자처럼 취급 표의 저장 줄과 뒤 문단 위치를 한/글처럼 맞춘다

## 최종 판정

**머지 보류 — R5 통합 제품의 최종 검증 진행 중.**

아래 입력의 Native/fresh WASM·직접 판독과 전체 필수 게이트를 완료한 뒤 최종 수용 판정을 갱신한다. 원격 comment·approve·push·merge·close는 실행하지 않았다.

## 접수 정보

- 원 PR: [PR #8572](https://github.com/edwardkim/rhwp/pull/8572), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `d9750b5604d3f23fd8645ae3f19dfc0cb293a70e`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 아래 표는 R5 실제 측정이며, 리베이스 전 R2/R3/R4 결과는 통합 분석과 기존 증적에서 구분한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: footnote-01 제목 표의 셀 문단 앞에 표안한글 삽입 후 저장.

독립 기대 동작: 표 위쪽 유지, 아래 표의 동일 간격, 7쪽·문단 소속 및 추가 Enter 재편집 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `9ded487923f1fa8d6e49e2ef1ef85fff4145d90c` | `96500d20a06a3ed82f683fb284e7f05183253cc5` |
| `ee831a3465fd8d58c589031cc084fda95a28be80` | `deb1b7ca097c579cf268923ad981a903bcf609df` |
| `a9cd16ad09ba6a280a1e7a6456342a9741619f15` | `8f05bf6ab5e28ee3b79ead1cdfcbb355fdb5edba` |
| `410c5dba40d0dbdf27a6da6312e5530003a56bb0` | `7bb8fa38baf14b5d7420a1aad925bfd87b2d18dc` |
| `d9750b5604d3f23fd8645ae3f19dfc0cb293a70e` | `beb26383c1e1f7c2f15d28587fdf3d0ce36a3620` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래는 제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 실제 R5 측정이다. Native·fresh WASM을 구분하고 아직 끝나지 않은 행은 진행 중으로 표시한다. 최종 수용은 전체 필수 검증 뒤 판정한다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [fix-typed.hwp](../../mydocs/pr/assets/tac-host-row-cell-growth/fix-typed.hwp) — 제출된 표 셀 편집 정상 저장본과 후속 문단 배치 | 7 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8572-fix-typed-hwp-2022-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8572/fix-typed-hwp-2020.pdf) |
| [actual-api-cell-typed.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8572/actual-api-cell-typed.hwpx) — 실제 fresh WASM 제목 셀 편집 후 HWPX 저장·다시 열기와 독립 Print의 전체 페이지 | 7 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8572-cell-typed-hwpx-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8572/actual-cell-typed-hwpx-2020.pdf) |
| [footnote-01.hwp](../../samples/footnote-01.hwp) — 원 PR 제목 셀 편집 전 동일 원본·표 위치의 전쪽 정상 대조군 | 6 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8572-original-footnote-hwp-native.tsv) | 진행 중 | [PDF](../../pdf/footnote-01-hwp-2020.pdf) |

[원본·PDF SHA와 보존 경로](assets/semanticist21_20261105/validation-r5/inputs.json), [페이지별 정밀 raw·쪽수·producer](assets/semanticist21_20261105/validation-r5/measurements.json)를 연결한다. 실제 실행 파일과 커밋 파일의 동일성을 확인했으며, 다른 문서의 높은 수치로 미달 쪽을 상쇄하지 않는다.

## 주장에 대응하는 페이지 증적

다음 이미지는 같은 R5 측정의 실제 PNG로 현재 Visual Sweep 함수를 실행해 만들었다. 원본과 PDF를 다시 출력하지 않았고, 해당 입력·쪽의 raw가 TSV와 같은지 확인했다. 직접 판독 완료 여부와 전체 통과 판정은 별도로 기록한다.

### native · 8572-fix-typed-hwp-2022 · 1쪽

원 PR의 표안한글 편집 저장본: 제목 표 확장·다음 표 간격 및 다음 쪽 첫 문단

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8572/native/8572-fix-typed-hwp-2022/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8572/native/8572-fix-typed-hwp-2022/overlay_001.png)

### native · 8572-fix-typed-hwp-2022 · 2쪽

원 PR의 표안한글 편집 저장본: 제목 표 확장·다음 표 간격 및 다음 쪽 첫 문단

![같은 원본 2쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8572/native/8572-fix-typed-hwp-2022/review_002.png)

![같은 원본 2쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8572/native/8572-fix-typed-hwp-2022/overlay_002.png)

## 회귀와 조판 원칙 준수

- [tests/cases/tac_host_row_follows_cell_growth.rs](../../tests/cases/tac_host_row_follows_cell_growth.rs)

- [최소 글꼴 집중 회귀](assets/semanticist21_20261105/validation-r5/focused-regression.txt)는 184건 PASS다. 전체 Rust 회귀 완료로 대신하지 않는다.
- R5 Native 전체는 128개 입력·301쪽, 최저 91.81%다. [전체 Rust 목록 대조](assets/semanticist21_20261105/validation-r5/full-rust-inventory-audit.json)에서 최소 글꼴 5개·55개 binary·12,735건 PASS·45건 ignored를 확인했고 누락·중복은 없다. [실제 브라우저·생성 타입 검증](assets/semanticist21_20261105/validation-r5/ui/README.md)도 완료했다. fresh WASM·새 base 대비 비교·보호 PR #8389/#8397·편람 게이트·lint·Native Skia의 최종 결과는 계속 통합한다.
- 원본 단위·참조 보존과 실물 배치의 절대 좌표 고정을 구분한다. 완료 페이지의 Native/fresh WASM 양쪽 raw 98% 이상과 독립 PDF 확대 판독을 확인한 핀만 유지하며, 회귀 첫머리를 실제 측정 근거로 갱신한다.
- 리베이스 전 R2/R3의 성공 기록과 현재 R5를 구분한다. source·측정 조건이 같은 완전한 증거만 재사용하고 현재 제품의 실패를 과거 통과로 덮지 않는다.
- 웹폰트 시간 제한·오프라인 검사는 관련 충돌이 없으므로 작업지시자 지시에 따라 반복하지 않는다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
