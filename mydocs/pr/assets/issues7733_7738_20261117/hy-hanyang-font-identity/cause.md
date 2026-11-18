# HY·한양 원본 신원과 로컬 재감지 보정

## 원인과 변경 범위

`2bb86f50d385ddef49433721c3f54da6c901ac7d`는 한양중고딕을 H2GTRM.TTF의
설치 이름과 같은 원본으로 연결했다. 실제 설치 파일의 SFNT name/cmap/윤곽 테이블은
서로 다른 글꼴이다. `SPJGT.otf`는 한양중고딕 HFT의 기호 분할 프로그램이며,
`H2GTRM.TTF`는 HY중고딕 TrueType이다. 후자에는 U+2550 `═`가 없으므로 그 경로의
0.5em 관측값을 원래 기호 폭이라고 해석하지 않는다. 형식 우선순위는 같은 원본의
TTF → TTC → OTF 후보 안에서 적용한다.

| 한양 HFT 원본 이름 | 같은 파일의 영문 이름 | 별개의 HY 원본 |
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

원본 프로그램 선택의 수정 전 실패와 CSS 오인 재감지의 수정 전 실패는 각각 독립 로그로
확인했다. 검사 초안의 문법·중복 질의·실행 0건·HMR 조회 인스턴스 오류는 결함 재현으로
세지 않는다. 현재 source의 Native 빌드와 전체 시각/필수 게이트는 진행 중이다.
이전 efe954의 전체 통과·시각 결과를 이 보정의 통과로 재사용하지 않는다.
