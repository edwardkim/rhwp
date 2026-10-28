# #7445의 미완료 문서 검토 범위 제외

## 사용자 지정과 현재 상태

2026-10-25 사용자는 #7445에 이관되어 완료되지 않은 문서를 이번 PR에서 모두
제외하도록 지정했다. 원격 API에서 하위 이슈 96개의 상태를 읽어 OPEN 85개와 CLOSED 11개를
구분하고, 코멘트가 있는 8개 이슈의 최신 코멘트 13개와 대표 이슈의 남은 코멘트 1개까지 대조했다. 본문의 과거
「96개 모두 OPEN」 설명은 현재 상태로 사용하지 않았다.

원본 경로와 보존 자산, SHA-256이 같은 별칭을 합쳐 132개 경로를 기록했다. 현재
존재하는 117개는 해시를 고정했으며, 이동 전 samples 경로 15개도 재유입 방지를 위해
기록했다. 종료된 이슈의 원본은 새 제외 대상으로 추가하지 않는다. 앞서 사용자가
별도로 지정한 편람·sample16 제외와 기존 삭제는 유지한다.

- [문서 경로·추적 이슈·SHA](../pr/assets/issue7445/pr-scope-20261025/documents.tsv)
- [OPEN 85개 상태와 범위 해석](../pr/assets/issue7445/pr-scope-20261025/issues.tsv)

## 포함하지 않은 비교 대상과 범위 정정

이슈 본문에 언급됐다는 이유만으로 정상 대조군까지 제외하지 않는다.
`exam_kor.hwp`, `lseg-05-tab.hwp`, `issue2004_cell_image_stack.hwp`,
`36367506-water-facility-approval.hwpx`, AIFT의 정상 HWPX 쌍둥이를 새 제외에 넣지 않았다.
#8100의 보존 묶음에서도 다른 이슈의 정상 대조군과 완료된 #8060 원본을 분리했다.

#7740은 범위 정정 기록이고 #7782는 이관 철회 기록이므로 별도 문서 제외를 만들지 않는다.
#8046·#8047·#8048의 정확한 이관 함수는 이미 활성 Rust source에 없음을 확인했다.
이 기록들을 근거로 일반 수식·WMF/EMF parser·보안 검사를 추가 제거하지 않는다.

## 반영 경로와 유지 범위

- 축별 공통 원장에 문서의 body·off-canvas·text-overlap·cell overflow·oracle·render-page
  수집 제외를 등록한다. baseline 수치와 허용 오차를 늘리지 않는다.
- Oracle의 재귀 수집과 명시 파일·저장 manifest, 페이지 정답지 생성기와 실행 게이트도
  같은 축별 원장을 읽도록 한다. 명령행으로 같은 원본을 재검증하는 Visual Sweep 자체는
  변경하지 않는다. 이슈 해결 작업은 원본을 계속 사용할 수 있다.
- 기본 renderer manifest에서 해당 미완료 원본의 5개 캡처 항목만 제거한다. 나머지
  109개 항목과 원래 JSON 줄바꿈을 유지한다.
- 정확한 기존 parser·serializer·보안·내용 소속 검사는 유지하며, 이관 문서의 전체
  피델리티 검토·추가 보정·완료 게이트와 구분한다. 전체 suite에서 별도 실패하면
  해당 원본과 기존 이관 범위를 대조하고 다른 문서의 baseline을 완화하지 않는다.
- 현재 실패한 #7379의 `unproven_empty_picture_table_frame_keeps_measured_flow_origin`
  한 함수는 같은 원본의 열린 #7753 범위로 분리한다. 나머지 정상 13개 함수는 유지한다.
  제거 전 전체 모듈과 helper, source commit·Git blob·SHA·유지/제거 함수·복원 조건은
  [#7753 원장](../pr/assets/issue7753/pr-scope-20261025/transfer.json)에 보존했다.

이는 사용자가 지정한 PR 범위 변경이며, 현재 차이가 모두 기존 base의 잘못된 회귀였다고
판정한 것이 아니다. 이관을 출력 개선·이슈 해결·PR 승인으로 보고하지 않는다. 원본
HWP/HWPX/PDF는 이동·삭제하지 않으며 renderer 구현과 지침은 바꾸지 않는다.

## 검증 계획

1. API 상태·정확한 입력·동일 바이트 별칭과 정상 대조군의 포함 여부를 재확인한다.
2. Rust·Python 경계가 상대/절대/Windows 경로에서 같은 축을 제외하는지 확인한다.
3. Oracle·페이지 생성기/실행 목록·기본 renderer manifest의 실제 수집 결과를 대조한다.
4. 관련 전수 래칫·남은 #7379 정상 검사와 자동 suite 수집을 실행한다.
5. Python 도구와 renderer contract, 문법·형식·diff 검사를 마친 뒤 커밋한다.

로그·원격 원문·중간 JSON은 ignored `output/pr-review/semanticist21-20261019/`에만
보존한다. 아직 끝나지 않은 검증은 완료로 표시하지 않는다. #7445와 미완료 하위 이슈를
이번 PR의 Closes 목록에 넣지 않는다.

## 실제 검증 결과

- Python 경계: 132개 경로 × 4개 표기 × 6개 축 = **3,168건 PASS**. 입력 117개 SHA와
  정상 대조군 5개도 확인했다. 실제 coverage 1,183개·page-smoke 1,234개·페이지 oracle
  생성기 1,020개 수집에 제외 문서가 없었다. 제외만 있는 페이지 게이트는 renderer를
  실행하기 전에 실행 대상 0개를 명시했다(`logs/issue7445-python-scope-3830.log`).
- Rust 경계: 같은 3,168건 PASS. parser 축과 정상 대조군을 제외하지 않는 것도 확인했다
  (`logs/issue7445-rust-boundary-3836.log`).
- Oracle 도구 전체 unit **120개 PASS**(`logs/issue7445-oracle-full-unit-3834.log`).
- renderer contract 전체 PASS(`logs/issue7445-renderer-contract-3835.log`). 허용 오차는
  유지하고, 승인된 캡처 5개와 대표 tier 1개 제거에 해당하는 목록·개수만 반영했다.
- 관련 정식 Rust **69개 실행, 65개 PASS·4개 FAIL**, exit 100
  (`logs/issue7445-rust-scope-3833.log`). #7379의 남은 13개는 모두 통과했다.
  본문 원장을 읽을 때도 수집과 같은 제외 조건을 적용하도록 초기 누락을 바로잡았다.

- 명시 파일 입력과 저장 manifest의 pair·unpaired 목록에서도 제외 입력만 빠지고 정상
  입력은 유지됨을 실제 함수 실행으로 확인했다(`logs/issue7445-manifest-observer-3839.log`).
- `cargo fmt --all -- --check`, 문서 링크 검사와 `git diff --check`를 통과했다.

남은 4개 실패의 원본은 이관 목록 밖의 다음 3개다. #7445 문서라는 근거 없이 제외하거나
baseline을 갱신하지 않는다. 이 수치를 전체 PR 검증 완료로 보고하지 않는다.

| 원본 | 현재 실패 축 |
| --- | --- |
| `samples/issue6133/156483831_poster_title_above_offset_float.hwp` | body overflow·off-canvas 각 1개 partition |
| `samples/hwpx/issue1535_coanchored_float_exclusion.hwpx` | text-overlap 1개 partition |
| `samples/issue1510_coanchored_float_tables.hwpx` | text-overlap 1개 partition |

최종 제출 cohort는 위 문서 원장의 경로뿐 아니라 실제 입력 SHA도 대조해 같은 바이트의
임시 경로가 검토 대상으로 다시 들어오지 않도록 선택한다. 원본 이슈의 해결용 직접
Visual Sweep까지 금지하는 변경은 아니다. API를 커밋 전에 다시 조회해 OPEN 85개가
같음을 확인했으며, 원격 comment·push·PR 생성·issue close는 수행하지 않았다.
