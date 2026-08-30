---
kind: pr-review
status: pending-user-validation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6467
issue: 6041
author: postmelee
---

# PR #6467 review - 실제 Canvas surface 예산 기반 적응형 render scale

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `pr_review/visual_fixture_evidence.md`,
  `pr_review/review_only_fast_pass.md`, `codex/docs_and_git_workflow.md`
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.

## metadata와 범위

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#6467](https://github.com/edwardkim/rhwp/pull/6467) |
| 관련 issue | [#6041](https://github.com/edwardkim/rhwp/issues/6041) |
| base / head | `codex/issue-6040-zoom-topology` / `codex/issue-6041-budget-first-render-scale` |
| stack base | `dfe27e18884cd067b0f4ccd0ed9141e20640fac5` |
| 최초 candidate | `1aa5a20419909baed4d1c37bcf28bbfdade98aa5` |
| 보정 code candidate | `e37d483fd` |
| 원격 상태 | Draft; 보정 candidate push·최신 CI·사용자 시각 검증 대기 |

이 PR은 GitHub native stack의 middle PR이다. #6040 source branch만을 base로 하며, 승인 전에는 #6042
상단 branch를 만들지 않는다. #6041의 화면 render scale 정책만 포함하고 #6040 배치와 #6042
가상화/LRU/scheduler는 변경하지 않는다.

## self-review

**보정 code candidate는 조건부 수용 권고한다.** 전역 예산과 비포커스 우선 강등은 실제 surface 비용이
과도한 경우에만 작동하고, 일반 문서·편집 페이지·출력 profile은 raw DPR을 유지한다.

최초 candidate의 `DEFAULT_CANVAS2D_LAYER_COUNT=4`를 모든 페이지 비용에 직접 적용한 부분은 blocker였다.
`kps-ai.hwp` 실제 DOM은 retained 페이지당 main Canvas 한 장뿐인데 4배 비용으로 계산해 화면 밖 페이지를
불필요하게 DPR 1.5로 낮췄다. 보정 candidate는 overlay/tree plane을 페이지별로 읽어 1~4 surface를
계산한다. 34% 4쪽과 `kps-ai.hwp`는 다시 모두 DPR 2이고, 실제 background/behind/front가 있는 KTX는
4-layer로 식별된다.

계획 파일이 source 전에 만들어지지 않은 절차 누락은 수행·구현 계획과 Stage 보고에 소급 없이 밝혔다.
이 누락 때문에 실제 코드 판정을 완화하지 않으며, 사용자 결과 승인 전에는 최종 보고와 다음 stack
layer를 진행하지 않는다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| focused planner | 13 pass |
| TypeScript | `npx tsc --noEmit` pass |
| Studio 전체 | 1,290 pass, 1 skip, 0 fail |
| production build | 244 modules pass, 기존 대형 chunk 경고만 확인 |
| diff | `git diff --check` pass |
| 실제 Canvas2D | 3개 문서×34/50/100% 수정 전·후 PNG 9개, SSIM 0.999885~0.999979, raw DPR 보존 |
| CanvasKit | layerCount 1·raw DPR 보존, warning/error 없음 |

Rust source/test/fixture와 PDF/SVG 출력은 바꾸지 않으므로 Rust lint 묶음과 출력 visual sweep은 범위에서
제외했다. 이 PR의 사용자-visible 대상은 Studio Canvas 해상도이므로 실제 브라우저의 physical/CSS 크기,
DPR, tier, layer count와 before/after 화면을 직접 판정했다.

비교 asset은 4쪽 실문서, `kps-ai.hwp`, 실제 4-layer `KTX.hwp`를 각각 34%·50%·100%에서 같은
1280×720 viewport로 캡처했다. 각 합성 PNG는 왼쪽 수정 전(#6040), 오른쪽 수정 후(#6041)이며 원본
PNG를 리사이즈·손실 압축 없이 붙였다. 사람이 9개를 모두 열어 문서 정렬, 텍스트/표/그림 선명도와
상태줄 배율을 확인했으며 제품 품질 차이는 없고 상태줄 렌더 시간만 달랐다.

## 위험과 후속 조건

- static flow가 DOM DIV로 실리는 페이지는 Canvas 상한을 한 장 보수적으로 더 잡을 수 있다. 모든 페이지를
  4장으로 잡던 초기 candidate와 달리 콘텐츠 기반이며, 예산을 넘지 않으면 해상도에 영향이 없다.
- 예산을 넘겨도 포커스 페이지는 품질을 낮추지 않아 `withinBudget=false`일 수 있다. 이는 편집 품질을
  메모리 수치보다 우선한 명시적 계약이다.
- 시간 표본 변동이 커 실제 속도 향상은 주장하지 않는다. 실측 샘플이 예산 이내면 최적화하지 않는 것이
  이번 정책의 올바른 결과다.
- 사용자 시각 검증, 보정 candidate의 PR CI와 latest head required aggregate를 확인하기 전에는 Ready/merge
  또는 #6042 시작을 권고하지 않는다.

## 최종 권고

**Draft middle PR로 조건부 수용, 사용자 검증 대기.** 페이지별 비용 보정 뒤 과도한 저해상도 blocker는
해소됐다. 작업지시자가 로컬 문서 줌 품질을 승인하고 원격 CI가 성공하면 #6041 결과를 고정하고 #6042를
현재 head에서 시작할 수 있다.
