---
kind: decision
status: active
canonical: mydocs/tech/tiny_model_macro_tools.md
last_verified: 2026-07-31
---

# 초소형 모델용 매크로 도구 축 설계 결정

- **이슈**: [#3633](https://github.com/edwardkim/rhwp/issues/3633) (로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) 신규 항목)
- **범위**: 매크로 3종의 설계 원칙과 명세. 1호 `digest` 는 구현 완료, 2·3호는 봉투 초안만 확정.
- **계약 테스트**: `tests/digest_macro_contract.rs`

## 1. 왜 — 초소형 모델의 제약 3가지

현행 에이전트 표면(#3608)은 "capabilities 로 배우고 → info 로 훑고 → export-text/search 로
파고 → edit 로 고치고 → ir-diff 로 검증"하는 다단 도구 체이닝을 전제한다. 로컬에서 도는
초소형 모델(4B급)은 이 전제가 성립하지 않는다:

| # | 제약 | 증상 |
| --- | --- | --- |
| 1 | **체이닝 불가** | 5회 연쇄 호출 계획을 세우지 못하고 2~3번째 호출에서 목적을 잃는다 |
| 2 | **컨텍스트 극소** | 도구 설명·중간 산출물 수천 토큰이 그 자체로 컨텍스트를 넘치게 한다 |
| 3 | **지시 이탈** | "다음에 뭘 하라"가 응답 안에 명시돼 있지 않으면 임의 행동으로 샌다 |

공백은 기능이 아니라 **호출 단위의 굵기**다. 코어 기능은 전부 있고, 한 번 호출로 끝나는
굵은 포장이 없을 뿐이다.

## 2. 대응 원칙 3가지 (제약 1:1 대응)

1. **원콜 워크플로** — 다단 파이프라인을 도구 **내부**로 옮겨 결정론적으로(모델 판단 개입
   없이) 수행한다. 모델은 1회 호출만 한다. 내부 단계는 전부 기존 코어 함수 재사용이며
   매크로에 새 해석 로직을 두지 않는다.
2. **40자 이내 도구 설명** — 매크로 도구의 MCP `description` 은 40자 이내로 극단 압축한다.
   도구 목록 자체가 컨텍스트 예산을 잠식하는 모델이 1차 소비자이기 때문이다. 길이는
   계약 테스트(`digest_registered_in_mcp_with_compact_description`)가 감시한다.
3. **nextStep 고정 유도** — 봉투의 `nextStep` 은 **고정 문자열 계약**이다. 모델이 다음
   행동을 지어내지 않고 받아 적게 한다. 문구 변경은 계약 테스트가 잡는 의도적 결정이어야
   한다(`digest` 의 원천은 `src/main.rs` 의 `DIGEST_NEXT_STEP`).

## 3. 매크로 3종 명세

### 3.1 1호 `hwp_digest` (CLI `digest`) — 구현 완료 (#3633)

- **CLI**: `rhwp digest <파일> [--max-chars N] [--json]` — 기계 전용 명령으로 항상 봉투
  한 줄 JSON 을 낸다. 실패 시 stdout 0바이트, 종료 코드는 #2707 계약(0/1/2).
- **내부 파이프라인** (전부 기존 원천 재사용): `load_document` → `info_json_value` 의
  format·pageCount·paraCount + `build_structure(Auto)` 최상위 노드 제목(최대 20개) +
  `extract_page_text_native` 페이지 0~2 발췌를 `--max-chars`(기본 2000) 문자에서 절단.
- **봉투**:

```json
{
  "schemaVersion": "1.0",
  "source": "<경로>",
  "format": "hwp5|hwpx|hwp3|hml",
  "pageCount": 16,
  "paraCount": 195,
  "outline": ["1. 소개", "2. 연관된 작업들", "..."],
  "excerpt": "첫 페이지 발췌...",
  "truncated": true,
  "nextStep": "더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json"
}
```

### 3.2 2호 `hwp_fill_and_verify` — 봉투 초안 (후속 이슈로 승격)

fields 조회 → fill-fields → 저장 → 재열기 검증을 원콜로 묶는다. 내부 단계는
`collect_all_fields`/`set_field_value_by_name_at`/`edit_serialize` 재사용.

```json
{
  "schemaVersion": "1.0",
  "source": "<경로>",
  "output": "<산출 경로>",
  "outputFormat": "hwp5|hwpx",
  "filledCount": 3,
  "filled": [{ "name": "성명", "occurrence": 0, "value": "..." }],
  "notFound": [],
  "ambiguous": [],
  "verify": { "reopened": true, "fieldsMatch": true },
  "truncated": false,
  "nextStep": "결과 확인은 fields --json <산출>, 눈검증은 export-svg <산출>"
}
```

### 3.3 3호 `hwp_replace_and_verify` — 봉투 초안 (후속 이슈로 승격)

replace-text(dry-run 계수) → 치환 → 저장 → 재열기 검증을 원콜로 묶는다. 내부 단계는
`replace_all_native`/`edit_serialize` 재사용.

```json
{
  "schemaVersion": "1.0",
  "source": "<경로>",
  "output": "<산출 경로>",
  "outputFormat": "hwp5|hwpx",
  "find": "구기관명",
  "replace": "신기관명",
  "replacedCount": 12,
  "verify": { "reopened": true, "residualFindCount": 0 },
  "truncated": false,
  "nextStep": "잔존 확인은 search --json <산출> <찾은말>, 눈검증은 export-svg <산출>"
}
```

초안 공통 규약: `verify` 는 산출물을 **재열기**해 얻은 실측이어야 하고(직렬화 반환값
재보고 금지), `nextStep` 은 1호와 같은 고정 문자열 계약이며, 판정 어휘
(filledCount/notFound/ambiguous/replacedCount)는 기존 `edit` 표면과 동형이어야 한다.

## 4. 인접 축과의 관계

- **weak-agent-proofing #3630 (P1 did-you-mean · P2 편집 --verify 내장 · P3 changedPages ·
  P4 nextCall)** — 상보적이다. #3630 은 기존 표면의 **오류·검증을 표면이 대신**하는 축이고,
  본 축은 **계획(체이닝)을 표면이 대신**하는 축이다. 접점 두 곳: 2·3호의 `verify` 내장은
  P2 와 같은 원리이므로 P2 가 먼저 머지되면 2·3호는 그 경로를 재사용한다. 성공 봉투의
  `nextStep`(고정 문자열)과 P4 의 오류 봉투 `nextCall`(교정 제안)은 역할이 다르므로 어휘를
  통일하되 병합하지 않는다.
- **역할 라우터 #3629 (capabilities/mcp-serve --profile)** — 라우터에 '초소형(tiny)'
  프로필을 추가해 매크로 3종 + 최소 도구만 노출하는 것이 본 축의 완성형이다. 다만 프로필
  스키마가 #3629 에서 확정되므로, **'초소형' 프로필 추가는 #3629 머지 후 후속 이슈로
  진행한다** (본 축은 프로필 없이도 단독 동작).

## 5. 결정 요약

| 결정 | 근거 |
| --- | --- |
| 매크로는 기존 코어 함수 재사용만 (새 해석 로직 금지) | CLI 계약(#2707)·판정 어휘 동형 유지, 드리프트 방지 |
| `digest` 는 `--json` 유무와 무관하게 항상 봉투 한 줄 | 기계 전용 명령 — 사람용 표면을 이원화하지 않는다 |
| `nextStep` 은 상수 문자열 + 계약 테스트 고정 | 지시 이탈 차단 — 유도문이 조용히 바뀌면 소비자 프롬프트가 깨진다 |
| MCP 설명 40자 이내를 테스트로 강제 | 컨텍스트 절약이 목적인 축에서 설명 비대화는 자기모순 |
| 2·3호는 봉투 초안만 선확정 | 어휘를 먼저 고정해 후속 구현 간 드리프트를 막는다 |
