---
kind: reference
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-30
---

# #6041 render surface 예산 검증

## 범위와 기준

- 기준: #6040 Stage 1 head `dfe27e18884cd067b0f4ccd0ed9141e20640fac5`
- 비교 대상: `codex/issue-6041-budget-first-render-scale`
- backend: Canvas2D
- browser viewport: 1280×720 CSS px, 문서 viewport 1260×558 CSS px
- raw DPR: 2
- Canvas2D surface 추정: page당 최대 4개 Canvas layer를 보수적으로 계산

화면 렌더 해상도만 바꾸는 Studio 작업이므로 PDF/SVG 출력 비교가 아니라 실제 Canvas의 CSS/물리
크기, tier, surface 비용과 같은 화면 캡처를 비교했다. print/PDF/SVG/highQuality 출력 경로는 정책에서
제외된다. 최종 사용자-visible 판정은 작업지시자의 로컬 조작 확인을 남겨 둔다.

## 34% 4쪽 실문서 품질 보존

fixture: `samples/21868765_별표2_보건소_분장사무.hwp`

- SHA-256: `ae694583e739ac48af97cb12ce573c2da9f4cb637721fdf84e5af4bf7ca17c13`
- 자동 4열, retained/visible 4쪽
- 기준과 변경본 모두 네 쪽 `screen`, effective DPR 2
- 쪽당 Canvas 540×764 physical px / 270×382 CSS px
- main Canvas 합계 1,650,240 physical px, 보수적 4-layer surface 25.18MiB
- JPEG 화면 캡처 SSIM: 0.999901. 상태줄 시간처럼 정책과 무관한 미세 차이만 있다.

![#6041 Stage 1 baseline at 34%](assets/issue6041_exact4_baseline_34.jpg)

![#6041 budget-first result at 34%](assets/issue6041_exact4_budget_first_34.jpg)

asset SHA-256:

- baseline: `9969d7582154c6a4768c98b5213ee305c49002d031ef44c75321463727090bd3`
- budget-first: `aae1878c0674ce13e4153dd3fda782dfe13a5b10161c7ba9739ab0a838c84d2e`

## 100% 실제 다중 쪽 문서 surface 절감

fixture: `samples/kps-ai.hwp`

- SHA-256: `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`
- 재현 순서: 25 → 34 → 36 → 50 → 60 → 100%
- 최종 retained 3쪽, visible 2쪽

| 지표 | Stage 1 기준 | budget-first | 변화 |
| --- | ---: | ---: | ---: |
| visible tier/DPR | 2쪽 모두 DPR 2 | 2쪽 모두 DPR 2 | 품질 보존 |
| offscreen prefetch | DPR 2 | DPR 1.5 | 필요한 한 단계만 하향 |
| retained physical px | 10,699,944 | 9,138,940 | -14.59% |
| retained main RGBA | 40.82MiB | 34.86MiB | -14.59% |
| 보수적 4-layer surface | 163.27MiB | 139.45MiB | -14.59% |
| visible physical px | 7,133,296 | 7,133,296 | 동일 |

## 시간 측정의 한계

60→100% 전환 완료를 같은 탭에서 각각 5회 측정한 중앙값은 기준 126ms, 변경본 132ms였다.
표본은 `[141,126,134,116,94]`와 `[122,132,142,322,100]`으로 변동이 크고 변경본에 outlier도 있어,
이 결과로 속도 향상을 주장하지 않는다. 이 PR의 정량 근거는 동일 visible 품질에서 줄어든 물리 픽셀과
추정 surface 메모리다.

## 자동 검증

- `node --test tests/render-surface-budget.test.ts`: 12 passed
- `npx tsc --noEmit`: passed
- `npm test`: 1,289 passed, 1 skipped, 0 failed
- `npm run build`: passed, 기존 대형 chunk 경고만 발생
- `git diff --check`: passed
