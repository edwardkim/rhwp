---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/trend_merge_dag_2026h2.md
last_verified: 2026-08-10
---

# 합침의 시대 — LLM N-병합 정세와 작업 계보 DAG 확장 설계서 (2026 H2)

- 좌표: 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) → 3년 선행 축(작업 계보,
  [#4401](https://github.com/edwardkim/rhwp/issues/4401) ·
  [PR #4406](https://github.com/edwardkim/rhwp/pull/4406))의 다음 지평
- 등급: **[지평]** — 이 문서는 코드 0줄이다. 완료 표기는 머지 링크와 함께만 한다는
  로드맵 규약에 따라, 여기 적힌 어떤 것도 구현 완료를 주장하지 않는다.
- 이슈: [#4407](https://github.com/edwardkim/rhwp/issues/4407)
- 착수 조건: [PR #4406](https://github.com/edwardkim/rhwp/pull/4406) 머지 **그리고**
  합류 수요 1건 실측(§7.2의 시나리오가 실제 파이프라인에서 발생). 착수는 근거가 결정한다.

출발 질문은 한 줄이다 — **"LLM 두 개가 합쳐지는 게 앞으로 나올까? N개가 될 수도 있고."**

답도 한 줄로 줄일 수 있다: **이미 왔고, N은 층마다 다른 모양으로 온다.** 이 문서의
전반부(§2–§5)는 그 지형을 층별로 전수 정리하고, 후반부(§6–§9)는 그 세계가 반드시
요구하게 될 인프라 — **합침의 증명** — 를 rhwp 작업 계보의 DAG 확장으로 설계한다.

---

## 0. 한 장 요약

| 층 | 무엇을 합치나 | 대표 기법 | 성숙도 | N개 확장 | rhwp 접점 |
|---|---|---|---|---|---|
| 1층 가중치 병합 | 같은 조상의 파인튜닝들 | soup·task arithmetic·TIES·DARE·진화적 병합 | **상품화** (오픈웨이트 일상) | 이미 N (soup 은 원래 N개) | 병합 레시피 = 계획서, 병합 산출 = 캡슐 (§9.4) |
| 2층 MoE 업사이클링 | N개를 전문가로 공존 | sparse upcycling·프랑켄MoE | 실용 (커뮤니티 실증) | 구조적으로 N | 동일 — "합쳐진 산출물"의 한 형태 |
| 3층 시스템 수준 | 서로 다른 모델을 제품 하나로 | 라우팅·증류·추론 합의(MoA) | **프런티어의 실제 방식** | 이미 N | 멀티 에이전트 합본 문서 = 같은 문제 (§7.2) |
| 검증 층 (공백) | 위 전부의 **출처와 재현성** | 카드·BOM·C2PA의 active asset 결속/ingredient 추가시 검증 기록과 레시피 재실행은 서로 다른 보장 | **재현 공백** | 합칠수록 커짐 | **여기가 우리 자리** — 해시 결속 + 재현 검증 (§6) |

핵심 논증: 합침 기술은 층마다 이미 왔거나 오고 있고, 합쳐진 결과물이 늘수록
"무엇을 어디서 얼마나 섞었나"를 **제3자가 재현할 방법**의 공백이 커진다. 카드의
작성자 입력, BOM의 구성 목록, C2PA의 active asset 무결성과 ingredient 추가 시점
검증 기록은 각각 유용하지만 병합 레시피 재실행과 같은 보장은 아니다. rhwp 계보
축은 이 재현 공백을 겨냥한다. 다만 단일
부모 해시 체인을 N개로 늘리는 것만으로는 충분하지 않다. 각 부모 산출을 자식의
구체적 입력 슬롯에 결속하고, 입력까지 포함한 실행 키로 재현해야 한다(§7).

---

## 1. 왜 문서 엔진 저장소에 이 문서가 있나

3년 선행 축의 논리 사슬을 다시 적는다:

1. **영수증** ([#4391](https://github.com/edwardkim/rhwp/issues/4391)) — 작업 **하나**가
   사실임을 증명한다 (입력·계획·산출 3해시, attest/verify).
2. **감사** ([#4393](https://github.com/edwardkim/rhwp/issues/4393)) — 작업 **집합**이
   재현됨을 회계한다 (재현율, 실패 명세).
3. **계보** ([#4401](https://github.com/edwardkim/rhwp/issues/4401)) — 작업 **역사**가
   이어졌고 사후에 바뀌지 않았음을 증명한다 (부모 1개의 해시 체인).

다음 질문은 필연이다: **역사가 사슬이 아니라 합류라면?** 문서 파이프라인에서 이미
자연 발생하는 형태다 — 분할(split) 후 병렬 편집한 산출물들을 합본하는 순간, 최종
산출물의 부모는 N개다. 모델 세계의 병합·MoE·라우팅도 구조가 같다: N개의 조상,
하나의 산출물, 그리고 "어떤 조상을 얼마나 섞었나"라는 검증 불가능한 주장.

문서 엔진이 이 문서를 갖는 이유는 감상이 아니다. rhwp 는 PR #4406의 제한된
run/replay 회귀에서 같은 입력·계획의 바이트 재현을 실측했고, 그 위에
영수증→감사→계보 후보를 쌓았다. 이 근거를 엔진 전체나 모든 외부 재료의 결정론으로
확대하지 않는다. 합류(DAG)는 이 사다리의 다음 단이며, 문서 작업에서 먼저 실증한
구조가 모델 아티팩트로 일반화될 수 있는지를 M4에서 따로 검증한다(§9.4).

---

## 2. 정세 1층 — 가중치 병합: 같은 계보 안의 산술

### 2.1 계보 (검증된 서지)

- **Model soups** (Wortsman 외, ICML 2022,
  [arXiv:2203.05482](https://arxiv.org/abs/2203.05482)) — 서로 다른 하이퍼파라미터로
  파인튜닝한 체크포인트들의 **가중치 평균**이 단일 최고 모델보다 정확도·강건성을
  올린다. 추론 비용 증가 0. "N개 병합"의 원형이 이미 N개였다.
- **Task arithmetic** (Ilharco 외, 2022 공개) — 태스크 벡터(파인튜닝 − 베이스)의
  덧셈·뺄셈으로 능력을 **합성하고 제거**한다. 병합이 "평균"에서 "대수"로 승격.
- **TIES-Merging** (Yadav 외, NeurIPS 2023) — 다태스크 병합의 간섭을 부호 충돌
  해소 + 상위 크기 절단으로 완화. N-태스크 병합의 실무 표준 중 하나.
- **DARE** (Yu 외, 2023,
  [arXiv:2311.03099](https://arxiv.org/abs/2311.03099)) — 델타 파라미터의 90%+ 를
  무작위 탈락시키고 남은 것을 재스케일해도 능력이 보존됨을 보임("Language Models
  are Super Mario"). 델타의 희소성 실증 — 병합 전 간섭 완화 플러그인으로 쓰인다.
- **SLERP** — 두 모델 구면 보간. 논문보다 실무(오픈웨이트 커뮤니티)의 기본기.
- **mergekit** — 커뮤니티 표준 도구. **YAML 레시피 선언형**이라는 사실이 이 문서
  후반의 설계와 직결된다: 병합은 이미 "계획서 실행"의 형태를 하고 있다(§9.4).
- **진화적 병합** (Sakana AI, 2024,
  [arXiv:2403.13187](https://arxiv.org/abs/2403.13187)) — 병합 레시피 공간을 진화
  탐색으로 자동화. 일본어 LLM × 수학 모델 병합(EvoLLM-JP)이 일본어 수학 추론에서
  70B급을 능가하는 7B를 만들었다. **레시피의 자동 탐색까지 왔다** — 사람의 직관
  없이 N개 조합이 발견되는 시대.

2023–24년 오픈웨이트 리더보드 상위권이 머지 모델로 채워졌던 시기가 실제로
있었다. "두 개 합치기"는 미래가 아니라 지나간 유행이 됐을 만큼 일상이다.

### 2.2 왜 되나 (직관)

같은 베이스에서 갈라진 자손들은 손실 지형의 **같은 분지** 안에 있다 — 사이 경로의
장벽이 낮아 선형 보간이 성능을 크게 깨지 않는다(선형 모드 연결성). git re-basin
계열(Ainsworth 외, 2022 공개)은 뉴런 순열 대칭을 정렬하면 **독립 학습된** 모델들
사이에서도 분지를 합칠 수 있음을 보였다 — "같은 조상" 전제를 부분적으로 무너뜨리는
연구 축이다.

### 2.3 한계 (정직하게)

- **공통 조상 전제.** 아키텍처가 다르면(차원·층수·토크나이저) 가중치 산술 자체가
  정의되지 않는다. 이종 병합 시도(비전에서 ZipIt 의 중복 특징 압축, 표현 스티칭
  연구)는 있으나 **프런티어 LLM 규모의 실증은 아직 없다.**
- **평가 민감성.** 병합 성능 보고는 벤치 선택에 민감하고, 리더보드 과적합 논란이
  병행했다. "병합이 항상 이득"은 사실이 아니다 — 간섭으로 능력이 깎이는 조합이
  흔하고, TIES/DARE 는 정확히 그 완화책으로 나왔다.
- **출처 소실.** 병합 산출물의 가중치에서 조상 기여를 **역산할 방법이 없다.**
  이 한계가 §6의 검증 공백 논증의 물질적 근거다.

### 2.4 N 확장의 현재

soup 은 원래 N개 평균이고, TIES/DARE 는 N 태스크 병합용이며, 진화적 병합은 N개
조합의 자동 탐색이다. **같은 계보 안에서 "N개"는 이미 상품이다.** 남은 연구
문제는 N이 아니라 "이종(異種)"이다.

---

## 3. 정세 2층 — MoE 업사이클링: N개의 공존

- **Sparse upcycling** (Komatsuzaki 외, 2022 공개) — dense 체크포인트를 MoE 로
  승격시키는 표준 경로. 훈련 비용을 아끼며 용량을 늘린다.
- **Mixtral 8x7B** (Mistral, 2023 말 공개) — 공개 가중치 MoE 가 상용 품질을
  낸다는 증명. 이후 공개 모델 진영에서 MoE 는 보편 선택지가 됐다.
- **프랑켄MoE** — mergekit 계열 도구로 파인튜닝 N개를 전문가로 꿰매고 라우터를
  얹는 커뮤니티 실전. 품질 편차는 크지만 "N개를 한 모델 안에 공존"이 개인
  수준에서 가능함을 보였다.

2층의 특징: 파라미터는 공존하되 융합은 아니다 — 라우터가 토큰마다 소수 전문가를
고른다. **능력은 합쳐지고 계산은 나뉜다.** 합침의 정치학으로 보면 1층(융합)과
3층(분업)의 중간형이다.

---

## 4. 정세 3층 — 시스템 수준: 프런티어의 실제 방식

프런티어 랩들은 서로 다른 대형 모델을 가중치로 합치지 않는다. 대신:

- **체크포인트 평균** — 훈련 말기 평균(EMA 포함)은 오래된 파이프라인 표준.
  1층 기술이 프런티어 내부에서는 "훈련 안정화 도구"로 쓰인다.
- **증류** — N 교사 → 1 학생. 프런티어 소형 모델 계열의 통상 제조 경로.
- **라우팅** — 요청마다 모델을 배분해 여러 모델을 **한 제품처럼** 판다. 빠른
  모델↔추론 모델 자동 배분은 2025년 이후 상용 제품의 기본 형태가 됐다.
- **추론 시점 합의** — **Mixture-of-Agents** (Wang 외, 2024,
  [arXiv:2406.04692](https://arxiv.org/abs/2406.04692)): 여러 LLM 을 층상으로
  쌓아 이전 층의 출력 전부를 보조 정보로 받게 하면, 공개 모델만으로 당대 최강
  단일 모델을 넘는 설정(AlpacaEval 2.0 65.1%)이 실증됐다.
- **투기적 디코딩** — 소형 제안자 + 대형 검증자 쌍이 "둘이 하나처럼" 작동하는
  추론 가속 표준.

판정: **프런티어에서 "N이 하나 되는" 일은 가중치가 아니라 시스템 계층에서 이미
일어났다.** 사용자 눈에 보이는 "모델 하나"가 내부적으로 몇 개인지는 이미 공개
정보가 아니다 — 그리고 이것이 §6의 공백을 프런티어까지 확장한다.

---

## 5. 시간축 전망 (확신도 표기)

| 전망 | 확신도 | 근거 |
|---|---|---|
| 같은 계보 병합의 일상화 지속 | **높음** | 이미 상품화·도구화 완료, 비용이 0에 가깝다 |
| 시스템 수준 합침(라우팅·증류·합의)의 확대 | **높음** | 프런티어 제품 구조가 이미 이것이다 |
| 합침 출처 증명 수요의 증가 | **높음** | SBOM→AIBOM 규제 흐름 + 오염 전파 사고의 필연성 (§6) |
| 이종 아키텍처의 준-병합(어댑터·스티칭 경유) 부분 실용화 | 중간 | 연구 축은 활발하나 규모 실증 부재 |
| 서로 다른 프런티어 2모델의 진짜 가중치 융합 | 낮음 | 표현 정렬이 미해결 — 증류+라우팅이 그 자리를 대신 채우는 중 |

함의: 어느 시나리오로 가든 **"합쳐진 무언가"의 총량은 단조 증가**한다. 그리고
아래 §6이 보이듯, 증가분만큼 검증 공백이 커진다.

---

## 6. 검증 공백 — 합침의 증명이 없다

### 6.1 무결성·진술·재현성은 서로 다른 보장이다

현존 수단은 한 묶음의 "자기 신고"가 아니라 서로 다른 층을 담당한다.

- **모델 카드와 허깅페이스 계보 메타데이터** — 조상·데이터·라이선스를 기록한다.
  허깅페이스의 [`base_model`](https://huggingface.co/docs/hub/main/model-cards#specifying-a-base-model)은
  병합 조상 여러 개도 표현하지만 저장소 작성자가 넣는 메타데이터다.
- **AIBOM** (AI Bill of Materials) — 모델·데이터·도구 등 구성요소의 목록과 교환
  형식을 다룬다. 생성 방식, 서명, 투명성 로그와 결합하면 신뢰 수준이 달라지므로
  AIBOM 전체를 하나의 검증 수준으로 단정하지 않는다.
- **C2PA** — 단순 자기 신고보다 강하지만 active asset과 ingredient의 보장을
  구분해야 한다. active asset은 hard binding으로 manifest와 자산을 결속하므로 현재
  자산 변경과 manifest/provenance 변조를 검출할 수 있다. 반면 composed asset에
  포함된 ingredient provenance는 실제 ingredient bytes가 보통 함께 있지 않아
  소비자가 hard binding을 같은 방식으로 다시 검증할 수 없다. C2PA 2.2 §7.3.2는
  ingredient가 추가될 당시 그 hard binding과 Content Credential 유효성을 검증하고,
  그 **시점의 validation record**를 active asset의 Content Credential에 포함한다고
  설명한다
  ([C2PA 2.2 explainer](https://spec.c2pa.org/specifications/specifications/2.2/explainer/Explainer.html),
  [technical specification](https://spec.c2pa.org/specifications/specifications/2.2/specs/C2PA_Specification.html)).
  이 기록은 ingredient bytes의 현재 재검증이나 병합 레시피 재실행과 동일하지 않다.
  서명된 assertion의 의미가 사실인지도 C2PA 자체가 가치 판단하지 않는다.

따라서 남은 공백은 **출처 수단 전체가 아니라 레시피 재현성**이다. 병합 모델이
"A 60% + B 40%"라고 기록해도 제3자가 정확한 입력 아티팩트·도구·연산 조건으로
재실행해 같은 가중치를 얻는지는 별도 계약이다. §2.3에서 봤듯 산출 가중치만으로
조상 기여를 일반적으로 복원할 수 없으므로, 레시피·입력 digest·실행 프로필을 함께
결속해야 한다.

### 6.2 오염 전파 — 공백의 비용

부모 하나에 결함(백도어·저작권 침해 데이터·PII)이 발견됐다고 하자. 물어야 할
질문은 "이 부모를 섞은 산출물이 어디까지 퍼졌나"다. 계보가 검증 가능하면 리콜
범위가 그래프 탐색 한 번이고, 신고제뿐이면 **전수 재감사 외에 방법이 없다.**
소프트웨어 공급망이 SBOM 을 강제하게 된 경로(Log4j 류 사고 → 규제)를 AI 가
그대로 밟을 것이라 보는 이유다.

### 6.3 문서 세계는 같은 문제를 먼저 겪는다

멀티 에이전트 합본 문서에서 표 하나가 틀렸을 때 — 어느 에이전트의 산출에서 온
값인가? 계보 없이는 답이 없고, 계보가 있으면 그래프 한 번이다. 문서 파이프라인은
모델 병합보다 **주기가 짧고 산출물이 작아** 검증 인프라의 실증장으로 먼저 온다.
rhwp 가 문서에서 먼저 닫고 모델로 일반화하는 순서(§9.4)가 그래서 성립한다.

### 6.4 rhwp 계보의 차별점

[PR #4406](https://github.com/edwardkim/rhwp/pull/4406)의 merge 후보가 제공하는 것:

1. **해시 체인** — 자식이 부모 캡슐 파일의 SHA-256 을 내장. 사후 변조는
   `parentOk:false` 로 폭로된다 (실측 테스트 고정).
2. **계보 불변식** — 부모의 산출 해시 == 자식의 입력 해시(`lineageOk`). "이전
   작업의 산출이 다음 작업의 입력"이라는 연대기의 정의 자체를 판정한다.
3. **재현 검증** — `--deep` 이 각 캡슐의 결속된 plan text를 재실행하고 입력·step·
   산출 해시를 대조한다. 현재 회귀가 증명하는 범위는 해당 run/replay 계획과 fixture다.

다음 단계에는 세 요소가 함께 필요하다. **(1) 다중 부모 표현, (2) 각 부모 산출과
자식 입력 슬롯의 해시 결속, (3) 계획뿐 아니라 입력·도구 조건까지 포함한 재실행
식별자**다. §7은 이 세 계약을 함께 확장한다.

---

## 7. 설계 — 작업 계보의 합류(DAG) 확장

### 7.1 현재 상태 (#4406, 요약)

```json
{
  "schemaVersion": "1.0",
  "kind": "workCapsule",
  "parent": { "capsule": "a.capsule.json", "sha256": "9f2c…" },
  "plan": { "...": "원본 output 보존" },
  "receipt": { "inputSha256": "…", "planSha256": "…", "outputSha256": "…" }
}
```

`rhwp lineage <머리캡슐>` 이 단일 사슬을 거슬러 3중 판정(parentOk / lineageOk /
reproduced)하고, 깨지면 exit 3 + `brokenAt`.

### 7.2 필연 시나리오 — 합류는 이미 파이프라인에 있다

- **시나리오 M (합본).** 분할 캡슐 s → 에이전트 3기의 편집 캡슐 a·b·c → 합본
  캡슐 d라면 5노드 DAG다. s가 여러 조각을 산출했다면 `outputs[]`, d가 세 문서를
  읽었다면 `inputs[]` digest가 먼저 필요하다. 현행 run 계획은 단일 `input`/
  `output` 계약이므로 이 시나리오는 현재 명령의 완료 기능이 아니라 다중 아티팩트
  계약을 요구하는 착수 사례다.
- **시나리오 R (재료).** `csv-to-table` 의 CSV나 `insert-image` 의 그림도 산출물의
  정체성을 결정한다. 현재 replay plan의 step 집합에는 이 두 명령이 없고, 경로
  문자열만 기록해도 재료 바이트는 결속되지 않는다. 이들을 parent로 승격하려면
  먼저 실행기가 외부 입력 슬롯과 그 바이트 digest를 영수증에 기록해야 한다.
- **모델 세계 대응.** mergekit YAML 의 `models:` 리스트가 정확히 `parents[]` 다.
  단, 각 model 항목의 정확한 가중치 digest와 실행 프로필까지 결속해야 문서 합본의
  스키마를 병합 레시피에 재사용할 수 있다(§9.4).

### 7.3 스키마 v1.1 — `parents[]`

```json
{
  "schemaVersion": "1.1",
  "kind": "workCapsule",
  "parents": [
    {
      "capsule": "a.capsule.json",
      "sha256": "…",
      "role": "primary",
      "binding": { "parentOutput": "output", "childInput": "inputs[0]", "sha256": "…" }
    },
    {
      "capsule": "b.capsule.json",
      "sha256": "…",
      "role": "material",
      "binding": { "parentOutput": "output", "childInput": "steps[1].image", "sha256": "…" }
    }
  ],
  "plan": { "...": "…" },
  "receipt": {
    "inputs": [
      { "slot": "inputs[0]", "sha256": "…" },
      { "slot": "steps[1].image", "sha256": "…" }
    ],
    "outputs": [ { "slot": "output", "sha256": "…" } ],
    "...": "…"
  }
}
```

- **role 규약.** `primary` = 주 문서·모델 입력, `material` = CSV·이미지·참조
  아티팩트다. 역할은 UI·정책 분류이며 검증 강도를 낮추지 않는다. 둘 다 0~N개를
  허용하되 모든 비-root edge는 `binding`이 필수다. 현행 단일 입력 발급기는
  `primary` 0~1개만 만들고, 다중 primary는 `inputs[]` 실행기가 생긴 뒤 연다.
- **edge 결속.** `sha256`은 부모 **캡슐 파일** digest다. `binding.sha256`은 부모의
  지정 `receipt.outputs[]` digest, 자식의 지정 `receipt.inputs[]` digest와 모두
  같아야 한다. 경로나 role만 같은 것은 계보가 아니다. 발급 시 부모 capsule과
  자식이 실제 읽은 입력 snapshot을 대조하고 하나라도 없거나 다르면 발급을 거절한다.
- **하위호환 정규화.** 읽기: v1.0 의 `parent`(객체) → `parents` 1원소
  (`role:"primary"`, `output`→`input`, 기존 두 receipt 해시로 binding 구성),
  `parent:null` → `parents:[]`. 쓰기: v1.1부터 `parents`와 slot digest를 쓴다.
  정규화는 lineage 로더 한 곳에서 수행한다.
- **발급 검증.** unknown role/slot, 중복 child slot, 빈 binding, 64자리 hex가 아닌
  digest, 같은 실파일의 자기 parent, 부모 수 상한 초과를 fail-closed로 거절한다.
- **CLI.** 기존 `--parent A`는 `primary:output→input` 의미를 유지한다. 추가 edge는
  `--parent-input <child-slot>=<capsule>#<parent-output-slot>`와
  `--parent-material ...` 같은 명시적 slot mapping을 반복 지정한다. capsule 경로만
  받는 material 플래그는 어떤 바이트를 결속하는지 알 수 없으므로 허용하지 않는다.

### 7.4 판정의 N-부모 일반화

| 축 | 단일 사슬 (v1.0) | DAG (v1.1) |
|---|---|---|
| `parentOk` | 부모 파일 해시 == 기록 해시 | **모든** 부모에 대해 각각 판정 |
| `lineageOk` | 부모 산출 == 자식 입력 | role과 무관하게 각 edge의 부모 output == binding == 자식 input slot을 판정 |
| `reproduced` | 링크당 재실행 1회 | 서로 다른 **실행 키**마다 1회 (§7.7 비용 회계) |

불변식의 정신은 유지된다: **모든 계보 edge는 부모 산출이 자식이 실제 읽은 특정
입력이라는 등식**이다. primary/material은 표현상의 역할 차이일 뿐, material을
`parentOk`만으로 통과시키지 않는다.

### 7.5 걷기 알고리즘 (의사코드)

```text
lineage(head, deep):
  visited = {}                       # (canonicalizedAccessPath, resolutionBase) → node id
  contentIndex = {}                  # fileSha256 → node ids (복사본 보고용, 병합 키 아님)
  queue = [(head, incomingEdge=None)]
  nodes, edges, broken = [], [], []
  while queue:
    (path, incoming) = queue.pop_front()
    if incoming and len(edges) >= MAX_EDGES:
      broken += [명시적 edge limit 오류]; break
    access = canonicalizeAccessPath(path) | 실패 → incoming을 broken으로 기록, 계속
    resolutionBase = canonicalizeResolutionBase(path.parent)
    visitKey = (access, resolutionBase)
    if visitKey in visited:
      edge = incoming을 기존 node의 digest·output slot과 대조해 판정
      edges += [edge]                # 다이아몬드 간선도 판정을 생략하지 않음
      continue
    if len(nodes) >= MAX_NODES:
      broken += [명시적 node limit 오류]; break
    bytes = read(access) | 실패 → incoming을 broken으로 기록, 계속
    fsha = sha256(bytes)
    node = strictParseAndValidate(bytes)
    node.id = nodes.len
    visited[visitKey] = node.id
    contentIndex[fsha] += [node.id]
    nodes += [node]
    if incoming:
      edges += [incoming을 fsha·node output·child input slot과 대조한 판정]
    if deep:
      executionKey = (node.planSha256, orderedInputSlotDigests,
                      actualToolVersion, actualExecutionProfileSha256)
      executionKey가 완전할 때만 동일 키 결과를 재사용하고, 아니면 node를 직접 재실행
    for p in node.parents의 선언 순서:
      queue 길이가 MAX_QUEUE 이상이면 명시적 limit 오류
      queue.push((resolve(p.capsule, base=resolutionBase), incomingEdge=(node.id, p)))
  valid = broken == [] and 모든 node·edge 판정 참
```

- **방문 주키는 `(canonicalized access path, resolution base)`다.** 같은 JSON
  bytes나 같은 file identity라도 접근 경로의 base가 다르면 내부 상대 parent가
  다른 대상을 뜻할 수 있다. 따라서 hardlink alias는 서로 다른 access path로 각각
  방문하며 절대 dedup하지 않는다. file identity는 symlink가 같은 실파일을 가리키는지
  보조 확인·보고하는 데만 쓰고 방문 주키를 대체하지 않는다. file hash도 무결성과
  동일 내용 보고에만 쓴다.
- **다이아몬드 1회 검증**: child→parent 방향으로 D→{B,C}→A인 그래프에서 같은
  실파일 A는 한 번만 파싱·재실행한다. B→A와 C→A 두 간선의 결속 판정은 모두 남긴다.
- **순환**: 해시 체인 특성상 진짜 순환은 만들 수 없으나(자식 해시를 부모가 미리 알 수
  없다), 손상·위조 입력과 자원 고갈 방어는 별개다. 노드·간선·queue 상한을 각각
  두고 `>=` 경계 회귀로 정확히 제한한다.

### 7.6 봉투 v1.1 (전체 예시)

```json
{
  "schemaVersion": "1.1",
  "head": "d.capsule.json",
  "mode": "dag",
  "nodes": [
    {
      "id": 0,
      "depth": 1,
      "capsule": "d.capsule.json",
      "fileSha256": "9999999999999999999999999999999999999999999999999999999999999999",
      "inputs": [
        { "slot": "inputs[0]", "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
        { "slot": "inputs[1]", "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" },
        { "slot": "inputs[2]", "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" }
      ],
      "outputs": [
        { "slot": "output", "sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd" }
      ],
      "reproduced": true
    },
    {
      "id": 1,
      "depth": 2,
      "capsule": "a.capsule.json",
      "fileSha256": "8888888888888888888888888888888888888888888888888888888888888888",
      "inputs": [
        { "slot": "input", "sha256": "1111111111111111111111111111111111111111111111111111111111111111" }
      ],
      "outputs": [
        { "slot": "output", "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }
      ],
      "reproduced": true
    },
    {
      "id": 2,
      "depth": 2,
      "capsule": "b.capsule.json",
      "fileSha256": "7777777777777777777777777777777777777777777777777777777777777777",
      "inputs": [
        { "slot": "input", "sha256": "2222222222222222222222222222222222222222222222222222222222222222" }
      ],
      "outputs": [
        { "slot": "output", "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" }
      ],
      "reproduced": true
    },
    {
      "id": 3,
      "depth": 2,
      "capsule": "c.capsule.json",
      "fileSha256": "6666666666666666666666666666666666666666666666666666666666666666",
      "inputs": [
        { "slot": "input", "sha256": "3333333333333333333333333333333333333333333333333333333333333333" }
      ],
      "outputs": [
        { "slot": "output", "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" }
      ],
      "reproduced": true
    },
    {
      "id": 4,
      "depth": 3,
      "capsule": "s.capsule.json",
      "fileSha256": "5555555555555555555555555555555555555555555555555555555555555555",
      "inputs": [
        { "slot": "input", "sha256": "0000000000000000000000000000000000000000000000000000000000000000" }
      ],
      "outputs": [
        { "slot": "parts[0]", "sha256": "1111111111111111111111111111111111111111111111111111111111111111" },
        { "slot": "parts[1]", "sha256": "2222222222222222222222222222222222222222222222222222222222222222" },
        { "slot": "parts[2]", "sha256": "3333333333333333333333333333333333333333333333333333333333333333" }
      ],
      "reproduced": true
    }
  ],
  "edges": [
    { "child": 0, "parent": 1, "role": "primary", "parentOutput": "output",
      "childInput": "inputs[0]", "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "parentOk": true, "lineageOk": true },
    { "child": 0, "parent": 2, "role": "primary", "parentOutput": "output",
      "childInput": "inputs[1]", "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "parentOk": true, "lineageOk": true },
    { "child": 0, "parent": 3, "role": "primary", "parentOutput": "output",
      "childInput": "inputs[2]", "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      "parentOk": true, "lineageOk": true },
    { "child": 1, "parent": 4, "role": "primary", "parentOutput": "parts[0]",
      "childInput": "input", "sha256": "1111111111111111111111111111111111111111111111111111111111111111",
      "parentOk": true, "lineageOk": true },
    { "child": 2, "parent": 4, "role": "primary", "parentOutput": "parts[1]",
      "childInput": "input", "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
      "parentOk": true, "lineageOk": true },
    { "child": 3, "parent": 4, "role": "primary", "parentOutput": "parts[2]",
      "childInput": "input", "sha256": "3333333333333333333333333333333333333333333333333333333333333333",
      "parentOk": true, "lineageOk": true }
  ],
  "roots": [4],
  "nodeCount": 5,
  "edgeCount": 6,
  "maxDepth": 3,
  "valid": true,
  "brokenAt": null,
  "broken": []
}
```

- v1.0 캡슐만으로 된 단일 사슬은 정규화 뒤에도 **기존 v1.0 봉투와 `links[]` key를
  그대로 출력**한다. v1.1 capsule 또는 다중 edge가 하나라도 있으면 위 v1.1 봉투를
  출력한다. 새 소비자는 `schemaVersion`과 `mode`로 분기하며, 미정의 alias에
  하위호환을 맡기지 않는다.
- edge별 `parentOk`/`lineageOk`를 두어 parent 배열 순서에 의미를 숨기지 않는다.
  `brokenAt`은 선언 순서 BFS에서 처음 발견한 파손으로 유지하고, `broken[]`은 같은
  순서의 전체 파손 목록이다. node `depth`는 head=1로 시작하고 `maxDepth`는 root까지
  가장 긴 node-depth다. `nodeCount`·`edgeCount`·`maxDepth`의 의미를 분리한다.
- exit 규약 불변: 0 유효 / 1 IO / 2 사용법 / 3 계보 깨짐(판정은 봉투 데이터).

### 7.7 비용 회계 (`--deep`)

- 재실행 횟수의 안전한 상한은 노드 수다. 캐시할 수 있는 단위는 **고유 실행 키**
  `(planSha256, 정렬된 입력 slot digest, 실제 toolVersion,
  실제 executionProfileSha256)`다.
  같은 plan text를 같은 경로의 서로 다른 입력 바이트에 적용한 두 캡슐은 별도 키다.
- 시나리오 M은 분할 s + 편집 a·b·c + 합본 d의 5노드다. 다섯 실행 키가 모두
  다르면 5회 재실행한다. 공통 조상은 DAG 방문 집합으로 한 번만 실행한다.
- `--deep` 없는 기본 모드는 파일·캡슐·edge 해시 대조만 수행한다. 1,000노드 성능은
  구현 뒤 benchmark로 측정하며 설계 문서에서 밀리초를 선결론 내리지 않는다.

---

## 8. 위협 모델 — 무엇이 잡히고, 무엇이 안 잡히나

| # | 공격 | 잡는 축 | 판정 |
|---|---|---|---|
| T1 | 부모 캡슐 사후 변조 | 기록 해시 대조 | `parentOk:false`, exit 3 — **v1.0에서 이미 실측 고정** |
| T2 | 부모 바꿔치기 (다른 캡슐로 교체) | 동일 (파일 해시가 다르다) | `parentOk:false` |
| T3 | 산출-입력 사슬 위조 (primary 또는 material 몰래 교체) | edge의 부모 output↔binding↔자식 input slot 3자 대조 | `lineageOk:false` |
| T4 | 결과만 그럴듯한 캡슐 (재실행 불일치) | `--deep` 재현 | `reproduced:false` |
| T5 | 재료 누락 신고 | 선언된 외부 input slot 전부에 parent binding을 요구 | 선언된 slot 누락은 발급 실패. 계획 밖 비밀 입력까지는 검출 못 함 |
| T6 | 동일 bytes·hardlink capsule의 상대 경로 의미 혼동 | canonicalized access path + resolution base 방문 키 | 별도 access path는 같은 file-id여도 별도 node. 같은 논리 조상이라는 주장은 stable identity/앵커 없이는 증명 못 함 |
| T7 | **역사 전체 재작성** (뿌리부터 전부 재발급) | **해시 체인만으로는 못 잡는다** | 외부 앵커 필요 — 타임스탬프·서명·투명성 로그는 **후속 축**으로 남긴다 (본 설계 범위 밖임을 명시) |

T7 을 숨기지 않는 것이 이 표의 요점이다. git 도 동일한 한계를 갖고(히스토리 강제
재작성), 서명·원격 사본이 그 앵커 역할을 한다. 계보 축의 다음 지평은 자연히
"앵커"가 되며, 그것은 이 문서의 범위 밖이다.

---

## 9. 단계 계획 (M1–M4)

| 단계 | 내용 | DoD (전부 실측 게이트) |
|---|---|---|
| **M1** 스키마+edge 결속 | `parents[]`·role·binding, receipt `inputs[]`/`outputs[]`, v1.0 정규화 | 기존 v1.0 입력의 봉투 key/exit 판정 불변 + v1.1 발급·재독 + 누락·중복 slot/unknown role/self-parent/mismatch 거절 + 참고문헌 원문 검증 |
| **M2** DAG 걷기 | access path+resolution base 방문, v1.1 `nodes[]`/`edges[]`/`broken[]`, 자원 상한 | 5노드·6-edge D→{A,B,C}→S 유효 + 모든 edge 판정 + 같은 base의 symlink 별칭 보조 식별 + hardlink alias를 서로 다른 계보로 방문 + 같은 bytes·다른 폴더 상대 parent 회귀 + 정확한 limit 경계 |
| **M3** audit×lineage | 폴더 전수에서 체인/DAG 자동 발견·일괄 감사 + 외부 input slot 완전성 | 체인 1 + DAG 1 자동 식별·회계, 선언 slot의 parent 누락 거절, 계획 밖 입력은 비검출 한계로 보고 |
| **M4** 모델 레시피 PoC | mergekit YAML·정확한 model digest·실행 프로필을 감싼 캡슐 | 실험 브랜치 한정. 같은 실행 키의 byte-identical 재현과 입력/프로필 하나 변경 시 cache 미재사용 |

각 단계는 독립 PR 이고, M1 착수 조건은 문서 머리에 적었다. **완료 표기는 머지
링크와 함께만** — 이 표의 어떤 칸도 지금 완료가 아니다.

### 9.4 M4 의 논지 — 왜 문서 엔진이 모델 병합 증명의 실증장인가

캡슐의 구조는 (계획, 영수증, 부모들)이고, 이 3요소는 대상이 무엇이든 성립한다:

| | 문서 작업 | 모델 병합 |
|---|---|---|
| 계획 | run 계획서 JSON | mergekit YAML 레시피 |
| 영수증 | 입력·계획·산출 SHA-256 | 조상 가중치·레시피·산출 가중치 SHA-256 |
| 부모들 | 이전 작업 캡슐들 | 조상 모델 캡슐들 |
| 재현 | 계획+입력 snapshot 재실행 → 같은 바이트 | 레시피+가중치 digest+실행 프로필 재실행 → 같은 직렬화 바이트 |

가중치 병합은 학습보다 연산 범위가 좁지만 부동소수점 연산 순서·device/kernel·
dtype·라이브러리 버전·직렬화 방식에 따라 바이트가 달라질 수 있다. 따라서 "순수
텐서 산술"만으로 결정론을 선언하지 않는다. M4는 실행 프로필을 해시하고 같은
프로필에서 byte-identical 재현이 실제로 되는지 확인하는 PoC이며, 수치 근사 동등성은
별도 판정으로 분리한다.

---

## 10. 로드맵 좌표와 정직 조항

- 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) 의 3년 선행 축 아래
  지평 항목으로 연결한다. 트랙 파일(R 번호) 승격은 착수 조건 충족 후 별도 PR 로
  한다 — 이 문서는 `track_*.md` 집계 밖의 `trend_` 접두어를 쓴다(집계 규약 준수).
- 서지 중 arXiv 식별자 4건(2203.05482 · 2311.03099 · 2403.13187 · 2406.04692)은
  본 문서 작성 시점에 원문 초록 대조로 검증했다. 나머지(task arithmetic ·
  TIES · SLERP · git re-basin · sparse upcycling · Mixtral 등)는 저자·연도·성과
  수준으로만 인용하고 식별자를 적지 않았다 — **검증 안 된 번호를 적지 않는 것**이
  이 로드맵의 서지 규약이며, 전량 원문 링크 검증은 M1 DoD 에 포함했다.
- 이 문서의 전망(§5)은 판단이지 실측이 아니다. 확신도 열이 그 구분선이다.

## 11. 참고문헌

1. Wortsman 외, "Model soups: averaging weights of multiple fine-tuned models
   improves accuracy without increasing inference time", ICML 2022,
   [arXiv:2203.05482](https://arxiv.org/abs/2203.05482). — §2.1
2. Ilharco 외, "Editing Models with Task Arithmetic", 2022 공개. — §2.1
3. Yadav 외, "TIES-Merging: Resolving Interference When Merging Models",
   NeurIPS 2023. — §2.1
4. Yu 외, "Language Models are Super Mario: Absorbing Abilities from Homologous
   Models as a Free Lunch", 2023,
   [arXiv:2311.03099](https://arxiv.org/abs/2311.03099). — §2.1
5. Akiba 외 (Sakana AI), "Evolutionary Optimization of Model Merging Recipes",
   2024, [arXiv:2403.13187](https://arxiv.org/abs/2403.13187). — §2.1
6. Ainsworth 외, "Git Re-Basin: Merging Models modulo Permutation Symmetries",
   2022 공개. — §2.2
7. Komatsuzaki 외, "Sparse Upcycling: Training Mixture-of-Experts from Dense
   Checkpoints", 2022 공개. — §3
8. Wang 외, "Mixture-of-Agents Enhances Large Language Model Capabilities",
   2024, [arXiv:2406.04692](https://arxiv.org/abs/2406.04692). — §4
9. mergekit (Arcee) — 병합 레시피 선언형 도구. — §2.1, §9.4
