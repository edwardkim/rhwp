# #7957 공동 처리 후속 백로그

## B1 — 크기 경계 생성 PDF의 Helvetica 글꼴·간격

2026-10-17 작업지시자는 이 생성본을 자원 정책·오류 복구 검증으로 한정하고 글꼴 차이를
후속 처리하도록 승인했다. 입력 허용 크기와 실패 후 복구의 통과를 조판 일치로 해석하지 않는다.

대상은 `samples/issue7962/`의 생성기와 16 MiB 경계 PDF다. 입력·SHA·생성 절차는 해당
폴더와 `samples/issue7957/README.md`에 보존한다. 현재 runtime `bba967e7e5d08e28b3ad19a247a622cea59d31ec`의
Native 최저 raw 일치율은 73.54167%, 같은 source의 fresh Docker WASM은 78.19253%다.
고정 devel `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`에도 같은 기존 차이가 있다.

실행 자료는 ignored `output/pr-review/issue7957/paint-order-20261017/`의 `candidate/scores/*/size/`,
기준 자료는 `../integration-20261017/base/scores/*/size/`에 있다. 추가 Nimbus Sans 공급 진단도
출력 차이를 해소하지 못했으며 제품 구현 변경이나 통과 증거로 삼지 않는다.

후속에서는 PDF의 Helvetica 지정·폭·문자 좌표와 실제 공급 face·측정/paint 경로를 추적하고,
동일 입력의 독립 PDF와 Native/fresh Docker WASM 전쪽 TSV·확대 판독을 확인한다.
페이지별 gate를 충족하기 전에는 이 생성본을 정상 조판 golden으로 추가하지 않는다.
