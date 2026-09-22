# PR #7334 리뷰 — 개체만 든 칸의 빈 글줄 높이와 혼합 부동 그림

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7334](https://github.com/edwardkim/rhwp/pull/7334) |
| 작성자 | `planet6897` 외부 contributor |
| 관련 이슈 | [#6761](https://github.com/edwardkim/rhwp/issues/6761), [#6782](https://github.com/edwardkim/rhwp/issues/6782) |
| 원 contributor head | `47b31d7411e0b8a8acb823c1382c1441bab49751` |
| 메인터너 보정 head | `1924a2a0d35bc1329bfb764a6c6955b1a5cb8b4d` |
| 원 code candidate CI | [CI #35683596884](https://github.com/edwardkim/rhwp/actions/runs/35683596884) 성공 |
| 보정 head CI | 이 trailing 기록 push 뒤 새로 실행되어야 함 |
| reviewer | `jangster77` |

원 contributor 변경은 개체만 든 표 셀에서 빈 문단의 기본 글줄 높이를 중복으로 더하지 않도록 하여,
#6761 문서의 불필요한 추가 쪽을 줄인다. 원 CI와 focused 검사는 통과했으나, 사용자가 Visual Sweep에서
표 4-1 일본 PS 마크의 하단이 셀 밖으로 나간 것을 발견했다. 따라서 원 CI 성공만으로 수용하지 않고,
보이는 배치 오류를 재현·보정했다.

## 입력과 기준 PDF

- 입력 `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp`는 이미 추적된
  원본이며 SHA-256은 `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860`이다.
  `rhwp info --json`의 마지막 저장 제품은 `hancom-office-2010` 8.5.8.1677이므로 한컴 변환 engine은
  2020을 사용한다.
- 기준 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf`도 이미 추적된 파일이며
  SHA-256은 `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79`이다. Creator는
  Hwp 2022이고 물리 103쪽이다.
- 대상 표는 PDF 물리 77쪽의 인쇄 쪽번호 55이며, rhwp는 #6761의 남은 전역 쪽수 차이 때문에 물리
  78쪽에 있다. Visual Sweep에는 픽셀을 바꾸지 않고 앞 77쪽을 빈 페이지로 채운 임시 매핑 PDF를
  사용해 PDF 77쪽을 rhwp 78쪽과 대조했다. 이 매핑 PDF는 증적 입력이 아니라 실행 중 생성한
  페이지 번호 정렬용 임시 파일이며 커밋하지 않았다.

## 발견한 보류 사유와 메인터너 보정

일본 행의 두 PS 그림은 저장 `cell.height`가 실제 행 높이보다 작고, 같은 빈 문단에
`InFrontOfText` 및 `TopAndBottom` 그림이 함께 있다. 저장 `LINE_SEG.vpos`를 각 그림의 문단 기준점으로
그대로 적용하면 두 번째 마크가 자기 셀 아래로 밀렸다. 수정 전에는 그림 경계가 셀
`304.1..384.3px`에 대해 `343.1..393.8px`까지 내려갔다.

`1924a2a0d`은 다음 저장 형상에만 실제 셀 content bottom으로 그림을 제한한다.

- 빈 문단, 하단 세로 정렬, 양의 저장 `vpos`를 가진 한 `LINE_SEG`;
- 저장 셀 높이가 실제 행 내부 높이보다 작음;
- non-TAC 그림만 있고 `InFrontOfText`와 `TopAndBottom`이 각각 하나 이상 공존함.

일반 부동 그림의 셀 밖 배치 의미를 바꾸지 않도록 위 형상 외에는 기존 좌표 계산을 유지했다.
또한 `japan_mixed_wrap_marks_stay_inside_their_cell` 회귀 검사를 추가해 두 마크 모두의 위·아래
경계가 자기 셀 안에 있어야 한다고 고정했다. 이 검사는 보정 전 head에서 실패하고 보정 후 통과했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| #6782 신규 일본 PS 셀 경계 회귀 | 수정 전 실패, 보정 후 통과 |
| #6761·#6782·#6194 focused nextest 10개 | 통과 |
| Native Visual Sweep, rhwp 78쪽 / PDF 물리 77쪽 | 구조 flag 0, 두 PS 마크 모두 일본 셀 안, overlay 생성 |
| fresh WASM Visual Sweep, 같은 페이지 | 구조 flag 0, 두 PS 마크 모두 일본 셀 안, overlay 생성 |
| 원 contributor GitHub Full CI | archive A-D 및 shard, lint, Native Skia, WASM, frontend 등 성공 |
| 보정·trailing head GitHub CI | push 뒤 확인 필요 |

Native와 fresh WASM의 page 78 결과가 동일했다. 전역 pixel match 90.066%, ink match 29.273%는
문서 전체의 기존 글꼴 래스터·쪽수 잔차를 포함한 보조 지표이므로 수용 판정에 사용하지 않았다.
수용 근거는 원본 PDF의 일본 행 내 두 PS 마크, 행 테두리, 중국·캐나다 인접 행 및 뒤따르는 표 행이
셀 경계를 넘지 않는 것을 Native/WASM 결과와 overlay로 직접 확인한 것이다.

## Visual Sweep 증적

| backend | review | overlay |
| --- | --- | --- |
| Native | [78쪽 review](../assets/pr7334_review/native_review_078.png) | [78쪽 overlay](../assets/pr7334_review/native_overlay_078.png) |
| fresh WASM | [78쪽 review](../assets/pr7334_review/wasm_review_078.png) | [78쪽 overlay](../assets/pr7334_review/wasm_overlay_078.png) |

## 최종 판정

**메인터너 보정 후 CI 대기.** 사용자가 지적한 셀 밖 PS 마크는 재현 회귀 검사와 Native·fresh WASM
Visual Sweep으로 해결했다. 원 contributor CI는 원 head만 증명하므로, `1924a2a0d` 및 이 trailing
증적 head의 새 CI가 완료된 뒤 `MERGEABLE`/`CLEAN`과 최종 head를 다시 확인해야 한다. #6761은
전체 문서가 rhwp 104쪽, oracle 103쪽인 잔여 조건이 있으므로 닫지 않는다.

## Merge 후 contributor PR comment 계획

실제 merge SHA와 최신 head CI가 확정된 뒤 원 PR에 한국어로 다음을 게시한다.

- 실제 merge SHA, 원 code candidate [CI #35683596884](https://github.com/edwardkim/rhwp/actions/runs/35683596884),
  보정 head CI를 구분해 명시한다.
- 저장 셀 높이·혼합 wrap·하단 정렬 형상, focused nextest 10건, Native/fresh WASM Visual Sweep의
  물리 PDF 77쪽 ↔ rhwp 78쪽 매핑과 확인 범위를 설명한다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
  연결하고 merge commit에 존재하는 다음 4장을 모두 보이게 넣는다.

  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7334_review/native_review_078.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7334_review/native_overlay_078.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7334_review/wasm_review_078.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7334_review/wasm_overlay_078.png`

#6761은 닫지 않고, 이번에 해결한 일본 PS 셀 밖 배치와 남은 전체 쪽수 차이를 같은 근거로 issue comment에
남긴다.
