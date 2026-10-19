# #7998 보존 테두리의 명시적 스타일 교체

- 날짜: 2026-10-19 (Asia/Seoul)
- 승인: stage15 결과 공유 뒤 다음 절편 진행.
- 시작 head: `356af501a`. [계획 4.4](../plans/task_m100_7998_impl.md#44-p4-잔여-범위-확정과-종료-순서--2026-10-19)의 두 번째 좁은 경계다.

## 수정 전 근거와 기대

[stage14](task_m100_7998_stage14.md)의 원본 phase 대조군에서 실제 셀 테두리를 빨강
Solid로 바꾸어도 옛 검정 paint 280개는 남았다. [stage15](task_m100_7998_stage15.md)는
그 조각들의 변/출처를 공통 IR에 연결했지만 실제 편집 소비는 미완료였다.
명시적으로 바꾼 변과 실제 공유 변의 소유 조각만 제거하고, 다른 변/내용/가림/자식 표를
보존하는 것이 이번 기대다. 빈 틈을 새 native 스타일로 교체하는 것은 명시적 편집 의도이며,
무편집 가져오기에서 원래 위상/틈을 바꾸는 것과 구분한다.

현재 setter는 오른쪽만 있는 JSON을 BorderFill 변경으로 인식하지 않고, 이웃 갱신은
항상 네 변을 소비했다. 대화상자는 배경만 바꿔도 네 변을 재전송하므로 실제 클릭한 변과
동일 값 확인을 구별한다. 원래 None 속성에 source paint가 있는 경우의 명시적 지우기도
검사한다. 삭제 조각은 속성 ID 역연산만으로 복원되지 않으므로 같은 command 저널에 연결한다.

## 구현과 검증 경계

실제 source→공통 소유→set_cell_properties_native의 변경 계획→BorderFill/이웃 변→
기존 layout/paint 소비와, 양 adapter 저장/재열기·실제 Studio undo/redo를 확인한다.
삭제 전에 현재 물리 셀과 소유를 재검증하며 좌표만으로 내용 그림을 분류하지 않는다.
변경 문단만 담은 undo 저널은 선형 히스토리의 역순과 현재 문단 지문을 전수 확인한다.

새 행열의 비표준 stencil 상속, fill/compound·표 밖 잔여까지의 일반 교체, 부분 병합 이웃,
크기/안여백과 동시 변경, zone 전체·중첩 경로 setter의 확장은 아직 완료 범위가 아니다.
부분 병합 이웃의 인증된 조각과 동시 크기 변경은 변경 전에 거절한다.
전체 회귀/lint/package/최신 devel 게이트는 P5에 둔다.

실행 자료는 ignored output/pr-review/issue7998/p4-border-replacement/에 보존한다.
이 문서는 검증 수치나 시각 완료를 선행 선언하지 않는다.


## 실제 구현 소비와 보호 경계

- model::border_paint::valid_side는 producer와 같은 물리 셀/shape 계약을 다시 확인한다.
  commands/border_paint_edit::plan은 같은 source의 반대 변 소유와 현재 target/이웃을
  대조해 교체할 직접 Rectangle control만 계획한다. 내용 그림·흰 fill·자식 표·다른 변은
  선택하지 않는다. 한 source의 반대 변이 없는 외곽은 표 밖 조각 closure를 먼저 해결해야
  하므로 변경 전에 거절한다. bbox만으로 내용/경계를 분류하지 않는다.
- set_cell_properties_native는 네 방향의 sparse JSON을 모두 인식하고, 생략한 속성은
  기존 BorderFill에서 가져온다. 실제 바뀐 변만 이웃 갱신에 전달한다. 대화상자의
  borderPaintSides 네 boolean은 실제 변 클릭을 나타낸다. 배경/대각선만 확인할 때
  동일한 None 값을 보내는 것과, 원래 source paint를 명시적으로 지우는 것을 구별한다.
- 기존 layout/paint는 새 native BorderFill과 남은 실제 controls를 그대로 소비한다.
  뒤에서 원래 검정 선을 숨기거나 전체 그림을 덮는 보정은 없다. 제거한 슬롯은 공통
  Paragraph::rebuild_char_offsets로 원문·글자모양·줄 시작/필드 위치와 함께 갱신한다.
- BorderPaintUndo는 바뀐 문단만 보관하며 Document snapshot 슬롯은 사용하지 않는다.
  applyCellBorderFillIds의 optional borderPaintUndoIds가 배치를 역순 전수 검증한 뒤
  속성 ID와 조각을 함께 복원한다. TS command는 ID 0도 실제 변경으로 보관하고,
  실패 rollback/undo/redo/discard에 연결한다. 4,096개 보관 한도·ID 소진은 변경 전에
  거절하며 자동 축출하지 않는다. set_document는 이전 문서의 보관 항목을 해제한다.

처음의 undo 검증은 Paragraph 전체의 JSON 지문을 비교했다. 실제 글자 입력/삭제 뒤
원문·서식·개체가 같아도 LineSeg의 baseline/column_start가 재계산되어 정상 undo가
거절되는 반례를 실행했다. 지문은 재계산 가능한 LineSeg를 제외하고 원문·문단/글자
서식·control/슬롯 등은 계속 확인한다. 성공 때는 캡처한 source LineSeg도 조각과 함께
되돌린다. 실제 문자 편집을 무조건 허용하거나 원문 변경을 덮어쓰는 완화가 아니다.
보정 전 FAIL / 후 PASS는 replacement_cases-pre-undo-fix.log와 최종 실행에 연결한다.

역연산 JSON 오류/없는 undo ID·stale 물리 소유·바깥 잔여·크기/안여백 동시 변경은 적용
전에 확인한다. 부분 병합/행열 상속·모든 작성 owner/continuation 일반화의 완료를
이번 작은 계약의 통과로 대신하지 않는다.

## 독립 한컴 Print 출처

phase-shifted-dot-right·phase-shifted-dot-none·thin-fill-occluder-right의 실제 편집
HWPX 세 개를 engine 2020으로 인쇄했다. 서비스가 기록한 한컴 버전은 11.0.0.9136,
pdf_print_method=0, pdf_output_mode=hancom2020_pdf_driver_one_up이다.
print-sources.json에 입력/PDF SHA·job ID·제품·출력 경로를 보존했고, undo 검증만
보정한 뒤 재생성한 실제 입력도 같은 SHA임을 확인했다. 같은 입력을 다시 인쇄하지 않았다.
원본 Helvetica의 한컴 대체 글꼴과 기존 점선 농도 차이는 남아 있으며, 편집 보정을
그 글꼴 문제의 해결로 보고하지 않는다.


## 최종 검증과 직접 판정

| 최종 제품의 범위 | Native | fresh Docker WASM |
|---|---|---|
| 원본 두 실물 + 공개 세 대조군 | 5건/16쪽 | 5건/16쪽 |
| Native API 편집 3개 × HWP/HWPX | 6건/6쪽 | 6건/6쪽 |
| 실제 WASM command 저장 3개 × HWP/HWPX | 6건/6쪽 | 6건/6쪽 |
| 전체 전쪽 raw 최저 | 97.57194% | 97.13390% |
| 편집본의 한컴 Print 대비 최저 | 99.14432% | 99.34314% |
| 90% 미달/쪽 누락/쪽수 차이/실행 오류 | 0 | 0 |

원시 TSV는 native-all-pages.tsv·wasm-all-pages.tsv, 명령/입력/수치는 validate.py·
각 results-source/edited/actual.json과 validation-summary.json에 보존했다.
최종 제품을 새로 출력한 원본 16쪽은 각 backend의 stage15 PNG와 전쪽 바이트 동일이다
(source-png-identity.json). 이것은 이전 checkpoint 비교이며 P5 최신 devel 비교를
실행했다는 뜻이 아니다. 제품이 바뀐 후의 원본/편집 캡처는 새로 산출했고, 같은 입력
SHA의 한컴 Print와 기존 무관한 기준만 재사용했다.

독립 Print·Native·fresh WASM의 같은 영역을 직접 판독해 오른쪽 공유 변만 빨강 실선이
되거나 없어지고, 아래쪽 원래 점선·자식 표·앞뒤 원문·얇은 흰 가림이 보존됨을 확인했다.
기존 Helvetica 대체 글꼴·선 농도 차이도 비교 HTML에 명시했다. 메인테이너가
**2026-10-19 “시각·편집 구조 통과”**로 판정했다. 이 판정을 외곽 잔여나 행열 계승의
완료로 확대하지 않는다.

실제 CanvasKit Studio의 SetCellBorderFillCommand에서 세 편집을 적용하고, 문자 X
삽입/삭제 뒤 undo로 원래 HWPX 바이트를 복원했다. redo의 편집 HWPX도 첫 적용과
바이트 동일하며 snapshot 슬롯은 0이다. 양 저장 6개를 실제 WASM에서 재열고 Native로
다시 읽어 남은 개체·셀·변·source·물리 지문이 Native API 결과와 같음을 확인했다.
studio-result.json·wasm-native-readback.log와 actual 전쪽 TSV에 연결한다.
실제 instantiate한 WASM SHA도 아래 최종 산출물과 동일하고 page error는 0이다.

정식 새 regression_suite_015의 issue_7998_pdf_border_paint_replacement **6건**은
source/편집본의 Native·fresh WASM 90% 이상과 직접 판독 후 등록했다. 기존 소유 5건·
periodic 4건·border editing 3건·clip/placement/owned lists 각 3건·#5959 역연산
4건을 더해 **Rust 31건 실제 실행/통과**다. 전수 검사로 부르지 않는다. TS command·
mutation routing·dialog 경유 검사 **24건**과 TypeScript 검사를 통과했다. 범위 적용은
첫 셀의 미변경 preview 변/BorderFill ID를 다른 셀에 복사하지 않고 sparse 변경으로
전달하도록 별도 검사했다. 이 TS 변경은 Rust compiler 입력을 바꾸지 않아 동일 최종
Native/WASM을 재사용했고, 실제 Studio 명령은 변경한 TS로 다시 실행했다.

회귀는 source의 실제 소유/다른 내용/원문·저장·원자적 오류·배치 역순 복원을 확인한다.
렌더링 golden이나 절대 px 핀은 추가하지 않았다. 실제 부분 병합 이웃의 독립 편집
Print는 이번에 실행하지 않았으며, 해당 보호 분기만으로 병합 일반화를 충족 처리하지
않는다. 전체 Rust/lint/Worker/package·최신 devel 게이트는 승인 계획대로 P5에 유지한다.

## 빌드 신원과 보존

| artifact | SHA-256 |
|---|---|
| Native | `450d90b5d4438d7f9c375153dc6ccc5c95b8ff8fd422742e2d3e9ddfad4d755e` |
| Docker WASM (19,450,376 bytes) | `966beb92ac7accdd204ce4af54e0eb36e05c12a8881d4054dea3551cebb99876` |
| JS glue | `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6` |
| 최종 runtime manifest | `f38ea8f58a7039ddcfe514994efe0abca49310ee661f7f6f049746e568049e62` |

명령/성공은 native-build-final.log·docker-wasm.log·build-artifacts.json에 있다.
수정 중 발견한 LineSeg undo 결함의 보정 전 빌드/검사는 별도 이름으로 보존했고 최종
결과에 합산하지 않았다. 중단한 이전 wasm-opt와 부분 pkg도 배포/검증 완료로 세지 않았다.
공유 target과 Docker named cache는 초기화하지 않았다. root pkg/public의 최종 JS·WASM은
바이트가 같으며 Studio 7700은 변경하지 않았다. 기존 추적 public/rhwp.js에는 새
저널 해제 API의 binding 7행만 반영한다. pkg·WASM binary·실행 로그는 커밋하지 않는다.

증거 root는 저장소 밖
/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-border-replacement이며
ignored output/pr-review/issue7998/p4-border-replacement로 연결했다.
이동 가능한 화면은 E:\tmp\issue7998-border-replacement-review\index.html이다.
원본·한컴 PDF·각 backend·실제 저장본·전쪽 TSV를 상대 경로로 보존했고 29개 asset/link를
확인했다. user-cdp-tabs.json에 실제 Studio 7793의 세 새 탭과 1쪽 표시를 기록했다.

다음 구현은 비표준 외곽의 표 밖 잔여·fill/compound 소유 closure를 완결한 뒤 새 행열
계승/삭제·복원으로 연결한다. R2 수평 문단 메트릭과 P4/P5 완료는 아직 남아 있다.
원격 push·PR 등록·이슈 종료는 하지 않았다.
