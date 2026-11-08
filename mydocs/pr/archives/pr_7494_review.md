# PR #7494 리뷰 — 직접 PDF 이미지와 편람 HWP 피델리티

## 최종 판정

**메인터너 보정 후 수용 가능** — #7751 HWP 해결 범위의 로컬·시각 검증과 정확한 code candidate CI를 완료했다. 원 PR 자체 대신 보정이 포함된 통합 PR을 수용 대상으로 삼는다.

원 head는 `a3f06bedb3bbd5402082349b955fdf1312a2f4bf`이며, 메인터너 보정은 최신
devel 기반 별도 통합 PR [#7964](https://github.com/edwardkim/rhwp/pull/7964)에서 검증한다. 원 fork를 다시 쓰거나 삭제하지 않는다.
code candidate `e8d7cbdeb5901e9b345abcc13e6670666a6a4057`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/38993598237) 성공과 trailing head의 CI·mergeability는 구분한다.
최종 merge 조건은 정확한 trailing head의 required checks 성공·MERGEABLE/CLEAN과 승인 범위다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#7494](https://github.com/edwardkim/rhwp/pull/7494), 직접 PDF 이미지 밝기·대비 |
| 작성자·base | ldj-diquest / devel, rhwp 첫 PR임을 작성자 전체 PR 조회로 확인 |
| 원 PR 규모 | 11파일, +571/−17 (원 PR API 접수 시점) |
| source fork | ldj-diquest/rhwp, `fix/direct-pdf-image-adjustments` |
| source head | `a3f06bedb3bbd5402082349b955fdf1312a2f4bf` |
| reviewer·상태 | edwardkim 요청 유지, 접수 시 OPEN/non-draft. merge 전 최신 상태 재확인 |
| 처리 경로 | collaborator_external_pr.md 9.1.1의 devel 체리픽 통합 |
| 통합 창구의 검토 역할 | jangster77의 self-review로 reviewer 요청을 추가하지 않는다. 원 PR reviewer 요청·원 기여 검토와 구분한다. |
| 원 기여 | 기능 6커밋의 author 보존, 원 PR의 devel merge 커밋 제외 |
| 이슈 | [#7751](https://github.com/edwardkim/rhwp/issues/7751) HWP 해결, [#7501](https://github.com/edwardkim/rhwp/issues/7501) 연결선 별도 |

## 원 기여와 메인터너 보정

원 기여는 Direct PDF가 일반 그림의 밝기·대비로 거부되던 경로를 열고,
Skia offset의 불필요한 255배, bake된 payload의 이중 효과와 미지원 효과의 손실을
막았다. 기존 Native PDF 테스트 진입점을 재사용한 점도 유지한다.

독립 한컴 Print와의 대조에서는 SVG를 정답으로 삼은 색조·원본 HWP 속성 순서,
이어받기 요구 높이·통째 표 종료 여백, 저장 줄 pen·overlay 바깥여백과 물리 Print
프레임에 추가 차이가 있었다. 메인터너가 단계별 원인·반례를 확인하고 공통 생산 결과로
보정했다. 실제 Studio DOM은 일반 색조를 CSS 승산으로 소비해 Canvas와 달랐으므로,
resolved PNG와 매개변수 포함 source key를 공유하게 고쳤다. frontend 색조식을 추가하지 않았다.

기존 `ci.yml`의 불필요한 개별 호출 추가·제거는 모두 제외했다. 현재 PR의 workflow diff는 0이다.
원 기여의 384쪽 SVG raster 증거와 메인터너의 383쪽 Native Skia/Canvas 증거를 혼동하지 않는다.

## 입력·기준·실제 생산 출처

- HWP: `samples/2025 행정업무운영 편람(최종).hwp`,10,687,488 bytes,
  SHA-256 `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`.
- Print: `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`,4,692,840 bytes, 383쪽,
  SHA-256 `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`.
  기존 [한컴2024 KoPub 변환 원장](../../tech/investigations/issue-7009/hancom_2024_kopub_oracle.json)의
  HOffice13.0.0.3901·PrintMethod0·job5a2b89ea-dda5-4343-838b-b169e7b9035a를 재사용했다.
- 실제 한컴 저장 색조 HWP 5개와 PDF 5개: [생성·SHA 원장](../../../pdf/issue7751/README.md).
  전부 50MiB 미만이며 통합 Git tree에 포함한다. 실제 원본 글꼴은 시스템 경로로 공급하고 배포하지 않는다.
- 비교 devel: `b7a5388034ee66f58a7e1d5363974e83b6374f57`.
  baseline 실제 producer는 `552514f423ec91f1b7171f6e6b0882a4efe034f2`다.
  renderer/paint/model/document_core/serializer와 HWP parser 경로의 diff0을 재확인했다.
  새 base의 차이는 PDF 가져오기이며, 이전 baseline을 새 SHA의 실행으로 바꾸어 적지 않았다.
- 최종 제품 source `542a8bcbfe00247f554706306f930a16c04a589f`에서 Native·fresh WASM을 새 빌드하고 전쪽을 재출력했다.
  96dpi·compat2024·Print이며 Native는 Skia PNG, WASM은 Canvas PNG다.
  Native는 cef058e45 + 기록한 fmt patch로 실제 생산됐으며 세 생산 파일 지문이 `542a8bcbfe00247f554706306f930a16c04a589f`와 같다. 이후 정식 테스트·문서 변경은 제품 source와 구분한다.
  Docker 부재 호스트의 `--no-opt` WASM 대체 경로와 codegen16을 명시했다.

## 페이지별 시각 검증 요약

`silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣)으로 판정했다. 양쪽 모두 전 383쪽·누락 0·90% 미만 0·고정 devel 대비 0.01%p 이상 하락 0이다. 전체 TSV 766행은 통합 PR 본문에 직접 포함했다.

| 범위·완료 페이지 | Native raw % | fresh WASM raw % |
| --- | ---: | ---: |
| 전쪽 최저 | 93.07066 | 92.98362 |
| 3쪽 | 100.00000 | 100.00000 |
| 130쪽 | 98.69152 | 98.45828 |
| 135쪽 | 98.05498 | 98.03088 |
| 241쪽 | 98.23048 | 98.27806 |
| 289쪽 | 99.74756 | 99.72209 |
| 290쪽 | 99.45606 | 99.44102 |
| 297쪽 | 99.23751 | 99.24374 |
| 375쪽 | 99.37858 | 99.42850 |
| 383쪽 | 100.00000 | 100.00000 |

독립 Print와 확대 판독한 대표 PNG 및 페이지별 원장: [시각 증적](../assets/pr7494_issue7751/README.md).

전 383쪽 Native/WASM의 내용·소유 순서를 대조했고 원본 각주 1·미주 0과 11쪽 각주 소속을
보존했다. 내부 64/32비트 usize sentinel만 대응했으며 실제 문단 번호는 바꾸지 않았다.
각주 영역도 독립 Print와 직접 확인했다. `not_evaluated`와 CI 녹색을 직접 판독의 대용으로 쓰지 않았다.

## 조판 원칙 준수 검토

| 항목 | 실제 경로·독립 근거·실행 증거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 원본 LineSeg·HU 여백·Print MediaBox·실제 RGB를 근거로 삼았다. 문서 ID/쪽번호 분기·출력 은폐 없이 공통 raster·overlay·저장 축 속성으로 선택한다. | 충족 |
| 측정·배치 일관성 | `scan.rs::fragment_extent`의 남은 행 높이/간격 → required_height/fragment_height → 기존 컷 예약 → layout의 같은 남은 행. 표 끝은 block/entry 수용 후 흐름 → stored end → layout_column으로 연결된다. | 충족 |
| 최종 출력 소비 | 공통 image resolver의 baked PNG/suppress_effects → PaintOp → Skia/Canvas/SVG/Direct PDF/JSON/DOM. 실제 RGB·lazy/inline 바이트·DOM screenshot·SVG browser·Direct PDF 실파일로 각각 확인했다. | 충족 |
| 분할·이어받기 | 이미 소비한 높이0 행의 간격6.8px 중복 예약을 제거했다. 컷 없는 경로와 컷 유닛을 보존한다. 통째 RowBreak의 fit 면제 하단566HU는 순차 후속 흐름에1회 전달하며 독립 프레임 소유 경로는 제외한다. p241 끝/후속 제목/표와 p289–290 Q13–16의 전체 셀 보존, 전체 쪽수/빈 쪽을 대조했다. | 충족 |
| 셀 도형 가로 프레임 | 전체·부분·내장 표의 원본para_shape_id → 공통 layout_cell_shape_with_parent_path의 resolved 문단 여백 → 최종 Group 원점 → 누적 자식 행렬에 1회 소비. TAC 확정 글줄 위치·문단이 아닌 기준·세로 원점은 유지한다. p135 일반사각형과p289의 8개 화살표는 독립 Print 좌표 ±2px·소유·포함·누락으로 보호한다. 두 backend의 나머지 381쪽 PNG는 byte 동일. | 충족 |
| 줄 소속·점유 높이 | 저장 inline pen과 기존 source-line indent를 재사용하고 표 바깥여백은 상위 원점에서1회 소비한다. 글상자 반각 재조판 뒤 공통 vpos를 재계산한다. 앞 개체 때문에 저장 시작이 이동하면 원래 끝의 재사용을 거절한다. treat_as_char만으로 같은 줄로 묶지 않는다. | 충족 |
| 사례·증거의 독립성 | 실제 저장 HWP/기존 Print와 합성 후 한컴 저장한 색조 5개를 구분한다. 서문/이어받기/Q&A는 before3FAIL → afterPASS, 셀 도형은수정 전 정식6검사중2FAIL→수정후6PASS(독립 x·그림안포함), 실제 DOM RGB는96/120/144 → 독립131/151/171, 종이 계약도 별도 beforeFAIL → afterPASS였다. 환경 오류·0건 실행은 검출/통과에서 제외했다. | 충족 |
| 기준값 변경 | SVG filter ID·잘못된 HWPX 필드 순서·과거 px 핀을 독립 Print/원본 속성·실제 RGBA/소유 관계로 교체했다. 정확한 source 메트릭은 유지하며 허용치·golden을 완화하지 않았다. base 미달3검사만 #7756/#7955에 보존·이관했다. | 충족 |
| 회귀 선행조건 | 편람 전 383쪽과 색조 5개의 양쪽 raw 90% 이상 확인 뒤 정식 case를 등록했다. 양쪽 raw 98% 이상인 완료 쪽에만 독립±2px 검사와 관계 검사를 함께 유지한다. 동등 동작은 100→99→98을 우선하고 WMF crop은 고유 130쪽을 사용한다. | 충족 |
| 확대 판독 | p3 본문/배경,90 최저 그림/본문,130 crop/괘선,224/226 저장 줄,241 이어받기 끝,135 강조 사각형,289 그룹 화살표와 일반선,290 Q&A,297 ※/법령명,375 두 단 서식,383 말미를 같은 배율로 대조했다. [대표 PNG](../assets/pr7494_issue7751/README.md), 색조는550 Canvas+550 DOM 독립 패치다. | 충족 |
| 주장·검증 범위 | `542a8bcbfe00247f554706306f930a16c04a589f`의 로컬 필수 검사와 code candidate의 GitHub Full CI를 구분한다. 전 383쪽 Direct PDF·모든 WMF의 일반 호환·다른 HWPX/#7772 전체 피델리티는 주장하지 않는다. | 충족 |

별도의 실제 호출 추적·기각 실험·분기별 원인과 관측값은
[1단계](../../working/task_m100_7751_stage1.md),
[5단계](../../working/task_m100_7751_stage5.md),
[9단계](../../working/task_m100_7751_stage9.md),
[12단계](../../working/task_m100_7751_stage12.md),
[13단계](../../working/task_m100_7751_stage13.md),
[14단계](../../working/task_m100_7751_stage14.md),
[15단계](../../working/task_m100_7751_stage15.md)에 연결한다.

## 최종 로컬·CI 검증

공용 절대 `target/pr-review`·jobs8로 Cargo를 순차 실행했습니다. fresh WASM은 Docker 부재 호스트의 `--no-opt` 대체 경로·codegen16이며, Native Skia3종과 전체 기본 nextest는 서로 구분했습니다. 전체 nextest에 SVG snapshot도 포함됩니다.

| 검사 | 결과 |
| --- | --- |
| `manifest-prepare` | exit 0 |
| `issue_7751_picture_tone_source` | Summary [   0.058s] 1 test run: 1 passed, 236 skipped |
| `issue_7751_raster_picture_tones` | Summary [   0.135s] 2 tests run: 2 passed, 229 skipped |
| `issue_7751_print_paper_frames` | Summary [   2.591s] 2 tests run: 2 passed, 230 skipped |
| `issue_7751_handbook_completed_boundaries` | Summary [   3.369s] 6 tests run: 6 passed, 230 skipped |
| `issue_6735_textbox_table_cell_shape` | Summary [   0.047s] 6 tests run: 6 passed, 227 skipped |
| `issue_5716_group_level_roundtrip` | Summary [   0.312s] 2 tests run: 2 passed, 220 skipped |
| `issue_5568_group_picture_crop` | Summary [   0.028s] 2 tests run: 2 passed, 223 skipped |
| `issue_2225_missing_picture_placeholder` | Summary [   1.791s] 2 tests run: 2 passed, 214 skipped |
| `render_p37_direct_pdf_export` | Summary [   0.885s] 14 tests run: 14 passed, 233 skipped |
| `native-skia-lib` | test result: ok. 3861 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 32.79s / test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s / test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s / test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `native-all-tests` | Summary [ 743.421s] 10595 tests run: 10595 passed (13 slow), 45 skipped |
| `fmt` | exit 0 |
| `clippy` | exit 0 |
| `wasm-clippy` | exit 0 |
| `workspace-build` | exit 0 |
| `workspace-clippy` | exit 0 |
| `manifest-check` | exit 0 |
| `unit-tier-check` | exit 0 |
| `visual-python` | 156 PASS |
| `studio-tests` | 2,023 PASS,0 FAIL/skip |
| `studio-typescript` | exit 0 |
| 실제 브라우저 Canvas·DOM·WMF crop·HFT·font binding | 8 PASS |


정확한 code candidate CI: [CI](https://github.com/edwardkim/rhwp/actions/runs/38993598237) 성공, [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38993598153) 성공, [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/38993598197) 성공, [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38993598196) 성공, [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38993598210) 성공, [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38993598174) 성공.
후행 문서 commit의 CI와 최종 merge 상태는 이후 별도로 확인한다.


기본 전체 회귀에 `svg_snapshot`도 포함한다. 새 회귀 source는`tests/cases/`에두고 파생harness/manifest는커밋하지않는다.
공용 절대`target/pr-review`·jobs8로 Cargo를순차실행했다. 이전head의중단전체nextest와빌드환경실패는PASS로세지않는다.
전쪽TSV·로그·provenance는실행중`output/pr-review/pr7494-20261016/`에보존하며,
종료시`/home/tsjang/rhwp-review-evidence/pr7494-20261016/`로핵심증거를해시확인후이관한다.

## 남은 차이와 후속 이슈

- 일반그림색조 5개는N/W/실제Direct PDF전쪽raw 100%다. unbaked흑백/미지원legacy효과거부는유지한다.
  편람3쪽Direct PDF성공을전 383쪽지원으로확대하지않으며63쪽연결선[#7501](https://github.com/edwardkim/rhwp/issues/7501),130쪽SVGpayload·289쪽Line화살표의명시적거부는남는다.
- 미세글자힌팅·전진폭·raster색차이가남는다.224쪽질문기점2.333px잔차는새px핀으로고정하지않았다.
  WMF저색수확대의일반팔레트선택은미확정이며6단계의독립반례/잔차를보존한다.
- HWPX별도피델리티와hwpspec전체쪽수는이번HWP383쪽해결로닫지않는다.
- [#7756](https://github.com/edwardkim/rhwp/issues/7756)과[#7955](https://github.com/edwardkim/rhwp/issues/7955)는OPEN유지.
  기존base도90%미만인동일원본/범위만이관하고공유원본·PDF·정상검사는유지했다.

## Merge 후 contributor PR comment 계획

1. merge와 devel의 asset 존재를 확인한 후 원 PR에 **한국어 존댓말로 감사와 첫 기여 환영**을 남긴다.
   원 기여의 Direct PDF 효과 보존과 메인터너의 독립 Print 색조·배치·DOM 보정을 구분한다.
2. 실제 통합 PR·merge SHA·정확한 CI 링크와 Native/fresh WASM 각 383쪽·최저 raw·미달 0을 적는다.
   [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 direct link로 남긴다.
   raw는 색상 경계 보정 전 2px 실루엣이며 엄격 pixel 일치율이나 사람의 판정 정확도가 아님을 구분한다.
3. merge SHA에 고정한 `mydocs/pr/assets/pr7494_issue7751/native_review_130.png`,
   `wasm_review_130.png`, `dom_color_review.png`를 실제 Markdown 이미지로 표시한다.
   현재 문서의 확인 범위·수치·남은 차이를 넘는 주장을 하지 않는다.
4. `--body-file`로 게시하고 API로 실제 본문·한글·이미지 URL을 재조회한다.
   [Discussion #858](https://github.com/edwardkim/rhwp/discussions/858)의 Slack 가입 안내를 덧붙인다.
5. superseded 원 PR은 통합 merge를 링크해 닫되 **fork branch는 보존**한다.
   #7751 close/comment와 #7445 표 완료 취소선·closed subissue 연결 해제를 확인한다.
   duration 갱신 결과 확인, clean 검토 worktree·임시 upstream head·local branch와 root output 전체 정리까지 완료한다.
