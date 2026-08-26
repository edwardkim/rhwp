---
kind: investigation
status: active
canonical: mydocs/tech/agent_roadmap/harness_scorecard.md
last_verified: 2026-08-15
---

# 하네스 스코어카드 — 주장마다 실행 명령 (#4389)

이번 주 업계 소식은 에이전트 하네스가 "관리형 플랫폼"으로 옮겨가는 흐름을
보여준다(대형 프레임워크의 하네스 GA 등 — 접속 2026-08-10:
[InfoQ](https://www.infoq.com/news/2026/08/agent-framework-harness-ga/),
[8개 SDK 지형](https://www.morphllm.com/ai-agent-framework)). 관리형 런타임의
공통 성질은 하나다 — **검증은 운영자만 할 수 있다.**

이 저장소는 반대편에 선다: **모든 하네스 성질 주장에 실행 명령이 달리고,
제3자가 클론 후 명령 하나로 전수 검증한다.** 비교는 실명 서열이 아니라 이
원리 하나로만 한다(전 조사 문서 공통 규율).

```bash
python tools/harness_proofs.py        # 6개 성질 실검증 — 하나라도 깨지면 exit 1
```

## 실검증 8종 (러너가 지금 판정 — 2026-08-15 로컬 실측 8/8 PASS)

| # | 성질 | 검증 명령 |
|---|---|---|
| P1 | **자기서술 결정론** — capabilities 2회 호출이 바이트까지 동일(모델·시각 무개입) | `rhwp capabilities` ×2 비교 |
| P2 | **명령 표면 전수 자기서술** — 모든 명령이 계약(exitCodes·jsonContract) 동봉, 표면 하한 68 | `rhwp capabilities` |
| P3 | **사용법 오류 사전** — 미지 옵션 = exit 2 + stdout 0바이트(반쪽 JSON 금지) | `rhwp info … --nope --json` |
| P4 | **실패 stdout 순수성** — 런타임 실패도 stdout 무오염(exit 1 + 0바이트) | `rhwp info no_such.hwp --json` |
| P5 | **출처 표지 S1** — 문서 파생 값의 신뢰 경계를 봉투가 스스로 밝힘 | `rhwp info <doc> --json` |
| P6 | **explain 결정론** — 서술이 생성 문장이 아니라 조립(드리프트 가드 가능 조건) | `rhwp explain <doc> --json` ×2 |
| P7 | **본문 도달성** — 원시 바이트를 어떻게 디코딩해도(UTF-8·UTF-16LE) 안 나오는 본문을 도구는 준다 | `rhwp export-text <doc> --json` + 원시 바이트 대조 |
| P8 | **주소 왕복** — `search` 가 준 쪽 주소가 `export-text` 가 그 줄을 실은 쪽과 일치 | `rhwp search <doc> "<본문 줄>" --json` |

P1~P6 은 전부 **도구가 예의 바른가**를 묻는다(결정론·종료 코드·stdout 순수성·표지).
P7~P8 이 처음으로 **이 도구가 없으면 못 하는 일이 무엇인가**를 묻는다(#4868).

- P7 실측(2026-08-15, `samples/hwp3-sample.hwp`): 본문 줄 425개 중 **420개(98.8%)** 가
  원시 디코딩 어디에도 없다. 판정 임계는 과반(50%)이다 — 표본 하나의 수치를 성질로
  굳히지 않는다. UTF-16LE 를 함께 대조하는 것은 공정성 때문이다(한글 문서의 여러
  바이너리 포맷이 UTF-16LE 로 문자열을 담으므로 UTF-8 만 보면 허수아비가 된다).
- P8 실측: 표적 줄(78자)에 대해 `search` 가 `page=15`, `export-text` 도 같은 쪽. 원시
  바이트 경로는 그 줄을 못 찾으므로 돌려줄 좌표 자체가 없고, 찾더라도 바이트 오프셋은
  쪽·문단 어느 좌표계로도 번역되지 않는다.
- P2 는 명령 수 **정확 일치**에서 **하한**으로 바뀌었다(#4870). 정확 일치이던 동안 표면이
  68→85 로 자라 러너가 상시 red 였다 — 상시 red 인 게이트는 게이트가 아니다.

## 계약 테스트·PR 로 검증되는 4종 (러너 밖 — 거짓 PASS 를 만들지 않는다)

| # | 성질 | 검증 경로 |
|---|---|---|
| C1 | **CAS 편집 유실 차단** — 계획 `preconditions.inputSha256`·`edit --expect-sha256`, 불일치=실행 0·저장 0 | `tests/run_plan_cas_contract.rs` 6본 · [PR #4381](https://github.com/edwardkim/rhwp/pull/4381) |
| C2 | **변이 자기검증 저널** — 워크스페이스에서 변이마다 본문 SHA-256 전/후 자동 기록 | `tests/mcp_workspace_contract.rs` · [PR #4361](https://github.com/edwardkim/rhwp/pull/4361) |
| C3 | **소비자 버전 대사** — `capabilities.schemaRegistry` 로 전 계약 축 기계 대조 | `tests/schema_registry_contract.rs` 4본 · [PR #4330](https://github.com/edwardkim/rhwp/pull/4330) |
| C4 | **공개 판정 실험** — 무안내 30분 첫 유효 산출, 측정 대장 공개 | [#4355](https://github.com/edwardkim/rhwp/issues/4355) · 프로토콜 [PR #4356](https://github.com/edwardkim/rhwp/pull/4356) |

## 운영 규약

1. 새 하네스 성질 주장은 **이 표에 행이 생겨야 주장이 된다** — 러너 검사 또는
   계약 테스트·PR 링크 없는 주장은 싣지 않는다.
2. C행이 devel 에 착지하면 가능한 것부터 러너(P행)로 승격한다.
3. 러너는 CI 편입 후보다(R12 CI 상시화와 합류) — 편입 전까지는 로컬·PR 실측을
   문서에 남긴다.
4. 신간·업계 대사는 `trend_harness_2026w32.md` 계열([PR
   #4385](https://github.com/edwardkim/rhwp/pull/4385) 리뷰 중 — 상대 링크는
   착지 후)이 담당하고, 이 문서는 **검증 가능한 성질의 대장**만 유지한다. W33 판은
   [trend_harness_2026w33.md](trend_harness_2026w33.md) — 범용 하네스의 플러그인화
   흐름과, 그 흐름이 도메인 도구에 남기는 자리(P7·P8)를 대사한다.
5. 러너의 임계는 **성질이 살아남을 만큼 느슨하게** 둔다. 표본 하나의 실측값을 그대로
   임계로 박으면 표본이 바뀔 때 성질이 아니라 표본을 검사하게 되고, 명령 수처럼
   자라는 값을 정확 일치로 박으면 게이트가 상시 red 가 된다(#4870 의 교훈).
