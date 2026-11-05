# PR #8448 리뷰 — fix(typeset): 저장 리셋으로 다음 쪽에 가는 문단 중간 줄의 자리차지 로고를 떠나는 쪽에 남긴다 (#8446)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8448](https://github.com/edwardkim/rhwp/pull/8448) · `planet6897` → `devel` |
| 원 head | `e9ac32cb443829a043c76faad3cc68f90a6a98ec` |
| 통합 적용 커밋 | `08149ad8d40d1ef87be74ab9412c6062b4a39262, a6a3ce454b3a3550f095a2a44795ac3a7f1fdc6d` |
| 최종 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 최종 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8446](https://github.com/edwardkim/rhwp/issues/8446) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(typeset): 저장 리셋으로 다음 쪽에 가는 문단 중간 줄의 자리차지 로고를 떠나는 쪽에 남긴다 (#8446)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [156569782-reset-line-logo-empty.hwp](../../samples/issue8446/156569782-reset-line-logo-empty.hwp) · [156569782-reset-line-logo-empty-2022.pdf](../../pdf/issue8446/156569782-reset-line-logo-empty-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8448/pr8448-156569782-reset-line-logo-empty-db022b31-native.tsv) · 99.87% (2쪽) | [TSV](assets/planet6897_20261103/pr_8448/pr8448-156569782-reset-line-logo-empty-db022b31-wasm.tsv) · 99.91% (2쪽) |
| [156667426-reset-line-logo.hwp](../../samples/issue8446/156667426-reset-line-logo.hwp) · [156667426-reset-line-logo-2022.pdf](../../pdf/issue8446/156667426-reset-line-logo-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8448/pr8448-156667426-reset-line-logo-6c93c2e3-native.tsv) · 96.75% (3쪽) | [TSV](assets/planet6897_20261103/pr_8448/pr8448-156667426-reset-line-logo-6c93c2e3-wasm.tsv) · 96.77% (3쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

저장 줄 사다리가 문단 중간에서 다음 쪽으로 되감겨도 앞 줄이 소유한 자리차지 로고는 떠나는 쪽에 남는다.

typeset/controls/shape_flow.rs의 anchor_line_rewinds_to_next_page와 controls.rs의 개체 배치가 같은 저장 줄 경계를 소비한다. 개체 원점과 후속 글줄의 쪽 소속을 함께 대조한다.

빈 앵커와 실제 글자가 있는 앵커의 실물 입력을 구분한다. 로고를 다음 쪽으로 옮겨 본문을 겹치게 만드는 반례와 다음 쪽으로 가야 하는 글줄의 보존을 검사한다.

관련 정식 회귀 파일: [issue_8446_reset_line_float_spill.rs](../../tests/cases/issue_8446_reset_line_float_spill.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `156569782-reset-line-logo-empty.hwp` 1쪽 · 저장 리셋 로고 | 99.87% · [review](assets/planet6897_20261103/pr_8448/pr8448-claim-p001-156569782-reset-li-native-review.png) / [overlay](assets/planet6897_20261103/pr_8448/pr8448-claim-p001-156569782-reset-li-native-overlay.png) | 99.91% · [review](assets/planet6897_20261103/pr_8448/pr8448-claim-p001-156569782-reset-li-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8448/pr8448-claim-p001-156569782-reset-li-wasm-overlay.png) |
| `156667426-reset-line-logo.hwp` 2쪽 · 리셋 로고 문서의 전쪽 최저 페이지 | 96.75% · [review](assets/planet6897_20261103/pr_8448/pr8448-claim-p002-156667426-reset-li-native-review.png) / [overlay](assets/planet6897_20261103/pr_8448/pr8448-claim-p002-156667426-reset-li-native-overlay.png) | 96.77% · [review](assets/planet6897_20261103/pr_8448/pr8448-claim-p002-156667426-reset-li-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8448/pr8448-claim-p002-156667426-reset-li-wasm-overlay.png) |

## 현재 대표 페이지 직접 판독

[판독 기록·실제 PNG SHA-256](assets/planet6897_20261103/direct-reading-current.md)에 같은 원본·쪽의 관측을 남겼습니다.

- native 1쪽: 리셋된 빈 줄을 포함한 보도자료에서 제목·본문·각주 뒤 담당 부서 표와 로고가 같은 쪽에 순서대로 남는다. 첫 줄·빈 줄이 뒤 로고 원점을 이중 전진시키지 않는지 대조했다.
- wasm 1쪽: 리셋된 빈 줄을 포함한 보도자료에서 제목·본문·각주 뒤 담당 부서 표와 로고가 같은 쪽에 순서대로 남는다. 첫 줄·빈 줄이 뒤 로고 원점을 이중 전진시키지 않는지 대조했다.

## 통합 최종 검증 조건

- 제품 `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973`의 전체 Rust 12,561개 통과와 Native Skia·CI 통합 모듈 21개·Rust lint를 확인했습니다.
- 최적화 fresh WASM의 실제 브라우저 실행과 공개 API 검사를 완료했습니다.
- 전쪽 측정 producer에서 Native/fresh WASM 각 80개 입력·406쪽은 모두 90% 이상입니다.
- Native의 당시 base 대비 406쪽 대조와 PR #8389·#8397의 원 TSV 표시값 무하락 대조는 완료했습니다.
- WASM의 당시 base 대비 전쪽 대조와 최신 base 기능 보존의 남은 실행은 아래와 같습니다.

- 대기: 동일 환경에서 당시 base 대비 WASM 80개 입력·406쪽의 0.1%p 하락 게이트 완료
- 새 base의 실제 Native/WASM 제품과 후보의 집중 대조 14쪽은 모두 같은 raw입니다. [실행·대조 원본](assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)을 확인했습니다.
- 대기: 주장 쪽 직접 판독·회귀 주석·수동 작성 리뷰와 PR 본문 최종 감사

이 대기 항목을 완료하지 않은 상태에서는 최종 검증 완료·승인으로 판정하지 않습니다.
[완료·대기 검증 원본 기록](assets/planet6897_20261103/intermediate-evidence-summary.md)

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->
