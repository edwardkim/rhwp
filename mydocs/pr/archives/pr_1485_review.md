# PR #1485 처리 보고서 — CanvasKit parity 구현 계획서 + drift 가드 (P30)

- PR: https://github.com/edwardkim/rhwp/pull/1485
- 제목: `render: document CanvasKit parity plan and guard drift`
- 작성자: seo-rii (collaborator)
- 연결: Refs #536 (멀티 렌더러 트래킹, close 안 함)
- base ← head: `devel` ← `seo-rii:render-p30`
- 처리일: 2026-06-29

## 1. 처리 결정

**admin merge.** #1447(P28)·#1469(P29) 후속 P30 — CanvasKit replay parity 다음 단계를
**런타임 변경 전에 문서로 명시**하고, 그 계획서가 살아있는 코드 touchpoint와 어긋나지 않도록
e2e 가드를 확장한다. 런타임 렌더링 동작 무변경(문서·가드·CI path filter만)으로 저위험.
MERGEABLE + CI 전부 pass + 충돌 0.

## 2. BEHIND 착시 분석 (집중 검토 지점)

`git diff devel pr1485` 단순 비교는 render-diff.yml 의 `preflight` job(146줄) 삭제·`mydocs/**`
제거처럼 보이나, **PR 이 한 변경이 아니다**. merge-base(`f13acedc`) 기준 PR 의 **순변경은
순수 추가뿐**임을 확인:

| 파일 | 순변경 |
|---|---|
| `.github/workflows/render-diff.yml` | **+2줄** (docs path filter 2개 추가) |
| `docs/canvaskit-parity-implementation.md` | +143 (신규 계획서) |
| `docs/text-ir-v2.md` | +10 (계획서 링크) |
| `rhwp-studio/e2e/renderer-contract.test.mjs` | +82 (계획 drift 가드) |

**삭제 0, 런타임 코드 0.** preflight job·`mydocs/**` 는 PR base 이후 devel 이 추가한 것으로,
3-way 머지가 정확히 보존한다(아래 검증). PR 이 BEHIND(rebase 필요)일 뿐 충돌 없음.

## 3. 머지 트리 검증 (착시 배제)

devel 임시 머지 트리에서 직접 확인:

| 검증 | 결과 |
|---|---|
| 자동 머지 | 충돌 없이 성공 |
| render-diff.yml `preflight` job | **보존** (devel 분 유지) |
| paths 블록 | devel 전체 항목(`mydocs/**` line 15 포함) + PR docs 2줄 추가 — 정확 병합 |
| `node e2e/renderer-contract.test.mjs` | **`renderer backend contract guard passed`** (exit 0) |

## 4. 변경 의의

- `docs/canvaskit-parity-implementation.md`: CanvasKit = 직접 replay 백엔드, Canvas2D = 호환
  기준, 미지원 작업은 진단·가드 fallback 으로 가시화 — 다음 widening 전 계약 명문화.
- e2e 가드: 계획서가 살아있는 touchpoint(`render-diff.yml` 등) + 필수 개념('CanvasKit direct
  replay', 'render-diff CI')을 계속 언급하는지 검증 → 계획-코드 drift 를 CI 가 차단.
- render-diff path filter 에 docs 2개 추가: 문서만 바뀐 계약 drift 도 가드를 돌린다.
- 비목표 명확: public canvas 기본값 불변, 런타임 replay widening 없음, 새 스키마 필드 없음.

## 5. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 시뮬레이션 | 0건 |
| 순변경 src/런타임 | 0건 (docs+e2e+ci path만) |
| 머지 트리 renderer-contract 가드 | passed |
| #1469(P29) 선례 | admin merge 동일 패턴(가드/CI만, 런타임 무변경) |

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1485_review.md`
