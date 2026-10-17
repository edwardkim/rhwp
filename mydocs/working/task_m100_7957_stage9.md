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

## 중간 판독과 공통 paint 순서 보정

`f1f13d226`의 Native에서는 close-stroke 2쪽·배치 수 3쪽·affine clip 3쪽이 전쪽
100%, 원본 19쪽이 최저 99.31446%였다. 실제 6쪽 연결선과 7쪽 글꼴은 개선됐지만,
6쪽의 뒤에서 가려져야 할 `Automotive`·`Global Test`가 앞에 다시 그려졌다.
이 상태를 완료로 판정하지 않는다. 추가 수정이 확인된 시점에 진행 중인 해당 Docker
최적화를 중단했다(exit 137, 의도한 중단이며 OOM·제품 실패가 아니다). 이전 단계의
완성된 Docker producer와 중간 Native producer는 보존한다.

원인은 path/image만 content operator의 계층적 source order로 정렬하고 텍스트는
나중에 별도의 큰 z-order로 추가하던 positional lowering이다. 좌표나 단어를 이용해
숨기지 않는다. TextExtractor가 같은 full parser의 페이지 ordinal과 Form 호출 경로를
기록하고, Tj buffer의 첫 paint 소속 → span sequence의 요청 내부 sidecar → import의
span/order 쌍 → shape·image·text 전체 z-order 정렬로 전달한다. public TextSpan 구조나
검색용 reading-order 의미를 변경하지 않는다. 시각 import에서 서로 다른 paint 소속의
span을 기하학적으로 합치거나 중복 제거하지 않는다. 비텍스트 mark 앞에서 대기 중인
텍스트 buffer를 먼저 방출한다. source-order sidecar의 entry·snapshot 복사는 기존 요청
ledger를 소비하며, 전체 할당 증명이 완료됐다는 주장은 여전히 하지 않는다.

독립 `text-paint-order-controls.pdf`는 페이지·중첩 Form·반복 중첩 Form에서 텍스트를
후속 불투명 사각형과 그림으로 덮는다. 원문 문자열은 IR에 남아야 하지만 최종 paint에서는
가려져야 한다. 변경 전에는 두 문자열이 덮는 개체 위에 보였다. 이 대조군의 변경 전
실루엣 값도 100%다. 유색 내부의 다른 글자색은 실루엣 점수로 잡히지 않으므로,
이 실패는 동일 영역의 실제 PNG·색상 및 개체 소유/순서로 입증한다.
