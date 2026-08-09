---
kind: investigation
status: active
canonical: mydocs/tech/agent_roadmap/trend_agent_runtime_2026h2.md
last_verified: 2026-08-09
---

# 동향: 에이전트 전용 런타임 — 웹의 선례와 rhwp workspace 제안 (W1, #4351)

[trend_survey_2026h2.md](trend_survey_2026h2.md) 의 후속 동향 1건이다. 같은
5관문(출처 실재·3판정·논지 기여·게이트 서술·캡)을 지나며, **제안이고 채택·번호
부여는 메인테이너 몫**이다. 이번 캡은 1건(W1)이다.

## 1. 동향 실측 — "사람용 표면을 제거한 런타임"이 실물이 됐다

Cloudflare 가 2026-08-07 **에이전트 전용 브라우저 엔진 Kitesurf** 를 발표했다.
요지(전부 아래 출처에서 확인):

- 사람용 브라우저의 표면(탭·테마·확장·시각 장식)을 제거하고, 에이전트가 실제로
  쓰는 것만 남겼다 — 구조 읽기·추출, 폼 채우기, 스크린샷(그라운딩), JS 실행,
  내비게이션.
- Chromium 없이 **Rust 로 새로 구현**, V8 isolate(Workers) 상주 실행. 메모리
  3~7배·CPU 약 3배 절감, Web Platform Tests 215,000+ 통과. 베타 무료, 오픈소스
  계획 언급.
- 한계도 공개: 비디오·WebGL 미지원, 복잡 페이지는 여전히 Chromium 필요.

출처(접속 2026-08-09):
[TechCrunch](https://techcrunch.com/2026/08/07/cloudflare-launches-kitesurf-a-browser-built-for-ai-agents/) ·
[MarkTechPost](https://www.marktechpost.com/2026/08/06/cloudflare-introduces-kitesurf-an-agent-first-web-browser-that-runs-entirely-in-v8-isolates-on-cloudflare-workers/) ·
[Mac Observer](https://www.macobserver.com/news/cloudflare-launches-kitesurf-browser-exclusively-for-ai-agents/)

**원리 추출**(실명 비교가 아니라 원리로): ① 에이전트 소비자는 픽셀이 아니라
**구조와 결정론**을 산다. ② 사람 표면을 빼면 자원·기동 시간이 크게 줄어 상주가
싸진다. ③ "무엇을 못 하는지"의 공개가 신뢰 표면이다.

## 2. rhwp 에의 사상(寫像) — 웹의 Kitesurf = 문서의 workspace

rhwp 의 "사람용 표면"은 studio(웹 에디터)다. 에이전트 축은 이미 CLI `--json`·
`mcp-serve` 세션(R71)·자기서술을 갖고 있으나, **상주 런타임으로서의 결합**이
비어 있다. 제안 W1:

### W1 `rhwp workspace` — 에이전트 전용 문서 런타임 (설계 제안)

- **한 줄** — 문서 N 개를 한 상주 프로세스에 열고, 픽셀 대신 **안정 노드 ID
  구조 트리**로 보고, 같은 ID 로 행동하고, 매 변이를 스스로 검증한다.
- **모양** — ① 워크스페이스: `mcp-serve` 확장(예: `--workspace <dir>`)으로 다중
  핸들 + 코퍼스 색인. ② 뷰: 문서→구역→쪽→블록/표/셀/필드의 안정 ID 트리 —
  a11y 트리의 문서판(`export-structure` 를 "상주·증분·안정 ID"로 승격). ③ 액션:
  기존 편집 표면을 노드 ID 주소로 라우팅. ④ 신뢰 루프: 매 변이 후 자동
  digest/verify 대조 + 저널(트랙 C 지문 체인과 접합). ⑤ 한계 공개: 렌더 픽셀이
  필요한 판정(시각 회귀)은 범위 밖 — render-diff 로 위임한다고 명시.
- **3판정: 부분** — 세션 3종·digest·verify·export-structure·프로파일이 전부
  실재한다. 공백은 결합(상주 멀티 문서 + 안정 ID + 자동 검증 루프)이지 부품이
  아니다. 신규 축이 아니라 **R76(문서 지능 서버)·R71·트랙 C 의 합류점**이다.
- **착수 게이트** — ① R76 의 판단("증분 재파싱 불가 전제에서 무엇을 캐시하나")과
  **같은 질문이므로 합류한다** — 별도 답을 만들면 드리프트다. ② 트랙 C R28(세션
  동시성 모델) 판단. ③ 메인테이너 채택(표면 신설이므로).
- **왜 1등 논지에 닿나** — HWP 도메인에 이 표면은 존재하지 않는다(#4327 §2·§3:
  경쟁 도구들은 변환·COM 자동화 층위). 웹에서 같은 원리가 대형 사업자의 실물로
  검증된 직후가, 문서 도메인에서 같은 자리를 선점할 시점이다. 엔진·세션·계약은
  이미 있고 남은 것은 결합이라 비용이 낮다.
- **DoD(채택 시)** — 에이전트가 워크스페이스를 열어 ID 트리로 문서를 찾고, ID
  액션으로 편집하고, 자동 verify 저널이 남는 왕복 1개가 계약 테스트로 고정된다.

## 3. 하지 않는 것

- "OS"·범용 런타임 주장 — 문서 도메인 밖은 범위가 아니다(비목표를 명시하는 것
  자체가 §1 원리 ③이다).
- R76 과 별개의 캐시 설계 — 게이트 ① 대로 합류만 한다.
- 이 문서에서의 구현 착수 — 채택 전 구현 금지(로드맵 운영 규칙 1).
