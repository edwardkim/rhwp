# HY·한양 원본 신원과 로컬 재감지 보정

## 원인과 변경 범위

`2bb86f50d385ddef49433721c3f54da6c901ac7d`는 한양중고딕을 H2GTRM.TTF의
설치 이름과 같은 원본으로 연결했다. 실제 설치 파일의 SFNT name/cmap/윤곽 테이블은
서로 다른 글꼴이다. 현재 직접 로드하는 파일은 OTF(CFF)이며 `.HFT` 원본 파일이 아니다.
문서의 HFT 글꼴 선언과 실제 공급하는 OTF 형식을 구분한다. `SPJGT.otf`는 설치된 한양중고딕의 기호 분할 OTF이며,
`H2GTRM.TTF`는 HY중고딕 TrueType이다. 후자에는 U+2550 `═`가 없으므로 그 경로의
0.5em 관측값을 원래 기호 폭이라고 해석하지 않는다. 형식 우선순위는 같은 원본의
TTF → TTC → OTF 후보 안에서 적용한다.

| 한양 계열 OTF의 한글 이름 | 같은 OTF의 영문 이름 | 별개의 HY TTF |
| --- | --- | --- |
| 한양중고딕 | HY Junggothic | HY중고딕 / HYGothic-Medium |
| 한양신명조 | HY Sinmyeongjo | HY신명조 / HYSinMyeongJo-Medium |
| 한양견고딕 | HY Gyeongothic | HY견고딕 / HYGothic-Extra |
| 한양견명조 | HY Gyeonmyeongjo | HY견명조 / HYMyeongJo-Extra |
| 한양그래픽 | HY Graphic | HY그래픽 / HYGraphic-Medium |
| 한양궁서 | HY Gungseo | HY궁서 |

[설치 파일의 실제 이름과 SHA-256](font-name-inventory.json)을 대조했다.
접두어만으로 같은 글꼴을 추정하지 않는다. 한양해서와 HYhaeseo처럼 실제 같은 파일의
이름인 경우는 보존한다. 원본 프로그램 후보와 SVG local()/임베드 파일 후보를 구분하며,
원본 부재 시 표시용 대체 체인을 다른 글꼴의 원본 설치 증거로 사용하지 않는다.

재감지의 Canvas 폭 변화는 이미 문서에 등록된 CSS 대체·웹폰트 때문에도 발생한다.
FontFace API가 있으면 정확한 `local("이름")` 로드로 로컬 존재만 확인하며, 그 결과를
실제 프로그램 바이트의 신원으로 인증하지 않는다. 실제 열거 자료에서는 각 face의
SFNT 이름과 PostScript 이름을 따로 수집한다. 명시 재감지 시 이전에 등록한 로컬 CSS face와
프로그램 캐시를 해제해 새 목록과 섞이지 않도록 한다. FontFace가 없는 환경의 제한적
존재 검사는 기존 동작을 보존한다.

## 검증 상태

로컬 수집·선언 프로그램 Node 검사 27건과 TypeScript 검사는 통과했다.
전체 Studio 검사는 2,814건 통과·2건 제외·실패 0건이다. 최소 글꼴 환경의
설치 프로그램 순서·HY/한양 신원·SVG 별칭 집중 회귀 5건도 모두 통과했다.
실제 환경 설정 버튼의 E2E는 문서와 무관한 글꼴의 재조판 방지, 관련 글꼴 갱신,
이전 CSS face 해제, 여섯 쌍 12개 face의 분리 수집·조회까지 통과했다.
관계형 회귀는 사설 글꼴을 요구하지 않는다.
[실제 Mac Chrome 수집](actual-macos-browser-collection.json)에서도 1,707개 face 중
대상 12개 이름이 모두 수집됐고, 여섯 쌍의 PostScript 신원 집합이 서로 겹치지 않음을 확인했다.
같은 family의 여러 스타일·분할 파일은 단일 프로그램으로 강제 합치지 않는다.

원본 프로그램 선택의 수정 전 실패와 CSS 오인 재감지의 수정 전 실패는 각각 독립 로그로
확인했다. 검사 초안의 문법·중복 질의·실행 0건·HMR 조회 인스턴스 오류는 결함 재현으로
세지 않는다. 현재 source의 Native 빌드는 완료했다. [새 Native 집중 검증](native-focused-validation.json)의
2·474·690쪽은 독립 한컴 글꼴 조건에서 각각 100.00%·100.00%·99.94%이며 보호값을 유지했다.
일반 자동 선택만 사용한 2쪽도 기존 98.25%에서 100.00%가 됐다. 자동 선택의 474쪽은
99.96%이므로 독립 한컴 HNC 프로그램 조건의 100% 결과와 구분한다.
같은 쪽의 review와 overlay를 모두 직접 판독했다. 이 결과는 Native 3쪽 집중 검사이며
새 fresh WASM·전쪽 보호 문서·전체 필수 게이트의 통과를 의미하지 않는다.

`native-skia`의 `export-png`도 같은 세 쪽에서 실행했다. 자동 선택에서 2쪽은
98.25%에서 100.00%로 올랐고 690쪽은 99.93%에서 99.94%로 올랐다. 474쪽은
자동 선택 99.96%, 독립 한컴 HNC 글꼴 공급 조건 100.00%를 기록했다. 2쪽의 제목과
양쪽 이중선 끝점을 직접 비교했다. [Native PNG 결과와 이미지](native-png/validation.json)

`8771cdcbc4`는 회귀 기대값과 문서만 바꿨다. 제품 소스 객체와 고정한 Native·WASM
파일의 해시를 대조해 `0fee3421a4`의 산출물을 재사용했다.
[산출물 재사용 근거](test-only-render-reuse.json)

초기 렌더, host 글꼴 입력, 로컬 글꼴 재감지의 브라우저 E2E 3건에서 같은 WASM을
실제로 초기화해 통과했다. Studio 배포 빌드는 정상 환경에서 해당 WASM을 포함했고,
서비스 워커 precache 75개 항목에 번들 `SpoqaHanSans-Regular.woff2`가 들어갔다.
최소 글꼴 격리에서 실행한 최초 배포 빌드는 글꼴 접근 제한으로 precache 항목이
누락돼 정상 환경의 배포 산출물로 검증을 마쳤다. [배포 빌드 영수증](studio-production-build.json)
이전 efe954의 전체 통과·시각 결과를 이 보정의 통과로 재사용하지 않는다.
