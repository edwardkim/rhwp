# Task M100 #4969 W10-Q3-E0 — 최신 devel 병합 후 재자격화 결과

- **결과 상태**: `qualified-red-baseline-reconfirmed`, 메인테이너 승인
- **승인일**: 2026-08-29 KST
- **checkpoint 상태**: 생성 승인·고정 완료
- **E0 checkpoint**: `406ee9e31d0e3c10c93897d355d03f0051dd5be1`
- **병합 upstream**: `b54f20e391023e04c9916c2b6d60af8ab1863369`
- **merge commit**: `b5fbed37c9ada3ec7a39931657d4bce386547626`
- **실행일**: 2026-08-29 KST
- **기계 판독 증적**:
  [`w10_q3_e0_requalification.json`](../../tech/investigations/issue-4969/w10_q3_e0_requalification.json)

## 판정

최신 devel 병합 뒤에도 Q3-E0는 **qualified-red-baseline-reconfirmed**다. default layer JSON과 CanvasKit plan은
최초 E0와 byte·BLAKE3가 같고 green control 21건이 모두 통과했다. 의도된 red는 E1~E4 미구현 경계 네 건에만
남았으며 예상 밖 실패는 없다. 따라서 Q3-E1 진입을 막는 제품 회귀는 발견되지 않았다.

## 병합·overlap 정정

E0 checkpoint 뒤 원격은 `b54f20e39102`까지 전진했다. tip 간 diff에는 shaping·WASM·Studio·integration source
차이가 함께 보였지만, 이는 대부분 Q3 branch 전용 변경이었다. 공통 merge-base `955abb5268c3`에서 양측 변경 집합을
다시 계산한 결과 실제 교집합은 `mydocs/orders/20260829.md` 한 파일뿐이었다. 충돌은 양쪽 작업 기록을 모두 보존해
해소했다. 제품 source·Q3 test의 수동 충돌 정정은 없었다.

병합 뒤 branch는 `21 ahead / 0 behind`다. 이 정정은 최초 preflight의 overlap 해석을 폐기하고, 이후 동기화
판정은 tip diff가 아니라 merge-base 양측 변경 집합으로 수행해야 한다는 근거다.

## default baseline 재계측

기존과 같은 debug warm fixture를 회차당 64 layer build, 총 9회 실행했다.

| 항목 | 병합 후 | 최초 E0 대비 |
| --- | ---: | ---: |
| aggregate p50 | 199,119,239 ns | -1.090% |
| aggregate p95 (nearest-rank) | 205,214,939 ns | -1.040% |
| build당 p50 | 3,111,238.11 ns | -1.090% |
| build당 p95 | 3,206,483.42 ns | -1.040% |
| layer JSON bytes | 619,562 | 동일 |
| CanvasKit plan bytes | 1,755 | 동일 |

```text
204018421 198989376 199119239 203743911 198055997
205214939 198994581 200760043 197994528
```

- layer JSON BLAKE3:
  `0b5212cc076c34dce706039d7c4da85936c0a6769e83f08baa7f158cbd9029de`
- CanvasKit plan BLAKE3:
  `a98e6933d76901cfa55a43205c758675a8864527f87f8bed7a689d16267cbb56`
- 9회 hash·byte length 불일치: 0
- `GlyphOutline` 공개: 0

성능 수치는 개선 방향이고 Q3-E5의 release SLA가 아니라 동일 debug fixture의 회귀 탐지 기준이다.

## green·red 계약

- shadow context, red 제외: **13 passed / 0 failed**
- generated `regression_suite_018` atomic source: **4 passed / 0 failed**
- composition handoff: **4 passed / 0 failed**
- green 합계: **21 passed / 0 failed**
- `issue_4969_q3_e0_red`: **0 passed / 4 failed**, 예상 밖 실패 0

red 네 건은 각각 per-slot native clear, strict WASM set/clear, request-gated explicit session, atomic variable
outline publication 부재다. 최신 devel이 이 경계를 우연히 부분 구현하거나 실패 원인을 바꾸지 않았다.

## Docker WASM·Studio

최신 Rust 제품 source를 포함해 표준 `docker compose --env-file .env.docker run --rm wasm`을 새로 실행했다.
release compile과 `wasm-opt`가 6분 06초에 성공했고 Studio production build도 통과했다.

| 항목 | 병합 후 | 최초 E0 대비 |
| --- | ---: | ---: |
| `pkg/rhwp_bg.wasm` | 9,759,816 bytes | +448 bytes (+0.0046%) |
| WASM SHA-256 | `31110f71a477f044d013986c9f7cd067cdb2f7be92840dea1cb40f5fed63e051` | 변경 |
| Studio main JS | 1,699,677 bytes | 동일 |
| main JS SHA-256 | `7d381a746ff460e43fdb66768fdc3aa9ff85a732ff2cbd0456904db144c09f3d` | 변경 |
| Studio CSS | 106,910 bytes | 동일 |
| CSS SHA-256 | `17e81926a0b122e311840b76b70f7315ea5d8b4a420184aabdbac8ca6b482f57` | 동일 |
| Vite | 239 modules / 960 ms | 통과 |

WASM·JS hash 변화는 병합된 upstream 제품 source를 새로 빌드한 결과다. default layer·CanvasKit hash와 E0
green/red 계약이 그대로이므로 Q3 activation의 부분 구현이나 default 출력 회귀 증거는 아니다.

## 보호 불변식

- Q2 old-Hangul predicate·segment selector 변경: 0
- Q3-E1~E4 구현·public API·제품 activation: 0
- request 없는 modern Hangul·Latin 진입: 0
- 새 integration source·generated suite·manifest·Cargo marker stage: 0
- private corpus·Hyper-V·한컴 Oracle·static instance font 사용: 0
- Docker/Studio 산출물 tracked diff: 0

## 다음 승인 경계

재자격화 결과와 증적 checkpoint는 승인·고정됐다. checkpoint 직전 `upstream/devel@d29b5fd4ba40`의 PDF 정답지
갱신 2 commit이 추가됐고 Q3 변경 집합과 교집합은 없다. 이를 merge `351817b21`로 충돌 없이 통합해 branch는
`23 ahead / 0 behind`다. 다음 경계는 Q3-E1 reversible native owner 구현 착수 승인이다. push·PR·GitHub
comment는 계속 별도 승인이다.
