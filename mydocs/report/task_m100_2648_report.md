# task_m100_2648 처리결과 보고서 — 머리말/꼬리말 LIST_HEADER 페이로드 파싱

- **이슈**: [#2648](https://github.com/edwardkim/rhwp/issues/2648)
- **브랜치**: `task/m100-2648-headerfooter-listheader` (base `devel` @ `3c54abfd`)
- **범위**: `src/parser/control.rs`
- **분류**: 결함 수정 (파서 누락)

## 1. 문제

`find_list_header_paragraphs`(파서)가 `HWPTAG_LIST_HEADER` 자식 레코드를 찾아 **그 뒤의
문단들만** 파싱하고, 레코드 자체의 페이로드(`list_attr`/`text_width`/`text_height`/
`text_ref`/`num_ref`)는 전혀 읽지 않았다. `parse_header_control`/`parse_footer_control`
이 이 함수만 호출하므로 `Header`/`Footer` 의 다섯 필드는 항상 `Default`(0)였다.

직렬화(`build_header_footer_list_header`, `serializer/control.rs:2340`)는 이 필드들을
무조건 사용해 `HWPTAG_LIST_HEADER` 레코드를 재생성하므로, 실제 파일이 담고 있던 값은
파싱→직렬화 왕복마다 0 으로 뭉개졌다.

같은 파일의 캡션 파서(`:427-431`)는 이미 이 레코드 페이로드를 읽는데, 머리말/꼬리말만
빠진 비대칭이었다.

## 2. 변경

`find_list_header_paragraphs` 옆에 `find_list_header_layout_and_paragraphs` 를 신설 —
직렬화 함수(`build_header_footer_list_header`)의 바이트 레이아웃을 역으로 읽어 레이아웃
필드와 문단을 함께 반환한다:
```
u16 para_count | u32 list_attr | u16(예약) | u32 text_width | u32 text_height
| u8 text_ref | u8 num_ref | u16 ext_flags | [u8;14] 예약
```
`parse_header_control`/`parse_footer_control` 양쪽에서 새 함수로 교체하고 다섯 필드를
채웠다. 기존 `find_list_header_paragraphs` 는 각주/미주/숨은설명 파서가 여전히 쓰므로
그대로 유지(불필요한 범위 확장 방지).

## 3. 검증

### 신규 테스트

`test_parse_header_control_reads_list_header_layout_fields` — 비영 `list_attr`
(`0x00020000`), `text_width=5000`, `text_height=3000`, `text_ref=7`, `num_ref=9` 를
담은 LIST_HEADER 레코드를 파싱해 다섯 값이 그대로 읽힘을 단언.

### red→green 실증

필드 대입 5줄을 제거(`let (_layout, paragraphs) = ...; header.paragraphs = paragraphs;`)
→ **FAILED**(`assertion left==right failed: list_attr 이 보존돼야 함`). 복원 → **통과**.

```
FAILED (수정 제거): 0 passed; 1 failed
GREEN  (수정 복원): parser::control 18 passed; 0 failed
```

### 회귀

```
cargo test --lib parser::      →  390 passed / 0 failed
cargo test --lib serializer::  →  403 passed / 0 failed
```

### 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소
  규약상 작업지시자 별도 승인 사항이라 실행하지 않았다.
- **parse→serialize→parse 완전 왕복 테스트**는 추가하지 않았다. 새 파서 함수가 직렬화
  함수(`build_header_footer_list_header`)의 정확한 역함수임을 바이트 단위로 대조
  확인했고(오프셋·길이 일치), 단위 테스트로 각 필드의 파싱을 직접 검증했다. 완전 왕복
  테스트는 기존 저장소에 header/footer 전용 parse↔serialize 하네스가 마련돼 있지 않아
  범위를 넘는다고 판단했다.
