# PR #8602 리뷰 — fix(save): 편집으로 빈 문단이 되면 끝 표시만 든 PARA_TEXT를 남기지 않는다

## 최종 판정

**머지 보류 — R5 통합 제품의 최종 검증 진행 중.**

아래 입력의 Native/fresh WASM·직접 판독과 전체 필수 게이트를 완료한 뒤 최종 수용 판정을 갱신한다. 원격 comment·approve·push·merge·close는 실행하지 않았다.

## 접수 정보

- 원 PR: [PR #8602](https://github.com/edwardkim/rhwp/pull/8602), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `338a75a1afd5bbe62843d71784a5c82b89613620`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 아래 표는 R5 실제 측정이며, 리베이스 전 R2/R3/R4 결과는 통합 분석과 기존 증적에서 구분한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 개체·전체 글을 지워 빈 문단으로 만든 뒤 저장.

독립 기대 동작: 끝 표시만 든 PARA_TEXT를 남기지 않음, 빈 문단 줄 정보와 한컴 열기 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `cad70bbaa4da6edbfb5348aa0fb91c83f1367241` | `228e426fd19b342b8c525beeca18c27152e0caa7` |
| `338a75a1afd5bbe62843d71784a5c82b89613620` | `6b6f2d60925d3db6506a1f49ba41194bcfad4a40` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래는 제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 실제 R5 측정이다. Native·fresh WASM을 구분하고 아직 끝나지 않은 행은 진행 중으로 표시한다. 최종 수용은 전체 필수 검증 뒤 판정한다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [rhwp-after.hwp](../../mydocs/pr/assets/emptied-paragraph-para-text/rhwp-after.hwp) — 제출된 정상 빈 문단 저장본 읽기; 현재 삭제 저장 생성과 구분 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8602-rhwp-after-hwp-2022-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8602/rhwp-after-hwp-2020.pdf) |
| [rhwp-after.hwp](../../mydocs/pr/assets/emptied-paragraph-para-text/rhwp-after.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8602-empty-deleted-hwp-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8602/empty-deleted-hwp-2020.pdf) |
| [empty-deleted.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8602/empty-deleted.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8602-empty-deleted-hwpx-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8602/empty-deleted-hwpx-2020.pdf) |

[원본·PDF SHA와 보존 경로](assets/semanticist21_20261105/validation-r5/inputs.json), [페이지별 정밀 raw·쪽수·producer](assets/semanticist21_20261105/validation-r5/measurements.json)를 연결한다. 실제 실행 파일과 커밋 파일의 동일성을 확인했으며, 다른 문서의 높은 수치로 미달 쪽을 상쇄하지 않는다.

## 주장에 대응하는 페이지 증적

다음 이미지는 같은 R5 측정의 실제 PNG로 현재 Visual Sweep 함수를 실행해 만들었다. 원본과 PDF를 다시 출력하지 않았고, 해당 입력·쪽의 raw가 TSV와 같은지 확인했다. 직접 판독 완료 여부와 전체 통과 판정은 별도로 기록한다.

### native · api-8602-empty-deleted-hwp · 1쪽

실제 삭제 후 빈 문단의 한컴 열기·Print 결과. PARA_TEXT 유무는 원문 레코드 검사로 입증한다.

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8602/native/api-8602-empty-deleted-hwp/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8602/native/api-8602-empty-deleted-hwp/overlay_001.png)

## 저장 손상의 독립 검출

RHWP 자체 재열기는 base·후보가 모두 통과하므로 원래 손상 검출의 증거로 사용하지 않는다.
[원시 CFB 레코드 대조](assets/semanticist21_20261105/validation-r5/api/pr8602-independent-record-comparison.json)는
rhwp 파서 없이 `olefile`·`zlib`·`struct`로 같은 공개 API 저장본을 읽었다.
삭제 대상 문단은 base에서 `char_count=1`·`PARA_TEXT=[0x000d]`, 후보에서는
`char_count=1`·`PARA_TEXT 없음`이며 줄 시작 `[0]`은 양쪽 보존된다.
이는 원 PR이 한컴에서 확인한 빈 문단 저장 계약과 정식 회귀의 기대값에 대응한다.

## 회귀와 조판 원칙 준수

- [tests/cases/emptied_paragraph_para_text.rs](../../tests/cases/emptied_paragraph_para_text.rs)

- [최소 글꼴 집중 회귀](assets/semanticist21_20261105/validation-r5/focused-regression.txt)는 184건 PASS다. 전체 Rust 회귀 완료로 대신하지 않는다.
- R5 Native 전체는 128개 입력·301쪽, 최저 91.81%다. [전체 Rust 목록 대조](assets/semanticist21_20261105/validation-r5/full-rust-inventory-audit.json)에서 최소 글꼴 5개·55개 binary·12,735건 PASS·45건 ignored를 확인했고 누락·중복은 없다. [실제 브라우저·생성 타입 검증](assets/semanticist21_20261105/validation-r5/ui/README.md)도 완료했다. fresh WASM·새 base 대비 비교·보호 PR #8389/#8397·편람 게이트·lint·Native Skia의 최종 결과는 계속 통합한다.
- 원본 단위·참조 보존과 실물 배치의 절대 좌표 고정을 구분한다. 완료 페이지의 Native/fresh WASM 양쪽 raw 98% 이상과 독립 PDF 확대 판독을 확인한 핀만 유지하며, 회귀 첫머리를 실제 측정 근거로 갱신한다.
- 리베이스 전 R2/R3의 성공 기록과 현재 R5를 구분한다. source·측정 조건이 같은 완전한 증거만 재사용하고 현재 제품의 실패를 과거 통과로 덮지 않는다.
- 웹폰트 시간 제한·오프라인 검사는 관련 충돌이 없으므로 작업지시자 지시에 따라 반복하지 않는다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
