# Stage 124 - HWP5 CommonObj bit 28 보존 구현 계획

## 목적

2025 행정업무운영 편람 HWPX를 rhwp HWP로 저장할 때 `CommonObj.attr` raw 우선 직렬화 경로가 이미 의미 모델에 있는 `hwp5_gen_shape_attr_bit28=true`를 잃지 않도록 보정한다.

## 범위

- 대상: `src/serializer/control.rs`의 HWP5 공통 개체 속성 직렬화.
- 대상 조건: `common.attr != 0`이면서 `common.hwp5_gen_shape_attr_bit28`이 true인 경우.
- 비대상: picture description tail, CharShape sentinel, 표 레이아웃 및 HWP 원본의 raw bit 제거.

## 설계

raw `attr`가 0이면 기존 `pack_common_attr_bits`가 의미 bit 28을 이미 인코드한다. raw `attr`가 0이 아닐 때에도 semantic true만 OR하여 동일한 직렬화 계약을 적용한다. semantic false일 때 raw bit 28은 제거하지 않는다. 따라서 파서가 원본 HWP raw 값을 보존한 모델은 바이트 의미를 유지하고, HWPX에서 생성된 불완전한 raw 값만 보정된다.

## 수용 기준

1. 2025 HWPX에서 생성한 HWP의 p261 picture 두 개와 인접 table `CTRL_HEADER`가 원본 HWP와 같이 bit 28을 가진다.
2. 비교 대상 세 개의 공통 의미 필드(treat-as-char, wrap, anchor)는 바뀌지 않는다.
3. Hancom Office 2020 같은 엔진 PDF raster 대조에서 기준선 109 changed pages / 392,833 pixels보다 악화하지 않는다.
4. p53 CharShape 잔차와 picture description tail은 별도 잔차로 결과에 명시한다.

## 페이지 수 수용 기준 확장 (미해결)

사용자 수용 기준은 HWP5 저장본의 Hancom PDF뿐 아니라 rhwp 자체 조판 결과도 기준 PDF와 같은 383쪽이어야 한다.

| 입력/산출물 | rhwp `dump-pages` | 기준 PDF 대비 |
| --- | ---: | ---: |
| 원본 HWPX | 386 | +3 |
| 원본 HWP | 393 | +10 |
| rhwp 저장 HWP (bit 28 보정 전 baseline) | 386 | +3 |
| Hancom 2020 기준 PDF | 383 | 0 |

원본 HWP가 HWPX보다 추가로 만드는 7쪽은 section 7(+1), section 10(+2), section 11(+4)의 표 분할에 집중된다. 세 대표 표의 의미 모델(크기, 행/열, RowBreak, anchor)은 일치하지만, HWPX table attr에는 `0x04000006`의 bit 26이 있고 원본 HWP에는 `0x00000006`이 있다. section 7의 첫 초과 표에는 이 차이가 없으므로 bit 26 하나로 전체 차이를 설명할 수 없다.

`--respect-vpos-reset`은 세 입력의 page count를 바꾸지 않았다. 따라서 이 옵션을 수용 기준 회피책으로 사용하지 않는다. page count 383 검증 전에는 Stage 125 결과 보고와 커밋을 진행하지 않는다.

세분화한 pagination 분석은 Stage 125에 분리한다. Stage 124의 bit 28 보정은 HWP5 저장 raw contract만 다루며, rhwp renderer의 383쪽 달성 책임을 대신하지 않는다.

## 검증 순서

1. 대상 Rust 통합 테스트와 HWP5 anchor trace를 실행한다.
2. 새 HWP를 Hancom Office 2020 `PrintToPDFEx`로 출력한다.
3. 원본 HWPX의 같은 엔진 PDF와 96 dpi, `pixelmatch threshold=0.1`, `includeAA=false`로 비교한다.
4. 결과를 Stage 125 보고서에 원시 레코드와 raster 지표로 기록한다.
