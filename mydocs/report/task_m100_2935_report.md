---
kind: report
status: done
task: m100-2935
issue: 2935
---

# task-m100-2935: hp:parameters의 stringParam(Command) CDATA 소실 수정 보고서

## 이슈

edwardkim/rhwp#2935 — `hp:parameters` 하위 `hp:stringParam name="Command"`가 CDATA로
인코딩된 경우 파서가 필드 명령 문자열(하이퍼링크 대상, 필드 명령 등)을 소실하는 문제.

이미 병합된 #2916 / PR #2927(`hp:equation`의 `hp:script` CDATA 누락)과 동일한 근본 원인
패턴이다.

## 근본 원인

`src/parser/hwpx/section.rs`의 `parse_field_parameters()`는 `hp:parameters`의 자식
요소 텍스트를 읽어 `Field::command`를 채우는 이벤트 루프에서 `Event::Text`와
`Event::GeneralRef`만 처리하고 `Event::CData`를 처리하지 않았다. quick-xml은
`<![CDATA[...]]>` 블록을 `Event::Text`가 아니라 별도의 `Event::CData` 이벤트로
방출하므로, 이 분기가 없으면 CDATA로 감싸진 콘텐츠(예: 쿼리스트링에 `&`가 포함된
하이퍼링크 URL, 비교 연산자 `<`/`>`를 포함한 필드 명령)가 조용히 사라진다.

## 수정

`Event::GeneralRef` 분기 뒤에 `Event::CData` 분기를 추가하여, CDATA 콘텐츠를
UTF-8로 디코드한 뒤 verbatim 재조립 버퍼(`raw`)에 이스케이프하여 추가하고,
`in_command`일 때는 기존 `Event::Text` 분기와 동일하게 `field.command`에도
반영하도록 했다. `header.rs::read_numbering_para_head_text()`와 #2927에서 수정된
`hp:script` 파서의 기존 패턴을 그대로 따랐다.

diff 규모: `src/parser/hwpx/section.rs` 순수 수정부 약 9줄 + 회귀 테스트 1개
(총 +35줄, 삭제 없음).

## 검증 (Red → Green)

1. **Red**: `Event::CData` 분기를 임시로 제거한 상태에서
   `test_parse_field_parameters_preserves_cdata_command`를 실행 →
   `field.command`가 빈 문자열(`""`)로 나와 실패 확인.
   ```
   left: ""
   right: "HYPERLINK \"https://example.com/?a=1&b=2\""
   ```
2. **Green**: `Event::CData` 분기를 복원한 뒤 동일 테스트 재실행 → 통과.
   ```
   test parser::hwpx::section::tests::test_parse_field_parameters_preserves_cdata_command ... ok
   ```
3. 기존 인접 테스트(`test_parse_memo_field_parameters_preserves_number_as_memo_index`,
   `parse_field_parameters_reassembles_nested_params_balanced`)도 회귀 없이 통과.
4. `cargo check --lib` 통과.

## 참고

- 작업 중 `C:` 드라이브 여유 공간이 0이 되어 Edit 도구가 `ENOSPC`로 실패하는 일이
  있었다. `rhwp-wt-s` 워크트리에서 `cargo clean`으로 target 디렉터리(약 7.4GiB)를
  정리해 여유 공간을 11GB로 확보한 뒤 작업을 재개했다. 이 워크트리에 국한된 조치이며
  다른 워크트리나 공용 자원은 건드리지 않았다.
- 브랜치 전환 시 동일 워크트리에 남아 있던 `src/parser/hwp3/*` 8개 파일의 미커밋
  변경(본 작업과 무관, 사용자가 만든 것으로 추정)은 건드리지 않고 그대로 두었다.
  `git stash`는 지시에 따라 사용하지 않았고, 대신 본 작업분만 `git diff` 패치로
  분리 저장한 뒤 `origin/devel` 기준 새 브랜치에 `git apply`로 재적용했다.
