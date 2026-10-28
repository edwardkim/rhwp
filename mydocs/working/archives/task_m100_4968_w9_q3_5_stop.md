# Task M100 #4968 — Stage W9-Q3-5 중단·재계획 보고서

- 작성일: 2026-08-26 KST
- 작업 브랜치: `task_m100_4968`
- 선행 커밋: `770032cf8` W9-Q3-4 bounded pair candidate
- 상태: **중단 조건 충족 — prototype 커밋 금지, 수정 계획 승인 완료**

## 1. 결론

Q3-4의 design-unit candidate를 실제 layout px로 환산하고 measurement와
`compute_char_positions`에 연결한 prototype은 공개 fixture에서 기대값을 재현했다. 그러나 exact Noto Sans KR
Regular source 2,519,996 bytes를 core에 `include_bytes!`로 내장한 결과, Docker 최종 WASM이 Q3-4 대비
3,048,193 bytes(33.95%) 증가했다.

이는 수행계획 9절의 "유의미한 WASM size·시간 회귀" 중단 조건이다. 기능 결과가 맞더라도 이 구조로 Q3-5를
완료 처리하거나 커밋하지 않는다. font name에서 core 내장 face를 고르는 임시 registry도 문서·브라우저가 실제로
선택한 face source와 별도 계보가 되므로 일반 해법으로 확장하지 않는다.

## 2. prototype에서 확인한 사실

### 2.1 적용 순서와 K0 불변

- pair design unit에 effective font size와 horizontal ratio를 한 번씩 적용했다.
- letter spacing은 기존 glyph별 항으로 유지해 pair delta를 다시 scaling하지 않았다.
- `kerning=false`는 source registry·hash·shaping을 호출하지 않는 fast path다.
- Q2 body K0 9개 그룹의 sample position SHA-256은 동결 baseline과 전항 일치했다.
- K1의 `AV` 경계는 ratio 100/90/80에서 각각 약 `-0.240/-0.216/-0.192 px` 이동했다.
- K1의 `AV To` 누적 pair delta는 각각 약 `-1.253/-1.128/-1.003 px`였고 spacing 0/-5/-10과 독립이었다.
- 대표 R100/S0 run에서 measurement bbox는 K0 286px, K1 283px였고 K1 마지막 position은
  283.333px였다. 폭 측정과 위치 계산이 같은 adjustment를 소비했다.

### 2.2 nominal identity와 세그먼트 기능 탐지

공개 body run 전체에는 `R100`, `S0`, `K1` 같은 숫자 토큰이 있다. Noto shaping은 이 구간에서 GSUB glyph
identity를 nominal cmap과 다르게 만들어 Q3-4의 1:1 gate가 run 전체를 닫았다. `AV`와 `To` 자체는 각각
`-18/-76` design unit로 정상 통과했다.

prototype은 gate를 느슨하게 풀지 않고 공백 경계의 최대 256 segment를 독립 판정했다. 실패 segment는 0
adjustment와 structured fallback을 유지하고, 증명된 segment만 적용했다. 이는 version 분기가 아니라 현재
segment의 실제 shaping 가능성을 보는 feature detection이다.

### 2.3 검증 증거

| 검증 | 결과 |
| --- | --- |
| `cargo check --locked --lib` | 통과 |
| exact capability·candidate·scale 외부 integration | 5 passed, 0 failed |
| Q2 K0 frozen position hash | 9/9 일치 |
| Q2 K1 ratio·spacing matrix | 기대 OpenType delta 재현 |
| debug `rhwp-q-kit` | 빌드 및 공개 fixture layer-tree probe 통과 |
| Docker WASM build | 통과, 5분 50초 |
| Q3-4 WASM | 8,978,378 bytes |
| Q3-5 prototype WASM | 12,026,571 bytes |
| 증가량 | **3,048,193 bytes / 33.95%** |
| prototype WASM SHA-256 | `39a179040c82af187ffe87e1a8e3922e17cc7b80c78ca563b8525fa7638033db` |

외부 integration용 임시 manifest·target은 제거했다. generated integration suite·manifest와 Cargo target marker는
생성하거나 변경하지 않았다. Docker가 만든 `pkg/`는 기존 ignore 정책 아래 검증 산출물이며 source diff에 없다.

## 3. 원인 판정

현재 layout은 `TextStyle`만 받고 실제 선택된 font bytes/source key를 받지 않는다. prototype은 이 결손을
`Noto Sans KR` 이름과 core 내장 bytes의 정적 대응으로 메웠다. 그 결과 다음 두 문제가 동시에 생겼다.

1. core WASM이 문서에서 사용하지 않을 때도 전체 CJK font source를 운반한다.
2. 이름이 같아도 문서 embedded face·브라우저 webfont·native selected face가 다른 경우, layout provenance와
   paint provenance가 갈라질 수 있다.

따라서 33.95% 증가는 단순 최적화 대상이 아니라 exact-source 책임이 잘못된 계층에 놓였다는 구조적 증거다.

## 4. 수정 수행계획 권고안

### Q3-5R1 — exact source handle 계약

- font selection 결과에 bytes를 복제하지 않는 source handle을 둔다.
- handle은 source key, face index, byte length, SHA-256과 source availability만 보존한다.
- `TextStyle`에는 font blob을 넣지 않는다.
- source가 없으면 기존 advance를 유지하고 `font-source-unavailable`로 닫는다.

### Q3-5R2 — layout-session provider와 per-face cache

- document embedded font, native selected font, WASM host 등록 font가 같은 provider 계약으로 source를 공급한다.
- `KerningPairEngine`은 layout session의 exact source key별로 한 번만 준비한다.
- 32 MiB font, 4,096 code point/glyph, 4,095 pair, 256 segment 상한을 함께 적용한다.
- backend는 layout이 결정한 positions만 재생하고 다시 shaping하지 않는다.

### Q3-5R3 — 공개 검증 face 분리

- 2.52MB Noto 전체를 제품 core에 넣지 않는다.
- 일반 제품 경로는 host/document가 제공한 exact source로 검증한다.
- native/WASM 자동 fixture에는 provenance와 라이선스가 명확한 작은 공개 kerning face를 사용한다.
- Noto Q2 fixture와 source digest는 OpenType truth·오프라인 boundary 증거로 유지하되 core bundle dependency로
  만들지 않는다.

### Q3-5R4 — 공통 run measurement와 line boundary

- base glyph advance와 pair adjustment map을 한 결과 객체로 만든다.
- token total, 긴 단어 char fallback, line measurement, `compute_char_positions`가 같은 결과를 소비한다.
- pair의 두 glyph가 서로 다른 줄로 갈 때 boundary-crossing adjustment를 제거하는 fixture를 추가한다.
- 자간·장평·script scale 순서와 K0 byte identity를 다시 동결한다.

### Q3-5R5 — 재진입 게이트

- core에 full font bytes가 포함되지 않음을 binary probe로 확인한다.
- Q3-4 대비 WASM 증가를 code-only 비용으로 다시 측정하고 유의미한 증가면 재중단한다.
- public fixture의 measurement·line break·positions와 native/Docker WASM parity를 모두 통과해야 Q3-5를
  완료한다.

## 5. 승인 요청 범위

권고안은 현재 prototype을 커밋하지 않고, full-font static registry를 제거한 뒤 Q3-5R1부터 다시 시작하는 것이다.
compact precomputed pair DB는 실제 selected face와 달라질 수 있어 권고하지 않으며, 33.95% 증가를 수용하는 안도
WASM 배포 비용과 불변식 때문에 배제한다.
