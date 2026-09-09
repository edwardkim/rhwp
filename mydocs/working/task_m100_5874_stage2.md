# #5874 Stage 2: PDF 출력 경계 보정

- Issue: #5874
- 선행 증거 commit: `ec0f8c6ff`.
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
- 입력 SHA-256은 [샘플 설명](../../samples/issue5874/README.md)에 고정했다.
  원본 1페이지 PDF와 italic 제거본은 7,162 pixels가 달라졌고, 페이지 수와 추출
  텍스트는 동일하다. 정규 대조본 PDF는 수정 전과 byte-identical이다.
- [전후 이미지](../pr/assets/issue_5874/before-after-review.png)를 직접 확인했다.
  둘째 줄 `기울인 글자`와 셋째 줄만 기울어지고 제목/일반 글자는 유지된다.
  [수정 PDF](../../pdf/issue_5874/after.pdf)를 보관했다. SVG의 `font-style="italic"` 40곳과
  overflow-cell 0건을 확인했으며 임시 SVG/JSON은 로컬 `output/issue5874/`에만 둔다.

## 시각 판정 경계

이번 결함은 SVG 생성 이후의 PDF 변환에 있다. 따라서 SVG를 다시 raster하는 visual sweep만으로
수정 여부를 판정하지 않고, 실제 `export-pdf`의 native PDF raster를 직접 비교했다.
작성자 한컴 screenshot은 기울임 유무의 독립 근거로만 사용했다. 로컬 PDF를 한컴 기준 PDF로
가장하거나 이 차이 픽셀 수를 fidelity 점수로 해석하지 않는다. 동일 한컴 글꼴/버전과의
완전한 픽셀 정합, Skia direct backend는 이번 7개 계약의 보증 범위가 아니다.

관련 PDF 계약 및 더 넓은 회귀와 lint 결과는 [Stage 3](task_m100_5874_stage3.md)에 기록했다.
아직 PR 생성/remote push/issue close는 하지 않았다.

## 이후 코멘트에 포함할 증거

- [전후 비교 PNG](../pr/assets/issue_5874/before-after-review.png)를 이미지로 본문에 표시한다.
- [작성자 한컴 기준 PNG](../pr/assets/issue_5874/reporter-hancom.png)를 함께 표시하되,
  다른 플랫폼/글꼴의 screenshot이며 정량 fidelity 기준 PDF가 아님을 명시한다.
- 최소 HWPX와 수정 전후 PDF 링크, 기본 PDF 경계에서의 재현/개선 사실, 검증 통과 여부를 적는다.
- raw log, 임시 PNG/SVG/JSON, 동일한 대조 PDF는 첨부하거나 커밋하지 않는다.
  아직 원격 게시 승인을 받지 않았으므로 코멘트는 게시하지 않는다.
