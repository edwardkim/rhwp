# #8153: Print 글리프 기하 보존

[계획](../plans/issue_8153_print_text_geometry_20261026.md)의 사양·실제 원본·독립
Print와 표준 Canvas 계약을 따른다. 원본과 PDF, 측정 임계값은 바꾸지 않는다.

## 수정 전 실제 실패

정식 Native 원문 SHA-256은
`ec04f96f6a73a4549e7c045001fd7a164e925f39e7e8bfad0804f881a319f8ea`다.
고정 `cb8d552a7d7ab4d001f8cf6fad4acb6520322573` 라이브러리에 연결해 빌드 exit 0,
실행 2.07초·exit 101이며 3건 중 1 PASS/2 FAIL이다. Print의 입력 1/4px 이동이
실제 alpha 질량 중심에서는 0px였고 반복 Print/Screen 전환도 같은 Print 경계에서
실패했다. 화면 격자 유지 대조는 통과했다. 합성 이동 계약이며 실물 문서의 정확도를
이 세 검사로 대신하지 않는다. 기존의 정확한 화면 격자 검사 두 건은 그대로 둔다.

정식 브라우저 원문 SHA-256은
`76b085a4a904b57b04e0d3b97901938e043361e8cb16e0e2cefe44c4136160e5`다.
고정 `b8ca19e50d`의 표준 최적화 WASM으로 실제 Chrome/FontFace/Canvas를 실행했다.
한 검사·실행 exit 1이며, 원본 배치·소속·glyph paint·Screen 모드 확인 뒤 Print의
실제 fillText가 `auto`를 소비하는 지점에서 실패했다. 파서나 import 오류가 아니다.
진단 사본의 설정을 제품 코드의 성공으로 세지 않고 같은 원문을 새 WASM에서 확인한다.

## 공통 정책과 실제 소비

`RenderProfile::Print`의 공통 기하 정책을 Native raster와 Canvas2D가 소비한다.
Native의 기존 글꼴 생성자는 이 정책일 때 subpixel 위치를 보존하고 hinting과
baseline snap을 끈다. 같은 생성자를 쓰는 일반 글자·겹침 글자와 직접 만드는
상자 숫자의 Font도 같은 helper를 소비한다. 화면 격자·기존 생성 번호 계약과
비래스터 벡터 기록의 글꼴 설정을 유지한다.

Canvas는 같은 프로필 정책을 표준 textRendering으로 전달한다. 실제 glyph paint에
적용하고 Screen 복귀 시 auto로 되돌린다. 이 속성을 지원하지 않는 구형 context는
기존 경로를 유지하며 지원을 관측하지 않고 기하 모드 성공으로 선언하지 않는다.
원본 좌표·장평·음수 자간·glyph fit의 조건·메트릭 DB·clip·그리기 순서는 바꾸지 않는다.
문서·쪽·글꼴·크기별 분기나 목표 점수에 맞춘 수치는 없다.

## 검증 상태

제품 변경 전 unit-tier 정책은 새 소스 테스트 모듈 증가 없이
4100 tests / 296 modules로 통과했다. Native와 브라우저 정식 원문을 정식 경로에
두었다. 코드 head `e496770a7d83910832808d9fc5b067c82ec4d949`의 Native 빌드는
372초·exit 0이며, 수정 전과 같은 원문 3건이 2.14초·3 PASS/0 FAIL이다.
화면/생성 번호 기존 2건·기존 105건을 합친 110건도
240.53초·110 PASS/0 FAIL이다.

영향 84조건에서 기존 100% 쪽의 이탈은 없지만 85·86·87쪽 등에는 작은 증가가 있다.
예를 들어 두 형식 모두 86쪽은 192→223, 87쪽은 221→238픽셀이다. 반면 80쪽은
252→220, 111쪽은 82→47, 289쪽은 64→23픽셀이다. 이 증가를 통과로 상쇄하지 않는다.
직접 연 86쪽 원본·수정 전/후·브라우저 진단 crop에서 수신·경유 글줄 자체의 원점이
독립 PDF보다 위에 있음을 확인했다. 원본 HU·행 상자 합의 조사로, 그림이 든 앞 행의
셀 여백 280HU가 빠지고 마지막 행으로 넘어간 별도 공통 조판 결함을 좁혔다.
글꼴 모드를 문서별로 되돌려 이 위치 차이를 숨기지 않고 행 높이 생산자를 추적한다.

같은 코드의 표준 최적화 fresh WASM 빌드는 exit 0으로 완료했다. 잡 8개,
codegen-units 16, Binaryen 8이며 WASM SHA-256은
`fa8587efda24d69dadfe3485f3654ff678c0ca2bc8b9c7aba687d25dfb87a4b7`이다.
수정 전과 동일한 정식 브라우저 원문 1건도 실제 Canvas 실행에서 PASS다.
전쪽 Native/fresh WASM과 증가 쪽 보정은 아직 남았다.
이 단계는 전쪽 100%·최종 테스트·PR 준비 완료가 아니다.

실행 로그·고정 producer·초안·proof는 ignored 작업 루트의
`handbook-print-text-geometry-candidate/`에 보존한다. upstream/devel 동기화,
원격 게시·push·PR 제출은 하지 않는다.
