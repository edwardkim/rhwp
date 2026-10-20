# 2026-10-20 사용자 지정 잔여 피델리티 이관

대표 이슈: #7445. 현재 검토 브랜치의 남은 시각 미달을 이관하고 PR을 준비하라는 사용자 지시에 따른 기록이다. 이관은 출력 결함 해결이나 통과 판정이 아니다.

관측 후보 D61 `a30dacfd037122ceacc65439ca03a1afc2bfe69c`, 고정 runtime base `efc52afb738bd408ac821c4e156101b8d26e5cbd`, 문서 전용 최신 base `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`를 구분한다. 원본/Print PDF별 전체 TSV를 바이트 그대로 보존했다. D66 자체 부분 행 실험 회귀 철회와 실제 face 요청 이름 보정 이후 최종 검증은 별도 기록한다.

## 입력별 추적

| 검증 조건 | 기존 추적 | 상세 |
| --- | --- | --- |
| added-issue7677-float_push_spacing_before_continuation-2024-hwp | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7677-float_push_spacing_before_continuation-2024-hwp/README.md) |
| added-issue7677-float_push_spacing_before_continuation-hwpx | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7677-float_push_spacing_before_continuation-hwpx/README.md) |
| added-issue7678-float_push_spacing_before_continuation-2024-hwp | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7678-float_push_spacing_before_continuation-2024-hwp/README.md) |
| added-issue7678-float_push_spacing_before_continuation-hwpx | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7678-float_push_spacing_before_continuation-hwpx/README.md) |
| added-issue7679-tac-after-text-hwpx | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7679-tac-after-text-hwpx/README.md) |
| added-issue7695-footnote-page-tail-hwpx | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7695-footnote-page-tail-hwpx/README.md) |
| added-issue7874-omitted-spacing-after-hwpx | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](added-issue7874-omitted-spacing-after-hwpx/README.md) |
| added-issue8060-71352_lpg_regulatory_impact-hwp | #8060 | [원본·PDF·base/후보·TSV](added-issue8060-71352_lpg_regulatory_impact-hwp/README.md) |
| counter-22081993 | 신규 묶음 하위 이슈 | [원본·PDF·base/후보·TSV](counter-22081993/README.md) |
| stored-empty-paragraph-56345 | #7806 | [원본·PDF·base/후보·TSV](stored-empty-paragraph-56345/README.md) |
| rowbreak-1480000-control | #7718 | [원본·PDF·base/후보·TSV](rowbreak-1480000-control/README.md) |
| rowbreak-1613000-control | #7727 | [원본·PDF·base/후보·TSV](rowbreak-1613000-control/README.md) |
| rowbreak-70833-control | #7728 | [원본·PDF·base/후보·TSV](rowbreak-70833-control/README.md) |
| footnote1937-control | #7732 | [원본·PDF·base/후보·TSV](footnote1937-control/README.md) |

## 활성 검사 분리와 보존

D66에서 기존 경계 33건·7677/7678의 5건·7644의 1건·8060의 2건은 모두 통과해 정상 독립 계약으로 유지했다. 7874의 `paragraphs_keep_spacing_after_omitted_by_stored_ladder` 1건은 실제 21.3px/독립 기대 28.9px로 실패해 활성 suite에서 분리했다. 아래 `.rs.txt`는 제거 전 코드 또는 정상 검사 보존 비교본이다. manifest는 canonical --prepare로 동기화하며 ignore/조용한 return을 추가하지 않았다.

- `tests/cases/issue_7677_rowbreak_host_line_after_last_fragment.rs` → [원문](tests/issue_7677_rowbreak_host_line_after_last_fragment.rs.txt).
- `tests/cases/issue_7678_float_host_post_text_line_split.rs` → [원문](tests/issue_7678_float_host_post_text_line_split.rs.txt).
- `tests/cases/issue_7874_stored_ladder_omitted_spacing_after.rs` → [원문](tests/issue_7874_stored_ladder_omitted_spacing_after.rs.txt).
- `tests/cases/issue_7644_stored_block_empty_paragraph.rs` → [원문](tests/issue_7644_stored_block_empty_paragraph.rs.txt).

전쪽 미달과 별개로 독립 근거가 있는 정상 경계 검사, 공유 parser/serializer·보안 계약, 71352의 정상 9·42쪽 관계 검사는 유지한다. 전체 피델리티 비교 대상에서 제외하는 문서와 정상 구조 계약에서 계속 읽는 공유 원본을 구분한다. 이관 문서를 PR의 개선 완료/종료 이슈에 포함하지 않는다.
