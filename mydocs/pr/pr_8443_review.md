# PR #8443 리뷰 — fix(edit): 편집 문단 뒤 원본 문단의 자리차지 개체 점유를 vpos 재계산에서 보존한다 (#8440)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8443](https://github.com/edwardkim/rhwp/pull/8443) · `planet6897` → `devel` |
| 원 head | `225bf3b5561558efed882d56ec17a0d03709266b` |
| 통합 적용 커밋 | `0bd3f9baa1dabac47d8de2a2cc882dca546a600c, 834ec801abac3ba8748e0afa5575a66fb1b7b84a, b511e84d01a1dcb927bf01ad3ec05e562ff33acd, 44e8fba98b242058643c04a53dcb02b90f0dee48, ed61e50b4bd30b651c45359ef096d4055c524530` |
| 최종 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 최종 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8440](https://github.com/edwardkim/rhwp/issues/8440) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(edit): 편집 문단 뒤 원본 문단의 자리차지 개체 점유를 vpos 재계산에서 보존한다 (#8440)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [pr8443-emc-edit.hwpx](assets/issue_8440/integration_20261103/pr8443-emc-edit.hwpx) · [emc_edit-2024.pdf](../../pdf/issue-8440/emc_edit-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8443/pr8443-pr8443-emc-edit-c0167c88-native.tsv) · 99.61% (14쪽) | [TSV](assets/planet6897_20261103/pr_8443/pr8443-pr8443-emc-edit-c0167c88-wasm.tsv) · 99.78% (14쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

편집 뒤 원본 문단의 문단 기준 자리차지 개체가 점유한 높이를 저장 vpos 재계산에서도 보존한다.

composer/line_breaking.rs의 재조판·구역 vpos 재계산에서 편집한 문단과 후속 원본 문단을 구분한다. 보존한 점유를 뒤 문단의 실제 저장 원점과 저장·재개방 출력이 소비하는지 대조한다.

EMC 실제 편집 결과의 7쪽을 독립 Print와 비교한다. 문단 기준 자리차지 개체를 소유하지 않는 일반 문단까지 점유 보존을 적용하지 않는 대조군을 유지한다. fresh WASM의 deleteText(0,114,0,1)→exportHwpx 결과도 Native 실제 편집 산출물과 바이트 단위로 일치했다. [WASM 실제 API 산출물 대조](assets/planet6897_20261103/wasm-producer-parity.md)를 연결한다.

관련 정식 회귀 파일: [issue_8440_edit_keeps_table_host_gap.rs](../../tests/cases/issue_8440_edit_keeps_table_host_gap.rs)

## 변경 전의 실제 가져오기·편집 결과

[동일 원본의 base/candidate API 산출물 대조](assets/planet6897_20261103/native-producer-before-after.md)와
[실제 WASM API와 Native 산출물 일치](assets/planet6897_20261103/wasm-producer-parity.md)를 함께 확인했습니다.

같은 EMC 원본에서 글자를 삭제하고 저장한 base 산출물의 주장 쪽(7쪽)은 60.61%였습니다.
현재 Native/WASM의 실제 편집 산출물과 독립 Print를 대조합니다. 아래 TSV는 변경 전 7쪽만 포함합니다.

[변경 전 편집 HWPX](assets/planet6897_20261103/before-producer/pr8443-emc-edit-base.hwpx) · [변경 전 선택 쪽 TSV](assets/planet6897_20261103/before-producer/pr8443-emc-edit-base-selected-native.tsv)

[review](assets/planet6897_20261103/before-producer/pr8443-emc-edit-base-p007-review.png) / [overlay](assets/planet6897_20261103/before-producer/pr8443-emc-edit-base-p007-overlay.png)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `pr8443-emc-edit.hwpx` 7쪽 · 앞 문단 글자 편집 뒤 표 점유와 뒤 제목 원점 | 99.92% · [review](assets/planet6897_20261103/pr_8443/pr8443-emc-edit-p007-native-review.png) / [overlay](assets/planet6897_20261103/pr_8443/pr8443-emc-edit-p007-native-overlay.png) | 99.98% · [review](assets/planet6897_20261103/pr_8443/pr8443-emc-edit-p007-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8443/pr8443-emc-edit-p007-wasm-overlay.png) |

## 현재 대표 페이지 직접 판독

[판독 기록·실제 PNG SHA-256](assets/planet6897_20261103/direct-reading-current.md)에 같은 원본·쪽의 관측을 남겼습니다.

- native 7쪽: 실제 EMC 편집 저장본의 전도성 RF·통신 포트·함체 포트 세 표와 사이 본문을 대조했다. 표 프레임·행 순서·표 뒤 간격을 보존하고 누락/겹침은 보이지 않는다. 작은 글자의 획·래스터 경계 차이는 별도로 남는다.

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
