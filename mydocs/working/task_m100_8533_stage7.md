# #8533 Print 대체 TTF와 JPEG 장치 프레임 보정

## 원인과 적용 경계

383쪽 첫 재검증에서 Native는 #8389의 쪽별 표시 하한을 유지했다. 일반 설치 환경의
WASM은 140쪽 99.98238%가 남았고, 글꼴 파일만 차단한 Mac 대조에는 추가 차이가 있었다.
후자는 설치 이름을 반환하는 CoreText 서비스 때문에 Chrome이 물음표 글리프를 선택했다.
`Haansoft Batang`의 숫자 `1` 요청으로 이를 별도 재현했으며, 파일 읽기와
`com.apple.fonts`·`com.apple.fonts.atsbridge` 조회를 함께 차단한 대조에서 정상 fallback을 확인했다.
파일 차단만 한 결과를 최소 글꼴 환경의 완료 증거로 재사용하지 않는다.

새 격리 조건에서도 30·168·194쪽의 반각 닫는 괄호 끝이 Canvas에서 한 픽셀 더 내려갔다.
동일한 공개 HCR TTF의 실제 윤곽을 같은 원점·크기로 재생한 진단에서는 세 쪽 모두
raw 100%였다. Print의 기하 계약에서만, 실제 적재 프로그램의 문자 범위·스타일 선택이
원 face 뒤의 정적 TTF를 선택한 경우 그 프로그램의 윤곽을 재생한다. 원 face가 공급하는
문자·Screen·2022 Print·문맥 클러스터·기울임/굵기/효과는 기존 painter를 유지한다.
프로그램과 윤곽 캐시는 바이트·항목 상한을 두며 미지원/상한 초과 시 기존 painter로 돌아간다.
조판 전진·줄 구성·문서 원문을 변경하지 않는다.

140쪽의 차이는 삽입 PNG에서 생성한 JPEG 인쇄 프레임이다. Native SVG와 WASM layer의
네 JPEG 자원은 각각 바이트와 SHA-256이 같았다. 그러나 Canvas의 프레임 경로만
브라우저 JPEG 복호화를 사용했고 일반 Print 경로의 공통 IJG 재생 RGB를 우회했다.
두 Canvas 진입점이 같은 복호화 함수를 소비하도록 연결했다. PDF·원본 그림·프레임 기하·
90% 문턱·2px 이웃·#8389 쪽별 하한은 바꾸지 않았다.

## 결함 검출과 보정 결과

- 기존 브라우저 대조군 2건은 보정 전에도 통과했다.
- 새 편람 140쪽 검사는 독립 PDF 영역 대비 raw 미일치 `[7, 1]`로 실제 실패했다.
  새 TTF fallback 검사도 기존 `fillText` 소비를 검출하여 실패했다.
- 같은 검사 4건은 보정 후 fresh WASM에서 모두 통과했다. 원 face의 `H`와 Screen의
  fallback 글자는 기존 painter에 남는 반례도 포함한다.
- 140쪽 고정 영역은 보정 전 Native 100%, fresh WASM 99.98238% 페이지에서 정했다.
  독립 PDFium 96 DPI PNG를 무보간 절단했고 source/PDF/PNG hash를
  `tests/fixtures/issue8533/handbook-print-frame.json`에 기록했다. 사유 글꼴 바이트는 추가하지 않았다.
- 앞선 전체 회귀는 12,537 실행, 12,536 PASS, 1 FAIL, 45 skip이었다. 실패는 Print에
  화면 메트릭을 활성화한 기존 일·생활균형 검사였다. 기존 원점·문자 소속 검사를 Screen과
  portable Print 양쪽에서 유지하고, 잘못된 활성 상태의 Print 거부도 검사하도록 보정했다.
  새 소스의 전체 회귀 완료로 앞선 수치를 재인용하지 않는다.

Mac fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
root pkg와 Studio public의 JS/WASM hash가 각각 일치했다. 이 단계의 WASM SHA-256은
`3be326bc73eb064fea3592cf4a88d099e3c0ea1e47bc6e1631a957cb375610d0`이다.
초기 등록 경계를 바로잡기 위해 중단한 첫 빌드는 완료 증거로 쓰지 않는다.

로그·진단 스크립트·전체 PNG 등 중간 파일은 ignored
`output/pr-review/issue8533-wasm-advance-20261104/`에 둔다.
새 Native/fresh WASM 383쪽 TSV·전체 회귀·필수 게이트·Studio 실제 소비 경로 확인을
마치기 전에는 PR 준비 완료로 표시하지 않는다.
