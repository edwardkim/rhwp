# PR #8155 리뷰 — 편람 중첩 도장 표의 공통 가로 앵커

## 최종 판정

**승인 — #8151의 144쪽 배치 개선과 회귀 검사 보완 범위의 로컬 검증 및 정확한 code head CI가 완료됐다.**
작성자 `jangster77`의 collaborator self-review다. reviewer는 지정하지 않았다. 사용자가 CI 완료 후 머지·후속 처리를 승인했다. 남은 머지 조건은 이 기록을 담은 trailing head의 CI 성공과 최신 mergeability·원격 SHA 확인이다. 아직 실행하지 않은 trailing CI·merge를 성공으로 기록하지 않는다.

## 접수·검증 신원

- [PR #8155](https://github.com/edwardkim/rhwp/pull/8155), base `devel`, 작성자 `jangster77`.
- code head `07daf7884185b4e9b030d290de6d27e39e3df741`, runtime `d81b8b4410779017d50a8891caa6a83ebec994d6`, 로컬 code/test `1c8294dd523fab69b303ccb4cb45bb55de003a72`.
- 고정 devel `2e087a5b4bc1515ad877e8020b30114639a59392`. PR 생성 직전까지 같은 SHA였다.
- Closes [#8151](https://github.com/edwardkim/rhwp/issues/8151). [#8153](https://github.com/edwardkim/rhwp/issues/8153)은 별도 후속 목표로 유지한다.
- 실제 원본·독립 Print·입력 SHA·producer·전쪽 TSV·PNG는 [최종 증적](../assets/issue_8151/README.md), 실제 명령은 [검증 명령](../assets/issue_8151/validation-commands.md)에 있다.

## 정확한 code head CI

아래 실행은 모두 같은 PR·repository·branch의 `07daf7884185b4e9b030d290de6d27e39e3df741`에 대한 실제 성공이다. source PR 또는 base의 CI로 대신하지 않았다.

| Workflow | 실행 | 실제 결과 |
| --- | --- | --- |
| Adapter inter-diff | [39763980913](https://github.com/edwardkim/rhwp/actions/runs/39763980913) | success |
| CodeQL | [39763980858](https://github.com/edwardkim/rhwp/actions/runs/39763980858) | success |
| Render Diff | [39763980811](https://github.com/edwardkim/rhwp/actions/runs/39763980811) | success |
| CI Impact Policy Controller | [39763980770](https://github.com/edwardkim/rhwp/actions/runs/39763980770) | success |
| Proptest roundtrip | [39763980877](https://github.com/edwardkim/rhwp/actions/runs/39763980877) | success |
| CI | [39763980825](https://github.com/edwardkim/rhwp/actions/runs/39763980825) | success |

## 로컬 검증과 시각 판정

전체 nextest **11,359 PASS / 0 FAIL / 45 SKIP**, 기존 focused 2 PASS·신규 focused 1 PASS,
Native Skia lib rhwp 3,859 PASS·12 IGNORE(workspace 추가 182 PASS), placeholder 2 PASS·직접 PDF export 14 PASS다.
fmt·native/WASM32/workspace all-target Clippy·workspace build·고정 base manifest가 exit 0이다.
호스트 locked wrapper의 release·wasm-opt fresh WASM 빌드와 실제 Canvas 출력을 완료했다.

두 편람은 원본·정상 KoPub Hancom 2024 Print와 같은 383쪽이다. 전체 Native/fresh WASM raw 최저 90% 이상,
144쪽 HWPX 98.84256/98.81159%, HWP 98.84240/98.81185%다. 독립 Print에서 도장 3개·안내문·앞뒤 본문·하단 그림을 같은 배율로 판독했다.
72개 입력 1,529쪽/backend를 base/후보로 대조한 3,058쌍 중 편람 외 70개 입력 1,526쌍은 raw·RGB·render tree가 같다.
원본·Print·compat·96dpi·실제 Hancom/KoPub 공급을 고정했으며, 최초 명시 글꼴 누락의 진단 WASM TSV는 최종 자료에서 교체했다.
전쪽 raw가 같더라도 실제 내용 보존을 대신하지 않도록 PNG·tree도 대조했다.

각 편람·backend에서 116·144쪽 개선, 85·86·87·179쪽 작은 raw 하락, 나머지 377쪽 동일이다.
최대 하락 0.02427%p, 0.1%p 이상 하락 0쪽이다. [픽셀 분해·독립 좌표·전후 crop](../assets/issue_8151/page-change-analysis.md)을 확인했다.
85·179쪽은 불일치 수가 같고 합집합 분모만 줄었다. 86·87쪽은 올바른 도장 x 이동으로 기존 본문 불일치의 2px 이웃 지원이 바뀌었다.
본문·Print의 기존 세로 차이가 새로 생긴 본문 이동은 아니며, 전쪽 100%가 완료됐다고 쓰지 않는다. 사용자 범위 조정에 따라 후속 #8153에서 처리한다.

## 조판 원칙 준수 검토

| 검토 항목 | 확인 근거 | 판정 |
| --- | --- | --- |
| 제출 전 로컬 검증·PR 상태 | 위 로컬 게이트와 실제 code CI, 일반 Open PR | 충족 |
| 기존 해결 계보 | #3930 → PR #4058, #3820 → `ba39fac28f0b28512edf3de358bd37d3d5538408` → PR #4517의 원래 저장·배치 계약 확인 | 충족 |
| 일반성·공통 처리 | 셀 overlay의 별도 inline x override를 제거해 기존 공통 중첩 앵커를 소비. 문서 ID·새 좌표 clamp 없음 | 충족 |
| 측정·배치 일관성 | `nested_overlay_outer_frame` 측정과 확정 y는 유지. `ctrl_area` → 중첩 `layout_table` → `compute_table_x_position`의 x 소비를 공통 결과로 연결 | 충족 |
| 리베이스 기능 보존 | base가 전진하지 않아 재리베이스 없음. #8148 전체 입력의 같은 환경 base/후보 비교로 기능 유지 확인 | 충족 |
| 실제 출력 소비 | 같은 Table bbox를 소비하는 Native Skia와 최적화 fresh WASM Canvas 실제 PNG | 충족 |
| 분할·이어받기 | 컷·요구/예약 높이·종료 코드 변경 없음. 전체 쪽수·기존 관련 회귀 유지 | 비해당 |
| 줄 소속·점유 높이 | 줄 구성·LineSeg 수용·세로 흐름 변경 없음. overlay의 앞 텍스트 pen 전진만 가로 원점에서 제외 | 비해당 |
| 독립 기대값·반례 | 실제 저장본과 여백 0 대조군. 원본 283HU 이동·y/크기/쪽수·순서·비겹침 검사, 같은 소스 base FAIL / 후보 PASS | 충족 |
| 기준값 변경 | 잘못된 기존 x 핀을 독립 Print stroke 중심의 96dpi 좌표 ±2px로 교정. 원본 4251HU 크기와 공통 y 유지. baseline/golden 완화 없음 | 충족 |
| 회귀 선행 조건 | 전체 90% 이상 및 144쪽 양쪽 raw 98% 이상·직접 판독 후 기존 검사 보완과 정식 `tests/cases` 신규 검사 | 충족 |
| 90% 후 확대 판독 | p144의 도장·안내문·하단 그림, 85/86/87/116/179의 같은 영역 Native/WASM/Print crop | 충족 |
| 주장·범위 대조 | ±2px 검사와 raw 2px 반경 구분. 다른 입력 무영향 확인과 작은 raw 하락 공개. 전쪽 100%는 후속 #8153 | 충족 |
| 검증 입력 커밋 | HWP/HWPX/Print 4개 파일의 실제 바이트가 검토 head와 같음. 전체 입력 원장 72개도 source/PDF SHA 고정 | 충족 |

## Merge 후 contributor PR comment 계획

실제 merge SHA를 확인하고 devel에 위 증적이 존재하는지 확인한 뒤, #8151 상태와 완료 comment 및 이 PR의 merge comment를 게시한다.
문안은 한국어 존댓말로 쓰고 code/trailing CI 실행 URL·실제 결과, 로컬 11,359 PASS, p144 양쪽 backend 값,
전체 TSV와 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
`mydocs/pr/assets/issue_8151/images/p144-head-{native,wasm}-{review,overlay}.png` 및 HWP 대조 이미지는 실제 merge SHA raw URL의 Markdown 이미지로 표시한다.
85·86·87·179쪽 작은 raw 하락과 후속 #8153을 명시하며 #8153은 닫지 않는다. `--body-file`로 게시 후 API로 본문·SHA·이미지 링크를 다시 읽는다.
그 뒤 duration refresh 결과를 확인하고 devel 동기화, 이번 임시 head branch 정리, 기본 worktree·공유 target 보존, 사용 종료한 output 전체 삭제를 수행한다.
