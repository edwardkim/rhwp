# #4467 Stage 2 완료 — 한국어 독자를 위한 로드맵 문체 보정

- **Issue**: [#4467](https://github.com/edwardkim/rhwp/issues/4467)
- **브랜치**: `task/4467-project-roadmap`
- **피드백**: [로드맵의 독자와 문체](../feedback/task_m100_4467_roadmap_audience.md)
- **완료일**: 2026-08-10 KST

## 1. 작업지시자 판정

프로젝트 로드맵의 주 독자는 프로젝트에 관심을 갖는 사람이며, 한국어를 모국어로 사용하는 독자를
기준으로 어휘와 논리적 전개를 다듬어야 한다. Stage 1 문서는 운영자와 AI 에이전트가 읽는 내부 문서처럼
관리 규칙과 외래어를 앞세운 문제가 있었다.

## 2. 논리 전개 보정

문서 순서를 다음과 같이 바꿨다.

1. 우리가 만들고 싶은 것
2. 지금 어디까지 왔는가
3. v0.5~v3.0 버전별 목표
4. 버전과 함께 계속 다듬는 일곱 분야
5. AI 활용 세부 로드맵의 위치
6. 상태 표시와 문서 관리 방법
7. 로드맵의 변천

각 버전은 “무엇을 이루는 단계인가 → 무엇을 다듬는가 또는 시작 전 확인할 것 → 언제 완성으로
보는가” 순서로 설명했다. 운영 규칙과 문서 권위 표는 독자가 제품 방향을 이해한 뒤 읽도록 뒤로 옮겼다.

## 3. 어휘 보정

- `canonical` → `기준 문서`
- `DoD` → `완료 기준`, `언제 완성으로 보는가`
- `착수 게이트` → `시작하기 전에 확인할 것`
- `release gate` → `출시 전 검증`
- `fixture` → `재현용 샘플`
- `corpus` → `실제 문서`, `실물 문서 모음`
- `P1~P7` → `문서 형식과 내부 구조`, `AI 활용과 자동화`처럼 분야 이름을 직접 표기
- `merge` → `병합`

HWP, HWPX, JSON, MCP처럼 기능을 식별하는 데 필요한 이름은 유지하고, WebAssembly는 첫 등장에
`웹어셈블리(WebAssembly)`로 함께 표기했다.

## 4. 연결 문서 정리

README의 안내 문구도 “현재 근거·착수 게이트·완료 정의”에서 “버전별 목표와 완료 기준, 지금 하는 일”로
바꿨다. `llms.txt`, 기술 문서 지도와 에이전트 로드맵 진입점의 `P5` 표기도 모두 “AI 활용과 자동화”로
교체했다.

## 5. 검증

```text
python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references
검사 문서: 532개 / 변경 파일: 12개 / redirect stub: 30개
내부 Markdown 상대 링크: 이상 없음

python3 scripts/check_document_metadata.py
메타데이터 검사 문서: 521개
문서 메타데이터: 이상 없음

git diff upstream/devel --check
통과

ROADMAP.md 공개 문체 금지어 재검색
본문의 canonical / DoD / 착수 게이트 / P1~P7 / fixture / corpus / release gate 없음
```

문서 보정만 수행했으므로 Rust·Studio·WASM·시각 테스트는 대상이 아니다.
