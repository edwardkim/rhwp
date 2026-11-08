# 설치 원본 프로그램과 문서별 글꼴 선택

옥수수 원본 OTF가 설치돼 있어도 문서의 호환 대체 이름이나 단일 face 조회 실패 때문에
돋움으로 그리던 경로를 보정했다. 같은 원본 family·style 안에서 실제 SFNT 형식의
TTF → TTC의 이름이 일치하는 face → OTF 순서로 선택하며, 원본 OTF는 다른 family의
대체 TTF보다 앞선다. 같은 family의 한글·영문 분할 프로그램도 보존한다.

이름만 존재하는 저장 probe는 실제 프로그램 공급과 구분한다. 이미 승인된 Local Font
Access 권한이 있는 문서 열기에서는 미해소 원본을 실제 카탈로그로 보완하고,
성공적으로 등록한 FontFace는 최종 Canvas2D 표시 체인에서 다시 대체하지 않는다.
승인되지 않은 권한을 자동으로 요청하지 않는다.

`installed_font_program_order.rs`의 세 검사는 합성 공급 snapshot과 저장소 공개 OTF
fixture만 사용한다. 옥수수 및 개인 글꼴을 차단한 macOS sandbox에서 세 검사 모두 통과했다.
비공개 글꼴 바이너리는 포함하지 않는다.

## 사용자가 지정한 가사 글꼴

기준 Print PDF의 가사는 원본 나눔고딕이 아닌 Haansoft Batang을 사용한다.
앞선 인쇄 비교 프로필의 성공을 일반 화면 성공으로 취급한 판단을 정정한다.
사용자가 일반 화면도 같은 프로그램을 쓰도록 선택했으므로 해당 원본의 digest에 묶인
문서별 사용자 대체 설정을 적용했다. 다른 문서에는 전파되지 않고 HWP 바이트도 변경하지 않는다.
실제로 로드한 대상 프로그램이 없으면 대체 성공으로 처리하지 않는다.

승인된 Local Font Access 검증 Chrome에서 재감지 버튼 없이 일반 파일 열기를 수행했다.
[1쪽 실제 화면](studio-p001.png)에서 가사의 문자·대시 간격을, [2쪽 실제 화면](studio-p002.png)에서
옥수수 원본 모양을 판독했다. 실제 초기화 응답의 WASM은 root pkg/public과 같은
`438b6ed62db8766014c7509a04fb4dcbf52abce51d24c6e2f08eef839506229e`다.
사용자의 기존 Chrome 탭은 직접 연결되지 않아 성공을 대신 주장하지 않는다.

화면 기본 TrueType 전진 계약과 기존 HFT·결측 계약, 앞서 실패한 6개 회귀를 포함한
Native Skia 자동 suite 49개가 통과했다. 일반 화면 기본 나눔고딕과 PDF의 대체 글꼴은
서로 다른 조건이며, 이 문서의 사용자 지정 설정과 일반 기본 출력도 구분한다.
전체 nextest·Native/fresh WASM 전쪽·편람·backend 검증은 별도 최종 게이트다.

상세 producer, 검사 및 로그 위치는 검증 기록 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/installed-programs-20261107/validation.json`)에 있다.
