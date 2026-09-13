# #7078 검토 반례

`high-contrast-step255.hwpx`는 커밋된 `samples/issue2470/36382471_masked.hwpx`의 합성 변형이다.
ZIP의 Contents/header.xml에서 그러데이션 시작색 #3057B9를 #000000, 끝색 #A0B4E6·#DFE6F7을
#FFFFFF로만 바꿨다. 이 합성 파일에 독립 한컴 정본이 있다고 주장하지 않는다.

기준 devel `1ae5ca295`와 네 PR 초기 체리픽 head `9a39b8f6b`의 같은 파일 SVG를 비교했다.
step=255에서 기존 510 stops·255 고유색·최대 채널 단차 2가 초기 후보 128 stops·64 고유색·단차 5로
바뀐다. 공통 expand_gradient_steps에 PDF 수용 한도를 적용하여 SVG까지 손상하는 반례다.
`fb09813f6`에서 #7078 코드만 되돌렸으며, 원 PR과 #7077은 머지 보류·미해결이다.

`pdf/pr7078-before-original-p1.pdf`와 `pdf/pr7078-capped-original-p1.pdf`는 합성 입력이 아닌
원래 `36382471_masked.hwpx`의 1쪽을 각각 기준/초기 후보 CLI `export-pdf -p 0`으로 출력한
검사 산출물이다. 독립 정본은 `pdf/issue2470/36382471_masked-2022.pdf`다.
MuPDF 96dpi에서 기준 PDF는 stitching sub-function 한도 오류와 흰색(255,255,255),
초기 후보는 오류 없이 (102,133,206), 독립 정본은 (102,133,207)을 낸다(x=400,y=417).
이는 PDF 누락 개선을 증명하지만 공통 렌더링의 띠 축소를 정당화하지 않는다.

보류 해제 조건: 공통 원본 stop/색/경계를 보존하고 PDF backend에서 표현 한도를 해결한 뒤
고대비·다중색·stepCenter·linear/radial 반례와 PDF 실제 raster를 검증한다.
