# #8533 원본 글꼴 체인의 실제 프로그램 준비 복구

## 남은 198쪽 차이와 소비 경로

Native와 WASM이 같은 공개 프로그램을 공급해도 WASM은 198쪽에 120개 실루엣
미일치 픽셀이 남았다. 모두 작은 글머리표의 반복 위치에 속했다. 독립 한컴 Print PDF의
해당 기호는 `Haansoft Batang`의 U+2219이며 반각 전진을 사용한다.

기존 `canvas_cluster_fit`은 원본 TTF의 cmap 결측이 확인됐을 때 작은 수학 기호의
윤곽을 누르지 않고 반각 칸 중앙에 놓는다. Native는 실제 프로그램을 공급받아 이 계약을
소비했지만, Sweep의 `prepareHftOtfFonts`는 `KoPub돋움체 Light,HCRBatang,…`
체인 전체를 한 family로 조회했다. 준비된 프로그램이 있어도 공급 proof가 0개였다.
Studio의 로컬/공개 프로그램 공급 두 경로에도 같은 전체 문자열 조회가 있었다.

원본 TTF는 체인의 첫 face만 조회하고 HFT 표시는 선언한 대체 face도 조회하도록
공통 helper를 사용한다. 요청의 전체 family 키는 그대로 두어 등록·소비가 일치한다.
다른 표시 프로그램을 원본 TTF로 승격하지 않는 반례와 CFF 분할 순서를 유지한다.
원본·PDF·숫자 기준·새 글꼴 바이트를 바꾸지 않았다.

## 집중 결과

- Sweep 공급자 검사: 보정 전 8 PASS / 1 FAIL. 보정 후 환경 준비 검사와 함께 25 PASS.
- Studio 공급자 검사: 보정 전 10 PASS / 2 FAIL. 보정 후 인쇄/묶음/일반 글상자 검사와
  함께 15 PASS. TypeScript 검사도 통과했다.
- 실제 fresh WASM의 Print 거부·portable 전환·화면 복원 브라우저 검사는 1 PASS다.
- 같은 공개 프로그램 조건에서 Native와 WASM의 198·220·272쪽은 모두 100%다.
  130쪽은 Native 99.96465%, WASM 99.96403%이며 #8389의 표시 하한 99.96%를 유지한다.
  전쪽 수용 여부는 이어지는 383쪽 전체 TSV로 판정한다.
- 기존 Native 필수 gate의 기본 동작은 유지한다. 명시적인 WASM manifest에도 같은
  #8389 하한을 비교할 수 있게 backend 인자를 추가했다. 잘못된 backend, 누락·중복,
  다른 입력/PDF·조건은 계속 보류하며 관련 Python 16건이 통과했다.

Rust 제품 소스와 빌드는 `8558aeed930aaf65c663ac55a05393f8d6823fc8`이다. 이 단계는
JavaScript/TypeScript 공급 경로와 검사를 보완했다. Native 실행 파일 SHA-256은
`b3e1afe12c2e994d909a33640f2de5c71b74612f3d616119d6fa99de50e2cfd4`,
fresh WASM은 `f54081389df95ef56eb88ec7e39bc538efa3049a5af23a73646c3ea280457d54`다.
Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 빌드이며 Docker 최적화로 주장하지 않는다.

## 격리와 실제 Studio

공개 다운로드 9개 프로그램과 저장소 공개 번들을 명시했다. macOS 외부 sandbox에서
개인·추가 시스템 글꼴 파일의 읽기를 차단하고 공개 Noto 파일 읽기 성공과 사용자
HancomEQN/HyhwpEQ 파일 읽기 거부를 실제 확인했다. Chrome은 외부 sandbox 아래서
실행했다. CoreText 이름 캐시까지 제거한 새 VM의 증거로 부르지 않는다.

전체 `ttfs/opensource`를 명시적으로 추가한 초기 대조군은 다른 Noto face가 선택됐으므로
정본 9개 프로그램 대조와 섞지 않는다. 공개 디렉터리까지 막은 초기 sandbox와 Chrome
중첩 sandbox 시작 실패는 유효한 시각 출력이 아니다. 성공한 실행의 입력·공급 환경·
프로그램 hash와 backend를 따로 기록한다.

실제 Studio의 편람 3쪽 클릭은 외곽 `group`(구역 0, 문단 4, 컨트롤 0, 쪽 index 2)을
선택했고 셀 조회 경고·command-error가 없었다. 초기 실제 fetch 응답의 WASM도 위 hash와
일치했다. Inspector 캐시에서 사라진 응답을 재요청한 값으로 대신하지 않고, 초기 fetch의
응답 clone을 브라우저 안에서 해시했다. 공급 경로 보완 뒤 최종 화면 검증은 다시 수행한다.

작업 로그·중간 파일은 ignored `output/pr-review/issue8533-wasm-advance-20261104/`에
보존한다. 전쪽·전체 회귀와 범위 게이트가 완료되기 전 PR 준비 완료로 표시하지 않는다.
