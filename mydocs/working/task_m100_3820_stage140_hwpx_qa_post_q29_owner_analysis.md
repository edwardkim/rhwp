# Stage 140: HWPX Q35·Q37·Q41·Q48 page owner 보정

## 목적

Stage 139에서 Q29 p295 완결과 Q30 p296 시작을 맞춘 뒤, Q&A 구간의 실제 PDF 분기점을
다시 대조한다. native HWP는 보조 근거로만 사용하고, PDF의 물리 쪽과 표 owner를 최종
정답지로 삼는다.

## 보존 계약

- 기준 PDF와 최종 HWPX 출력은 모두 383페이지여야 한다.
- PDF p298에는 Q34와 완결된 Q35가, p299에는 Q36과 완결된 Q37이, p300에는 Q38이 있어야 한다.
- PDF p301의 Q41, p304의 Q48은 마지막 response와 blank-bottom row를 같은 페이지에 둔다.
- 새 릴리스 준비 중이므로 이 Stage도 로컬 분석·커밋까지만 수행하며 merge, push, PR 생성을
  하지 않는다.

## 분석

### Q35·Q37

Q35는 6x5/15-cell, 높이 18084, outer bottom 566인 RowBreak 표이며 마지막 response는
3줄+2줄이다. Q37도 같은 6x5/15-cell RowBreak 표지만 높이 23988, 마지막 response가
2줄+7줄이다. 기존 HWPX는 각각의 tail을 다음 페이지로 넘겨 Q36과 Q38을 밀어냈다.

PDF p298~p300 대조에서 Q35는 p298, Q37은 p299에 완결되어야 함을 확인했다. 두 geometry와
line segment 서명에 64px allowance를 한정 적용했다.

### Q41·Q48

Q41은 6x5/15-cell, 높이 29772, outer bottom 0, 마지막 response의 6줄+6줄 저장 frame이다.
HWPX가 Q41 tail을 p302로 넘겼지만 PDF p301은 Q41 전체를 수용하고 p302를 Q42부터 시작한다.

Q48은 6x5/15-cell, 높이 15385, outer bottom 0, 마지막 response의 3줄 단일 문단이다.
64px은 부족했고, 저장 frame의 실제 row total을 수용하는 96px에서 PDF p304 완결과 p305 Q49
시작이 재현됐다. 서명은 profile, geometry, 마지막 response 행, 빈 rowspan cut을 모두 검사해
다른 표에는 적용되지 않는다.

### 11절 병렬 규정 표

Q&A의 synthetic tail 네 개를 제거하면 HWPX는 379쪽이고 103x2 규정 표는 53 fragment였다.
PDF의 규정 표는 57 fragment이므로 reserve를 단계적으로 대조했다.

| reserve | 전체 쪽수 | 규정 표 fragment |
| --- | --- | --- |
| 56px | 379 | 53 |
| 100px | 380 | 54 |
| 140px | 382 | 56 |
| 160px | 383 | 57 |

160px는 Q&A에서 사라진 synthetic tail을 되살리지 않고 PDF와 같은 57개 규정 표 fragment를 만든다.

## 구현

- Q35 3·2, Q37 2·7, Q41 6·6, Q48 3 lineSeg response tail의 exact HWPX saved-frame guard를
  `scan_block_table_split_rows`에 추가했다.
- 각 guard는 cut allowance, fully-consumed fit, physical overflow tolerance에 같은 allowance를
  적용해 row-cut과 실제 draw owner를 일치시킨다.
- HWPX 103x2 병렬 규정 표의 reserve는 160px으로 고정해 57 fragment와 383페이지를 복원했다.

## 결과

- HWPX render-tree: 383페이지.
- Q35 p298, Q37 p299, Q41 p301, Q48 p304 owner가 PDF와 일치한다.
- HWPX 규정 표는 57 fragment가 됐다.
- 남은 시각 차이: HWPX는 부록 앞 p310 blank와 규정 표 말미 p366~p367의 owner가 PDF보다 한
  페이지 이르다. 전체 쪽수만 맞추는 추가 reserve 조정은 금지하며, 이 차이는 Stage 141에서
  page-break/master-page 계약과 규정 표 말미 cut을 별도로 분석한다.

## 시각 증적

- `/tmp/rhwp-3820-stage140-q35-pdf-owner-1/p298-comparison.png`
- `/tmp/rhwp-3820-stage140-q35-pdf-owner-1/p299-comparison.png`
- `/tmp/rhwp-3820-stage140-q36-q37-pdf-owner-1/p299-p300-comparison.png`
- `/tmp/rhwp-3820-stage140-giant-160-map-1/`

## 검증

`CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout --quiet`
를 실행했다. 3개 테스트가 모두 통과했고 HWPX/HWP 저장 레이아웃과 383페이지 계약을 확인했다.

## 상태

focused regression 통과. Stage 141에서 남은 부록 전환 owner를 분석한다. merge, push, PR 생성은
금지한다.
