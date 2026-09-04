## 개요

HWP3 글자 모양의 `text_color`를 rhwp 내부 `CharShape.text_color`로 보존하도록 수정합니다.

Closes #1692

## 변경 내용

- HWP3 기본 8색 인덱스 `0~7`을 `ColorRef(0x00BBGGRR)`로 변환
- `convert_char_shape`에서 `Hwp3CharShape.text_color`를 누락하지 않도록 보완
- SO-SUEOP HWP3/HWPX/PDF 샘플을 추가하고 HWP3 글색 보존 회귀 테스트 추가

## 시각 검증

- PDF 기준: `pdf/SO-SUEOP-2024.pdf` 22쪽
- HWP3 수정본: `samples/SO-SUEOP.hwp` export-svg `-p 20`
- HWPX 기준본: `samples/SO-SUEOP.hwpx` export-svg `-p 22`
- 결과: PDF/HWP3/HWPX 모두 파란 주석/해설 글색 확인

## 검증

- `cargo fmt --check`
- `git diff --check`
- `env CARGO_INCREMENTAL=0 cargo test task1692 --lib`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1692`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
