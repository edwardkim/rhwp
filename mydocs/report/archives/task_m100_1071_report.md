# Task M100-1071 최종 보고서

## 1. 작업 개요

- GitHub Issue: #1071
- 제목: `[layout] HWPX TAC 도형 paragraph layout 가로 위치 + 한글자처럼 캐럿 이동`
- 작업 브랜치: `local/task_m100_1071`

## 2. 문제

`samples/hwpx/shape-001.hwpx` 에서 다음 문제가 있었다.

```text
1. HWPX TAC 다각형 두 개가 HWP 정답지보다 서로 붙어 보임
2. HWP/HWPX 모두 TAC 다각형을 편집 커서가 한 글자처럼 통과하지 못함
3. TAC 다각형 사이 연속 space 구간에서 커서 이동 간격이 불균등하게 느껴짐
```

## 3. 원인

첫 번째 문제는 HWPX `hp:secPr` 처리와 관련된 control slot 불일치였다.

```text
char_offsets stream:
  [secPr][colPr][shape1][" "][" "][shape2]

기존 HWPX para.controls:
  [colPr][shape1][shape2]
```

`hp:secPr` marker는 `char_offsets` 에 반영되지만 `para.controls` 에
`SectionDef` 로 materialize 되지 않아 TAC 도형 control position 이 한 칸씩 밀렸다.

두 번째 문제는 커서 offset 이동이 아니라 cursor rect 좌표 계산 문제였다.
렌더 트리의 TextRun에는 TAC 도형 자리표시자 `U+FFFC` 가 섞여 있었고,
기존 cursor rect 계산은 이 자리표시자를 실제 편집 문자처럼 따라갔다.

## 4. 구현

주요 변경:

```text
1. HWPX hp:secPr 를 Control::SectionDef 로도 materialize
2. raw control text position 과 편집용 logical control position 분리
3. SectionDef/ColumnDef 는 커서 이동 단위에서 제외
4. Shape/Table/Picture/Equation/Footnote/Endnote 는 logical inline unit 으로 취급
5. cursor rect 계산에서 TAC 도형 bbox 와 실제 para.text 문자 위치를 합쳐 caret stop 구성
6. TAC 도형 사이 연속 space 구간은 앞 도형 오른쪽과 뒤 도형 왼쪽 사이를 균등 분배
```

수정 파일:

```text
src/parser/hwpx/section.rs
src/document_core/helpers.rs
src/document_core/queries/doc_tree_nav.rs
src/document_core/queries/cursor_rect.rs
src/document_core/converters/hwpx_to_hwp.rs
src/renderer/layout/paragraph_layout.rs
```

테스트:

```text
tests/issue_1067_shape_rotation.rs
tests/issue_1071_tac_cursor_nav.rs
tests/hwpx_to_hwp_adapter.rs
```

## 5. 검증

실행한 검증:

```text
cargo fmt --check
cargo check
cargo test --test issue_1071_tac_cursor_nav
cargo test --test issue_1067_shape_rotation
cargo test --test issue_598_footnote_marker_nav
cargo test --lib logical_positions_ignore_section_and_column_controls
cargo test --test hwpx_to_hwp_adapter
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
issue_1071_tac_cursor_nav: 4 passed
issue_1067_shape_rotation: 7 passed
issue_598_footnote_marker_nav: 4 passed
logical_positions_ignore_section_and_column_controls: passed
hwpx_to_hwp_adapter: 49 passed, 0 failed, 11 ignored
WASM build: success
```

WASM 산출물은 `pkg/` 와 `rhwp-studio/public/` 에 동기화했다.

## 6. 작업지시자 판정

작업지시자 판정:

```text
동작 테스트 통과
```

확인된 항목:

```text
1. shape-001.hwpx 의 다각형과 다각형 사이 공백 출력 성공
2. HWP/HWPX 모두 TAC 다각형을 한 글자처럼 통과하는 커서 이동 성공
3. 연속 space 구간의 커서 이동 UX 개선 통과
```

## 7. 결론

#1071은 완료 처리한다.

이번 정정으로 HWPX `shape-001.hwpx` 는 HWP 정답지와 같은 TAC control stream 기반의
가로 배치를 가지며, 편집 커서도 TAC 다각형과 연속 space 구간을 자연스럽게 통과한다.
