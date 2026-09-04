# planet6897 PR 일괄 검토 Stage 1 - 메인터너 보정: 중첩 lineseg와 OLE 0 ID 보존

## 목적

`planet6897`의 비드래프트 PR #4928, #4932, #4933, #4938, #4940을 최신
`upstream/devel` 위 검토 브랜치에 누적한 뒤 발견한 두 라운드트립 경계를 보정한다.

## 보정 내용

1. #4940의 구역 저장 lineseg 권위 판정은 본문뿐 아니라 표 셀과 중첩 표 셀의 문단까지
   재귀한다. 셀 재조판도 같은 `section_sized` 판정을 사용해, 0 높이로 저장된 숨은 셀
   블록을 다시 펼치지 않는다.
2. #4932의 HWPX `hp:ole@id`는 OWPML 스키마에서 `xs:nonNegativeInteger`이므로 `0`도
   유효하다. `Option<u32>`으로 속성 부재와 명시적 `0`을 구분해, 저장 시 원래 `id=0`을
   유지한다.

## 회귀 근거

- `parser::hwpx::section::tests::issue4669_explicit_zero_id_is_not_rewritten_to_instid`
  - `id="0"`과 별도 `instid`를 함께 파싱해 두 값이 분리 보존되는지 확인한다.
- `document_core::commands::document::tests::issue4898_section_authority_includes_nested_table_cells`
  - 중첩 표 셀의 양수 lineseg가 section 권위에 포함되고, 같은 구역의 0 높이 lineseg는
    재조판 대상에서 제외되는지 확인한다.

## 범위

- 변경: `src/document_core/commands/document.rs`, `src/parser/hwpx/section.rs`
- 문서: 이 stage 기록 한 건
- 외부 PR 원격 push와 GitHub 리뷰 제출은 보정 검증 뒤 별도 승인으로 진행한다.
