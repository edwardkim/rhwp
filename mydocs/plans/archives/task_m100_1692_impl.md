# Task #1692 구현 메모

## 구현 방향

1. `src/parser/hwp3/mod.rs`에 HWP3 기본 8색 인덱스 → `ColorRef` 변환 함수를 추가한다.
2. `convert_char_shape`에서 `hwp3_cs.text_color`를 내부 `CharShape.text_color`로 변환해 대입한다.
3. 단위 테스트로 색상 인덱스 매핑과 `convert_char_shape` 보존을 고정한다.
4. `tests/issue_1692.rs`에서 `SO-SUEOP.hwp`와 `SO-SUEOP.hwpx`를 함께 파싱해 파란 글자색 보존을 확인한다.

## 근거

- `mydocs/tech/한글문서파일구조3.0.md` 표 16은 글자 모양의 `글자색` 필드를 `0~7 (검정~흰색)`으로 정의한다.
- `samples/SO-SUEOP.hwp` raw 스타일에서 `주해(궁서9)`, `정답` 스타일의 HWP3 `text_color`는 `1`이다.
- `samples/SO-SUEOP.hwpx`의 같은 스타일은 `textColor="#0000FF"`로 변환되어 있다.

## 검증 예정

- `cargo fmt --check`
- `git diff --check`
- `cargo test task1692 --lib`
- `cargo test --test issue_1692`
- 필요 시 `cargo clippy --all-targets -- -D warnings`
