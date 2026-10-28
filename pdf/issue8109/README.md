# #8109 글꼴 출처 대조군

수동 작성한 PDF로 표준 Type1 프로그램의 부재, 프로그램 선언은 있으나 읽을 수 없는 상태,
프로그램 상태를 알 수 없는 경우를 구별한다. 한컴 저장본이 아닌 합성 resource 계약 입력이며,
성공적으로 열리는 각 PDF 자체가 독립 시각 기준이다. `unresolved-descriptor.pdf`는 기존
자원 제한 거절을 보호하는 오류 입력이며 렌더링 fixture에 포함하지 않는다.

`generate_controls.py`는 object dictionary·content stream·xref offset을 명시해 PDF11개를
생성한다. fontTools와 시스템 `fonts-urw-base35` 패키지가 필요하다. `manifest.json`은 원본
font와 PDF 해시를 기록한다. `standard-substitute.hwpx`는 이 생성기의 산출물이 아니다.

`standard-substitute.hwpx`는 수정 전 importer로 `pdf/issue7998/body-list.pdf`를 가져온 뒤
공통 모델의 `Helvetica` 선언에 TrueType 대체 `Arial`을 명시해 만들었다. 새 PDF 정책에
의존하지 않고 HWP의 독립 alternative face/type byte와 HWPX → HWP → HWPX 보존을 검사한다.
생성 절차와 독립 한컴2020 Print는 `mydocs/working/task_m100_8109_stage1.md`, 현행 시각 검증은
`mydocs/working/task_m100_8109_stage2.md`에 기록했다.

프로그램이 있는 대조군은 PDF 텍스트 문서 내부에 CFF 프로그램만 포함한다. 원본은 Debian의
`fonts-urw-base35` 패키지가 배포하는 Artifex/URW `NimbusSans-Regular.otf`이며 저작권자는
2015 URW Software 및 2013–2014 (URW)++ Design & Development다. 라이선스는 GNU AGPL
version3이며 다음 font 예외를 포함한다.

> As a special exception, permission is granted to include these font
> programs in a Postscript or PDF file that consists of a document that
> contains text to be displayed or printed using this font, regardless
> of the conditions or license applying to the document itself.

원본 소스와 라이선스: https://github.com/ArtifexSoftware/urw-base35-fonts.
독립 font 프로그램과 상용 font 파일은 포함하지 않았다.
