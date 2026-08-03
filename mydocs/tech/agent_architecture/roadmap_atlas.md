---
kind: canonical
status: active
canonical: mydocs/tech/agent_architecture/roadmap_atlas.md
last_verified: 2026-08-03
---

# 열린 에이전트 로드맵 7개 전수 지도

> 2026-08-03 기준 열린 에이전트 표면 로드맵은 **일곱 개**다.
> 각각은 근거가 탄탄한데, **서로가 서로를 모른다.**
> 이 문서는 일곱을 전수로 읽고 **층 배치 · 전제 · 상호 의존 · 중복 · 모순**을 확정한다.
> 층 모델의 논증은 [4층 성숙도 모델](layer_model.md), 축 진입점은 [README](README.md),
> 원 제안은 [#3880](https://github.com/edwardkim/rhwp/issues/3880).

**이 문서의 목적은 순위표가 아니라 지도다.** 어느 로드맵이 더 중요한지가 아니라,
**어느 것이 어느 것 없이는 성립하지 않는지**를 적는다.

모든 주장에 **이슈 번호 · PR 번호 · 코드 경로 · 실측 출력** 중 하나가 붙는다.
근거를 대지 못하는 항목은 **"확인되지 않음"**(§5)으로 적었다.

**측정 기준** — 실측은 2026-08-03, `<저장소>/target/release/rhwp.exe`(`rhwp v0.8.2`)와
`upstream/devel` 기준 워크트리. 바이너리 커밋 동일성은
[확인되지 않음](layer_model.md#9-확인되지-않음) U-1.

---

## 0. 읽는 법

### 0.1 "머지"의 정의 — 먼저 못박는다

이 문서는 **"닫힌 PR" 을 "머지" 로 세지 않는다.** 실측하면 전체 200건 중
OPEN **22** · CLOSED **167** · MERGED **11** 이고, **`mergedAt` 값이 있는 것은 11건뿐**이다
(`gh pr list --author kevin9327 --state all --limit 200 --json number,state,mergedAt`).

그래서 이 문서는 **산출물이 워크트리에 실재하는지**를 개별 확인한 것만 "착지"로 적는다.
확인 방법은 파일 존재(`ls`)·소스 문자열(`grep`)·바이너리 동작(실행) 셋이다.

### 0.2 각 로드맵 항목의 형식

**한 줄**(무엇을 약속했나) → **층**([성숙도 사다리](layer_model.md#12-명명-규약--이-축이-쓰는-표기) L1~L4)
→ **전제**(시작되려면 무엇이 먼저 있어야 하나) → **막는 것**(없으면 무엇이 못 나아가나)
→ **진행률**(체크박스 + 산출물 실재) → **겹침** → **종료 조건**(끝났다고 말할 조건).

가운데 둘 — **전제와 막는 것** — 이 이 문서의 핵심이다. 나머지는 그 판단의 근거다.

---

## 1. 한눈에

### 1.1 층 배치

| 이슈 | 개설 | 한 줄 | 층 |
|---|---|---|---|
| [#3608](https://github.com/edwardkim/rhwp/issues/3608) | 07-30 | 마일스톤 현황판 M1~M30 | **L1** (+ M18~M20·M24 는 L3, M21 은 L2, M26~M30 은 L4) |
| [#3719](https://github.com/edwardkim/rhwp/issues/3719) | 08-01 | 6층 구현 스택의 상위 지도 | **L1** |
| [#3787](https://github.com/edwardkim/rhwp/issues/3787) | 08-02 | 간접 인젝션 내성 구현 S1~S10 | **L2** |
| [#3793](https://github.com/edwardkim/rhwp/issues/3793) | 08-02 | 보안 문서 축 신설 11편 | **L2** |
| [#3796](https://github.com/edwardkim/rhwp/issues/3796) | 08-02 | 재작업 제거 — 작업 순서 고정 | **L2 횡단** |
| [#3828](https://github.com/edwardkim/rhwp/issues/3828) | 08-02 | 유입 다리 4개 | **L3** |
| [#3869](https://github.com/edwardkim/rhwp/issues/3869) | 08-03 | 설치 없는 실행(WASM) | **L3** |

**층 분포가 말하는 것** — 일곱 중 L1 이 둘, L2 가 셋, L3 가 둘, **L4 는 0** 이다.
그리고 L3 두 건은 **devel 에 아무것도 없다**(§2.6·§2.7).
즉 오늘 열린 로드맵은 **아래 두 층에 몰려 있고, 도달 층은 전부 종이 위에 있다.**

### 1.2 의존 그래프

```
L4   M26~M30 (미착수 — 옳다)
      ▲ 전제
L3   #3828 유입 다리 4개 ──── inspect 사용 ────┐      #3869 설치 없는 실행 ≡ #3608 M24
      ▲ 전제: capabilities 자기서술            │       ▲ 전제: 봉투 동등성 기준(B2·B6)
L2   #3787 인젝션 내성 ◀── 계약 ──▶ #3793 보안 문서 축 ┘
     #3796 재작업 제거 (횡단 — 나머지 여섯 전부의 작업 순서)
      ▲ 전제: 봉투가 부분 목록을 부분이라 말한다
L1   #3608 마일스톤 현황판 ◀── 상위 지도 ──▶ #3719 6층 아키텍처
      ▲ 구멍 8건 (layer_model §4)
```

### 1.3 진행률 한눈에 (실측)

| 이슈 | 체크박스 | 산출물 실재 | 열린 PR |
|---|---:|---|---:|
| [#3608](https://github.com/edwardkim/rhwp/issues/3608) | **8 / 196** | 명령 61·json 31·MCP 39 | 다수 |
| [#3719](https://github.com/edwardkim/rhwp/issues/3719) | **2 / 8** | 세션 12종 동작 | 3건 |
| [#3787](https://github.com/edwardkim/rhwp/issues/3787) | 체크박스 없음 | `inspect` 3축 동작 · 출처 표지 전 봉투 | 1건 |
| [#3793](https://github.com/edwardkim/rhwp/issues/3793) | 체크박스 없음 | **11편 5,626줄 실재** | 0건 |
| [#3796](https://github.com/edwardkim/rhwp/issues/3796) | 체크박스 없음 | `tools/agent_preflight.py` 실재 | 1건 |
| [#3828](https://github.com/edwardkim/rhwp/issues/3828) | 체크박스 없음 | **0 / 4** (`grep` 전부 0) | 4건 |
| [#3869](https://github.com/edwardkim/rhwp/issues/3869) | 체크박스 없음 | **0 / 6** (코드 0) | 2건 |

> **체크박스를 진행률로 읽지 말 것.** [#3608](https://github.com/edwardkim/rhwp/issues/3608) 의
> 8/196 은 4.1% 가 아니다 — 갱신되지 않은 것이다(§3.3 D3).
> 다섯 로드맵은 체크박스 자체가 없다.

---

## 2. 로드맵 전수

### 2.1 [#3608](https://github.com/edwardkim/rhwp/issues/3608) — 에이전트 표면 전면 커버리지 (Stage 6~8)

**한 줄** — 전 명령을 실측 매트릭스로 고정하고 마일스톤 M1~M30 의 현황판이 된다.
개설 2026-07-30, [#2659](https://github.com/edwardkim/rhwp/issues/2659) Stage 1~5 의 후속.

**층** — 본체는 **L1**(M1~M17). 다만 §8 의 M18~M30 이 세 층에 걸쳐 있다 —
**L2**: M21 퍼징 · M22 스펙 적합성 · M23 코퍼스 /
**L3**: M18~M20 바인딩 · M24 WASM · M25 문서 지능 서버 /
**L4**: M26~M30 클라우드·접근성·생태계·지표·표준화.

**일곱 중 유일하게 네 층에 걸쳐 있다.** 이것이 이 로드맵의 힘이자 한계다 —
전 범위를 담고 있어 권위가 되지만, **자기 안에서 "무엇을 먼저"를 답할 수 없다.**
M20 과 M26 이 같은 문서의 같은 형식으로 나란히 적혀 있으면, 둘의 착수 시점 차이를
표현할 자리가 없다.

**전제** — 없다. 최하위 로드맵이다.

**막는 것** — 셋이다.

1. **M18~M20** 이 L3 바인딩 전부. [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가
   기존 둘의 표류 20건(치명 3건)을 찾았다
2. **M21** 이 L2 견고성. 실측: `fuzz/fuzz_targets/` 6종 존재,
   `grep -ril fuzz .github/` = **0**
3. **M24** 가 L3 WASM. [#3869](https://github.com/edwardkim/rhwp/issues/3869) 와 같은 축(§3.2)

**진행률 (실측)**

체크박스 **8 / 196 (체크됨 8)**. 그러나 실측은 훨씬 앞서 있다 — M2(세션 조회·렌더)의
7개 항목이 전부 미체크인데 세션 도구 12종이 실제로 뜬다. §3.3 참조.

M18~M20 실재 확인:

```
bindings/python  — .py 36개        bindings/node  — .ts 48개
bindings/Native/src/lib.rs  376줄   bindings/csharp/RhwpNative.cs  63줄
bindings/swift/Sources/  2파일 274줄
```

M20 은 "미착수"로 적혀 있으나 **디렉터리는 다른 계약이 이미 점유**하고 있다
([#3879](https://github.com/edwardkim/rhwp/pull/3879) §8: 노출 함수 4개, 봉투에
`schemaVersion` 도 종료 코드도 판정 표현도 없음).

**겹침**

| 겹치는 곳 | 상대 | 성격 |
|---|---|---|
| M24 WASM/브라우저 | [#3869](https://github.com/edwardkim/rhwp/issues/3869) | **같은 축**(§3.2) |
| §6.7 실행 3층 | [#3719](https://github.com/edwardkim/rhwp/issues/3719) S4 | 상위 지도 분담(의도적) |
| §6.6 도메인 매크로 15종 | [#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-6 | 8종만 재게재(의도적) |
| M6 온보딩 | [#3828](https://github.com/edwardkim/rhwp/issues/3828) B2 | 부트스트랩 경로 중복 후보 |
| §1-D 제외 30종 | [#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-1 | **규칙이 이미 깨짐**(§3.4) |

**종료 조건**

- M1~M17 이 **실측 매트릭스로** 닫힌다 — 체크박스가 아니라 `capabilities` 재실측으로
- M18~M30 을 **별도 로드맵으로 분리**하거나, 현황판 역할만 남기고 착수 판단은 층 모델에 위임
- §1-D 제외 목록이 **실측과 일치**한다(§3.4 해소)

---

### 2.2 [#3719](https://github.com/edwardkim/rhwp/issues/3719) — 6층 아키텍처 통합 지도

**한 줄** — [#3608](https://github.com/edwardkim/rhwp/issues/3608)·[#3630](https://github.com/edwardkim/rhwp/issues/3630)·[#3703](https://github.com/edwardkim/rhwp/issues/3703)
셋이 같은 표면을 나눠 서술하던 것을 하나의 구현 스택(S0~S6)으로 묶는다. 개설 2026-08-01.

**층** — **L1**. 그리고 **L1 의 판정 기준을 소유한다** — §4 횡단 불변식 7과 exit 코드 사전이
전 층 조각의 수용 기준이다.

**전제** — [#3608](https://github.com/edwardkim/rhwp/issues/3608) 의 커버리지 매트릭스.

**막는 것** — 두 가지다.

1. **불변식 7이 모든 신규 조각의 통과 조건**이다. 특히 불변식 4("부분 목록 금지")와
   6("stdout 순수성")이 [layer_model §4](layer_model.md#4-l1-의-구멍이-l3-를-막는다--실례-8건)
   여덟 구멍의 판정 근거다
2. **§6 판정 대기 2건**(`hwp_doc_transaction` vs 계획 실행기 / `hwp_form_autopilot` vs
   계획서 템플릿)이 결론 나야 L2·L3 조각의 중복 구현을 막는다

**진행률 (실측)**

체크박스 **2 / 8**(§3-1 의 `dump-pages`·`export-doclang` 만 체크).
§8 마일스톤 매핑표는 M1·M2·M4·M16 을 ✅ 완료로 적는다.

실측 대조:

| §8 주장 | 실측 | 판정 |
|---|---|---|
| M2 세션 조회·렌더 ✅ | 세션 도구 12종 전부 뜸 | **참** |
| M4 보호 문서 ✅ | `--password` 미배선 7건([#3839](https://github.com/edwardkim/rhwp/pull/3839)) | **거짓** |
| M16 MCP 표면 v2 ✅ ("내성 4종으로 실질 달성") | didYouMean 동작 확인(`rhwp infoo` → "가장 가까운 명령은 'info'") | 부분 참 |
| L1 잔여 3(`render-diff`·`export-png`·…) | `render-diff` json=true 실측 | **이미 해소됨** |

§2-1 스냅샷(2026-08-01)과 오늘의 차이는 크다 — CLI 명령 54 → **61**, `--json` 21 → **31**,
MCP 무상태 23 → **39**, 계약 테스트 215 → **523**
(전체 대조표는 [층 모델 §2.2](layer_model.md#22-l1--표면-있는가)).

**이틀 만에 절반 이상 바뀌었다.** 로드맵 본문의 실측 수치는 **이틀이면 낡는다** —
이것이 이 축이 "숫자를 문서에 박지 말고 재현 명령을 박아라"로 수렴하는 이유다.

**겹침**

- [#3608](https://github.com/edwardkim/rhwp/issues/3608) 과 **역할 분담이 명시**돼 있다
  (아키텍처는 #3719, 현황판은 #3608). 의도적 중복
- **층 기호가 [#3880](https://github.com/edwardkim/rhwp/issues/3880) 과 충돌**한다(§3.1)
- §7 큐 리스크의 "열린 PR 10건 내외" ↔ [#3796](https://github.com/edwardkim/rhwp/issues/3796) §6 동일 규율

**종료 조건**

- §5 층별 DoD 6개(S1~S6)가 전부 충족되고 각각 계약 테스트로 고정
- §6 판정 대기 2건의 결론이 **머지**된다(오늘 PR [#3826](https://github.com/edwardkim/rhwp/pull/3826) 열림)
- §4 불변식 7이 **드리프트 가드로 자동 검사**된다 — 오늘 exit 3 사전만 가드가 있다

---

### 2.3 [#3787](https://github.com/edwardkim/rhwp/issues/3787) — 간접 프롬프트 인젝션 내성 (구현)

**한 줄** — 신뢰할 수 없는 문서가 에이전트를 조종하지 못하게 막는 표면 계약 S1~S10.
개설 2026-08-02.

**층** — **L2**. 단 **S1(출처 표지)은 L1 을 직접 바꾼다** — 전 봉투에 필드를 추가하기 때문이다.

**전제** — L1 봉투. 표지를 실을 봉투가 먼저 있어야 한다.

**막는 것** — L3 의 소비자 확대. 신뢰할 수 없는 파일을 다루는 소비자가 붙으려면
"우리가 무엇을 탐지하고 무엇을 탐지하지 못하는가"가 계약으로 있어야 한다.

**진행률 (실측)**

| 조각 | 상태 | 근거 |
|---|---|---|
| S1 출처 표지 | **착지** | 오늘 확인한 조회 명령 8개 전부가 `untrustedContent`·`untrustedFields` 보유 |
| S2 인젝션 신호 | **착지** | `inspect injection --json` → `injectionSignals`·`highestConfidence`·`scanScopes` |
| S3 은닉 텍스트 | **착지** | `inspect hidden-text --json` → `hiddenText`·`hiddenCharCount`·`thresholdPt` |
| S4 유니코드 기만 | **착지** | `inspect unicode --json` → `findings`·`kindCounts`·`severityCounts` |
| S5~S8 경계·자원·핸들 | **착지** | `tests/boundary_integrity_contract.rs` 실재 |
| S9 위협 문서 | **[#3793](https://github.com/edwardkim/rhwp/issues/3793) 이 흡수** | 지정 경로 `mydocs/tech/prompt_injection_model.md` 는 **없음**(§3.5) |
| S10 악성 코퍼스 | **열림** | PR [#3867](https://github.com/edwardkim/rhwp/pull/3867) |

**겹침**

- **S9 ↔ [#3793](https://github.com/edwardkim/rhwp/issues/3793) 전체.** 문서 1편으로 계획한 것을
  다른 로드맵이 11편으로 구현했다. 경로도 다르다(§3.5)
- **§5 오탐 규율 ↔ [#3796](https://github.com/edwardkim/rhwp/issues/3796) §5.** 같은 문장이
  두 로드맵에 있다(의도적 승계로 보이나 어느 쪽도 상대를 인용하지 않는다)
- **`inspect` ↔ [#3828](https://github.com/edwardkim/rhwp/issues/3828) B3 레시피.**
  레시피가 이 축의 산출물을 쓰는데 어느 쪽도 상대를 전제로 적지 않았다(§3.6)

**종료 조건**

- S10 머지 + **악성 코퍼스가 red→green 을 실증** — 방어 전엔 통과하고 후엔 잡힘
- 정상 `samples/*.hwp` 전수에서 **오탐 0** 재확인
- `inspect` 의 자기서술 구멍 해소([layer_model §4 B5](layer_model.md#46-b5--하위-명령이-자기서술에-없다))
  — 탐지가 있어도 부를 수 없으면 방어가 아니다

---

### 2.4 [#3793](https://github.com/edwardkim/rhwp/issues/3793) — 에이전트 보안 문서 축 신설

**한 줄** — rhwp 최초의 보안 문서 축 `mydocs/tech/agent_security/` 11편.
"코드보다 문서가 먼저인 이유"를 논증한다. 개설 2026-08-02.

**층** — **L2**.

**전제** — [#3787](https://github.com/edwardkim/rhwp/issues/3787) 의 구현. 문서가 실물을
서술해야 하므로 서술 대상이 먼저 있어야 한다. 다만 이 로드맵의 논지는 **계약이 코드보다
먼저**라는 것이라(§6: "충돌하면 문서를 고치고 코드를 맞춘다") 전제 관계가 단순 선후가 아니다.

**막는 것** — L3 채택. 소비 에이전트 작성자가 `consumer_guide.md` 만 읽고 안전하게 붙일 수
있어야 외부 소비자가 늘어난다.

**진행률 (실측) — 일곱 중 유일하게 산출물이 전부 실재한다**

`mydocs/tech/agent_security/` **11편 5,626줄**(README 110 · attack_surface 605 ·
consumer_guide 700 · detection_policy 542 · disclosure 181 · glossary 374 ·
hidden_content 680 · indirect_prompt_injection 668 · test_corpus 542 ·
threat_model 585 · unicode_deception 639).

수용 기준 5개 중 **셋은 충족**(권위 표 등재 · 근거 병기 · #3787 상호 링크, 직접 확인),
하나는 **부분 확인**(프런트매터는 2편만 확인 — §5 U-2), 하나는 **미검증**
(`consumer_guide.md` 단독 사용 가능성 — 외부 사용자 시험 없음).

**겹침**

- [#3787](https://github.com/edwardkim/rhwp/issues/3787) S9(§3.5)
- **축의 범위 번짐(관찰)** — PR [#3826](https://github.com/edwardkim/rhwp/pull/3826) 이
  `agent_security/session_verify_decision.md`·`session_transaction_decision.md` 를
  이 디렉터리에 넣는다. 두 문서는 **보안 문서가 아니라 세션 도구 신설 여부 판정**이다.
  보안 축이 "판정 보관소"로 번지는 첫 사례다

**종료 조건**

- **이미 수용 기준을 대체로 충족**했다. 산출물이 실재하고 권위 표에 등재됐다
- 다만 **현행성 유지 방식이 정해져야 한다**(§3.13 D13) — 이 축의 `README.md` 는
  개설 하루 만에 실물과 어긋났다. 갱신 주기 없이 닫으면 문서가 조용히 낡는다
- 이 로드맵은 **닫아도 되는 첫 후보**다. 축이 살아 있으므로 이슈가 열려 있을 이유가 약하고,
  남은 일은 이슈가 아니라 **유지 규칙**이다

---

### 2.5 [#3796](https://github.com/edwardkim/rhwp/issues/3796) — 재작업 제거 (작업 순서 고정)

**한 줄** — 실측된 재작업 원인 8종(R1~R8)을 **작업 순서 자체로** 막는다. 개설 2026-08-02.

**층** — **L2 횡단**. 산출물이 표면이 아니라 **기여 절차**다.

**전제** — 없다. 도구([#3795](https://github.com/edwardkim/rhwp/pull/3795))가 이미 착지했다.

**막는 것 — 나머지 여섯 로드맵 전부다.**

§2 의 단계 0~5(참조 구현 읽기 → 여섯 축 동시 작성 → 선검사 → 빌드 1회 → 명시적 커밋 →
PR 전 3종 검사)는 **모든 표면 조각의 작업 순서**다. 빌드 3회를 1회로 줄이는 것이
[#3828](https://github.com/edwardkim/rhwp/issues/3828) B1 에도, [#3869](https://github.com/edwardkim/rhwp/issues/3869) W1 에도,
[#3787](https://github.com/edwardkim/rhwp/issues/3787) S10 에도 똑같이 적용된다.

**그런데 여섯 로드맵 어디에도 이 사실이 없다.** [#3880](https://github.com/edwardkim/rhwp/issues/3880)
§0 이 지적한 그대로다 — "이미 만들었는데 아무도 모른다".

**진행률 (실측)**

| 항목 | 상태 |
|---|---|
| `tools/agent_preflight.py` | **실재** |
| `mydocs/manual/agent_preflight_guide.md` | **실재** |
| §7 수용 기준 "깨끗한 devel 에서 오탐 0" | **미충족** — PR [#3872](https://github.com/edwardkim/rhwp/pull/3872) 가 오탐 2건 수정 중 |
| 여섯 로드맵에 명시 | **0 / 6** |
| 플레이북 편입 | 확인되지 않음(§5 U-3) |

오탐의 성격이 흥미롭다 — PR [#3872](https://github.com/edwardkim/rhwp/pull/3872) 본문에 따르면
**스키마를 출력하는 명령이 자기 스키마 안의 오류 설명 문자열("알 수 없는 옵션")에 걸려
스스로를 미구현으로 신고**했다. 이 로드맵 §5 가 "헛울리는 검사기는 곧 무시당하고,
무시당하면 없느니만 못하다"고 적은 그 실패를 **검사기 자신이 겪었다.**

**겹침** — §5 오탐 규율은 [#3787](https://github.com/edwardkim/rhwp/issues/3787) §2 ·
[#3793](https://github.com/edwardkim/rhwp/issues/3793) §3-④ 와, §6 "열린 PR 10건 내외"는
[#3719](https://github.com/edwardkim/rhwp/issues/3719) §7 리스크 1 과, §4 근거 규율은
[#3793](https://github.com/edwardkim/rhwp/issues/3793) 수용 기준 3 과 같은 말이다.
**어느 쪽도 상대를 인용하지 않는다.**

**종료 조건**

- 선검사가 **깨끗한 devel 에서 오탐 0** (PR [#3872](https://github.com/edwardkim/rhwp/pull/3872) 머지)
- **여섯 로드맵 각각에 "이 축의 작업 순서는 #3796 을 따른다" 한 줄**이 붙는다
- [에이전트 표면 플레이북](../../manual/agent_surface_playbook.md)에 단계 0~5 가 편입된다 —
  플레이북이 절차의 권위이므로, 두 문서가 다른 순서를 말하면 그 자체가 재작업 원인이 된다

---

### 2.6 [#3828](https://github.com/edwardkim/rhwp/issues/3828) — 에이전트 유입 다리 4개

**한 줄** — "정확한 도구 이름을 이미 아는 에이전트"를 전제하지 않는 진입 경로 4개.
개설 2026-08-02.

**층** — **L3**.

**전제** — 둘이다.

1. **L1 자기서술.** B1 키워드 검색과 B2 매니페스트는 `capabilities` 선언을 대상으로 한다.
   선언이 실물과 다르면 다리가 **에이전트를 계약 밖으로 안내**한다
   ([layer_model §4 B4·B5](layer_model.md#45-b4----json-침묵-무시--exit-사전-이탈))
2. **[#3787](https://github.com/edwardkim/rhwp/issues/3787) 의 `inspect`.** B3 레시피 중
   "문서 안전성 점검"이 이 명령을 쓴다(§3.6)

**막는 것** — 첫 접촉 경로 전부. 그리고 L4 의 채택 지표(M29) — 유입이 없으면 지표가 0 이다.

**진행률 (실측) — 0 / 4**

```
$ grep -c '"explain"' src/main.rs               → 0     (B4)
$ grep -c 'export-agent-manifest' src/main.rs   → 0     (B2)
$ grep -c 'capabilities --search' src/main.rs   → 0     (B1)
$ ls mydocs/manual/recipes                      → 없음   (B3)
```

B1 `capabilities --search`([#3836](https://github.com/edwardkim/rhwp/pull/3836)) ·
B2 `export-agent-manifest --json`([#3843](https://github.com/edwardkim/rhwp/pull/3843)) ·
B3 레시피 5편 1,285줄([#3835](https://github.com/edwardkim/rhwp/pull/3835)) ·
B4 `rhwp explain`([#3832](https://github.com/edwardkim/rhwp/pull/3832)) — **네 건 전부 열림**.
마무리(llms.txt·지식지도 링크) PR 은 미착수.

PR [#3835](https://github.com/edwardkim/rhwp/pull/3835) 는 `llms.txt`·
`mydocs/manual/agent_knowledge_map.md` 갱신을 **레시피 PR 안에 포함**했다 —
수용 기준의 "마무리 PR 1건"과 다른 형태다.

**레시피 5편 중 03 은 미완**이다. PR 파일 목록 실측: `01`·`02`·`04`·`05`·`06` 만 있고
`03_redact_before_sharing.md`(배포 전 PII 마스킹)가 없다.
[#3828](https://github.com/edwardkim/rhwp/issues/3828) 진행 코멘트가 사유를 적었다 —
작업 워크트리가 `edit redact` 병합 이전 스냅샷이라 명령을 실행할 수 없었고,
**"실행 없이 출력을 지어내는 것은 이 묶음의 원칙을 정면으로 어긴다"**.
근거 규율을 지킨 결과의 공백이므로 이 문서는 이를 결함으로 세지 않는다.
다만 `01`·`04` 본문이 이미 `03` 을 상호 참조하므로, 머지 시점에 **깨진 링크**가 된다.

**겹침**

| 겹치는 곳 | 상대 | 성격 |
|---|---|---|
| B2 부트스트랩 | [#3608](https://github.com/edwardkim/rhwp/issues/3608) M6 온보딩 | **미판정 중복 후보** |
| B4 `explain` | [#3608](https://github.com/edwardkim/rhwp/issues/3608) §6.6 `digest` | 둘 다 "문서를 싸게 파악" — 경계 미정 |
| B3 레시피 | [#3793](https://github.com/edwardkim/rhwp/issues/3793) `consumer_guide.md` | 목표 서사 vs 안전 계약 (역할 다름) |
| B3 안전성 점검 | [#3787](https://github.com/edwardkim/rhwp/issues/3787) `inspect` | **상호 무기재**(§3.6) |

**종료 조건**

- B1~B4 **머지** + `llms.txt`·에이전트 지식 지도에 링크
- **B4 와 `digest` 의 경계가 문서로 확정**된다 — 둘 다 "문서를 싸게 파악"인데,
  어느 것을 언제 쓰는지가 없으면 다리가 갈림길이 된다
- 다리가 안내한 명령이 **전부 계약 안에 있다**([layer_model §4 B4·B5](layer_model.md#45-b4----json-침묵-무시--exit-사전-이탈) 해소)

---

### 2.7 [#3869](https://github.com/edwardkim/rhwp/issues/3869) — 설치 없는 실행 (WASM 에이전트 표면)

**한 줄** — 모든 진입로가 공유하는 첫 관문(rhwp 바이너리 확보)을 없앤다.
`pip install rhwp` / `npm install rhwp` 만으로 동작하는 WASM 표면 W1~W6. 개설 2026-08-03.

**층** — **L3**.

**전제** — L1 봉투 정합. W2("WASM 반환값이 CLI `--json` 봉투와 같은 모양")가
**논리적 전제**다([layer_model §3.1](layer_model.md#31-논리적-전제--아래가-없으면-위의-명제가-성립하지-않는다)) —
`run` 과 `bench --json` 이 실패 시 stdout 규약의 예외이고(B2),
`export-tables -o --json` 이 봉투 대신 사람 문장을 내므로(B6),
**"같은 모양"의 기준이 오늘 정의되지 않는다.**

**막는 것** — 샌드박스 안의 에이전트 전부. 이 로드맵 §2 의 표현대로
**"나머지는 배수지만 이건 0에서 1"** 이다.

**진행률 (실측) — 코드 0**

`src/wasm_api.rs` 는 **7,621줄 · `wasm_bindgen` 372곳**(실측)이지만 **렌더링 지향**이다.
PR [#3873](https://github.com/edwardkim/rhwp/pull/3873) 의 실측 인용에 따르면
WASM 표면에 **있는 것**은 `getDocumentInfo`·`getStructure`·`searchText`·`getFieldList`·
`setFieldValue`·`replaceOne/All`·`exportHwp/Hwpx`·`extractThumbnail`,
**없는 것**은 `digest`·`extract-data`·`export-tables`·`table-to-csv`·`inspect` 3종·
`redact`/`sanitize`·`run`·**`capabilities` 자기서술 일체**다.

같은 PR 이 원인을 **기능 부족이 아니라 크레이트 경계**로 지목했다 —
로직은 lib 에 있는데 **봉투를 만드는 층이 `[[bin]]` 전용**이다
(`capabilities_command_entries()`·`mcp_serve.rs`·`agent_profiles.rs`).

조각별로는 W1·W2·W5·W6 이 **설계 문서만** 있고(PR
[#3876](https://github.com/edwardkim/rhwp/pull/3876) 6편 2,701줄 ·
[#3873](https://github.com/edwardkim/rhwp/pull/3873) 4편),
**W3(Python 휠)·W4(npm)는 미착수**다.

**겹침 — 이 로드맵의 가장 큰 문제**

**[#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 와 같은 축**이다(§3.2).
[#3880](https://github.com/edwardkim/rhwp/issues/3880) §0 이 "어제 세우고 나서야 알았다"고
적은 그 항목이고, PR [#3873](https://github.com/edwardkim/rhwp/pull/3873) 본문도
"관련 이슈 #3869(설치 없는 실행)와 같은 축이라 정렬했다"고 인정한다.
그럼에도 **두 문서 디렉터리가 동시에 열려 있다.**

**종료 조건**

- `pip install` 만으로 `digest`·`fields`·`fill`·`search`·`inspect` 가 **바이너리 없이** 동작
- W2 봉투 동등성이 **계약 테스트로 고정** — 그러려면 L1 B2·B6 이 먼저 닫혀야 한다
- WASM vs 네이티브 성능을 **실측해 문서화**(느린 구간을 숨기지 않는다)
- **M24 와의 관계가 하나로 정리**된다(§3.2)

---

## 3. 발견한 중복과 모순

> **이 절이 이 문서의 성과다.** 숨기지 않고 표로 적는다.
> 각 항목은 **무엇이 어긋났는가 / 근거 / 왜 문제인가 / 해소 조건** 순이다.

### 3.0 한눈에

| # | 무엇 | 종류 | 심각도 근거 |
|---|---|---|---|
| **D1** | 층 기호 `L1`·`L3`·`L4` 가 두 뜻 | 모순 | "L4 를 먼저"가 정반대 두 해석 |
| **D2** | WASM 축이 두 곳에서 자란다 | **중복** | 문서 디렉터리 2개 동시 개설 |
| **D3** | "진행률의 유일 기준"이 셋 | 모순 | 같은 마일스톤에 답이 셋 |
| **D4** | 명시적 제외 30종의 규칙이 이미 깨짐 | 모순 | 실질 커버리지 계산식이 성립 안 함 |
| **D5** | S9 가 지정한 문서 경로가 실물과 다름 | 모순 | 로드맵이 없는 파일을 가리킴 |
| **D6** | `inspect` 를 쓰는 쪽과 만드는 쪽이 서로 무기재 | 누락 | 착수 순서가 안 잡힘 |
| **D7** | 선검사가 여섯 로드맵 어디에도 없음 | 누락 | 만들어 놓고 안 쓰임 |
| **D8** | 판정 대기 2건의 결론이 로드맵에 미반영 | 지연 | 중복 구현 위험 존속 |
| **D9** | 열린 PR 22건 — 자기 규율 위반 | 모순 | 두 로드맵이 "10건 내외"를 못박음 |
| **D10** | 열린 PR 을 "이미 해결됐다"로 계산 | 모순 | 로드맵이 낙관 쪽으로 왜곡 |
| **D11** | 퍼징 타깃 수가 로드맵과 실물이 다름 | 모순 | 4종 vs 실측 6종 |
| **D12** | 하루에 tech 하위 신규 디렉터리 5개 | 관찰 | 파편화가 이 축의 문제 자체 |
| **D13** | 보안 축 문서가 하루 만에 낡았다 | 모순 | L2 산출물이 문서인데 실물과 다름 |

---

### 3.1 D1 — 층 기호가 두 뜻이다

**무엇** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) 의 `L1`~`L6` 과
[#3880](https://github.com/edwardkim/rhwp/issues/3880) 의 `L1`~`L4` 가 같은 기호로 다른 것을 가리킨다.

**근거** — #3719 §1 표(L1 = CLI `--json`, L3 = MCP 세션, L4 = 계획 실행기)와
#3880 §2 블록(L1 = 표면, L3 = 도달, L4 = 표준).

**왜 문제인가** — "L4 를 먼저 하자"가 **계획 실행기 v2**(높은 우선순위)를 뜻하는지
**M26~M30 표준화**(#3880 이 "아직 이르다"고 명시)를 뜻하는지 구별할 수 없다.
두 해석의 우선순위는 정반대다.

**해소** — [layer_model §1.2](layer_model.md#12-명명-규약--이-축이-쓰는-표기) 가
구현 스택 `S0~S6` / 성숙도 사다리 `L1~L4` 로 갈랐다. 이 표기가 두 이슈 본문에도 반영돼야
완전히 해소된다.

---

### 3.2 D2 — WASM 축이 두 곳에서 자란다

**무엇** — [#3869](https://github.com/edwardkim/rhwp/issues/3869)(설치 없는 실행)과
[#3608](https://github.com/edwardkim/rhwp/issues/3608) M24(WASM/브라우저 에이전트 표면)가
같은 축인데, **각각 별도의 문서 디렉터리를 만드는 PR 이 동시에 열려 있다.**

**근거 (실측)**

```
PR #3876  [#3869]  mydocs/tech/agent_runtime/       6편 2,701줄
    surface_spec.md 717 · envelope_parity.md 524 · cost_model.md 462
    failure_dictionary.md 449 · entrypoint_decision.md 378 · README.md 171

PR #3873  [M24]    mydocs/tech/wasm_agent_surface/  4편
    self_description.md 498 · browser_bridge.md 509
    zero_install_onboarding.md 465 · README.md 159
```

두 PR 다 2026-08-03 개설. PR #3873 본문이 **스스로 인정**한다 —
"관련 이슈 #3869(설치 없는 실행)와 같은 축이라 정렬했다".

**왜 문제인가**

1. `zero_install_onboarding.md`(#3873)와 `entrypoint_decision.md`(#3876)는 **같은 질문**
   ("어느 경로로 시작하나")에 답한다. 두 문서가 다른 답을 하면 소비자가 어느 쪽을 믿을지 모른다
2. `self_description.md`(#3873)와 `envelope_parity.md`(#3876)는 **같은 대상**
   (WASM 이 자기를 어떻게 서술하고 CLI 봉투와 어떻게 맞추나)을 다룬다
3. 디렉터리가 둘이면 `mydocs/tech/README.md` 권위 표에 **같은 주제가 두 줄**이 된다

**해소 조건** — 셋 중 하나를 택한다.

| 안 | 내용 | 대가 |
|---|---|---|
| A | 한 디렉터리로 병합 | 두 PR 중 하나를 리베이스 |
| B | 역할 분담 명시 (M24 = 브라우저 / #3869 = 패키지 배포) | 경계 문서 1편 추가 필요 |
| C | [#3869](https://github.com/edwardkim/rhwp/issues/3869) 를 M24 로 흡수 | [#3880](https://github.com/edwardkim/rhwp/issues/3880) §3-3 의 제안 |

**본 문서의 판단** — B 가 아니라 **A 또는 C**. 근거: 두 PR 의 문서 목록이 겹치는 항목이
2/4·2/6 로 절반에 가깝고, 겹치는 항목이 **둘 다 canonical** 이다.
canonical 이 둘이면 그건 canonical 이 아니다.

---

### 3.3 D3 — "진행률의 유일 기준"이 셋이다

**무엇** — 같은 마일스톤의 완료 여부에 대해 세 개의 답이 있다.

**근거**

| 출처 | M2(세션 조회·렌더) | M4(보호 문서) |
|---|---|---|
| [#3608](https://github.com/edwardkim/rhwp/issues/3608) §7 체크박스 ("진행률의 유일 기준") | 7항목 **전부 미체크** | 8항목 **전부 미체크** |
| [#3719](https://github.com/edwardkim/rhwp/issues/3719) §8 매핑표 | **✅ 완료** | **✅ 완료** |
| 실측 (2026-08-03) | 세션 도구 12종 **전부 뜸** → 참 | `--password` 미배선 **7건** → 거짓 |

체크박스 전체는 **8 / 196**. 이 값을 진행률로 읽으면 4.1% 인데, `--json` 계약이
21 → 31 로 늘어난 이틀 동안 체크박스는 거의 움직이지 않았다.

**왜 문제인가** — 두 로드맵이 **서로 다른 방향으로 틀렸다.**
#3608 은 비관 쪽으로(다 된 걸 미체크), #3719 는 낙관 쪽으로(안 된 걸 ✅).
둘을 합쳐도 참이 나오지 않는다.

**해소 조건**

- 체크는 **머지 링크와 함께만** — #3608 §7 이 이미 규정한 것을 지킨다
- ✅ 는 **실측 재현 명령과 함께만** — "M4 완료"가 아니라
  "`--password` 를 선언한 N 개 명령이 전부 암호 문서를 연다(가드: `tests/…`)"
- 또는 **체크박스를 버리고 재현 스크립트로 대체**한다.
  이틀에 절반이 바뀌는 표면에서 손으로 유지하는 체크박스는 반드시 낡는다

---

### 3.4 D4 — 명시적 제외 30종의 규칙이 이미 깨졌다

**무엇** — 두 로드맵이 "`diagnostic` + `internal` 카테고리 30종은 기계 계약 명시 제외"라고
적었는데, **그 카테고리 안에서 이미 3건이 `--json` 계약을 갖는다.**

**근거 (실측)**

```
$ rhwp capabilities | (카테고리별 json 보유 집계)
diagnostic 25 중 json=true : dump-pages · ir-diff · render-diff   ← 3건
internal    5 중 json=true : 0건
```

[#3719](https://github.com/edwardkim/rhwp/issues/3719) §2-1 은
"54 중 30(진단 25 + 내부 5)은 §3-1 의 명시적 제외이므로 실질 커버리지는 **21 / 24**"
라고 계산한다. 오늘 같은 방식으로 계산하면 분모·분자가 둘 다 틀린다 —
제외 카테고리 안에 계약 보유가 3건 있으므로 `31 - 3 = 28` 이 비제외 계약이고,
비제외 명령은 `61 - 30 = 31` 이다.

**왜 문제인가** — "카테고리 = 제외"라는 규칙이 이미 예외 3건을 가지므로,
**제외의 근거가 카테고리가 아니라 개별 판정**이 됐다. 그런데 두 로드맵의 문장은
여전히 카테고리로 말한다. 새 조각이 들어올 때 "이건 diagnostic 이니 제외"라는
잘못된 판단이 나온다.

그리고 이 규칙 공백이 [layer_model §4 B4](layer_model.md#45-b4----json-침묵-무시--exit-사전-이탈)
와 직결된다 — `dump`·`diag` 는 제외인데 `--json` 을 **침묵으로 삼킨다.**
제외 명령이 미지 옵션을 어떻게 다뤄야 하는지가 어느 문서에도 없다.

**해소 조건** — 제외를 **명령 단위 목록**으로 바꾸고, 각 항목에 제외 근거를 붙인다.
그리고 **제외 명령의 미지 옵션 정책**을 exit 사전에 추가한다.

---

### 3.5 D5 — S9 가 지정한 문서 경로가 실물과 다르다

**무엇** — [#3787](https://github.com/edwardkim/rhwp/issues/3787) S9 는 위협 문서를
`mydocs/tech/prompt_injection_model.md` 로 지정했다. 그 파일은 **없다.**

**근거 (실측)**

```
$ ls mydocs/tech/prompt_injection_model.md
No such file or directory

$ ls mydocs/tech/agent_security/indirect_prompt_injection.md
mydocs/tech/agent_security/indirect_prompt_injection.md      ← 실물은 여기
```

[#3793](https://github.com/edwardkim/rhwp/issues/3793) 이 S9 한 편을 **11편 축으로 확장**해
구현했다. 결과는 더 낫지만 **#3787 본문은 갱신되지 않았다.**

**왜 문제인가** — #3787 을 읽고 S9 를 착수하는 사람은 없는 파일을 만들려 하고,
그 순간 **12번째 보안 문서**가 생긴다. 축의 canonical 이 둘이 된다.

**해소 조건** — #3787 S9 행을 "[#3793](https://github.com/edwardkim/rhwp/issues/3793) 이
`mydocs/tech/agent_security/` 로 흡수 — 완료"로 갱신한다.

---

### 3.6 D6 — `inspect` 를 쓰는 쪽과 만드는 쪽이 서로를 모른다

**무엇** — [#3828](https://github.com/edwardkim/rhwp/issues/3828) B3 레시피 중
"문서 안전성 점검"은 [#3787](https://github.com/edwardkim/rhwp/issues/3787) S2~S4 가 만든
`inspect` 명령을 쓴다. **어느 쪽도 상대를 전제로 적지 않았다.**

**근거** — #3828 §B3 본문에 #3787 인용 없음. #3787 §3 조각표에 소비처 언급 없음.
[#3880](https://github.com/edwardkim/rhwp/issues/3880) §0 이 이 항목을 명시적으로 지적했다.

**추가 발견** — 그리고 그 `inspect` 는 **자기서술에 하위 명령이 없다**
([layer_model §4 B5](layer_model.md#46-b5--하위-명령이-자기서술에-없다)).
즉 레시피가 이 명령을 쓰는 순간 **capabilities 만 읽은 에이전트는 호출을 만들지 못한다.**
두 로드맵이 서로를 몰라서, 그 사이의 구멍도 아무도 안 봤다.

**해소 조건** — #3828 B3 에 "선행: #3787 S2~S4" 한 줄, #3787 조각표에 "소비처: #3828 B3" 한 줄.
그리고 B5 를 L1 조각으로 승격.

---

### 3.7 D7 — 선검사가 여섯 로드맵 어디에도 없다

**무엇** — [#3796](https://github.com/edwardkim/rhwp/issues/3796) 이 만든 선검사
(`tools/agent_preflight.py`)가 **나머지 여섯 로드맵의 작업 순서를 바꿨는데**,
여섯 군데 어디에도 그 사실이 없다.

**근거** — 도구는 워크트리에 실재한다. #3608·#3719·#3787·#3793·#3828·#3869 본문에
`agent_preflight` 언급 0건. [#3880](https://github.com/edwardkim/rhwp/issues/3880) §0 의 지적과 동일.

**왜 문제인가** — 빌드 3회 → 1회가 조각당 12분이다.
오늘 열린 PR 22건 규모에서 이 절약이 반영되지 않고 있다.

**추가 모순 ① — 수용 기준 미충족** — 이 로드맵 §7 의 수용 기준
("깨끗한 devel 에서 오탐 0")이 **오늘 미충족**이다.
PR [#3872](https://github.com/edwardkim/rhwp/pull/3872) 가 오탐 2건을 고치는 중이고,
그 오탐의 성격이 §5("헛울리는 검사기는 무시당하고, 무시당하면 없느니만 못하다")가
경고한 바로 그것이다.

**추가 모순 ② — 오탐이 잘못된 결함 보고를 낳았다**

[#3828](https://github.com/edwardkim/rhwp/issues/3828) 진행 코멘트(2026-08-03)는
"별건 — 선검사가 반복해서 잡는 **기존 결함**"으로
"`export-capabilities-schema --bare` 를 capabilities 가 선언하는데 CLI 가 받지 않는다,
별도 이슈로 올리는 게 맞다"고 보고했다. 실측하면 결함이 아니다.

```
$ rhwp export-capabilities-schema --bare   → exit 0, stdout 12,295 B (유효 스키마)
```

PR [#3872](https://github.com/edwardkim/rhwp/pull/3872) 의 진단이 옳다 —
**스키마를 출력하는 명령이 자기 스키마 안의 오류 설명 문자열에 걸려 스스로를
미구현으로 신고**한 것이다. 즉 **오탐 하나가 "별도 이슈로 올려야 할 결함" 보고까지
만들어냈다.** [#3796](https://github.com/edwardkim/rhwp/issues/3796) §4 근거 규율
("주석은 근거가 아니다. 코드가 근거다")에 **"검사기 출력도 근거가 아니다"** 가 빠져 있다.

**해소 조건** — 여섯 로드맵 각각에 한 줄 + PR #3872 머지 + 플레이북 편입 +
§4 근거 규율에 "검사기 출력은 재현으로 확인한 뒤에만 결함으로 쓴다" 추가.

---

### 3.8 D8 — 판정 대기 2건의 결론이 로드맵에 없다

**무엇** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §6 이 "착수 전 결론 필요"로
남긴 2건(`hwp_doc_transaction` vs 계획 실행기 / `hwp_form_autopilot` vs 계획서 템플릿)의
결론이 PR [#3826](https://github.com/edwardkim/rhwp/pull/3826) 에 있는데 **열려 있다.**

**근거** — PR #3826 은 코드 변경 0건, 판정 문서 2편(`session_verify_decision.md` 108줄 ·
`session_transaction_decision.md` 125줄). 결론은 "둘 다 신설 안 함 — L4 계획 실행기가
상위 호환". 워크트리에 두 파일은 **없다**(실측).

**왜 문제인가** — 로드맵 본문은 여전히 "판정 대기"다. 이 상태에서 누군가 M3 의
`hwp_doc_transaction` 항목을 보고 착수하면 **결론이 이미 난 것을 구현**한다
([#3796](https://github.com/edwardkim/rhwp/issues/3796) R8: "이미 해결된 문제를 처음부터 다시 풂").

**해소 조건** — PR #3826 머지 + #3719 §6 판정 대기 절 갱신 + #3608 M3 체크리스트 갱신.

---

### 3.9 D9 — 열린 PR 22건, 두 로드맵의 자기 규율 위반

**무엇** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §7 과
[#3796](https://github.com/edwardkim/rhwp/issues/3796) §6 이 **"열린 PR 은 10건 내외"** 를
못박았는데, 오늘 실측 **22건**이다.

**근거 (실측)**

```
$ gh pr list --author kevin9327 --state open --limit 50 | wc -l
22
```

22건 중 **12건이 2026-08-03 하루에 개설**됐다.

**왜 문제인가** — 이건 절차 위반이 아니라 **구조적 병목**이다.
L3 조각이 전부 열린 PR 안에 있다(#3828 4건 · #3869 2건 · M18~M20 1건).
큐가 안 빠지면 [layer_model §4](layer_model.md#4-l1-의-구멍이-l3-를-막는다--실례-8건) 의
구멍을 다 고쳐도 **L3 는 devel 에 도달하지 않는다.**

그리고 이 규율의 출처가 실제 사건이다 — 2026-07-22 에 열린 PR 이 폭주해 30건이
일괄 close 됐다(#3796 §6). 같은 일이 반복될 조건이 갖춰져 있다.

**해소 조건** — 신규 PR 을 멈추고 머지·리베이스에 집중. 다음 조각은 **이슈로만 예약**.
[layer_model §7](layer_model.md#7-지금-무엇을-먼저-해야-하는가) 순서 2 가 이것이다.

---

### 3.10 D10 — 열린 PR 을 "이미 해결됐다"로 계산한다

**무엇** — 열린 PR 을 "해결됨"·"착지"로 적는 일이 **세 곳에서 반복**된다.
한 번의 실수가 아니라 **패턴**이다.

**근거 (실측)**

**① [#3869](https://github.com/edwardkim/rhwp/issues/3869) §0 표**
— 서두가 "과장하지 않기 위해 먼저 적는다 — **아래는 이미 해결됐다**"이다.

| "해결됨" 항목 | 근거로 든 것 | 실측 |
|---|---|---|
| 이름을 모른다 → `capabilities --search` | [#3836](https://github.com/edwardkim/rhwp/pull/3836) | **열린 PR.** `grep 'capabilities --search' src/main.rs` = 0 |
| 부트스트랩 왕복 → `export-agent-manifest` | [#3843](https://github.com/edwardkim/rhwp/pull/3843) | **열린 PR.** `grep 'export-agent-manifest' src/main.rs` = 0 |

**② [#3828](https://github.com/edwardkim/rhwp/issues/3828) 진행 코멘트(2026-08-03)**
— 표 제목이 **"착지"** 인데 세 항목(#3836·#3835·#3832)이 전부 열린 PR 이다.
실측 확인: `explain` 0건 · `capabilities --search` 0건 · `mydocs/manual/recipes/` 부재.

**③ [#3608](https://github.com/edwardkim/rhwp/issues/3608) 진행 코멘트(2026-08-03)**
— 제목이 **"M18~M25 문서 축 일괄 착지"** 이고 "문서 18편 7,542줄"을 합산한다.
그 다섯 PR(#3873·#3878·#3877·#3879·#3876)은 **오늘 전부 열려 있다.**
실측: 다섯 디렉터리 중 워크트리에 존재하는 것 **0개**.

**왜 문제인가** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §2 가
**"머지된 것과 열린 PR 을 분리한다 — 합산은 로드맵을 낙관 쪽으로 왜곡한다"** 고
명시했는데, 그 규율이 **세 곳에서 적용되지 않았다.**
그리고 하필 ①은 "과장하지 않기 위해"라고 서두를 단 절에서 그랬다.

용어의 문제이기도 하다 — **"착지"에 정의가 없다.** PR 개설을 착지로 읽으면
[D9](#39-d9--열린-pr-22건-두-로드맵의-자기-규율-위반)(열린 PR 22건)과 겹쳐,
**"다 됐는데 큐만 밀렸다"** 는 그림이 만들어진다. 실제로는 L3 전체가 devel 에 없다.

**해소 조건**

- **"착지" 를 "머지" 로만 쓴다.** PR 개설은 "제출"이다
- 진행 표에 **상태 열**(제출 / 머지)을 필수로 넣는다
- 이 규율을 [#3796](https://github.com/edwardkim/rhwp/issues/3796) §4 근거 규율에 편입한다 —
  "재현할 수 없으면 재현할 수 없다고 쓴다"의 확장이다

---

### 3.11 D11 — 퍼징 타깃 수가 로드맵과 실물이 다르다

**무엇** — [#3608](https://github.com/edwardkim/rhwp/issues/3608) M21 은
"cargo-fuzz 타깃 **4종**(HWP5/HWP3/HWPX/HML 파서)"이라 적었다. 실측은 **6종**이다.

**근거 (실측)**

```
$ ls fuzz/fuzz_targets/
parse_hml.rs  parse_hwp.rs  parse_hwp3.rs  parse_hwpx.rs
parse_ooxml_chart.rs  parse_wmf.rs          ← 로드맵에 없는 2종

$ grep -ril fuzz .github/ | wc -l
0                                            ← CI 에서 한 번도 안 돈다
```

**왜 문제인가** — 두 방향으로 틀렸다.
로드맵은 **있는 것을 적게 세고**(4 vs 6), **없는 것을 있다고 세지는 않지만**
"인프라 존재"와 "CI 실행"을 구분하지 않는다. PR [#3877](https://github.com/edwardkim/rhwp/pull/3877)
제목이 그 간극을 정확히 짚는다 — "인프라는 있는데 CI 에서 안 돌고 있었다".

**이미 자기 신고돼 있다** — [#3608](https://github.com/edwardkim/rhwp/issues/3608)
2026-08-03 코멘트에 "현황판 드리프트 — 갱신이 필요합니다" 절이 있고,
M21 첫 항목이 미체크인데 이미 머지됐다는 것(그리고 실제로는 6종이라는 것)과
`grep -ril fuzz .github/` = 0 을 함께 적었다. **본문이 아직 안 고쳐졌을 뿐이다.**
이 문서는 그 신고를 독립 실측으로 재확인했다.

**해소 조건** — M21 항목을 "타깃 6종 유지 + **CI 워크플로 편입**"으로 갱신.
DoD 를 "타깃이 있다"가 아니라 **"CI 에서 돈다"** 로 바꾼다.
그리고 **코멘트의 자기 신고가 본문에 반영되는 경로**를 만든다 —
코멘트에만 있는 정정은 본문만 읽는 사람에게 도달하지 않는다.

---

### 3.12 D12 — 하루에 신규 문서 디렉터리 5개 (관찰)

**무엇** — 2026-08-03 하루에 열린 PR 이 `mydocs/tech/` 아래에 신규 디렉터리 5개를 만든다.
`wasm_agent_surface/`([#3873](https://github.com/edwardkim/rhwp/pull/3873), M24) ·
`agent_runtime/`([#3876](https://github.com/edwardkim/rhwp/pull/3876), #3869) ·
`fuzzing/`([#3877](https://github.com/edwardkim/rhwp/pull/3877), M21) ·
`document_intelligence/`([#3878](https://github.com/edwardkim/rhwp/pull/3878), M25) ·
`bindings/`([#3879](https://github.com/edwardkim/rhwp/pull/3879), M18~M20).
**이 축(`agent_architecture/`)이 여섯 번째**다.
현재 `mydocs/tech/` 하위 Markdown 은 **194편**이다(실측).

**왜 적는가** — 판단이 아니라 관찰인 이유는 디렉터리 신설 자체가 나쁘지 않기 때문이다
(`agent_security/` 11편은 성공 사례다). 다만 **이 축이 존재하는 이유가 파편화**이므로,
같은 날 파편이 5개 늘어난 사실은 기록돼야 한다.

**주의할 점** — `mydocs/tech/README.md` 권위 표가 여섯을 전부 등재해야 고아 디렉터리가
생기지 않는다. 다섯 PR 중 그 표를 함께 고치는 것은 3건(#3873·#3878·#3879)이고,
#3876·#3877 은 자기 디렉터리 안 README 만 만든다(실측: PR 파일 목록).

---

### 3.13 D13 — 보안 축 문서가 하루 만에 낡았다

**무엇** — [#3793](https://github.com/edwardkim/rhwp/issues/3793) 의 산출물인
[agent_security/README.md](../agent_security/README.md) 가 **오늘 실물과 다르다.**

**근거 (실측)**

문서(`last_verified: 2026-08-02`)는 이렇게 적는다.

> 현재 `rhwp edit` 의 하위 명령은 `fill-fields`·`replace-text`·`set-cell` **3종뿐**
> (2026-08-02 실측). 개인정보 마스킹 명령(`edit redact`)은 … **설계된 것이고 아직 없다.**

실물은 여섯이다.

```
$ rhwp edit
사용법: rhwp edit <fill-fields|replace-text|set-cell|insert-image|redact|sanitize> …
```

**왜 문제인가** — 방향이 반대일 뿐 이 문서가 스스로 경고한 실패다.
문서는 "아직 없는 필드를 있는 것처럼 쓰면 그 문서는 즉시 거짓말이 된다"고 적었는데,
**있는 것을 없다고 적는 것도 같은 거짓말**이다. 그리고 이 문장은 하필
`edit redact` 의 역할 분리를 설명하는 자리에 있어, 읽는 사람이
"마스킹 기능은 아직 못 쓴다"고 결론 내린다.

L2 의 산출물이 **문서**이므로 이것은 문체 문제가 아니라 **신뢰 문제**다.
표면이 이틀에 절반씩 바뀌는 동안(§2.2 실측: `--json` 21 → 31) `last_verified` 는
하루면 낡는다.

**해소 조건** — 두 방향이 있다.

1. **재검증 주기를 표면 변경 속도에 맞춘다** — 표면 관련 문서의 `last_verified` 가
   N일 넘으면 CI 가 경고
2. **숫자·목록을 문서에서 빼고 재현 명령만 남긴다** — 이 문서가 이미 그렇게 하고 있다
   ("현재 동작은 항상 `rhwp edit` 와 `rhwp capabilities` 로 확인한다"). 그런데
   그 지시문 **바로 위에** 낡은 목록이 있다. **지시문과 목록을 함께 두면 목록이 이긴다**

---

## 4. 종료 조건 종합

| 이슈 | 이 로드맵이 끝났다고 말할 수 있는 조건 | 오늘 |
|---|---|---|
| [#3608](https://github.com/edwardkim/rhwp/issues/3608) | M1~M17 실측 매트릭스 충족 + M18~M30 분리 + §1-D 제외가 실측과 일치 | 미충족 (D3·D4) |
| [#3719](https://github.com/edwardkim/rhwp/issues/3719) | 층별 DoD 6개 충족 + 판정 대기 2건 결론 머지 + 불변식 7 가드화 | 미충족 (D8) |
| [#3787](https://github.com/edwardkim/rhwp/issues/3787) | S10 머지 + 오탐 0 재확인 + `inspect` 자기서술 구멍 해소 | 미충족 (S10 열림, B5) |
| [#3793](https://github.com/edwardkim/rhwp/issues/3793) | 11편 실재·등재·상호링크 + **현행성 유지 규칙 확정** | 산출물 충족, 유지 규칙 미정 (D13) |
| [#3796](https://github.com/edwardkim/rhwp/issues/3796) | 오탐 0 + 여섯 로드맵에 명시 + 플레이북 편입 | 미충족 (D7) |
| [#3828](https://github.com/edwardkim/rhwp/issues/3828) | B1~B4 머지 + 링크 + `explain`↔`digest` 경계 확정 | 미충족 (0/4) |
| [#3869](https://github.com/edwardkim/rhwp/issues/3869) | `pip install` 동작 + W2 계약 테스트 + 성능 실측 + M24 관계 정리 | 미충족 (코드 0, D2) |

**종료 조건을 스스로 적은 로드맵은 넷**(#3608 §2 · #3719 §5 · #3787 §4 · #3793 §5)이고,
**셋은 수용 기준만 있고 종료 조건이 없다**(#3796 · #3828 · #3869).
수용 기준은 "이 조각이 통과했나"이고 종료 조건은 "이 축이 끝났나"다 — 다른 질문이다.

---

## 5. 확인되지 않음

| # | 항목 | 왜 확인 못 했나 |
|---|---|---|
| **U-1** | 닫힌 PR 167건 중 devel 반영 건수 | GitHub API 상 `mergedAt` 이 있는 것은 11건뿐. 이 문서는 **산출물 실재를 개별 확인한 것만** 근거로 썼다 |
| **U-2** | [#3793](https://github.com/edwardkim/rhwp/issues/3793) 11편 **전부**의 프런트매터 스키마 준수 | `threat_model.md`·`README.md` 2편만 직접 확인. 나머지 9편은 미확인 |
| **U-3** | [#3796](https://github.com/edwardkim/rhwp/issues/3796) 단계 0~5 의 **플레이북 편입 여부** | `agent_surface_playbook.md` 본문을 이 작업에서 읽지 않았다 |
| **U-4** | [#3828](https://github.com/edwardkim/rhwp/issues/3828) B4(`explain`)와 `digest` 의 **기능 중복 정도** | `explain` 이 devel 에 없어 출력을 대조할 수 없다. PR [#3832](https://github.com/edwardkim/rhwp/pull/3832) 본문만으로는 판정 불가 |
| **U-5** | [#3869](https://github.com/edwardkim/rhwp/issues/3869) 의 WASM 성능 주장 | 이 PC 에서 WASM 빌드를 시도하지 않았다. PR #3876 의 비용 실측은 인용이며 재현하지 않았다 |
| **U-6** | Node 바인딩의 실행 동작 | [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 이미 명시 — `node_modules` 부재로 `vitest`/`tsc` 미실행 |
| **U-7** | 일곱 로드맵에 대한 **저장소 방향 결정권자의 판단** | 코멘트 실측: #3608 **14** · #3719 **3** · #3828 **1** · 나머지 넷 **0**. 그런데 그중 **17건이 로드맵 작성자 본인의 진행 갱신**이고, 타인 코멘트는 #3608 의 `jangster77`(2026-07-31, 통합 PR #3647 착지 통지) **1건뿐**이다. 로드맵의 **내용**에 대한 외부 검토는 확인되지 않았다 |

> U-7 은 이 지도 전체의 한계다. 이 문서의 층 배치·중복 판정은 **본문 대조와 실측**에 근거하며,
> 저장소의 방향 결정권자가 동의했다는 뜻이 아니다.

---

## 6. 관련 문서

- [축 지도 · 읽는 순서 · 지금 할 일](README.md) — 인접 축·절차 문서·이슈 전체 목록은 여기에 있다
- [4층 성숙도 모델](layer_model.md) — 층 구분의 논증, L1 구멍 8건 전수
- [불변식 전수](invariants.md) · [결정 대장](decision_log.md) · [미해결 공백](open_gaps.md)
- [에이전트 보안 문서 지도](../agent_security/README.md) —
  [#3793](https://github.com/edwardkim/rhwp/issues/3793) 의 산출물(§2.4)
- 상위 이슈 [#3880](https://github.com/edwardkim/rhwp/issues/3880),
  일곱 로드맵은 §1.1 표의 링크
