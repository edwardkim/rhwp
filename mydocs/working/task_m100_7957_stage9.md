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

## 16 MiB 자원 대조군의 범위 판정

작업지시자는 크기 경계 생성본을 자원 정책 검증으로 한정하고 Helvetica 글꼴·간격 차이는
[백로그 B1](../plans/task_m100_7957_backlog.md)로 분리하도록 승인했다. 해당 TSV의 미달과
producer를 그대로 기록하며 현재 공동 처리의 조판 통과 문서에 포함하지 않는다.
`exceeds_32.pdf`의 출력 차이는 계속 이번 보정 범위다.

## 고정 runtime의 Native 검증

`bba967e7e5d08e28b3ad19a247a622cea59d31ec`의 별도 보존 CLI로 360쪽을 출력했다.
자원 전용 생성본 1쪽을 제외한 359쪽은 전쪽 raw 90% 이상이며, 원본 19쪽의 최저값은
99.31522%, 공통 nested 대조군의 최저값은 90.14458%다. 고정 devel 대비 측정 가능한
공통 대조군에서 0.01 percentage point 이상 하락은 없다. Native 결과만으로 fresh WASM
완료를 선언하지 않는다. 최종 producer·전쪽 TSV는 ignored `paint-order-20261017/`에 보존한다.

글꼴 projection 정식 검사 3건도 통과했다. 첫 실행은 suite manifest를 갱신하지 않아
검사가 0건 선택됐으므로 통과로 세지 않았다. 기존 공용 target을 유지한 채 manifest를
명시적으로 준비한 뒤 3건이 실제 실행됐으며, 캐시 삭제나 소스 touch는 하지 않았다.

## 필수 회귀에서 발견한 연속 텍스트 경계

고정 runtime `bba967e7`의 360쪽 Native/fresh Docker WASM을 완료했다. 자원 전용 1쪽을
제외한 359쪽은 양쪽 모두 raw 90% 이상이고 원본 19쪽 최저값은 Native 99.31522%,
WASM 99.40106%다. 새 8개 관계 검사와 공개 API/Worker 92 assertion도 통과했다.
이 결과를 다음 보정 source의 결과로 바꿔 쓰지 않는다.

이후 필수 `issue_7958_pdf_positioned_word_gaps` 검사가 FAIL했다. 독립 생성본은 같은 줄의
두 Tj text object 사이에 물리적 단어 간격을 가진다. 새 paint order가 모든 Tj의 소속을
분리해, 중간에 다른 paint가 없는 이 경우에도 기존의 `가나 다라` 복원을 막았다.
최종 PNG가 100%여도 편집 가능한 문자열 계약이 충족되지 않았으므로 검사를 완화하지 않는다.

연속 텍스트는 비텍스트 mark 전까지 같은 paint group에서 기존 간격 복원을 허용한다.
path/image paint와 Form 진입·종료는 group을 분리하며 최종 z-order는 여전히 첫 원본
operator의 계층적 source order다. 중간의 도형·그림을 넘어서 텍스트를 합치지 않는다.
단어 간격의 수정 전 FAIL / 보정 후 PASS와 가림·Form 반례를 먼저 확인한 뒤 새 producer로
Native/fresh Docker WASM과 필수 gate를 다시 판정한다. 자료는 ignored `paint-group-20261017/`에 둔다.
