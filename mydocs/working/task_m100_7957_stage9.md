# #7957–#7963·#7966 — 추가 PDF의 남은 출력 보정

작업지시자는 `exceeds_32.pdf`의 남은 연결선·글꼴·간격 차이도 이번 공동 처리에서
보정하도록 승인했다. 19쪽 raw 최저 96.58220%는 변경 전 진단값이며, 직접 판독에서
드러난 결함을 면제하지 않는다. 앞 절편 `17c14d8d7`의 Native/Docker producer는
변경 전 증거로 보존하고 같은 313쪽 검사를 중간 구현마다 반복하지 않는다.

## 독립 근거와 소비 경로

- PDF의 `s`는 `h S`와 같다(ISO 32000-1, path-painting operators).
  원본 6쪽의 분홍 원형선과 파란 세로선 여섯 개가 모두 `s`를 쓴다.
  `PathExtractor::close_and_stroke`는 이미 있으나 `document.rs`의 페이지·Form
  path interpreter 두 곳이 `CloseAndStroke`를 전달하지 않았다. 그 결과 path가
  다음 paint까지 남아 색상이 섞이거나 후속 상태 변경으로 소실된다.
  두 interpreter → 같은 extractor → `finalize_path` → `extract_paths` → 공통
  PDF lowering → Native/Canvas paint를 확인한다. 별도 문서·좌표 조건을 넣지 않는다.
- 독립 생성 `close-stroke-controls.pdf`는 페이지와 두 단계 중첩 Form에서
  `s`·`h S`·열린 `S`를 대조한다. 변경 전 실제 Native에서 두 쪽 모두 붉은 `s`
  선의 픽셀이 0이었다. 정상 `h S`·`S` 및 뒤의 채운 사각형도 함께 보존한다.
- PDF FontDescriptor의 굵기 700은 IR의 `CharShape.bold`까지 보존됐다.
  `MalgunGothicBold`·`Arial-BoldMT`가 설치 프로그램의 family와 연결되지 않아
  실제 paint와 측정이 다른 fallback을 썼다. 로컬 SFNT 이름표·OS/2 weight와
  원본의 descriptor를 대조하며 폰트 바이트는 배포하지 않는다. Bold full face
  정체성을 Regular로 합치지 않고 canonical 이름·metric 결정면을 각각 검증한다.

## 검증 경계

작은 경계와 실제 6·7쪽의 직접 비교를 먼저 확인한다. 코드가 고정된 뒤 영향을 받는
Native/fresh Docker WASM의 전쪽 TSV·대표 PNG, 요청 정책·공개 API/Worker·필수 회귀를
판정한다. 점수와 연결선·글꼴/간격의 직접 판독을 구분한다. 기존 Helvetica 자원 경계
대조군의 미달은 별도 관측으로 보존하며 숨기거나 이 문서의 통과로 대신하지 않는다.

증적은 ignored `output/pr-review/issue7957/fidelity-20261017/`에 보존한다.
현재 승인은 로컬 보정·검증과 이미 승인된 devel 대상 Open PR까지다. merge·이슈 종료는
수행하지 않는다. `parserAllocationBoundsProven=false`의 미완료 경계도 유지한다.
