# PR #8227 리뷰 — PDF 편집 우선 본문 흐름

> **2026-10-29 통합 후보 검증:** 아래 과거 결과는 각 측정 당시 head의 기록이다. 한국어 가정통신문은 과거와 같은 글꼴·PDF 래스터 조건에서 Native 두 쪽의 렌더 PNG와 점수가 이전과 동일하다. 현재 시스템 글꼴 조건과 fresh WASM은 이 재현 결과와 구분한다. [동일 조건 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json). 통합 최종 판정과 남은 조건은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)을 따른다.

## 최종 판정

**#8227의 편집 우선 본문 흐름은 통합 브랜치에 반영됐고 관련 검사가 통과했다. 통합 PR의 최종 판정은 별도 남은 조건을 따른다.**
원 head의 단독 실행이나 원격 승인을 뜻하지 않는다. 원 변경을 누적 적용한 아래 통합 후보에 대한 판정이며,
통합 PR의 최신 head CI와 검토가 완료되면 작업지시자가 승인한 머지와 후속 처리를 진행한다. 작업지시자가 승인한 통합 PR은 남은 검사와 미해결 기준을 처리한 뒤 제출한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#8227](https://github.com/edwardkim/rhwp/pull/8227) / planet6897 / devel |
| 체리픽 당시 원 head | `a34d1fb70dcdc2e1f692efaa63bfd2d77ee0e74c` |
| 2026-10-28 재조회 원 head | `fe855d85935c47b3ff5fd1b93839e5926f468828`; 7개 커밋의 patch-id가 체리픽 당시 원 커밋과 각각 동일 |
| 원 PR의 이슈 연결 | 원 PR 자동 닫기 연결 없음; 통합 PR의 자동 닫기로 전환하지 않음 |
| 원 head CI 참고값 | SUCCESS; 통합 후보 CI와 구분 |
| 검증한 통합 code/test head | `2e902abacf328bccdd7780606666ac3b1c5c1ce9` |
| 시각 production source | `ae8c413f84a01a4cc4591357056183709f21b150` |
| 고정 기준 devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 실제 적용·중복 | 4개 적용 / 3개 patch 중복 제외 |
| reviewer·조회 시점 상태 | jangster77 지정 유지 / OPEN / mergeable=MERGEABLE; 머지 전 재조회 필요 |

브랜치는 `review/integration-20261026-document-updates`다. [적용 순서·SHA·단계 기록](<../../../mydocs/pr/archives/pr_8227_review_impl.md>)과
[원 head·reviewer·댓글·스레드 재조회](<../../../mydocs/working/assets/review_integration_20261026/validation/source-pr-preflight.json>), [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)을 연결한다.

## 변경과 실제 검증 범위

fidelity와 editable을 명시적으로 구분한다. editable에서는 실제 구역 본문 문단이 존재하고 단·쪽의 이어짐이 보존되어야 하며, 기존 fidelity 출력도 별도로 검사한다.

원 PR 제목: feat(pdf-import): 본문 문단을 구역 본문 흐름으로 가져오는 편집 우선 배치 옵션 — 단·쪽을 넘는 문단 잇기 (#8220, #8221 위)

주요 구현 경로: [pdf_import.rs](<../../../src/cli/commands/pdf_import.rs>), [extended.rs](<../../../src/cli/metadata/capabilities/extended.rs>), [public.rs](<../../../src/cli/metadata/help/public.rs>), [read.rs](<../../../src/cli/metadata/mcp/read.rs>), [body_regions.rs](<../../../src/parser/pdf/body_regions.rs>), [editable.rs](<../../../src/parser/pdf/editable.rs>), [mod.rs](<../../../src/parser/pdf/mod.rs>).

보호 검사: [issue_7998_pdf_inferred_gap_tabs.rs](<../../../tests/cases/issue_7998_pdf_inferred_gap_tabs.rs>), [issue_8220_pdf_editable_layout.rs](<../../../tests/cases/issue_8220_pdf_editable_layout.rs>), [issue_8220_pdf_exam_paragraph_boundaries.rs](<../../../tests/cases/issue_8220_pdf_exam_paragraph_boundaries.rs>). 전체 nextest 또는 해당 Native/Studio 실행의 실제 결과에 포함된다.

편집 가능 모드의 2010 PDF 7쪽에서 「결국르포르는…」 첫 글줄 위치를 독립 PDF 좌표 x=603.98px, y=388.54px와 각각 ±2px로 비교하는 고정형 검사를 추가했다. 이전에 측정한 편집 전 7쪽 일치율은 Native 99.82145% / fresh WASM 99.77620%다. 편집 후 문단의 관계 검사는 그대로 유지한다. 해당 파일의 검사 4개가 통과했으며 [좌표 근거](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/pdf2010-exam-p7-coordinates.json>)와 [검사 실행](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/pdf-editable-coordinate-test.json>)을 기록한다.

별도로 만든 추가 실험에서 같은 시험의 편집을 HWP로 저장·재열기한 입력은 편집 문장이 있는 4쪽 Native 65.17416% / fresh WASM 64.97946%, 전16쪽 최저 1.58239% / 1.63218%다. 한컴 2020 Print 5쪽과 재열기 출력의 내용과 쪽 배치가 크게 다르다. 이는 원 PR이 제출한 편집 전 시각 검증이나 메모리 내 편집 관계 검사의 결과가 아니다. 이 추가 실험을 근거로 #8227의 제출 범위가 미해결이라고 판정하지 않는다. [실험 입력·독립 Print와 전쪽 측정](<../../../mydocs/pr/assets/pr8227-editable-after-save/evidence.json>).

동일한 저장 HWP·PDF·현재 글꼴 환경으로 새 base와 후보의 Native 4~6쪽을 추가 비교한 결과 PNG가 모두 동일했다. 이 확인은 해당 세 쪽에 한하며 WASM이나 다른 쪽까지 같다는 뜻은 아니다.

이 PR에 별도 단독 보정은 추가하지 않았다. 누적 통합에 필요한 컴파일 호환 보정과 다른 PR의 공통 경로 보정은 통합 후보에 포함된다.

## 검증 입력과 결과

[전체 로컬 실행·명령·exit·검증 SHA](<../../../mydocs/working/assets/review_integration_20261026/validation/final-rust-validation.json>)에 fmt·세 Clippy·workspace build·고정 base 정책·전체 nextest·Native 실행 결과를 보존한다.
Studio 단위 검사는 2,607개 통과·기존 skip 2개이며 실제 브라우저와 fresh 최적화 WASM도 실행했다. [production source 동일성](<../../../mydocs/working/assets/review_integration_20261026/validation/production-source-continuity.json>)과 [WASM 빌드 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/final-wasm-build.json>)로 검증 head를 구분한다.

| 입력 조합 | 전체 쪽수 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: | ---: |
| pdf-2010-fidelity | 16 | [98.80885%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-2010-fidelity.tsv>) | [98.69530%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-2010-fidelity.tsv>) |
| pdf-2020-fidelity | 15 | [99.22374%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-2020-fidelity.tsv>) | [98.69112%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-2020-fidelity.tsv>) |
| pdf-mac-fidelity | 15 | [98.52190%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-mac-fidelity.tsv>) | [98.44719%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-mac-fidelity.tsv>) |
| pdf-2010-editable | 16 | [98.80885%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-2010-editable.tsv>) | [98.69835%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-2010-editable.tsv>) |
| pdf-2020-editable | 15 | [99.13134%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-2020-editable.tsv>) | [98.69133%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-2020-editable.tsv>) |
| pdf-mac-editable | 15 | [98.51800%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-mac-editable.tsv>) | [98.44191%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-mac-editable.tsv>) |

해당 조합은 모두 독립 PDF의 전쪽과 대응하며 90% 미만·후보 쪽수 불일치·누락이 없다. 실제 경로·입력/PDF SHA·명령은 [전쪽 실행 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/visual-validation.json>),
[커밋 blob과 실제 입력의 해시 감사](<../../../mydocs/working/assets/review_integration_20261026/validation/tracked-input-audit.json>), [실제 글꼴·PDF 래스터 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/font-and-raster-provenance.json>)에 있다.

[동일 조건 base/head 판정](<../../../mydocs/working/assets/review_integration_20261026/validation/base-head-comparison.json>)에서 Native·WASM 각각 64개 조합·910개 기준 쪽을 비교했고 2%p 이상 하락은 0쪽이다. 미세 하락 여부·최대 하락은 원장의 실제 값을 따른다.
기준의 공식 사양 83쪽/71쪽과 cellnotes 12쪽/11쪽 오류는 기준 실패로 보존하고 모든 기준 쪽을 별도 진단 비교했다.
새 editable 3모드는 구 버전에 없으므로 독립 PDF·실제 본문 소유 검사로 별도 검증했다.

[실제 브라우저 PDF import·설정·본문 소유](<../../../mydocs/working/assets/review_integration_20261026/validation/pdf-layout-ui-review.json>)에서 2010 16쪽·2020 15쪽·mac 15쪽의 두 모드를 확인했다. editable 실제 본문 문단은 각각 255·256·267개이며 비 PDF 대조군도 보존했다. WASM 중간본은 실제 브라우저 import로 만들었고 임시 HWPX를 fixture로 커밋하지 않는다.

## 2026-10-28 원 PR 입력과 쪽 확인

원 PR의 시험지 PDF 가져오기에서 2010 원본 7쪽 fidelity의 Native/fresh WASM은 99.83396% / 99.77416%, editable은 99.82145% / 99.77620%다. 전16쪽 최저는 각각 98.80885% / 98.69530%, 98.80885% / 98.69835%다. 편집 모드로 가져온 뒤 추가 편집한 상태의 시각 수치는 별도로 확인해야 한다. 위 수치는 2026-10-27 후보의 기록이며 최신 head의 재측정 결과는 아니다. [측정 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/visual-validation.json>)에서 해당 PDF·가져오기 모드와 출처를 확인한다.

## 별도로 확인한 2010 PDF 1쪽과 3쪽

원 PR에서 들여쓰기와 쪽을 넘는 문단 머리를 판독한 2010 시험지 PDF의 편집 가능 모드에 대해, 2026-10-27 측정의 1쪽은 Native 99.69372% / fresh WASM 98.69835%, 3쪽은 Native 99.75461% / fresh WASM 99.66472%다. 해당 입력의 전 16쪽과 실제 PDF·가져오기 경로는 [측정 원장](../../working/assets/review_integration_20261026/validation/visual-validation.json), 쪽별 값은 [Native TSV](../../working/assets/review_integration_20261026/validation/tsv/native/pdf-2010-editable.tsv)와 [WASM TSV](../../working/assets/review_integration_20261026/validation/tsv/wasm/pdf-2010-editable.tsv)에 있다. 이 수치는 후속 #8184 체리픽 전의 결과이며 편집 후 저장·재열기한 별도 입력의 결과가 아니다.

## 조판 원칙 준수 판정

| 항목 | 판정과 근거 |
| --- | --- |
| 제출 전 로컬 검증 | 완료된 공통 검증은 [선택 회귀 110개](../../working/assets/review_integration_20261026/validation/post-rebase-8328/fontless-branch-regression.json), [후속 회귀 24개](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json), [Rust lint·build](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr-local-rust-gates-20261029.json)과 [시각 도구·CI 구문](../../working/assets/review_integration_20261026/validation/post-rebase-8328/prepr-visual-ci-checks.json)에 기록한다. 전체 Rust 회귀는 작업지시자 지시에 따라 재실행하지 않았고 원격 CI는 별도다. |
| 기존 해결의 선행 분석 | 충족 — [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>); 원 이슈·원 해결·보정 SHA를 실패별로 추적 |
| 구현 근거와 일반성 | 충족 — 위 실제 소비 경로와 독립 기대값. 문서 ID 보정·출력 은폐 없이 공통 결과 사용 |
| 측정·배치 일관성 | 충족 — 공통 계산·저장 줄·프레임 결과의 실제 소비와 전쪽 Native/WASM 출력 대조 |
| 리베이스 기능 보존 | 충족 — #8116 새 base의 15개 계약·4개 문서와 기존 17개 계약·전쪽을 함께 검증. 한 충돌에서 두 수용 조건을 보존했으며 커밋 대조와 실행 원장에 기록 |
| 최종 출력 소비 | 충족 — 실제 Native PNG·fresh WASM Canvas, 적용되는 CanvasKit 경로를 구분 |
| 분할·이어받기 | 비해당 — 표 조각 컷·이어받기 규칙은 변경하지 않음 |
| 줄 소속과 점유 높이 | 충족 — 원본 LineSeg/개체 줄·공통 flow 결과·실제 문단/개체 소유 대조 |
| 독립성·반례 | 충족 — 실제 원본·독립 Print 또는 원본 payload/외부 경로·편집 대조군. 임시 스크립트를 정식 회귀로 대신하지 않음 |
| 기준값 변경 | 비해당 — 이 PR의 baseline·golden·허용치를 완화하지 않음 |
| 회귀 추가 선행 조건 | 충족 — 관련 입력 전쪽 Native와 해당 보강 단계의 fresh WASM 최저 90% 이상 선행 확인. 보강 회귀는 소유·순서·내용 포함 관계이며 최종 최적화 WASM도 별도 완료 |
| 수식·간격 확대 판독 | 충족 — 관련 변경·최저 쪽과 독립 Print의 직접 비교. 원 PR 잔여 차이·정책 fallback은 별도 공개 |
| 구현 주장과 검증 범위 | 충족 — 고정 source와 실제 결과를 연결. 원 head 단독 CI·정량 성능·미실행 경로로 범위 확장하지 않음 |

## 시각 증적과 남은 조건

[대표 비교·overlay](<../../../mydocs/working/assets/review_integration_20261026/validation/review-panels.json>)와 [전쪽·확대 증거 목록](<../../../mydocs/working/assets/review_integration_20261026/validation/README.md>)을 연결한다. 시각 지표는 색상 경계 보정 전 raw 실루엣이며 글꼴 예외로 gate를 우회하지 않았다.

원 fork head는 변경하지 않았다. 실제 편람 수정 #8268·충돌 #8161은 제외하며 추가 PR을 체리픽하지 않는다.

## Merge 후 contributor comment 계획

현재는 게시·close·merge를 실행하지 않는다. 추후 승인된 통합 머지 뒤 최종 merge SHA·CI URL과
Visual Sweep 정본·SHA 고정 raw 이미지·원 기여와 보정 범위·남은 차이를 정중한 한국어로 안내한다.
작업지시자가 승인한 후속 처리로 본문 파일을 게시하고 API에서 실제 내용과 링크를 재확인한다.

## 원 PR의 2020 PDF 1쪽과 3쪽

원 PR은 2020 시험지의 editable 가져오기에서 1쪽 첫 줄 들여쓰기와 3쪽 이어지는 문단 머리를 직접 판독했다. 기여자의 고정 커밋과 Git blob이 같은 [1쪽 Native overlay](../assets/issue_8220_editable/png/2020_native_overlay_p001.png), [1쪽 WASM overlay](../assets/issue_8220_editable/png/2020_wasm_overlay_p001.png), [3쪽 Native review](../assets/issue_8220_editable/png/2020_native_review_p003.png), [3쪽 WASM review](../assets/issue_8220_editable/png/2020_wasm_review_p003.png)를 연결한다.

같은 2020 PDF의 별도 2026-10-27 통합 후보 측정은 1쪽 Native 99.69910% / WASM 98.72790%, 3쪽 99.44626% / 98.88209%다. 이는 원 PR 이미지의 실행 수치나 후속 체리픽 이후의 새 측정이 아니다. [원본 PDF·이미지와 TSV 해시](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8227-source-2020-pages.json).
