# PR #9201 검토 — 도형 캡션 편집

## 최종 판정

**보호 미달 두 쪽 보정 완료 — 대상 문서 Native·fresh WASM 160쪽이 페이지별 보호값을 충족했다.**

[메인터너 보정 원인](../assets/integration_open_prs_20261120/maintainer-cause.md) · [전쪽 결과](../assets/integration_open_prs_20261120/maintainer-repair/verification.json). 최종 통합 승인에는 새 코드의 전체 검증이 필요하다.
도형 캡션의 편집·재열기와 통합 보정은 검증했다. 원 PR에서 제시한 캡션 편집 결함의
해결 근거와 공통 제출 조건을 아래에 기록했다. [통합 검증](../assets/integration_open_prs_20261120/README.md).

## 접수와 적용

| 항목 | 확인 |
| --- | --- |
| 원 PR·작성자·리뷰어 | [#9201](https://github.com/edwardkim/rhwp/pull/9201) · `semanticist21` · `jangster77` |
| 원 head | `77cd2377e562349ff4b0d4fbbf7f9ecf64b90a47`, 열린 PR, MERGEABLE, 6파일 |
| 적용 순서 | 첫 번째. `80c7eff7f4e9a46331aa5908d1b16e2e09f6a417` |
| 고정 기준·제품 후보 | `0a437a7eefb19bfe81fc12866887e4548ea9554d` · `25f9587840d09b08af2b37be178ab93379a7b9dc` |

## 원 기여와 메인터너 보정

글상자가 없는 사각형의 캡션을 셀 주소 `65534`로 읽고 편집한다. 실제 캐럿과 히트 테스트가
같은 주소를 돌려준다. 원 PR의 `shape_caption_ref` 공개 범위 변경은 기준의
`9ffd6d78bc745d061daf759c13460fae9176b83d`에 이미 있었다.

#9207 적용 뒤 도형 캡션이 공용 셀 너비 구조체를 사용하도록 마지막 통합 커밋
`25f9587840d09b08af2b37be178ab93379a7b9dc`에서 연결했다. 기존 방향별 캡션 너비와
여백을 전달하고, 셀 전용 한 줄 압축은 캡션에서 요청하지 않는다. 캡션 주소·소유와 저장
복원은 관계형 회귀로 확인했다. [회귀 검사](../../../tests/cases/shape_caption_cell_text.rs).

## 검증과 시각 증적

- 고정 기준의 fresh WASM은 캡션 `65534` 문단 조회에서 실패했다. 후보에서는 캡션 조회와
  삽입, 문단 나누기·합치기, 캐럿과 클릭 주소를 확인했다. HWP와 HWPX를 각각 두 번 저장·다시 열어
  주소 `65534`와 문구 `그림  검토 시작`이 보존됐다. [실제 편집 결과와 실행 해시](../assets/pr-9201/evidence.json).
- 비교 대상은 통합 후보가 생성한 사각형과 캡션의 저장 HWP다. 기존 Cairo PDF를 교체하고
  동일 HWP를 MCP의 한컴 2020(11.0.0.9136)에서 Hancom PDF 1-up으로 인쇄했다.
  MCP는 Print method 0을 기록했고, PDF 메타데이터는 `Hwp 2020 0.0.0.0`과
  `Hancom PDF 1.3.0.550`이다. 96 DPI 기준 1쪽의 raw 2px 실루엣은 고정 devel과
  최종 후보에서 Native와 최적화한 fresh WASM 모두 100.00000%다.
  [인쇄 기록과 해시](../assets/pr-9201/evidence.json).
- 도형 테두리, 그림 번호, 마지막 글자의 줄 소속을 직접 대조했다. 테두리 모서리의 안티앨리어싱에
  미세한 차이가 있다. [같은 배율 확대](../assets/pr-9201/caption-p001-enlarged.png).
- [원본](../../../tests/fixtures/shape_caption_cell_text/shape-caption-edit.hwp) ·
  [독립 Print](../../../pdf/pr-9201/shape-caption-edit-hancom-2020.pdf) ·
  [Native TSV](../assets/pr-9201/tables/native/shape-caption.tsv) · [WASM TSV](../assets/pr-9201/tables/wasm/shape-caption.tsv).

| 확인 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 합성 사각형의 캡션 1쪽 | [review](../assets/pr-9201/shape-caption-native-001-review.png) · [overlay](../assets/pr-9201/shape-caption-native-001-overlay.png) | [review](../assets/pr-9201/shape-caption-wasm-001-review.png) · [overlay](../assets/pr-9201/shape-caption-wasm-001-overlay.png) |

최종 최적화 WASM의 Node 소비 경로에서 HWP·HWPX 각각 두 번 저장하고 다시 읽었다.
네 저장 모두 캡션 문구와 편집 주소 `65534`를 보존했다.
[최종 제품 SHA와 실행 결과](../assets/integration_open_prs_20261120/final-product-consumers.json).

## Merge 후 contributor PR comment 계획

캡션 주소를 통일한 기여에 감사하고, 한 줄 셀 PR과 통합할 때 캡션 너비 호출을 맞춘
이유와 반복 저장 결과를 설명한다. 통합 CI·병합 SHA·devel 반영을 확인한 뒤 위 1쪽의
Native·fresh WASM review와 overlay를 병합 SHA에 고정한 raw 이미지로 표시한다.
이미지 응답·해시와 게시 후 본문을 확인한다. [Visual Sweep 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
