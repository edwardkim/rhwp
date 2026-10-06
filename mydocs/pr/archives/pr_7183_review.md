---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7183_review.md
last_verified: 2026-09-16
---

# PR #7183 검토

## 판정

**승인** — 분할 표의 그림만 있는 문단 복원 범위. 검증한 변경 범위에 대한 로컬 검토 판정이며 전체 이슈 해결 판정과 구분한다.
메인터너 보정 `f2e21164c029320d1e9b42a8c8ad1751c69fcb1d`에서 #7118의 실행 회귀·코드 원칙 위반과 #7184의 끝 컷/경계 증거 부족을 해소했다.
최종 판정은 **승인** — 메인터너 보정을 포함한 통합 변경 범위다. [통합 PR #7197](https://github.com/edwardkim/rhwp/pull/7197)의 제출 head `7665912859016c9d46a326e86f000c1a394e4862`에서 원격 CI도 통과했다.
[2회차 결과·제한 사항](../../working/task_m100_6970_open_pr_stage2.md)과 [최종 검증 결과](../assets/pr7118_7187_review_stage2/gate-results.json)를 기준으로 한다.
이 기록은 통합 PR code CI 성공 뒤의 trailing 문서다. 최종 문서 head의 CI·병합 조건을 재확인한 뒤 승인된 merge·후속 처리를 수행한다.

## 검토 대상

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#7183](https://github.com/edwardkim/rhwp/pull/7183) |
| 제목 | 수정(renderer): 분할 표 칸에서 글자 없이 그림만 든 문단이 어느 조각에도 배치되지 않던 것을 고친다 (#7182) |
| 작성자 / reviewer | davindev (기존 기여자) / jangster77 사전 요청 |
| 원 head | `fb50aa6cfaa054ff845bdf211b574a6cf99d98b6` |
| source base / 규모 | devel / 3 files, +116/-10, 1 commits |
| 원 PR 상태 | OPEN / draft=False / MERGEABLE / CLEAN |
| source CI | [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/35055413849) 및 전체 rollup 실패/대기 없음; 검토 종료 전 source head 불변 확인 |
| 통합 base | `8d45f242baa1a565357aaa38e9f459595b1e756c` |
| 통합 code head | `f2e21164c029320d1e9b42a8c8ad1751c69fcb1d` (메인터너 보정·검증 증적 포함) |
| branch | `codex/open-pr-review-20260916` |
| 충돌 해결 | 텍스트 충돌 없음. |

원 commit은 `-x`로 누적 적용했고 작성자·메시지·co-author를 보존했다.
source/local SHA와 실행 명령은 [검증 기록](pr_7183_review_impl.md), 공통 제한은 [2회차 보고](../../working/task_m100_6970_open_pr_stage2.md)에 있다.
route: collaborator_external_pr + intake/local_validation/multi_pr_update_branch/visual_fixture_evidence.
#7118의 1,000줄 초과 변경은 별도 코드·실행·시각 검토로 취급했고 admin merge는 하지 않았다.

## 변경·증거 심사

`table_partial.rs`에서 row 범위가 정해진 **uncut** fragment의 control-only 문단 구제에 Picture를
포함한다. cut fragment에는 구제를 확장하지 않아 기존 유닛 소유를 보존한다. 세로 가운데 정렬의
콘텐츠 높이에 TAC 그림 높이를 반영하며, 음수 offset의 상단 이탈 보정은 Center에만 확장한다.
Top 경로의 완전 이탈 조건을 그대로 두는 것을 확인했다.

focused의 `issue_7182_rowbreak_cell_picture_only_paragraphs_render_inside_their_cells` 및 기존
#4059 Square 계약이 통과했다. 입력 4쪽·그림 11개와 각 이미지가 소유 셀 내부에 들어가는 실제 bbox를
검사한다. 한컴 PDF는 MCP 2020, 12.0.0.4605로 생성한 4쪽이다.
Native와 fresh WASM 1–4쪽을 직접 비교했고, 대표 3쪽에서 사진 9개와 셀 소유가 유지된다.
사진 복원 범위에 차단 결함은 없으며 글꼴·줄끝·표 하단의 기존 차이를 완전 fidelity로 포장하지 않는다.

합성 입력의 임의 다중 그림/여러 빈 문단 전체를 전수 검증한 것은 아니다. 이 PR은 source 컷이 없는
RowBreak fragment의 그림 복원 범위로 수용한다. CellBreak의 소유권 재설계로 확장해 해석하지 않는다.

## 공통 조판 원칙

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | 실제 그림-only fixture와 MCP 정본 |
| 측정·배치 일관성 | 충족 | control-only 높이/valign 및 소유 셀 bbox |
| 분할·이어받기 | 충족 | uncut row owner만 구제; cut_units 있는 경로 유지 |
| 줄 소속·점유 높이 | 충족 | 11개 이미지 소유 셀 내부 검증 |
| 증거 독립성 | 충족 | 동일 입력의 Native/WASM/한컴 4쪽 |
| 기준값 변경 | 비해당 | 없음 |
| 주장·검증 범위 | 충족 | 사진 복원에 한정, 완전 fidelity 아님 |

## 증적

최종 공통 검증: focused 66/66, 전체 nextest 9933/9933(기존 skip 51), Clippy 3종, Native Skia 3종 통과.
새 WASM host `--no-opt` 진단 빌드와 Native의 6문서 22쪽 Visual Sweep을 완료했다.
Docker 배포 최적화 경로는 미실행이며 표준 원격 CI 통과로 대체 표기하지 않는다.
[최종 실행 결과](../assets/pr7118_7187_review_stage2/gate-results.json) · [시각 실행](../assets/pr7118_7187_review_stage2/visual-results.json) · [입력 원장](../assets/pr7118_7187_review_stage2/fixture-manifest.json).

![직접 확인한 비교/브라우저 증적](../assets/pr7118_7187_review_stage2/visual/wasm/pr7183/compare_003.png)

시각 검토 정본: [Visual Sweep](../../manual/verification/visual_sweep_guide.md).
Native/WASM의 PNG 라벨과 본문을 구분해 직접 열었다. HWP3 저장 비교 이미지는 Visual Sweep의
`make_compares`로 **후보를 한컴이 출력한 PDF(왼쪽)**와 **원본 한컴 PDF(오른쪽)**를 합친 것이다.
그 이미지의 기본 `rhwp` 라벨은 Native renderer 출력을 뜻하지 않는다.
1회차(보정 전)의 [입력/PDF 해시 원장](../assets/pr7118_7187_review/fixture-manifest.json), [시각 지표](../assets/pr7118_7187_review/visual-metrics.json),
[focused 이력](../assets/pr7118_7187_review/focused.log)를 함께 보존했다. 낮은 pixel/ink proxy는 완전 일치로 해석하지 않는다.

직접 사용한 기존 HWP/HWPX는 기존 Git 경로를 재사용했다. 1회차의 한컴 PDF와 누적 저장 HWP는 `788ab292b`에 이미 포함돼 있다.
2회차는 기존 Git 입력·PDF를 재사용하고 새 비교 이미지와 로그를 보존했다. 아직 merge하지 않았으므로 source PR/issue는 닫지 않는다.

## 통합 code CI 완료

- code candidate `7665912859016c9d46a326e86f000c1a394e4862`, base `8d45f242baa1a565357aaa38e9f459595b1e756c`.
- [CI Full](https://github.com/edwardkim/rhwp/actions/runs/35071582466), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35071582467), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35071582672), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/35071582435), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/35071582468) 모두 성공.
- 원 PR head 불변·OPEN과 개별 CI 실패/대기 없음을 재확인했다. 원본 #7118의 충돌은 통합 브랜치에서 해결되어 있으며 직접 병합하지 않는다.
- 후속 문서만 single-parent commit으로 추가한다. 실제 merge SHA·trailing CI·duration 결과·종료 상태는 확인 후 GitHub 후속 comment에 확정값을 남긴다.

## Merge 후 contributor PR comment 계획

- [통합 PR #7197](https://github.com/edwardkim/rhwp/pull/7197) 반영 사실과 감사, 실제 merge SHA, code CI 및 최종 trailing CI URL을 적는다. 보정 없는 원 PR 자체를 승인했다는 표현은 쓰지 않는다.
- RowBreak 표의 그림 전용 문단을 소유 조각에 배치하고, TAC 그림 높이와 가운데 정렬의 음수 offset을 보정했습니다.
- 4쪽의 그림 11개와 실제 셀 내부 bbox를 검사했습니다. Native/fresh WASM 1–4쪽과 한컴 PDF를 직접 비교했고 3쪽의 그림 9개가 소유 셀 안에 배치됩니다.
- Visual Sweep 4쪽, 후보 0/4, 평균 pixel match 86.68677%, 평균 visual_accuracy_proxy 40.69627%.
- 수치는 자동 일치율 보조값이며 사람 판정 정확도·완전 시각 일치를 뜻하지 않는다. 내용 픽셀 지표가 높으면 raster가 더 비슷하고 낮으면 위치·형태 차이의 직접 검토가 필요하다.
- 그림 복원 범위의 해결이며 기존 글꼴·줄끝·표 하단 차이와 임의의 CellBreak 소유권 재설계를 포함하지 않습니다. 관련 이슈 #7182는 검증한 신고 증상 해결로 병합 후 종료합니다.
- [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크하고 아래 PNG의 merge commit 고정 raw URL을 실제 이미지로 포함한다.

![후속 comment 대표 증적](../assets/pr7118_7187_review_stage2/visual/wasm/pr7183/compare_003.png)

- 로컬 focused 66/66·전체 nextest 9933 pass/51 skip·Native Skia 4112/2/4와 fresh WASM host 진단 경로를 구분해 적는다.
- 최종 head CI 성공 및 devel 증적 존재 확인 뒤 원 PR을 통합 반영으로 close한다. contributor fork branch는 삭제하지 않는다.
