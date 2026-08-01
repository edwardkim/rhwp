# PR 초안 (파킹) — 열린 PR 볼륨 제한(10건)으로 미개설

- 브랜치: `kevin9327/rhwp` `task/3658-endcut-tail` (fork 푸시 완료, origin/devel f80b910aa 리베이스 기준)
- base: `edwardkim/rhwp` `devel`
- 개설 시점에 볼륨이 빠지면 아래 제목·본문 그대로 `gh pr create` 하면 된다.

## 제목

fix(layout): #3658 중첩 표 분할 종료 조각의 꼬리 문단 유실 수정

## 본문

---

## 문제

다문단 셀의 중첩 표가 RowBreak 연속 페이지에서 마지막 꼬리 문단을 남긴 채 종료됩니다. #3595/PR #3642 가 행 범위 유도를 고친 뒤에도, 컷이 빈 `end_cut` 으로 완료를 선언하면 continuation 조각이 생성되지 않아 남은 꼬리가 **어느 쪽에도 렌더되지 않고 영구 유실**됩니다.

재현: `samples/task2097/75544_pii_bunseok.hwpx` p60 — `○○금융회사 대표이사 △△△` 서명 문단 소실, `20 . . .` 날짜 줄은 셀 하단에 클립.

## 근인

컷 종료 판정 자체는 옳습니다(마지막 유닛까지 포함한 컷이므로 continuation 이 없어야 맞음). 결함은 **종료 조각의 렌더 회계**에 있습니다.

1. 종료 조각의 `offset_within_start` 가 0.0 으로 고정되어 렌더가 start_row 를 처음부터 재적층 — 재적층 높이가 행 그리드보다 커지고(75544 pi=527: 766px vs 747px) 초과분이 셀 하단 밖으로 밀립니다.
2. 밀린 꼬리 줄이 "셀 하단 초과 줄 드롭"(다음 쪽 소속 줄 제외용)에 걸려 잘립니다. 종료 조각은 이어받을 다음 쪽이 없으므로 영구 유실입니다.

## 구현

`NestedTableSplit` 에 `terminal`(이 조각이 셀 콘텐츠의 마지막 조각) 플래그를 도입하고, 종료 조각은:

1. **기표시 밴드 오프셋** — 이전 쪽에 이미 보인 start_row 상단 밴드만큼 `offset_within_start` 를 부여해 잔여 콘텐츠가 행 그리드 안에 들어가게 합니다(중복 렌더 제거). 표시 상자는 포함 행 그리드 높이를 유지해 셀 클립이 꼬리를 자르지 않게 합니다.
2. **셀 하단 초과 줄 드롭 면제** — continuation 이 없으므로 드롭하지 않습니다. 재적층 드리프트로 수 px 넘치는 마지막 줄은 오버플로를 감수하고 렌더합니다.

per-중첩행 컷 경로(`table_partial.rs`)도 `end_cut=[]` 이면 종료 조각으로 표시합니다. 1행 표 경로는 종전 규약 그대로입니다.

## 검증 (red→green 실행)

- **red**: 수정 전 소스에서 `nested_table_tail_paragraph_is_rendered` 실패 실측 — `표의 마지막 문단이 렌더되지 않았다: "○○금융회사대표이사△△△"`. (종전 `#[ignore]` 회귀를 활성화)
- **green**: 수정 후 `issue_3595_nested_split_row_identity` 2/2 통과.
- 무회귀: 같은 경로의 인접 회귀 `issue_1073_nested_table_split`(3/3)·`issue_2007_nested_cell_pagination`(1/1) 통과. origin/devel(f80b910aa) 리베이스 후 세 타깃 재실행 green. rustfmt(변경 3파일) clean, `cargo clippy --lib` 경고 0. 프로파일 release-test.
- 처리 문서: `mydocs/report/task_m100_3658/`.

## 시각 전/후 (p60)

![p60 전/후 비교](https://raw.githubusercontent.com/kevin9327/rhwp/task/3658-endcut-tail/mydocs/report/assets/task3658_p60_before_vs_after.png)

- 수정 전: 상자가 날짜 줄에서 클립되고 서명 문단이 문서 전체에 없음.
- 수정 후: 날짜 줄·서명 문단이 상자 안에 렌더되고 상자가 정상 종결. 같은 페이지의 나머지 콘텐츠는 변화 없음.

closes #3658

🤖 Generated with [Claude Code](https://claude.com/claude-code)
