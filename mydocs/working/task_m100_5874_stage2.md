# #5874 Stage 2: PDF 출력 경계 보정

- Issue: #5874
- 선행 증거 commit: `1b3b667af`.
- 범위: 기본 SVG 기반 PDF에서 정규 face로 fallback된 italic text만 합성한다.
- 구현 전 계획: usvg의 요청 style/실제 glyph face/수평 baseline을 확인한 다음 XML attribute의
  정확한 위치에 local shear를 추가한다. 기존 translate/scale과 glyph advance를 보존한다.
- 실제 italic/oblique는 이중 합성하지 않는다. 혼합 face/다중 baseline/세로쓰기/경로 위 text는
  통째로 변형하지 않고 경고한다. 원본 문서 내용은 경고에 포함하지 않는다.
- 검증 예정: 격리 fontdb 계약 테스트, 원본 재현 PDF의 전후 raster/추출 텍스트,
  관련 PDF fallback/subset/subSVG 계약, fmt 및 Clippy.
- Skia direct PDF 및 Text IR v2 authority는 별도 범위다.

## 구현과 실측 결과

- `pdf_synthetic_italic.rs`와 기본 PDF 호출부를 보정했다. 정상 face와 선택된 fallback face를
  구별하며, 실제 baseline을 고정하는 `x' = x + 0.25 * (baseline - y)`를 사용한다.
- `issue_5874_pdf_synthetic_italic` focused nextest: **7/7 통과**, 0.015초.
  실제 style metadata가 italic/oblique인 face, 정규 fallback, 혼합 face 경고, 기존 변환/회전,
  baseline/advance, ID 충돌, 임베드 Type0 font 보존을 격리된 fontdb로 검사했다.
- 초기 두 실행은 파생 harness가 새 case 또는 fmt 후 바뀐 가중치 배정을 반영하지 않아
  `0 tests`/exit 4로 종료했다. 성공으로 세지 않았다. fmt 이후 manifest를 재생성하고
  실제 `regression_suite_023`의 module 등록과 **7건 실행**을 확인했다.
- [실측 JSON](../pr/assets/issue_5874/after-comparison.json)에 입력/바이너리/구현 파일의 SHA-256을
  고정했다. 원본 1페이지 PDF와 italic 제거본은 7,162 pixels가 달라졌고, 페이지 수와 추출
  텍스트는 동일하다. 정규 대조본 PDF는 수정 전과 byte-identical이다.
- [전후 이미지](../pr/assets/issue_5874/before-after-review.png)를 직접 확인했다.
  둘째 줄 `기울인 글자`와 셋째 줄만 기울어지고 제목/일반 글자는 유지된다.
  [수정 PDF](../../pdf/issue_5874/after.pdf)와 [원본 SVG](../pr/assets/issue_5874/source.svg)를 보관했다.

## 시각 판정 경계

이번 결함은 SVG 생성 이후의 PDF 변환에 있다. 따라서 SVG를 다시 raster하는 visual sweep만으로
수정 여부를 판정하지 않고, 실제 `export-pdf`의 native PDF raster를 직접 비교했다.
작성자 한컴 screenshot은 기울임 유무의 독립 근거로만 사용했다. 로컬 PDF를 한컴 기준 PDF로
가장하거나 이 차이 픽셀 수를 fidelity 점수로 해석하지 않는다. 동일 한컴 글꼴/버전과의
완전한 픽셀 정합, Skia direct backend는 이번 7개 계약의 보증 범위가 아니다.

다음 단계에서 관련 PDF 계약 및 더 넓은 회귀와 lint를 수행한다. 아직 PR 생성/remote push/
issue close는 하지 않았다.
