# rhwp-work-receipt 레퍼런스

SKILL.md 는 라우터다. 여기서 해당 단의 본문을 읽는다.

| 파일 | 단 | 고정하는 계약 |
|------|----|----------------|
| [replay-attest.md](replay-attest.md) | 영수증 | 3해시, attest/verify, `--expect-output-sha256` |
| [capsule-chain.md](capsule-chain.md) | 캡슐 | `--capsule` / `--parent`, 불변, 상대 경로 |
| [audit-accounting.md](audit-accounting.md) | 감사 | 비재귀 `*.capsule.json`, `reproducedRate` |
| [lineage-chronicle.md](lineage-chronicle.md) | 계보 | `parentOk` · `lineageOk` · `reproduced` · `brokenAt` |
| [exit-codes.md](exit-codes.md) | 공통 | exit 3 = 판정, 1 = IO, 2 = 사용법 |
| [pitfalls.md](pitfalls.md) | 공통 | `toolVersion`, 귀속/서명 비주장 |
| [decision-tree.md](decision-tree.md) | 라우팅 | 요청 → 단 → 명령 |
| [envelope-field-catalog.md](envelope-field-catalog.md) | 사전 | 키·타입·null 의미 |
| [recipe-index.md](recipe-index.md) | 색인 | 워크스루·픽스처 교차표 |

생성기 [`_gen_pack.py`](_gen_pack.py) 는 픽스처·예제 골격을 다시 만든다.
수기로 고친 레퍼런스 md 는 생성기가 덮어쓰지 않는다.
