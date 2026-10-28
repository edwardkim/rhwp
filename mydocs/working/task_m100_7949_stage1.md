# Task #7949·#7950 — 1단계 마스크 구현과 Native 선행 확인

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 기준 SHA: `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`
- 승인: 사용자의 “승인합니다”로 [공동 구현계획](../plans/task_m100_7949_impl.md) 착수
- 상태: 마스크 보정 Native 선행 확인. 최종 Docker WASM·정식 회귀·시각 gate는 미완료

## 구현과 소비 경로

`PdfImageHandle::decode`에서 예산이 있는 경우 지원하는 명시적 1비트 Mask를 읽는다.
`image_mask_limited.rs`가 참조·크기·Decode·행 패딩·길이를 확인하고 기존 제한 decoder로
스트림을 해제한다. 마스크와 base는 공통 unit square에서 픽셀 중심을 대응시켜 RGBA로 합성한다.
Decode 기본값에서 0은 불투명, 1은 투명이며 `[1 0]`은 반전이다.
독립 근거는 [Adobe PDF Reference 1.7 §4.8.5 pp.350–351](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf)다.

합성 후 기존 handle이 bbox·CTM을 설정하고 제품 extract가 clip의 visible bounds를 적용한다.
lowering의 `to_png_bytes_with_budget`가 RGBA를 PNG color type 6으로 기록하고 기존 BinData·그림
객체가 이를 paint에 공급한다. bbox나 페이지 원점을 바꾸지 않았다.
JPEG로 저장할 때 새 RGBA를 조용히 불투명하게 바꾸지 않고 PNG 사용 오류를 반환한다.

마스크는 raw/Flate, Decode `[0 1]`·`[1 0]`, 기본 보간(false)을 지원한다.
색상 키 배열·SMask·독립 stencil·중첩 Mask·다른 mask codec·DecodeParms·보간 요청은
명시적 미지원으로 남긴다. 크기·길이·타입 오류와 예산 초과를 구분한다.
새 픽셀 버퍼는 할당 전에 요청 예산에 계상하고 스트림·반복 행에도 요청 확인을 적용한다.

## 선행 실행 결과

- `cargo build --locked --profile release-test --bin rhwp --features native-skia,pdf-import`,
  공용 target `/home/edward/mygithub/rhwp/target/pr-review`, jobs14: 성공(3분21초).
- ignored `output/pr-review/issue7949/probe-mask.py`: raw·반전·9px 행 패딩·서로 다른 해상도·
  Flate의 5입력을 실제 CLI로 HWPX 저장한 뒤 BinData PNG의 alpha/색상을 독립 기대값과 비교해 통과했다.
  정식 회귀나 실물 조판 완료를 대신하지 않는 진단이다.
- 사용자 PDF `261016-mof-69009.pdf`는 이전의 `PDF_IMAGE_UNSUPPORTED` 대신 1쪽으로
  열리고 HWPX 저장에 성공했다. 입력 SHA는 수행계획에 고정했다.
- Native 전쪽 실루엣은 **82.7978%**, 원본 1쪽/Native 1쪽으로 **시각 gate 미충족**이다.
  원본과 Native PNG를 직접 대조해 그림 5회의 표시를 확인했지만 오른쪽 표의 채움/흰 글자와
  글꼴·간격 차이가 남았다. PDF 진단의 unsupportedPaths는 15다. 마스크가 열기만 해결했다고
  보고하며 이 차이를 정상 조판 완료나 base 대비 무회귀로 판정하지 않는다.

로그·명령·원본 대조 PNG·TSV는 전용 작업공간의 ignored
`output/pr-review/issue7949/`에 보존했다. 기준 명세 PDF도 로컬 reference에 보존한다.
다음 단계는 기본 feature 전환과 최종 Docker WASM 검증이다. 전체 회귀·최종 시각 판정과
남은 출력 차이의 처리는 통합 단계에서 구분하며 현재 상태로 PR을 제출하지 않는다.
