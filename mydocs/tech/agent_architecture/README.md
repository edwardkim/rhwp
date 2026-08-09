---
kind: guide
status: active
canonical: mydocs/tech/agent_architecture/layer_model.md
last_verified: 2026-08-03
---

# 에이전트 표면 아키텍처 문서 지도

`mydocs/tech/agent_architecture/`는 rhwp 에이전트 표면의 **층 구조와 작업 순서**를 다룬다.
개별 기능의 설계가 아니라 **"지금 무엇을 먼저 해야 하는가"** 가 이 축의 주제다.

---

## 왜 이 축이 생겼는가

2026-08-03 현재 열린 에이전트 표면 로드맵은 **일곱 개**다 —
[#3608](https://github.com/edwardkim/rhwp/issues/3608) ·
[#3719](https://github.com/edwardkim/rhwp/issues/3719) ·
[#3787](https://github.com/edwardkim/rhwp/issues/3787) ·
[#3793](https://github.com/edwardkim/rhwp/issues/3793) ·
[#3796](https://github.com/edwardkim/rhwp/issues/3796) ·
[#3828](https://github.com/edwardkim/rhwp/issues/3828) ·
[#3869](https://github.com/edwardkim/rhwp/issues/3869).

**각각은 근거가 탄탄한데, 서로가 서로를 모른다.** 예를 들어:

- [#3869](https://github.com/edwardkim/rhwp/issues/3869)(설치 없는 실행)와
  [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24(WASM 브라우저 표면)는
  **같은 축**인데, 오늘 각각 별도의 문서 디렉터리를 만드는 PR 이 동시에 열려 있다
- [#3828](https://github.com/edwardkim/rhwp/issues/3828) B3(레시피)은
  [#3787](https://github.com/edwardkim/rhwp/issues/3787) 이 만든 `inspect` 명령을 쓰는데,
  **어느 쪽도 상대를 전제로 적지 않았다**
- [#3796](https://github.com/edwardkim/rhwp/issues/3796)(재작업 제거)이 만든 선검사가
  **다른 여섯 로드맵 전부의 작업 순서**를 바꿨는데, 그 사실이 여섯 군데 어디에도 없다

층은 있는데 **층 사이의 순서가 없다.** 이 축이 그 자리를 채운다.

> **이슈는 닫히지만 판단은 남아야 한다.**
> [#3880](https://github.com/edwardkim/rhwp/issues/3880) §5 가 이 축의 신설을 수용 기준으로
> 요구한 이유다 — 순서에 대한 판단이 이슈 본문에만 있으면 이슈와 함께 사라진다.

---

## 이 축이 무엇이 아닌지

과장은 아키텍처 문서의 흔한 실패다. **하지 않는 것을 먼저 적는다.**

- **새 기능 제안이 아니다.** 이 축의 모든 항목은 이미 열린 이슈·PR 이거나
  실측으로 재현한 현재 동작이다. 새 명령·새 도구·새 마일스톤을 제안하지 않는다
- **로드맵을 대체하지 않는다.** 로드맵 본문의 권위는 각 이슈에 있다.
  이 축은 **그 사이의 순서**만 정한다
- **구현 스택의 재설계가 아니다.** [#3719](https://github.com/edwardkim/rhwp/issues/3719) 의
  6층 스택(엔진 → CLI → MCP → 세션 → 계획 → 매크로)은 그대로 유효하다.
  이 축의 4층은 **다른 축**이다 — [층 모델 §1](layer_model.md#1-층-기호-충돌--먼저-정리해야-할-것) 참조
- **진행률 대시보드가 아니다.** 숫자는 이틀이면 낡는다.
  이 축은 숫자 대신 **재현 명령**과 **막고 있는 것**을 적는다
- **품질 판정이 아니다.** 어느 로드맵이 더 낫다고 말하지 않는다.
  **어느 것이 어느 것 없이는 성립하지 않는지**만 말한다

---

## 읽는 순서

처음 읽는다면 위에서 아래로. 특정 작업 중이라면 해당 문서로 바로 간다.

| 순서 | 문서 | 언제 읽나 |
|---|---|---|
| 1 | **이 문서** | 축의 전제와 "지금 할 일"을 알아야 할 때 |
| 2 | [4층 성숙도 모델](layer_model.md) | **층 구분의 근거**를 알아야 할 때. **권위 문서** |
| 3 | [로드맵 7개 전수 지도](roadmap_atlas.md) | 특정 로드맵의 전제·겹침·종료 조건을 볼 때 |
| 4 | [불변식 전수](invariants.md) | 이 표면이 **지키기로 한 규칙**을 확인할 때 |
| 5 | [결정 대장](decision_log.md) | "왜 이렇게 했는지"를 뒤집기 전에 |
| 6 | [미해결 공백](open_gaps.md) | 무엇이 아직 안 닫혔는지 볼 때 |
| 7 | [관측성 계약](observability_contract.md) | R80 호출 통계의 수집·금지 경계를 보기 전에 |
| 8 | [MCP 스펙 개정 추종 대장](mcp_spec_ledger.md) | MCP 스펙이 개정됐을 때 — 우리 표면 접점과 추종 판정 절차 |

**새 조각을 착수하려는 사람**은 [층 모델 §6 층 판정 절차](layer_model.md#6-층-판정-절차--새-조각이-오면-어디에-넣나)
로 바로 가도 된다. 세 질문으로 그 조각이 어느 층인지 정한다.

**로드맵 하나를 손보려는 사람**은 [로드맵 지도 §3 중복·모순](roadmap_atlas.md#3-발견한-중복과-모순)
을 먼저 본다. 손대려는 곳이 이미 다른 로드맵과 겹칠 수 있다.

**기존 결정을 뒤집으려는 사람**은 [결정 대장](decision_log.md)을 먼저 본다.
뒤집는 것이 문제가 아니라 근거를 모른 채 뒤집는 것이 문제다.

---

## 네 층 요약

| 층 | 질문 | 속하는 로드맵 | 오늘 |
|---|---|---|---|
| **L1 표면** | 있는가 | [#3608](https://github.com/edwardkim/rhwp/issues/3608) · [#3719](https://github.com/edwardkim/rhwp/issues/3719) | 넓다. **정합이 안 맞는다** |
| **L2 신뢰** | 믿을 수 있는가 | [#3787](https://github.com/edwardkim/rhwp/issues/3787) · [#3793](https://github.com/edwardkim/rhwp/issues/3793) · [#3796](https://github.com/edwardkim/rhwp/issues/3796) · M21 | 문서·탐지 착지. **퍼징 CI 0** |
| **L3 도달** | 쓸 수 있는가 | [#3828](https://github.com/edwardkim/rhwp/issues/3828) · [#3869](https://github.com/edwardkim/rhwp/issues/3869) · M18~M20·M24 | **devel 에 거의 없다** |
| **L4 표준** | 남는가 | M26~M30 | 미착수. **지금은 옳다** |

**핵심 판단** — **L1 의 구멍이 L3 를 막는다.**
봉투 키 하나, 자기서술 문장 하나 같은 작은 것이, 바인딩·WASM·유입 다리 같은 큰 것을
막는다. 실례 8건의 전수는 [층 모델 §4](layer_model.md#4-l1-의-구멍이-l3-를-막는다--실례-8건) 에 있다.

---

## 지금 무엇을 해야 하는가 — 현재 답

> [#3880](https://github.com/edwardkim/rhwp/issues/3880) §3 의 순서를 2026-08-03 실측으로
> 보강한 것이다. **근거 없는 순서는 취향이므로 항목마다 근거를 붙인다.**
> 전문과 #3880 과의 차이는 [층 모델 §7](layer_model.md#7-지금-무엇을-먼저-해야-하는가).

### 1. L1 봉투 구멍 중 미착수 4건

[#3880](https://github.com/edwardkim/rhwp/issues/3880) T1·T3 은 PR
[#3882](https://github.com/edwardkim/rhwp/pull/3882) 로 이미 열렸다. 남은 것은 넷이다.

| 구멍 | 실측 증상 | 상태 |
|---|---|---|
| T2 `--json` 침묵 무시 | `rhwp dump <문서> --json` → exit 0, 사람 텍스트 18,642 B | 정책 판단 필요 |
| T4 실패 stdout 예외 | `run` 194 B · `bench --json` 407 B (사전은 "0바이트") | 계약 테스트 걸림 |
| B5 하위 명령 미선언 | `rhwp inspect --json <문서>` → exit 2. `edit` 도 동일 | **미제기** |
| B6 `-o` 조합 봉투 소실 | `export-tables … -o … --json` → 사람 문장 124 B | **이슈 미승격** |

**근거** — 넷 다 **L3 의 논리적 전제**다. 특히 B6 는
[#3869](https://github.com/edwardkim/rhwp/issues/3869) W2("WASM 봉투가 CLI 와 같은 모양")의
기준을 정의 불가로 만든다. 그리고 넷 다 작고 서로 독립적이다.

### 2. 열린 PR 22건 → 10건 이하

**근거** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §7 과
[#3796](https://github.com/edwardkim/rhwp/issues/3796) §6 이 **"열린 PR 은 10건 내외"** 를
스스로 못박았는데 오늘 실측 22건(12건이 하루에 개설)이다.
L3 조각이 **전부** 이 큐 안에 있으므로, 큐가 안 빠지면 1번을 아무리 잘 해도
L3 는 devel 에 도달하지 않는다.

### 3. 선검사를 나머지 여섯 로드맵에 명시

**근거** — `tools/agent_preflight.py` 가 devel 에 실재하는데
여섯 로드맵 어디에도 언급이 없다. 빌드 3회 → 1회는 조각당 12분이다.
동시에 선검사 자신의 오탐(PR [#3872](https://github.com/edwardkim/rhwp/pull/3872))을 닫는다 —
오탐 하나가 잘못된 결함 보고까지 만들어냈다
([로드맵 지도 §3.7](roadmap_atlas.md#37-d7--선검사가-여섯-로드맵-어디에도-없다)).

### 4. WASM 축 통합 판정

**근거** — [#3869](https://github.com/edwardkim/rhwp/issues/3869) 와
[#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 가 같은 축인데
**문서 디렉터리 2개**(`agent_runtime/` · `wasm_agent_surface/`)가 동시에 열려 있고,
겹치는 문서가 절반 가까우며 **둘 다 canonical** 이다.
판단과 선택지는 [로드맵 지도 §3.2](roadmap_atlas.md#32-d2--wasm-축이-두-곳에서-자란다).

### 5. M21 퍼징 CI 편입

**근거 (실측)**

```
$ ls fuzz/fuzz_targets/ | wc -l      → 6      (로드맵은 4종이라 적는다)
$ grep -ril fuzz .github/ | wc -l    → 0      (CI 어디에서도 안 돈다)
```

DoD 를 "타깃이 있다"가 아니라 **"CI 에서 돈다"** 로 바꾼다.

### 6. M18~M20 표류 20건 정리

**근거** — [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 Python·Node 전수 비교로
표류 20건(치명 3건 실행 확인)을 찾았다. **바인딩이 늘기 전에** 정리해야 예외가
언어 수만큼 복제되지 않는다.

### 7. L4 착수 판단

**지금은 하지 않는다.** [#3608](https://github.com/edwardkim/rhwp/issues/3608) §8 말미의
원칙 — "근거 없는 항목은 넣지 않는다" — 을 착수 판단에 적용하면, M26~M30 의 근거가 될
외부 채택 실측이 아직 없다. **L3 가 굳은 뒤 다시 본다.**

---

## 근거 규율

이 축의 문서는 다음을 지킨다. 어기면 되돌린다.

1. **모든 주장에 근거** — 이슈 번호 · PR 번호 · 코드 경로 · 실측 명령 출력 중 하나
2. **근거를 못 대면 "확인되지 않음"** — 각 문서 말미에 목록으로 모은다.
   추측을 사실처럼 적은 아키텍처 문서는 반년 뒤 잘못된 우선순위의 근거가 된다
3. **"착지"는 "머지"만** — PR 개설은 "제출"이다.
   열린 PR 을 진행률에 합산하면 로드맵이 낙관 쪽으로 왜곡된다
   ([#3719](https://github.com/edwardkim/rhwp/issues/3719) §2 규율,
   [로드맵 지도 §3.10](roadmap_atlas.md#310-d10--열린-pr-을-이미-해결됐다로-계산한다) 참조)
4. **숫자보다 재현 명령** — 표면이 이틀에 절반씩 바뀐다.
   숫자를 적을 때는 반드시 측정일과 재현 명령을 함께 적는다
5. **중복·모순을 숨기지 않는다** — 발견한 것이 이 축의 성과다

---

## 측정 기준

이 축의 모든 실측은 **2026-08-03**, `<저장소>/target/release/rhwp.exe`(`rhwp v0.8.2`,
로컬 릴리스 빌드)와 `upstream/devel` 기준 워크트리에서 얻었다.

재현 명령:

```bash
# 표면 규모
rhwp capabilities | jq '{명령:(.commands|length),
                          json:([.commands[]|select(.json)]|length),
                          게이트:([.commands[]|select(.requiresFeature)]|length)}'
rhwp capabilities --mcp | jq '.tools | length'
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | rhwp mcp-serve \
  | jq -r '.result.tools[].name' | grep -cE '^hwp_(open|close|doc_)'

# 계약 테스트 규모
grep -h '^#\[test\]' tests/*contract*.rs | wc -l

# 퍼징 CI 편입 여부
grep -ril fuzz .github/ | wc -l
```

2026-08-03 결과: 명령 **61** · `--json` **31** · MCP 무상태 **39** · 서버 총 노출 **51** ·
세션 **12** · 계약 테스트 **523**(61파일) · 퍼징 CI 참조 **0**.

**주의** — 측정에 쓴 바이너리와 `upstream/devel` 의 커밋 동일성은 확인하지 않았다.
간접 근거는 [층 모델 §9 U-1](layer_model.md#9-확인되지-않음).

---

## 이 축을 갱신하는 법

- **문서 역할 분담** — 층 구조는 [층 모델](layer_model.md), 로드맵별 사실은
  [로드맵 지도](roadmap_atlas.md), 지켜야 할 규칙은 [불변식](invariants.md),
  결정의 논증은 [결정 대장](decision_log.md), 미해결 항목은 [공백 목록](open_gaps.md).
  **같은 사실을 두 문서에 적지 않는다** — 한 곳에 적고 나머지는 링크한다
- **로드맵이 하나 닫히거나 새로 열리면** [로드맵 지도](roadmap_atlas.md) §1·§2 를 갱신한다
- **[층 모델 §4](layer_model.md#4-l1-의-구멍이-l3-를-막는다--실례-8건) 의 구멍이 닫히면**
  해당 항목의 "상태"를 머지 링크와 함께 갱신한다. **PR 개설로는 닫지 않는다**
- **새 중복·모순을 발견하면** [로드맵 지도 §3](roadmap_atlas.md#3-발견한-중복과-모순) 에
  D 번호를 붙여 추가한다. 해소된 항목은 지우지 말고 해소 근거를 적는다
- **실측 수치를 갱신할 때는** 측정일과 재현 명령을 함께 바꾼다.
  숫자만 바꾸면 다음 사람이 어떻게 잰 건지 모른다

---

## 관련 문서

### 인접 축

| 축 | 무엇 | 이 축과의 관계 |
|---|---|---|
| [에이전트 보안](../agent_security/README.md) | 문서가 에이전트를 조종하는 경로 | **L2 의 권위**. [#3793](https://github.com/edwardkim/rhwp/issues/3793) 산출물 |
| [경량 에이전트 내성](../weak_agent_proofing.md) | 약한 모델의 오사용 방지 | L1 내성 계약의 전신 |
| [에이전트 경계 무결성 계약](../agent_boundary_contract.md) | 경로·자원·핸들 경계 | L2 경계 계약 |
| [외부 바인딩 공통 기반](../bindings_foundation.md) | IR 스키마 버저닝·파이썬 1호 | L3 바인딩 축(M18~M20) |

### 절차

- [에이전트 표면 플레이북](../../manual/agent_surface_playbook.md) — 표면 추가의 절차·수용 기준
- [에이전트 선검사 가이드](../../manual/agent_preflight_guide.md) — [#3796](https://github.com/edwardkim/rhwp/issues/3796) 의 도구
- [에이전트 지식 지도](../../manual/agent_knowledge_map.md) · [에이전트 실패 사전](../../manual/agent_troubleshooting_guide.md)
- [CLI 명령 레퍼런스](../../manual/cli_commands.md) — **현재 동작은 항상 `rhwp capabilities` 로 재확인**

### 이슈

- [#3880](https://github.com/edwardkim/rhwp/issues/3880) — 탑다운 로드맵. 이 축의 발원
- 일곱 로드맵: [#3608](https://github.com/edwardkim/rhwp/issues/3608) ·
  [#3719](https://github.com/edwardkim/rhwp/issues/3719) ·
  [#3787](https://github.com/edwardkim/rhwp/issues/3787) ·
  [#3793](https://github.com/edwardkim/rhwp/issues/3793) ·
  [#3796](https://github.com/edwardkim/rhwp/issues/3796) ·
  [#3828](https://github.com/edwardkim/rhwp/issues/3828) ·
  [#3869](https://github.com/edwardkim/rhwp/issues/3869)
