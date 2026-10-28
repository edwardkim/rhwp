---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7180_review.md
last_verified: 2026-09-16
---

# PR #7180 검토

## 판정

**승인** — HWP3 내어쓰기 여백과 줄 상자의 이중 들여쓰기 수정 범위. 검증한 변경 범위에 대한 로컬 검토 판정이며 전체 이슈 해결 판정과 구분한다.
메인터너 보정 `f2e21164c029320d1e9b42a8c8ad1751c69fcb1d`에서 #7118의 실행 회귀·코드 원칙 위반과 #7184의 끝 컷/경계 증거 부족을 해소했다.
최종 판정은 **승인** — 메인터너 보정을 포함한 통합 변경 범위다. [통합 PR #7197](https://github.com/edwardkim/rhwp/pull/7197)의 제출 head `7665912859016c9d46a326e86f000c1a394e4862`에서 원격 CI도 통과했다.
[2회차 결과·제한 사항](../../working/task_m100_6970_open_pr_stage2.md)과 [최종 검증 결과](../assets/pr7118_7187_review_stage2/gate-results.json)를 기준으로 한다.
이 기록은 통합 PR code CI 성공 뒤의 trailing 문서다. 최종 문서 head의 CI·병합 조건을 재확인한 뒤 승인된 merge·후속 처리를 수행한다.

## 검토 대상

| 항목 | 확인값 |
| --- | --- |
| 원 PR | [#7180](https://github.com/edwardkim/rhwp/pull/7180) |
| 제목 | 수정(parser/hwp3): 내어쓰기 문단의 첫 줄 기준 여백을 일반 규칙으로 옮긴다 (#7172) |
| 작성자 / reviewer | planet6897 (기존 기여자) / jangster77 사전 요청 |
| 원 head | `6989418dee9998d7044bfd807d34f4b6f907fadb` |
| source base / 규모 | devel / 6 files, +175/-15, 1 commits |
| 원 PR 상태 | OPEN / draft=False / MERGEABLE / CLEAN |
| source CI | [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/35016722579) 및 전체 rollup 실패/대기 없음; 검토 종료 전 source head 불변 확인 |
| 통합 base | `8d45f242baa1a565357aaa38e9f459595b1e756c` |
| 통합 code head | `f2e21164c029320d1e9b42a8c8ad1751c69fcb1d` (메인터너 보정·검증 증적 포함) |
| branch | `codex/open-pr-review-20260916` |
| 충돌 해결 | 텍스트 충돌 없음. |

원 commit은 `-x`로 누적 적용했고 작성자·메시지·co-author를 보존했다.
source/local SHA와 실행 명령은 [검증 기록](pr_7180_review_impl.md), 공통 제한은 [2회차 보고](../../working/task_m100_6970_open_pr_stage2.md)에 있다.
route: collaborator_external_pr + intake/local_validation/multi_pr_update_branch/visual_fixture_evidence.
#7118의 1,000줄 초과 변경은 별도 코드·실행·시각 검토로 취급했고 admin merge는 하지 않았다.

## 변경·증거 심사

`convert_para_shape_with_layout_contract`가 음수 indent를 첫 줄 여백으로 정규화하고,
`hwp3_para_line_box`가 정규화한 여백에 음수 indent를 다시 더하지 않는다. 양수 indent와 오른쪽
여백은 유지한다. 암호 문서 전용 조건 해제는 한컴 저장본과의 독립 대조로 확인했다.

Git에 이미 보존된 실제 HWP3 입력을 누적 CLI로 HWP5 저장 후, 한컴 HWP5 기준과 **내용이 대응되는
3,699개 문단의 왼쪽/오른쪽 여백·indent 튜플이 전부 일치**했다. 서로 3,699/3,702 문단이므로
레코드 순번을 그대로 zip한 수치는 증거에서 제외하고 텍스트 대응을 사용했다.
focused 1개와 관련 #1692 계약이 통과했다. 양수 indent 반례는 원 PR의 기존 unit 테스트 수정에 있다.

한컴 engine 2020으로 원본·누적 저장본을 각각 출력한 PDF는 모두 264쪽이다.
직접 Visual Sweep 비교의 254·255쪽 참고문헌에서 후속 항목의 정렬과 내어쓰기를 확인했다.
글꼴/공백·하이픈 차이는 문서 전체 완전 일치로 판정하지 않는다. 이 판정은 여백 변환 범위다.

## 공통 조판 원칙

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | 독립 한컴 HWP5 레코드와 위 적용/비적용 조건 |
| 측정·배치 일관성 | 충족 | 정규화한 여백→줄 상자 소비 |
| 분할·이어받기 | 비해당 | 컷 알고리즘 변경 없음 |
| 줄 소속·점유 높이 | 비해당 | 줄/객체 재조판 규칙 변경 없음 |
| 증거 독립성 | 충족 | 기존 원본·한컴 HWP5·새 저장본 PDF 직접 대조 |
| 기준값 변경 | 비해당 | baseline 완화 없음 |
| 주장·검증 범위 | 충족 | 위 저장 필드 범위 및 미해결 축 구분 |

## 증적

최종 공통 검증: focused 66/66, 전체 nextest 9933/9933(기존 skip 51), Clippy 3종, Native Skia 3종 통과.
새 WASM host `--no-opt` 진단 빌드와 Native의 6문서 22쪽 Visual Sweep을 완료했다.
Docker 배포 최적화 경로는 미실행이며 표준 원격 CI 통과로 대체 표기하지 않는다.
[최종 실행 결과](../assets/pr7118_7187_review_stage2/gate-results.json) · [시각 실행](../assets/pr7118_7187_review_stage2/visual-results.json) · [입력 원장](../assets/pr7118_7187_review_stage2/fixture-manifest.json).

![직접 확인한 비교/브라우저 증적](../assets/pr7118_7187_review/hwp3_saved_hancom_p254.png)

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
- HWP3 내어쓰기의 첫 줄 여백을 정규화하고 줄 상자의 이중 들여쓰기를 수정했습니다.
- 내용 대응 3,699개 문단의 여백·indent가 한컴 HWP5와 모두 일치했고, 양쪽 한컴 PDF가 264쪽입니다. 254–255쪽 참고문헌 정렬을 직접 확인했습니다.
- 후보 저장본과 원본의 한컴 PDF를 Visual Sweep make_compares로 직접 비교했습니다. 이 비교에는 자동 후보 수·pixel match·visual_accuracy_proxy를 산출하지 않았으며 임의 수치를 쓰지 않습니다. 이미지의 기본 rhwp 라벨은 한컴이 출력한 후보 저장본을 뜻합니다.
- 수치는 자동 일치율 보조값이며 사람 판정 정확도·완전 시각 일치를 뜻하지 않는다. 내용 픽셀 지표가 높으면 raster가 더 비슷하고 낮으면 위치·형태 차이의 직접 검토가 필요하다.
- 다른 글꼴·공백·하이픈 차이를 전체 문서의 시각 일치로 해석하지 않습니다. 관련 이슈 #7172는 검증한 신고 증상 해결로 병합 후 종료합니다.
- [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크하고 아래 PNG의 merge commit 고정 raw URL을 실제 이미지로 포함한다.

![후속 comment 대표 증적](../assets/pr7118_7187_review/hwp3_saved_hancom_p254.png)

- 로컬 focused 66/66·전체 nextest 9933 pass/51 skip·Native Skia 4112/2/4와 fresh WASM host 진단 경로를 구분해 적는다.
- 최종 head CI 성공 및 devel 증적 존재 확인 뒤 원 PR을 통합 반영으로 close한다. contributor fork branch는 삭제하지 않는다.
