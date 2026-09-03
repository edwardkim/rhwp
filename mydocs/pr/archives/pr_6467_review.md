---
kind: pr-review
status: approved-pending-base-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-03
pr: 6467
issue: 6041
author: postmelee
---

# PR #6467 review - 실제 Canvas surface 예산 기반 적응형 render scale

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `pr_review/visual_fixture_evidence.md`,
  `pr_review/review_only_fast_pass.md`, `pr_review/rework_and_exceptions.md`,
  `codex/docs_and_git_workflow.md`, `hyper_waterfall_docs_guide.md`
- 작성자·self-review: `postmelee`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.

## metadata와 범위

| 항목 | 2026-09-03 Ready 재자격화 |
| --- | --- |
| PR | [#6467](https://github.com/edwardkim/rhwp/pull/6467) |
| 관련 issue | [#6041](https://github.com/edwardkim/rhwp/issues/6041) |
| base / head | `codex/issue-6040-zoom-topology` / `codex/issue-6041-budget-first-render-scale` |
| 현재 stack base | `11737990bc8ae7d1bb78e20c0c7b3ac958c45043` |
| 최초 stack base | `dfe27e18884cd067b0f4ccd0ed9141e20640fac5` |
| 2026-09-02 restack base | `c2932ff30fbc45e3d89eefad7c75a71518acde33` |
| 최초 candidate | `1aa5a20419909baed4d1c37bcf28bbfdade98aa5` |
| 보정 code candidate | `e37d483fd` |
| 2026-08-31 정리 전 head | `5fc2542005ca271c9ac3452ce11416e7a0855ba7` |
| 2026-09-03 restack code candidate | `06d045d031dc9abc9d9cfefba037d481094673c2` |
| 원격 상태 | Draft, #6458 merge와 직접 `devel` base 전환 대기 |

이 PR은 계획된 GitHub native stack의 2/3 PR이다. #6040 source branch만을 base로 하며 후속 순서는
#6042 가상화/LRU/scheduler PR이다. 본 PR은 surface 예산 기반 정책만 포함한다. #6521에서 실험한
예산 이내 저배율 overview ceiling은 화질 저하 대비 이득 부족으로 비채택했으며 이 PR에 섞지 않는다.
`Closes #6041`은 사용하지 않고 `Refs #6041`을 유지한다.

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
| 예산 발동 실측 | issue6280 21쪽 문서 200% physical Canvas 57.04M→35.65Mpx, 37.49% 감소 |
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
- issue6280 200%에서 visible DPR 2를 보존하고 offscreen DPR만 1로 낮춰 steady-state physical Canvas
  pixel과 raw RGBA backing-store 등가값이 37.49% 감소했다.
- 100→200% 완료 시간 중앙값은 기준 1352ms, 변경본 1360ms로 사실상 동일하다. wall-clock 속도 향상은
  주장하지 않으며, 실측 샘플이 예산 이내면 최적화하지 않는 것이 이번 정책의 올바른 결과다.
- 위 시간에는 UI 자동화와 약 500ms 안정화 관측이 포함된다. 직접 UX latency로 재사용하지 않는다.
  #6521에서 네 완료 경계를 실험했지만 화질 정책은 비채택했고, 필요한 관찰 계약만 #6042에서 재검토한다.
- restack head의 PR CI와 latest head required aggregate를 확인하기 전에는 Ready/merge를 권고하지 않는다.

## 2026-09-03 restack 재자격화

- #6454의 geometry 1·2·3·6·50·500쪽과 실문서 3개의 smooth/direct/resize 27조합에서 공유
  zoom-frame snapshot이 진입 gate를 통과하지 못해 제품 변경 없이 `NOT_PLANNED`로 종료됐다.
- #6467 고유 7개 커밋을 최신 `devel@eb2ea3add` 위에서 재자격화된 #6458 head `11737990b` 위로
  재배치했다. 이번 재적층에는 제품·test 충돌이 없었다.
- 새 #6458 zoom-path 테스트의 prototype fixture에 #6467의 `renderSurfacePlan`과
  `renderSurfaceDecisions` 초기 상태를 추가했다. 제품 class의 optional guard로 테스트를 우회하지 않았다.
- TypeScript, Studio 1,386건(1,385 pass·1 policy skip), production build 248 modules,
  E2E manifest 126/126과 `git diff --check`를 통과했다.
- 실제 1280×720 Canvas2D·20쪽 실문서의 자동 34%에서 3열, 편집 영역 중심 오차 0.17px,
  visible 6쪽·retained 3쪽의 raw DPR 2 유지, 연결된 Canvas와 콘솔 warning/error 0을 확인했다.

## 최종 권고

**승인, #6458 merge 대기.** 제품 정책은 기존 보정 candidate와 같고 최신 `devel` 재적층 뒤 전체
Studio·production build를 다시 통과했다. #6458이 승인·merge되어 이 PR의 직접 base가 `devel`로
전환되면 exact head required checks를 확인하고 Ready로 전환한다. merge는 그 뒤에도 사용자 승인 전까지
수행하지 않는다. 실행 기록은
[정리 계획](pr_6467_review_impl.md), [Stage 4](../../working/task_m100_6041_stage4.md),
[Stage 5](../../working/task_m100_6041_stage5.md)를 따른다.
