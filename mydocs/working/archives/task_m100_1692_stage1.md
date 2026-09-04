# Task #1692 Stage 1 - HWP3 글자색 보존

## 변경 내용

- HWP3 글자 모양의 1바이트 글자색 인덱스를 rhwp 내부 `ColorRef`로 변환한다.
- `convert_char_shape`가 `text_color`를 누락하지 않도록 보완한다.
- SO-SUEOP HWP3 원본과 HWPX 변환 기준 샘플을 함께 회귀 테스트에 사용한다.

## 샘플 기준

- `samples/SO-SUEOP.hwp`: 원본 HWP3, 현재 rhwp 48쪽
- `samples/SO-SUEOP.hwpx`: HWPX 변환 기준, 현재 rhwp 48쪽
- `pdf/SO-SUEOP-2024.pdf`: 한컴 PDF 기준, 46쪽

## 확인 결과

- 수정 전 `samples/SO-SUEOP.hwp` 21쪽 SVG는 `fill="#000000"` 위주로 출력되어 파란 글색이 손실됐다.
- `samples/SO-SUEOP.hwpx` 기준 샘플에도 파란 글색 `fill="#0000ff"`가 존재한다.
- HWP3 raw 스타일에서 `text_color=1`이 HWPX `#0000FF`에 대응함을 확인했다.
- 수정 후 `samples/SO-SUEOP.hwp` 21쪽 SVG에 `fill="#0000ff"` 425개가 출력된다.
- 비교 기준 `samples/SO-SUEOP.hwpx` 같은 본문 페이지는 23쪽이며, SVG에 `fill="#0000ff"` 156개가 출력된다.

## 시각 판정 자료

- PDF 기준: `pdf/SO-SUEOP-2024.pdf` 22쪽
- HWP3 수정본: `samples/SO-SUEOP.hwp` export-svg `-p 20` (`SO-SUEOP_021.svg`)
- HWPX 기준본: `samples/SO-SUEOP.hwpx` export-svg `-p 22` (`SO-SUEOP_023.svg`)
- PNG 판정:
  - PDF 22쪽: 파란 필기/주석과 하단 해설 글색 확인
  - HWP3 수정본 21쪽: 같은 영역의 파란 글색 복구 확인
  - HWPX 기준본 23쪽: 같은 본문 페이지에서 파란 글색 확인
- 픽셀 기준:
  - PDF 22쪽: blue-ish 픽셀 5012
  - HWP3 수정본 21쪽: blue-ish 픽셀 2592
  - HWPX 기준본 23쪽: blue-ish 픽셀 1237
- 판정: #1692 범위인 글색상 손실은 복구됨. 도형/표 배치, 페이지 overflow, HWP3/HWPX 페이지 밀림은 #1692 범위 밖 잔여 이슈다.

## 검증

- `cargo fmt --check` 통과
- `git diff --check` 통과
- `env CARGO_INCREMENTAL=0 cargo test task1692 --lib` 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1692` 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과

## 정리

- 임시 렌더 산출물은 커밋 대상에서 제외되도록 정리했다.
