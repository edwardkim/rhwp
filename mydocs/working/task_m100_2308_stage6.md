# M100 #2308 Stage 6 — 통합·시각 검증·문서화

## 기준

- 브랜치: `issue-2308-render-normalized-derived-state`
- 비교 기준: `upstream/devel@cbddc1cd87084b60685da9a2b4369a4511d86173`
- 코드 기준: Stage 5 `6438a4cfb`
- 완료일: 2026-07-23
- 상태: focused·OVR 완료, 전체 게이트 승인 대기

## 최종 코드 동일성

Stage 2~5를 재구성한 뒤 `src/`와 `tests/`를 기존 구현 완료본
`1f2054faafdf0d82f6fa7634f01f4d2537f42036`과 비교했다. 파일 차이는 0건이다. 이력 재구성은
검증된 최종 구현을 바꾸지 않고 Stage 경계와 문서 추적성을 복원했다.

## OVR

```text
python3 tools/object_visual_regression.py \
  samples/76076_regulatory_analysis.hwp \
  samples/issue2004_cell_image_stack.hwp \
  -o /private/tmp/issue2308-hw-ovr \
  --diff-against upstream/devel
```

| 샘플 | 페이지 | 개체 | 회귀 |
| --- | ---: | ---: | ---: |
| `76076_regulatory_analysis.hwp` | 82→82 | 9→9 | 0 |
| `issue2004_cell_image_stack.hwp` | 8→8 | 0→0 | 0 |

- 비교: 현재 `6438a4cfb` vs `upstream/devel@cbddc1cd8`
- 허용 오차: ±2px
- 합계: 회귀 0건
- 결과: `/private/tmp/issue2308-hw-ovr/ovr_diff.md`

OVR은 한컴 없이 실행하는 geometry 보조 근거다. 한컴 before/after/OVL 사람 판정은 전체 검증
승인 게이트에 남긴다.

## 문서화

- 수행·구현 계획의 Stage 1~6 경계를 유지했다.
- Stage별 완료보고서를 `task_m100_2308_stage1.md`부터 `stage6.md`까지 분리했다.
- `mydocs/tech/rendering_engine_design.md`에 source IR, revision cache, sparse overlay,
  invalidation/fallback 계약을 반영했다.
- 최종 보고서에 Stage별 커밋과 focused/OVR 결과를 기록했다.

## 남은 승인 게이트

1. 전체 release test와 clippy
2. WASM build
3. Studio unit/E2E
4. 한컴 기준 before/after/OVL 최종 시각 판정
5. 원격 push, draft PR, 이슈 결과 코멘트
