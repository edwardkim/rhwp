---
name: rhwp-codex
description: rhwp 에이전트 대전(Codex)으로 전 명령 표면을 항해합니다. mydocs/manual/agent_codex/ 의 정본 교본 — 철학 4규약(판정=데이터·결정론·출처 표지·원본 무훼손), 요청→명령 판단 트리, 71개 명령의 가족별 장(계약·출처 표지·실픽스처 실측 봉투 표본)을 순서대로 안내하고, 재생성·신선도 검사 절차까지 다룹니다. 트리거 — 사용자가 "rhwp 사용법/전체 명령/뭘 쓸지 모르겠다", "봉투 예시 보여줘", "명령 교본/코덱스", "대전 재생성/문서 신선도", "rhwp capabilities 항해" 등을 요청할 때.
---

# rhwp-codex — 에이전트 대전 항해 Skill

## 입장 순서 (30초)

1. [철학 4규약](../../../mydocs/manual/agent_codex/00_서문.md) — 판정은 exit 3
   봉투 데이터, 같은 계획=같은 바이트, 출처 표지(untrustedFields), 편집은
   `-o`+`--dry-run`.
2. [판단 트리](../../../mydocs/manual/agent_codex/01_판단트리.md) — 요청을
   7갈래(파악·수확·편집·변환·검증·보안·대량)로 갈라 장 번호를 얻는다.
3. 해당 장(10~85)의 **실측 표본**을 흉내낸다 — 표본은 전부 저장소 픽스처에
   실제로 돌린 봉투다(지어낸 예시 0). 명령을 못 찾으면
   `rhwp capabilities --search <키워드>`.

## 유지보수 (대전이 낡아 보이면)

```bash
cargo build --bin rhwp
python tools/gen_agent_codex.py          # 재생성 (표본 재실행)
python tools/gen_agent_codex.py --check  # 신선도 검사 — 차이면 exit 3
```

생성 장(frontmatter 에 `generated:` 표지)은 수기 수정 금지 — 생성기의 표본
계획·가족 표를 고쳐 재생성한다. 커버리지는 tests/agent_codex_contract.rs 가,
스킬-명령 정합은 스킬 표류 가드가 지킨다.

## 경계

- 봉투 **필드** 정의는 대전이 아니라 지식지도 §2-2 사전이 단일 출처다.
- 진단·프로브 장(85)은 개발자 표면 — 통상 문서 작업엔 쓰지 않는다.
