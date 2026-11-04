# PR #8444 리뷰 — fix(layout): TAC 표 문단 뒤 줄의 TAC 개체를 줄 높이로 쪽 끝에 수용하고 그 줄 그림을 저장 줄 원점에 둔다 (#8441)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8444](https://github.com/edwardkim/rhwp/pull/8444) · `planet6897` → `devel` |
| 원 head | `14ee038f2194e255f7ca3c14d74f7bfefd480936` |
| 통합 적용 커밋 | `904d2fb128b0ab4a2dff32c12a96bf56f7781d90, e70c0555e915d7d21f5efa3b604980d67fe27ee8` |
| 최종 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 최종 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8441](https://github.com/edwardkim/rhwp/issues/8441) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(layout): TAC 표 문단 뒤 줄의 TAC 개체를 줄 높이로 쪽 끝에 수용하고 그 줄 그림을 저장 줄 원점에 둔다 (#8441)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [156578788-tac-logo-line.hwp](../../samples/issue8441/156578788-tac-logo-line.hwp) · [156578788-tac-logo-line-2022.pdf](../../pdf/issue8441/156578788-tac-logo-line-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156578788-tac-logo-line-359ee733-native.tsv) · 99.57% (2쪽) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156578788-tac-logo-line-359ee733-wasm.tsv) · 99.56% (2쪽) |
| [156669499-tac-logo-line.hwpx](../../samples/issue8441/156669499-tac-logo-line.hwpx) · [156669499-tac-logo-line-2022.pdf](../../pdf/issue8441/156669499-tac-logo-line-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156669499-tac-logo-line-2f73c60e-native.tsv) · 99.96% (1쪽) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156669499-tac-logo-line-2f73c60e-wasm.tsv) · 99.98% (1쪽) |
| [156715172-tac-table-picture.hwpx](../../samples/issue8441/156715172-tac-table-picture.hwpx) · [156715172-tac-table-picture-2022.pdf](../../pdf/issue8441/156715172-tac-table-picture-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156715172-tac-table-picture-2651c3f0-native.tsv) · 99.09% (4쪽) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156715172-tac-table-picture-2651c3f0-wasm.tsv) · 99.1% (4쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

TAC 표 뒤 줄의 그림·도형은 그 줄의 점유 높이로 쪽 끝 수용을 판단하고, 실제 paint는 저장된 소유 줄 원점을 따른다.

layout.rs·layout/tac_picture_item.inc.rs·layout/shape_item.inc.rs의 원점과 typeset/controls/shape_flow.rs의 TableHostShapeFlow 요구 높이를 함께 대조한다. 그림의 높이와 host 줄의 전진을 중복 계상하지 않는다.

세 실물 로고 입력의 표·로고·뒤 문단 순서를 확인한다. 쪽 끝에서 이미 놓인 줄과 다음 쪽으로 이월할 줄을 별도로 검사해 한 경로의 통과를 다른 경로의 증거로 대신하지 않는다.

관련 정식 회귀 파일: [issue_8441_tac_shape_line_page_fit.rs](../../tests/cases/issue_8441_tac_shape_line_page_fit.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `156578788-tac-logo-line.hwp` 1쪽 · TAC 그림 줄 높이 | 99.95% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-native-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-native-overlay.png) | 99.95% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-wasm-overlay.png) |
| `156669499-tac-logo-line.hwpx` 1쪽 · 둘째 로고 입력의 쪽 끝 수용과 표 아래 배치 | 99.96% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156669499-tac-logo-native-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156669499-tac-logo-native-overlay.png) | 99.98% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156669499-tac-logo-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156669499-tac-logo-wasm-overlay.png) |
| `156715172-tac-table-picture.hwpx` 4쪽 · TAC 표 뒤 포스터의 저장 줄 원점과 내어쓰기 | 99.09% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p004-156715172-tac-tabl-native-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p004-156715172-tac-tabl-native-overlay.png) | 99.10% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p004-156715172-tac-tabl-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p004-156715172-tac-tabl-wasm-overlay.png) |

## 현재 대표 페이지 직접 판독

[판독 기록·실제 PNG SHA-256](assets/planet6897_20261103/direct-reading-current.json)에 같은 원본·쪽의 관측을 남겼습니다.

- native 1쪽: 첫 쪽 제목·본문 뒤 담당 부서 표와 로고가 같은 쪽에서 차례로 보존된다. 표 아래 로고와 쪽번호가 독립 Print와 대응하며 본문/그림을 아래 쪽으로 밀거나 누락하지 않는다.
- native 1쪽: 보도정정자료 제목 상자·본문 뒤 담당 부서 표와 두 로고가 같은 쪽에 보존된다. TAC 로고를 다음 쪽으로 밀거나 본문과 겹치는 현상은 보이지 않는다.
- wasm 1쪽: 보도정정자료 제목 상자·본문 뒤 담당 부서 표와 두 로고가 같은 쪽에 보존된다. TAC 로고를 다음 쪽으로 밀거나 본문과 겹치는 현상은 보이지 않는다.

## 통합 최종 검증 조건

- 제품 `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973`의 전체 Rust 12,561개 통과와 Native Skia·CI 통합 모듈 21개·Rust lint를 확인했습니다.
- 최적화 fresh WASM의 실제 브라우저 실행과 공개 API 검사를 완료했습니다.
- 전쪽 측정 producer에서 Native/fresh WASM 각 80개 입력·406쪽은 모두 90% 이상입니다.
- Native의 당시 base 대비 406쪽 대조와 PR #8389·#8397의 원 TSV 표시값 무하락 대조는 완료했습니다.
- WASM의 당시 base 대비 전쪽 대조와 최신 base 기능 보존의 남은 실행은 아래와 같습니다.

- 대기: 동일 환경에서 당시 base 대비 WASM 80개 입력·406쪽의 0.1%p 하락 게이트 완료
- 대기: 새 base 22947f6a90의 실제 WASM 제품과 후보의 기능·시각 집중 대조 완료
- 대기: 주장 쪽 직접 판독·회귀 주석·수동 작성 리뷰와 PR 본문 최종 감사

이 대기 항목을 완료하지 않은 상태에서는 최종 검증 완료·승인으로 판정하지 않습니다.
[완료·대기 검증 원본 기록](assets/planet6897_20261103/review-preparation-a7e5ce7112.json)

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->
