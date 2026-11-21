# PR #9207 검토 — 한 줄 셀의 저장과 편집

## 최종 판정

**보호 미달 두 쪽 보정 완료 — 대상 문서 Native·fresh WASM 160쪽이 페이지별 보호값을 충족했다.**

[메인터너 보정 원인](../assets/integration_open_prs_20261120/maintainer-cause.md) · [전쪽 결과](../assets/integration_open_prs_20261120/maintainer-repair/verification.json). 최종 통합 승인에는 새 코드의 전체 검증이 필요하다.
한 줄 설정의 저장·편집과 정상 저장 줄 보존은 통합 후보에서 확인했다.
[통합 검증](../assets/integration_open_prs_20261120/README.md).

## 접수와 적용

| 항목 | 확인 |
| --- | --- |
| 원 PR·작성자·리뷰어 | [#9207](https://github.com/edwardkim/rhwp/pull/9207) · `JinHo-von-Choi` · `jangster77` |
| 원 head·관련 이슈 | `1c85b170b112dd2ff53e026bc76c73eac904dc3e`, 열린 PR, MERGEABLE, 65파일 · [#7940](https://github.com/edwardkim/rhwp/issues/7940) |
| 적용 순서 | 네 번째. `bdb74fa17e3a49c034cb47763ebc7af98b289de6`부터 `10f1c07f8d157e23f8013f54aa916db07145f562`까지 7개 |
| 고정 기준·제품 후보 | `0a437a7eefb19bfe81fc12866887e4548ea9554d` · `25f9587840d09b08af2b37be178ab93379a7b9dc` |

## 원 기여와 메인터너 보정

선택한 셀이 한 줄로 바뀌면 기존 줄 분할을 무효화하고, 편집한 긴 글자와 캐럿이
셀의 실제 내용 폭 안에 들어가도록 한다. 저장된 한 줄과 일반 줄바꿈 셀이 각자의 속성을
보존하며 이웃 셀에는 편집이 전파되지 않는다. 중첩 표 안쪽 셀을 지정할 수 없는 대화상자
경로에서 해당 항목이 비활성화되는 조건을 유지했다.

공용 셀 너비 구조의 변경을 #9201의 도형 캡션과 함께 적용했다. 보정 커밋
`25f9587840d09b08af2b37be178ab93379a7b9dc`는 캡션의 방향별 너비를 전달하며
한 줄 압축 적용은 셀의 선언이 소유한다. [도형 캡션 병행 검증](pr_9201_review.md) ·
[한 줄 셀과 일반 줄 대조 검사](../../../tests/cases/issue_8947_squeeze_cell_stored_line.rs).

## 검증과 시각 증적

- 실제 Studio에서 긴 입력 앞뒤의 캐럿이 같은 줄에 남고 실행 취소·다시 실행을 통과했다.
  HWP와 HWPX 각각 두 번 저장·재열기한 뒤 대상/이웃 값 `SQUEEZE/BREAK`를 독립 XML로
  확인했다. 고정 기준의 같은 편집 재현은 `BREAK/BREAK`였다.
  [재열기 UI](../assets/pr-9207/studio-production-cell.png) · [저장과 페이지 증적](../assets/pr-9207/evidence.json).
- 결재 문서 2쪽과 부대시설 사용료 감면기준 1쪽을 Windows 데스크톱 한컴 2024
  (13.0.0.3901)의 Hancom PDF Print로 재인쇄해 Native·fresh WASM 각 3쪽을 대조했다.
  결재 문서의 Native raw는 1쪽 98.27%, 2쪽 97.79%이고 fresh WASM은 98.31%, 97.35%다.
  1쪽의 제목은 같은 셀의 한 줄에 있다. 부대시설 사용료 감면기준은 두 출력 모두 99.97%이며,
  `SQUEEZE` 15문단이 같은 셀·한 줄·원래 순서를 보존했다. 웹한글기안기와 Cairo PDF는
  공식 비교 기준에서 제외했다.
  [결재 문서 Print](../../../pdf/issue8947/36481164-squeeze-title-cell-2024.pdf) ·
  [감면기준 Print](../../../pdf/issue9085/facility_fee_reduction_criteria-2024.pdf) ·
  [전쪽 TSV·인쇄 영수증·원본 PNG](../assets/pr-9207/hancom-2024/evidence.json).
- 화학제품 103쪽, 한컴 사양서 71쪽과 시행령 74쪽의 고정 기준 출력도 별도로 비교했다.
  시행령의 기존 보호 TSV와 한컴 2024 재인쇄 대조는 비교 조건이 다르다.
  [고정 기준과 대표 증적](../assets/pr-9207/evidence.json).

| 페이지 | Native | fresh WASM |
| --- | --- | --- |
| 결재 문서 제목 1쪽 | [review](../assets/pr-9207/hancom-2024/squeeze-native-001-review.png) · [overlay](../assets/pr-9207/hancom-2024/squeeze-native-001-overlay.png) | [review](../assets/pr-9207/hancom-2024/squeeze-wasm-001-review.png) · [overlay](../assets/pr-9207/hancom-2024/squeeze-wasm-001-overlay.png) |
| 결재 문서 2쪽 | [review](../assets/pr-9207/hancom-2024/squeeze-native-002-review.png) · [overlay](../assets/pr-9207/hancom-2024/squeeze-native-002-overlay.png) | [review](../assets/pr-9207/hancom-2024/squeeze-wasm-002-review.png) · [overlay](../assets/pr-9207/hancom-2024/squeeze-wasm-002-overlay.png) |
| 감면기준 1쪽 | [review](../assets/pr-9207/hancom-2024/para-squeeze-native-001-review.png) · [overlay](../assets/pr-9207/hancom-2024/para-squeeze-native-001-overlay.png) | [review](../assets/pr-9207/hancom-2024/para-squeeze-wasm-001-review.png) · [overlay](../assets/pr-9207/hancom-2024/para-squeeze-wasm-001-overlay.png) |

최종 Studio의 실제 브라우저에서 HWP·HWPX 각각 두 번 저장하고 다시 열었다. 네 결과 모두
대상 셀과 이웃 셀의 한 줄 속성 차이와 두 셀의 텍스트를 유지했고, 캐럿은 대상의 같은 줄에 있었다.
[최종 제품 SHA와 실행 결과](../assets/integration_open_prs_20261120/final-product-consumers.json).

결재문서 1쪽은 실제 Studio의 Canvas2D·CanvasKit compat/default에서 화면·인쇄 프로필로 확인했다. 대상 쪽의 글꼴 폴백은 0건이었다.
[실제 Studio 캡처와 검증](../assets/integration_open_prs_20261120/actual-studio-renderers/README.md).

## Merge 후 contributor PR comment 계획

대상 셀의 한 줄 설정과 이웃 셀의 줄바꿈을 함께 보존한 기여에 감사한다. 메인터너 보정이
필요했던 공용 너비 호출과 두 형식의 재열기 결과를 설명한다. 통합 CI·병합 SHA·devel
반영 확인 뒤, 원 PR의 각 주장과 같은 문서·쪽의 review·overlay를 병합 SHA의 이미지로
표시한다. 편람 130쪽은 Native 증거를 사용한다. 이미지 응답·해시와 게시 본문, #7940
상태를 확인한다. [Visual Sweep 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
