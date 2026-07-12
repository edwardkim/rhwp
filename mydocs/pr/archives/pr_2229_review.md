# PR #2229 리뷰 - Canvas2D 정적 그림 표 셀 clip 보존

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | [#2229](https://github.com/edwardkim/rhwp/pull/2229) |
| 제목 | `task 2228: 표 셀 clip을 정적 그림 레이어에 보존` |
| 작성자 | `jangster77` |
| base | `devel` |
| 관련 이슈 | [#2228](https://github.com/edwardkim/rhwp/issues/2228) |
| 코드 커밋 | `2929cb6f1` (`task 2228: 표 셀 clip을 정적 그림 레이어에 보존`) |
| 규모 | 문서 작성 시점 코드 변경: 6 files, +345/-80 |
| 작성 시점 상태 | Open. GitHub Actions와 merge 가능 상태는 문서 후속 커밋을 포함한 최신 head 기준으로 다시 확인한다. |

## 변경 범위

- `rhwp-studio/src/view/flow-image-clip.ts`
  - PageLayerTree를 순회하며 상위 `clipRect`의 누적 교집합을 flow image마다 보존한다.
  - 완전히 clip 바깥인 그림은 정적 DOM 그림 레이어에서 제외한다.
- `rhwp-studio/src/view/page-renderer.ts`
  - 일부만 보이는 그림에 clip wrapper를 두고 bbox, crop, 회전, 좌우 반전을 기존 DOM 그림 경로와 같은 좌표계에서 적용한다.
- `rhwp-studio/tests/flow-image-clip.test.ts`
  - 중첩 표 셀 clip 계보와 clip 바깥 그림 제외를 회귀 테스트로 추가한다.
- 계획서, 단계 기록, 결과 보고서를 함께 보존한다.

## 원인과 검토 결과

[#2228](https://github.com/edwardkim/rhwp/issues/2228)의 중복 붉은 `창립교회` 그림은 폰트 문제가 아니다.
Canvas2D 정적 flow 그림을 DOM layer로 분리할 때 `PageLayerTree`의 `clipRect(tableCell)` 문맥을 버린 것이
원인이다. 같은 binary 그림을 참조하는 TopAndBottom control 두 개가 표 셀 밖에서도 독립 DOM frame으로
생성됐다.

이번 수정은 문서별 좌표나 그림 ID를 하드코딩하지 않고 트리의 clip 교집합을 그대로 전달한다. 따라서 표 셀뿐
아니라 clipRect 아래에 배치되는 다른 flow 그림에도 같은 규칙을 적용한다. blocking finding은 없다.

## 기준 자료와 시각 확인

- 원본: `samples/issue2217/20200830.hwp`
  - SHA-256: `8e7a95cf591944bff56050879fa90251921ec57e28eac66d40c6fb8ad103016f`
- 기준 PDF: `pdf/issue2217/20200830-2020.pdf` (HWP 2020, 4쪽)
  - SHA-256: `ea0110e5c6325f7d2b22620017791b8ab5e53768f84dfa2c0656390d931c6563`
- 기준 PDF 2쪽 raster: `mydocs/pr/assets/pr_2229/pr_2229_hwp2020_20200830_p2.png`
- 수정 head Canvas2D 2쪽: `mydocs/pr/assets/pr_2229/pr_2229_canvas2d_20200830_p2.png`

기준 PDF와 Canvas2D 화면 모두 우측 하단에 붉은 `창립교회` 그림이 한 번만 표시된다. 이번 검토는 전체
폰트 fidelity를 판정하는 sweep이 아니라, 표 셀 밖에 동일 그림이 반복 생성되는 사용자-visible 회귀의 직접
수용 기준을 확인한 것이다. Headless Chrome Canvas2D에서 2쪽 flow image layer는 정상 그림 3개만 생성했고,
셀 밖 TopAndBottom control 두 개는 생성되지 않았다. 콘솔 error/warning은 없었다.

## 검증 결과

- `cd rhwp-studio && npm test`: 206 passed, 0 failed
- `cd rhwp-studio && npx tsc --noEmit`: 통과
- `cd rhwp-studio && npm run build`: 통과
- `cd rhwp-chrome && npm run build`: 통과, 로컬 extension bundle만 생성하고 배포하지 않았다.
- `git diff --check upstream/devel...HEAD`: 통과

작업지시 범위에 따라 Cargo 전체 테스트와 clippy는 이번 PR 전 로컬 검증에서 제외했다. 이 PR은
TypeScript 프론트엔드 렌더 경로만 변경한다.

## 최종 권고

[#2228](https://github.com/edwardkim/rhwp/issues/2228)의 수용 기준인 표 셀 clip 보존과 중복 그림 제거는
코드, 회귀 테스트, 실제 Canvas2D 화면에서 확인됐다. 문서 후속 커밋을 포함한 최신 PR head의 GitHub Actions가
통과하고 작업지시자 승인되면 merge를 권고한다.
