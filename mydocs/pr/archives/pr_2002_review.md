# PR #2002 검토 — Issue #1995: 다수 전면 non-TAC 이미지를 각 페이지 단독 배치

- 작성일: 2026-07-07 / planet6897 → devel / CI 11 pass, MERGEABLE
- 연결 이슈: #1995 (5차 10k 서베이 #1996 발견분)

## 요지

과소 페이지 대표 케이스(청년주거 연구, rhwp 171 vs 한글 268)의 근본 원인을 **진단 정정**
— RowBreak 표 문제가 아니라 **한 문단의 전면 이미지 96장이 단일 앵커에 스택**되던 결함.
수정: 문단 내 본문높이 60%↑ non-TAC 그림이 **2장 이상**이면 각각 `force_new_page` 단독
배치 (속성 기반 가드, 단일 그림 문단 불변). 부수: InFrontOfText/BehindText **tac 표**의
zero-height 방출 예외 2곳(typeset/pagination — 인라인 콜아웃 흐름 높이 예약).

## 검토

- 진단 정정(서베이 상관관계의 우연성 규명)과 속성 기반 가드가 정확. 결과 171→**270**
  (한글 오라클 pyhwpx 268, +2 잔여는 기록·후속).
- tac 표 예외는 자기 핀(1086/1156/1692) + byeolpyo/편람 무회귀로 검증 — 로컬 OVR의
  issue1835(TAC 표 샘플)도 회귀 0 확인.
- typeset.rs +104줄 — 재성장 감시 축 누적 (Phase P 흡수 대상 목록에 반영).
- 재현 파일(12.9MB)은 미포함, 합성 회귀 테스트로 대체 — 타당.

## 게이트 결과 (devel `b2fa0f5d` + PR)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 11 pass / 1 skip |
| cargo fmt / clippy | 통과 / **경고 0** |
| cargo test --profile release-test --tests | **2,913 통과 / 실패 0** (신규 핀 포함) |
| OVR baseline 5샘플 | **추가 변동 0** (기지 #1936발 3건 동일 — tac 표 예외의 issue1835 영향 없음) |
| **시각 판정** | **통과** (작업지시자, WASM 빌드 studio 확인, 2026-07-07) |

## 판단

머지 권고 — 시각 판정 포함 전 게이트 통과. #1995는 `Closes` 선언 없음 + 잔여 +2 기록
있음 — close 여부는 작업지시자 판단.
