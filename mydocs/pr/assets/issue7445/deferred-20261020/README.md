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

## 이관 전 검사 코드와 복원 원장

#7834와 같은 방식으로 전체 검사 모듈을 `.rs.txt`에 보존했다. [복원 원장](transfer-inventory.json)에
이관 전 source commit·Git blob·이전/보존 경로·바이트 수·SHA-256·모든 검사 함수와 현재 활성
여부를 기록했다. 기존 4개 사본도 이관 직전 원본과 바이트 단위로 대조했다. 아래 사본은
자동 수집 대상이 아니며, 정상 검사 보존 사본을 새로운 실행 suite로 추가하지 않는다.

| 검사 모듈 | 처리 | 보존 원문 |
| --- | --- | --- |
| `issue_7644_stored_block_empty_paragraph.rs` | 정상 검사 유지; 이관 전 전체 원문 보존 | [전체 파일](tests/issue_7644_stored_block_empty_paragraph.rs.txt), 1건 |
| `issue_7677_rowbreak_host_line_after_last_fragment.rs` | 정상 검사 유지; 이관 전 전체 원문 보존 | [전체 파일](tests/issue_7677_rowbreak_host_line_after_last_fragment.rs.txt), 3건 |
| `issue_7678_float_host_post_text_line_split.rs` | 정상 검사 유지; 이관 전 전체 원문 보존 | [전체 파일](tests/issue_7678_float_host_post_text_line_split.rs.txt), 2건 |
| `issue_7679_tac_table_after_host_line.rs` | 정상 HWP 검사 유지; HWPX 전체 피델리티와 구분 | [전체 파일](tests/issue_7679_tac_table_after_host_line.rs.txt), 1건 |
| `issue_7695_footnote_page_tail_saved_lines.rs` | 정상 HWP 검사 유지; HWPX 전체 피델리티와 구분 | [전체 파일](tests/issue_7695_footnote_page_tail_saved_lines.rs.txt), 1건 |
| `issue_7874_stored_ladder_omitted_spacing_after.rs` | 활성 검사 1건 제거; 원문 보존 | [전체 파일](tests/issue_7874_stored_ladder_omitted_spacing_after.rs.txt), 1건 |
| `issue_8060_joint_stored_row_restart.rs` | 정상 검사 유지; 이관 전 전체 원문 보존 | [전체 파일](tests/issue_8060_joint_stored_row_restart.rs.txt), 2건 |

- 원본 source는 공개 이관 커밋 `65ef09ad8`의 직전 커밋
  `91ce04bdf0ac285d5e53a88f15a3e690baca2575`에서 추출했다. 사본을 현재 구현에 맞춰
  편집하거나 assertion을 바꾸지 않았다. 제거한 함수는 7874의 1건뿐이며 나머지는 정상 계약
  보존 사본이다. 8060의 두 활성 검사도 그대로 유지했다.
- 7679·7695의 사본은 정상 HWP의 검사 원문이며 이관된 HWPX가 같은 시각 품질이라는
  주장이 아니다. `counter-22081993`는 당시 전쪽 시각 대조군으로, 이 이관에서 제거한 전용
  Rust 함수가 없으므로 존재하지 않는 회귀 코드를 만들어 기록하지 않았다.
- 복원 때 전쪽 Native/fresh WASM raw 90% 이상·쪽수/내용/소유·확대 판독을 먼저 확인한다.
  그 뒤 원장의 `source_path`로 해당 전체 모듈을 복사하고 위 README들의 원본 이동 대응만
  연결한다. 정확한 독립 기대값을 유지하고 `rust-test-suite-manifest.mjs --prepare` 및
  `--check --base-ref <검증 base>`와 해당 집중 회귀를 실행한다. 원문 사본 자체를 통과시키거나
  활성 코드와 중복 등록하지 않는다.
