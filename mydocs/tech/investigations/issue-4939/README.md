---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4939.md
last_verified: 2026-08-16
---

# Issue #4939 폰트 규칙 기준선과 원장

이 디렉터리는 폰트 메트릭과 fallback 규칙의 현재 상태를 재현 가능한 기준선으로 고정하고,
각 규칙의 의미와 근거를 분리해 기록하는 조사 공간이다.

## 권위와 소비 경계

- [수행계획](../../../plans/task_m100_4939.md)이 범위와 단계별 승인 게이트의 정본이다.
- [원인 계보 보고서](../../../report/font_metrics_fallback_causal_lineage_20260816.md)는 역사와 보호
  불변식의 선행 근거다.
- 이 디렉터리의 JSON은 조사 snapshot이다. 제품 runtime의 canonical registry가 아니다.
- `src/`, `rhwp-studio/src/`, `web/`은 이 디렉터리의 JSON을 import하거나 생성 입력으로 사용하지 않는다.
- 비공개 10k 코퍼스의 원문, 파일명, 절대 경로와 식별 가능한 목록을 기록하지 않는다.

## 단계별 산출물

| 단계 | 산출물 | 상태 |
| --- | --- | --- |
| Stage 1 | `font_rule_ledger.schema.json`, `font_rule_sources.json`, source fixture와 boundary test | 작성됨 |
| Stage 2 | `font_rule_candidates.json`, `font_rule_baseline.json`, `task_m100_4939_baseline_manifest.md` | 작성됨 |
| Stage 3 | `font_rule_candidates.json` | 미착수 |
| Stage 4 | `font_rule_ledger.json`, `font_rule_ledger_summary.md` | 미착수 |
| Stage 5 | baseline 재생성과 최종 감사 | 미착수 |

Stage 1은 실제 source mapping을 수집하지 않는다. owner와 안정 selector의 존재, 규칙 행 스키마,
grouped mapping·ordered chain·algorithmic predicate의 확장 계약만 고정한다. 실제 collector와
canonical JSON writer는 Stage 2에서 구현했다. Stage 2의 candidate는 30개 source selector의
폐합 snapshot이며, 실제 mapping 행 전수 확장은 Stage 3의 승인 범위다.

- [W0 baseline manifest](task_m100_4939_baseline_manifest.md)
- `font_rule_candidates.json`: owner·selector·source digest 기준선
- `font_rule_baseline.json`: metric table·lookup projection·fixture·gate 기준선

## Stage 1 검증

```bash
node --test scripts/tests/font_rule_ledger.test.mjs
node scripts/font_rule_ledger.mjs boundary \
  --sources mydocs/tech/investigations/issue-4939/font_rule_sources.json
python3 scripts/check_markdown_links.py mydocs/tech/investigations/issue-4939
git diff --check
```

성공 조건은 invalid ledger가 거부되고, 중복 `ruleId`가 거부되며, owner나 selector가 사라졌을 때
0건 성공 대신 명시적 오류가 발생하는 것이다.
