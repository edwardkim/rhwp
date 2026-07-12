# PR #1930 검토 — Task #1929: HWP5 그림 imgDim 왕복 소실 수정

- 작성일: 2026-07-05 / planet6897 → devel / 시간순 4/7 (05:16)
- 연결 이슈: #1929 (#1916 계열 raw-부재 재구성 결손)

## 요지
PICTURE extra 꼬리(18B)의 원본 크기 칸(offset 9..17)을 IR `img_dim`과 양방향 연결:
파서 적재 + 직렬화기 non-raw 경로에서 img_dim 우선 기록((0,0)은 crop 폴백 유지).

## 검토
- 바이트 오프셋 명세와 코드 일치(u32 LE ×2). native HWP5는 raw verbatim이라 2-round 안정.
- img_dim은 렌더 미소비 필드(HWPX 직렬화·게이트 전용) — 렌더 A/B 불필요 판단 타당.
- 인메모리 핀 2건 + 서베이 잔여 2건 실파일 3/3 보존 확인.

## 판단
머지 권고. #1929 close 가능(본문 명시 없음 — 승인 시 처리).

## 결합 게이트 결과 (devel `d982067b` + 7건 시간순 통합, 2026-07-05)

fmt 통과 / clippy 경고 0 / `cargo test --profile release-test --tests` **2,889 통과·실패 0**
(신규 핀 11건 포함) / OVR baseline 5샘플 **개체 회귀 0건** (기준 00014ecf)
