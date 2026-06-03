# task_m100_1116 Stage 20 - HWP3-origin 영문 폰트 치환 보정

## 작업 시각

- 2026-05-25 23:44 KST

## 사용자 지시

- 파일별로 한컴오피스와 영문 폰트가 미세하게 다름.
- 한컴오피스 3mm 격자 정답지를 참고.
- 소스 수정 승인 수신 후 진행.

## 분석 결과

대상 파일:

- `samples/hwp3-sample16-hwp5-2010.hwp`
- `samples/hwp3-sample16-hwp5-2018.hwp`
- `samples/hwp3-sample16-hwp5-2022.hwp`
- `samples/hwp3-sample16-hwp5-2024.hwp`
- `samples/hwp3-sample16.hwp`

확인 내용:

- p3 본문/글상자 Latin 글자모양의 영어 슬롯은 모두 `HCI Poppy` 계열.
- `2010/2018/2024` 저장본은 `HCI Poppy`가 `alt_type=2`로 들어와 기존 HFT 치환 경로에서 `Palatino Linotype`으로 변환됨.
- `2022` 저장본은 `HCI Poppy`가 `alt_type=1`, HWP3 원본은 `alt_type=0`으로 들어와 기존 HFT 치환 경로를 타지 않음.
- 그 결과 SVG에 `font-family="HCI Poppy,...`가 그대로 출력되고, 실제 브라우저에서는 미설치 폰트 fallback으로 산세리프 계열이 사용되어 한컴오피스 정답지와 영문 폭/모양이 달라짐.

## 수정 내용

### `src/renderer/style_resolver.rs`

- `resolve_legacy_latin_font()` 추가.
- 영어 슬롯(`lang_index == 1`)의 legacy HCI/HWP3 영문 폰트는 `alt_type`이 `0/1/2` 어느 값이어도 한컴 HFT 치환과 같은 결과를 우선 적용.
- 핵심 보정:
  - `HCI Poppy` -> `Palatino Linotype`
  - `HCI Hollyhock` 계열 -> `HY중고딕`
  - `HCI Columbine`/OCR 계열 -> `Calibri`
  - 일부 장식 영문 face -> 기존 HFT 치환과 동일한 HY 계열

### `tests/issue_1116.rs`

- `sample16_hwp5_2022_page3_latin_font_matches_legacy_hancom_mapping` 추가.
- `sample16_hwp3_page3_latin_font_matches_legacy_hancom_mapping` 추가.
- 2022 저장본과 HWP3 원본 p3의 Latin `C` SVG가 `Palatino Linotype`으로 출력되고, `HCI Poppy` unresolved family가 남지 않는지 검증.

## 산출물

- 재생성 위치: `output/poc/render-spacing/stage20-font-diff-after/`
- 확인 결과:
  - `hwp3-sample16`: `C Palatino=6 HCI=0`
  - `hwp3-sample16-hwp5-2010`: `C Palatino=6 HCI=0`
  - `hwp3-sample16-hwp5-2018`: `C Palatino=6 HCI=0`
  - `hwp3-sample16-hwp5-2022`: `C Palatino=6 HCI=0`
  - `hwp3-sample16-hwp5-2024`: `C Palatino=6 HCI=0`

## 검증

- `cargo test --test issue_1116 -- --nocapture` 통과.
- `cargo build --bin rhwp` 통과.
- `target/debug/rhwp export-svg ... -p 2 --show-grid=3mm` 5개 파일 재생성 및 SVG font-family 확인.
- `cargo test --test issue_1105 -- --nocapture` 통과.
- `cargo test --test issue_1086 -- --nocapture` 통과.
- `cargo test --test issue_1035_alignment -- --nocapture` 통과.
- `cargo fmt --all -- --check` 통과.
- `git diff --check` 통과.

## 남은 작업

- 작업지시자 확인 후 커밋 및 PR 준비 단계 진행.
- PR 생성은 별도 승인 후에만 진행.
