# Task M100 #2124 Stage 2 완료 보고 - 공식 metrics snapshot

- 이슈: #2124
- 단계: Stage 2 - 공식 metrics snapshot
- 작성일: 2026-07-10
- 브랜치: `task2124-frontend-baseline`
- 기준 커밋: `upstream/devel` `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 선행 단계: `mydocs/working/task_m100_2124_stage1.md`

## 1. 완료 요약

schema v2 공식 frontend metrics snapshot과 재현성 manifest를 저장했다. 측정 시 전체 worktree에는
#2124 도구·문서 변경이 있었지만 측정 대상 제품 소스는 clean이었다.

## 2. 산출물

| 파일 | 내용 |
|------|------|
| `mydocs/metrics/frontend/2026-07-11/metrics.json` | 함수별 evidence와 환경을 포함한 공식 JSON |
| `mydocs/metrics/frontend/2026-07-11/summary.md` | reviewer용 요약 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_baseline_manifest.md` | commit, scope, 도구 hash, font inventory, 검증 |

## 3. 핵심 수치

| Group | Files | Code lines | Total CC | CC>25 / sum | CC>100 | Max CC |
|------|------:|-----------:|---------:|------------:|-------:|-------:|
| Studio runtime | 145 | 54,007 | 9,505 | 47 / 3,346 | 6 | 453 |
| Chrome extension | 15 | 2,002 | 444 | 4 / 132 | 0 | 40 |
| Firefox extension | 15 | 1,999 | 444 | 4 / 132 | 0 | 40 |
| Safari extension | 3 | 1,136 | 299 | 3 / 115 | 0 | 43 |
| Shared frontend | 9 | 737 | 107 | 0 / 0 | 0 | 14 |
| VS Code extension | 4 | 1,238 | 159 | 0 / 0 | 0 | 23 |
| npm editor wrapper | 2 | 241 | 19 | 0 / 0 | 0 | 8 |
| legacy `/web` | 10 | 5,795 | 828 | 4 / 207 | 0 | 86 |

전체는 203 files, 67,155 code lines, Total CC 11,805, 전체 Top 20 합 2,581이다.

## 4. 재현성

| 항목 | 값 |
|------|----|
| generatedAt | `2026-07-11T03:01:51.538Z` |
| Node / platform | `v24.15.0` / Darwin arm64 `25.5.0` |
| metrics script SHA-256 | `6984e6eb7b019e76c040d98360c403449994da0275d03e4dd0978c8a315a496b` |
| metrics lock SHA-256 | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| Studio lock SHA-256 | `a9992df61824d3778c206e59ad89ecd8156e2835af728752e9ffc77bee4885dc` |
| `HEAD` / `upstream/devel` | `3077f96d1f9931c50d6d62be77b389d4f66470a9` |
| 측정 대상 source | clean |

snapshot은 advisory다. 후속 PR은 schema v2 `--compare` 결과를 제공하되 사전 리뷰 없이 threshold를
fail gate로 승격하지 않는다.
