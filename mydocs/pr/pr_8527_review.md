# PR #8527 리뷰 — feat(number): 개요·문단 번호 8~10수준을 한/글처럼 읽고 쓰고 그린다

## 최종 판정

**머지 보류 — 빈 번호 모양의 본문 거리 보정과 새 후보의 최종 검증이 필요함.**

아래 입력의 Native/fresh WASM·직접 판독과 전체 필수 게이트를 완료한 뒤 최종 수용 판정을 갱신한다. 원격 comment·approve·push·merge·close는 실행하지 않았다.

## 접수 정보

- 원 PR: [PR #8527](https://github.com/edwardkim/rhwp/pull/8527), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `7243067034a9cb2d64bb0e6f8938f496461b1c34`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 아래 표는 R5 실제 측정이며, 리베이스 전 R2/R3/R4 결과는 통합 분석과 기존 증적에서 구분한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 8~10수준 번호 문서 열기와 두 형식 저장·재열기.

독립 기대 동작: 10수준별 번호·빈 형식·개요 탐색·원본 머리 레코드 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `1d6c685106306a297cc1fcf2d09b3aaf56732969` | `2052a7921253b0b0445feed7735ba54917223749` |
| `7243067034a9cb2d64bb0e6f8938f496461b1c34` | `d3ee4e0da83d360d132462b46e0671902700a536` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래는 제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 실제 R5 측정이다. Native·fresh WASM을 구분하고 아직 끝나지 않은 행은 진행 중으로 표시한다. 최종 수용은 전체 필수 검증 뒤 판정한다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [levels.hwp](../../tests/fixtures/numbering_levels/levels.hwp) — HWP 8~10수준 번호와 빈 서식 | 1 | [95.16596%](assets/semanticist21_20261105/validation-r5/tsv/8527-levels-hwp-2022-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8527/levels-hwp-2020.pdf) |
| [levels.hwpx](../../tests/fixtures/numbering_levels/levels.hwpx) — HWPX 8~10수준 번호와 빈 서식 | 1 | [95.16596%](assets/semanticist21_20261105/validation-r5/tsv/8527-levels-hwpx-2022-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8527/levels-hwpx-2020.pdf) |
| [scaffold_outline_a.hwpx](../../mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_a.hwpx) — 10수준 확장 후 기존 개요 번호 정상 대조군 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/control-scaffold-a-native.tsv) | 진행 중 | [PDF](../../pdf/issue8540/scaffold_outline_a-2024.pdf) |
| [scaffold_outline_b.hwpx](../../mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_b.hwpx) — 10수준 확장 후 기존 개요 번호 정상 대조군 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/control-scaffold-b-native.tsv) | 진행 중 | [PDF](../../pdf/issue8540/scaffold_outline_b-2024.pdf) |

[원본·PDF SHA와 보존 경로](assets/semanticist21_20261105/validation-r5/inputs.json), [페이지별 정밀 raw·쪽수·producer](assets/semanticist21_20261105/validation-r5/measurements.json)를 연결한다. 실제 실행 파일과 커밋 파일의 동일성을 확인했으며, 다른 문서의 높은 수치로 미달 쪽을 상쇄하지 않는다.

## 주장에 대응하는 페이지 증적

다음 이미지는 같은 R5 측정의 실제 PNG로 현재 Visual Sweep 함수를 실행해 만들었다. 원본과 PDF를 다시 출력하지 않았고, 해당 입력·쪽의 raw가 TSV와 같은지 확인했다. 직접 판독 완료 여부와 전체 통과 판정은 별도로 기록한다.

### native · 8527-levels-hwp-2022 · 1쪽

원 PR의 8~10수준 문단 번호 표시

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8527/native/8527-levels-hwp-2022/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8527/native/8527-levels-hwp-2022/overlay_001.png)

## 확대 판독에서 확인한 추가 결함

R5 원본의 개요 9·10수준은 번호 문자열이 비어 있지만 본문과의 거리 50%를 갖는다.
현재 출력은 약 6.67 CSSpx 왼쪽으로 당겨져 있다. 전쪽 90% 통과로 완료를 선언하지 않고
[독립 근거·생산/측정/배치 소비 경로·보정 계획](assets/semanticist21_20261105/empty-numbering-head-analysis.md)에
따라 공통 머리 기하를 보정한다. 수정 전 관계 검사는 HWP·HWPX 둘 다 실패했다.
아래 R5 증적은 보정 전 결과이며 새 후보의 완료 증거로 재표기하지 않는다.

## 회귀와 조판 원칙 준수

- [tests/cases/numbering_levels_eight_to_ten.rs](../../tests/cases/numbering_levels_eight_to_ten.rs)

- [최소 글꼴 집중 회귀](assets/semanticist21_20261105/validation-r5/focused-regression.txt)는 184건 PASS다. 전체 Rust 회귀 완료로 대신하지 않는다.
- R5 Native 전체는 128개 입력·301쪽, 최저 91.81%다. [전체 Rust 목록 대조](assets/semanticist21_20261105/validation-r5/full-rust-inventory-audit.json)에서 최소 글꼴 5개·55개 binary·12,735건 PASS·45건 ignored를 확인했고 누락·중복은 없다. [실제 브라우저·생성 타입 검증](assets/semanticist21_20261105/validation-r5/ui/README.md)도 완료했다. fresh WASM·새 base 대비 비교·보호 PR #8389/#8397·편람 게이트·lint·Native Skia의 최종 결과는 계속 통합한다.
- 원본 단위·참조 보존과 실물 배치의 절대 좌표 고정을 구분한다. 완료 페이지의 Native/fresh WASM 양쪽 raw 98% 이상과 독립 PDF 확대 판독을 확인한 핀만 유지하며, 회귀 첫머리를 실제 측정 근거로 갱신한다.
- 리베이스 전 R2/R3의 성공 기록과 현재 R5를 구분한다. source·측정 조건이 같은 완전한 증거만 재사용하고 현재 제품의 실패를 과거 통과로 덮지 않는다.
- 웹폰트 시간 제한·오프라인 검사는 관련 충돌이 없으므로 작업지시자 지시에 따라 반복하지 않는다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
