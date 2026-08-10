# task_m100_4543 stage1 — 5년 축 앵커 (투명성 로그) Y5-M1~M3

- 이슈: #4543 (설계서 #4448 착공) / 브랜치: task_m100_4543 (스택 최상단 적층)
- 산출물: src/anchor_log.rs(줄 해시 체인·머클 코어) + anchor add/checkpoint/
  verify + lineage --anchor-log(anchoredOk 6번째 축) + 7+1표면 + 계약 3종 +
  대전 재생성(75명령 델타 2장)

## 설계 이행 (horizon_year5_anchor.md 그대로)

- prevEntryHash = 직전 **줄 원문 바이트** SHA-256 — 캡슐(파일 바이트)·서명
  (파일 바이트)과 같은 대상 규약, 세 체계 정합.
- 깨진 로그에는 append 거부(exit 3) — 변조 위에 도장 찍기 금지.
- 머클: 루트 산출·경로 증명·검증(잎→루트 재계산) — 7년 축(연합 번들)이 이
  경로를 실어 나를 토대.
- 도구 경계 정직: 공표는 운영 절차 — 봉투 어떤 필드도 공표를 주장하지 않음.
- anchoredOk 미등재 = false 이되 체인 안 깨짐(등재 강제는 게이트 직무 — 축마다
  한 일), opt-in 무파손(signerOk 선례).

## 실측

- anchor_contract 3/3: 등재 3건 연번 → 체크포인트(root 64hex) → verify 머클
  경로 참 → 미등재 exit 3 → **중간 줄 1글자 변조**를 다음 줄 기록 해시가
  폭로 + 등재 거부 → lineage 축 opt-in/데이터 판정.
- 가드: cli_json·router·provenance(+빈 로그 미등재 레시피)·사전(+9행, 212)·
  하네스/서명/영수증/감사/계보 회귀 전건 · node 466(패리티 우산 표 anchor) ·
  봉투 44 · 대전 --check 멱등 · clippy 0.
