# #7078 검토 반례

`high-contrast-step255.hwpx`는 커밋된 `samples/issue2470/36382471_masked.hwpx`의 합성 변형이다.
ZIP의 Contents/header.xml에서 그러데이션 시작색 #3057B9를 #000000, 끝색 #A0B4E6·#DFE6F7을
#FFFFFF로만 바꿨다. 이 합성 파일에 독립 한컴 정본이 있다고 주장하지 않는다.

기준 devel `1ae5ca295`와 네 PR 초기 체리픽 head `9a39b8f6b`의 같은 파일 SVG를 비교했다.
step=255에서 기존 510 stops·255 고유색·최대 채널 단차 2가 초기 후보 128 stops·64 고유색·단차 5로
바뀐다. 공통 expand_gradient_steps에 PDF 수용 한도를 적용하여 SVG까지 손상하는 반례다.
`fb09813f6`에서 #7078 코드만 되돌렸으며, 초기 검토에서는 원 PR과 #7077을 보류했다. 아래 메인터너 보정에서 해당 원인을 해결했다.

`pdf/pr7078-before-original-p1.pdf`와 `pdf/pr7078-capped-original-p1.pdf`는 합성 입력이 아닌
원래 `36382471_masked.hwpx`의 1쪽을 각각 기준/초기 후보 CLI `export-pdf -p 0`으로 출력한
검사 산출물이다. 독립 정본은 `pdf/issue2470/36382471_masked-2022.pdf`다.
MuPDF 96dpi에서 기준 PDF는 stitching sub-function 한도 오류와 흰색(255,255,255),
초기 후보는 오류 없이 (102,133,206), 독립 정본은 (102,133,207)을 낸다(x=400,y=417).
이는 PDF 누락 개선을 증명하지만 공통 렌더링의 띠 축소를 정당화하지 않는다.

보류 해제 조건: 공통 원본 stop/색/경계를 보존하고 PDF backend에서 표현 한도를 해결한 뒤
고대비·다중색·stepCenter·linear/radial 반례와 PDF 실제 raster를 검증한다.


## 메인터너 보정 검증

`0acc011e3`은 공통 stop/색/경계를 그대로 두고 PDF 변환기의 Type 3 함수만 계층화한다.
고대비 반례는 510 stops·255 고유색을 보존한다. 함수의 원본 구간·색·opacity는 전부 유지하며
함수 하나의 자식 수만 PDF reader의 256개 제한에 맞춘다.

- `pdf/pr7078-maintainer-original-p1.pdf`: 기존 실문서의 보정 후 PDF.
  MuPDF 96dpi (400,417)=(103,133,206), 독립 한컴=(102,133,207), 함수 한도 오류 없음.
- `pdf/pr7078-maintainer-contrast-p1.pdf`: 같은 고대비 합성 반례의 보정 후 PDF, 함수 한도 오류 없음.
- 제품 PDF 함수 값 검증: HWP step 100/129/255/1024 × stepCenter 0/8/50/92/100 × 선형/원형,
  다중색·불투명도 1/256/257/1024구간. 모든 원 구간 중점의 실제 PDF 함수 값과 원본 값을 대조해 통과했다.
- 기존 `expand_gradient_steps` 및 #6822 step=100 계약은 변경하지 않았다.

개별 검토·최종 검증 결과는 `mydocs/pr/archives/pr_7078_review.md`를 따른다.

추가 Poppler 96dpi 비교에서 기존 무상한 PDF와 보정 PDF는 실문서·고대비 반례 두 페이지 전체 픽셀이 각각 동일했다.
같은 출력의 비교 전용 flat PDF는 중복 커밋하지 않고 보정 PDF 정본과 비교 결과를 유지한다.
