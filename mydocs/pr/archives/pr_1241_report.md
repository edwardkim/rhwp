# PR #1241 처리 보고서

- **작성일**: 2026-06-02
- **PR**: #1241
- **제목**: `미주 연속 인라인 수식 다행 병합 해소 (closes #1239)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1239
- **검증 브랜치**: `local/pr1241-verify`
- **기준 브랜치**: `local/devel` (`47e1151e`)
- **PR head**: `3af90e8d42b9a570cc4c3b6dc389bcd4c8ca43a0`
- **검증 병합 커밋**: `b1c4b459 Merge PR 1241 verification`

## 1. 처리 요약

PR #1241을 현재 `local/devel` 기준 검증 브랜치에 병합했다.

변경 핵심은 미주 수식-only 문단에서 연속 인라인 수식(treat-as-char TAC)이 같은 char position으로 복원되어 같은 줄에 병합되는 문제를, 렌더 줄 배정 단계에서 LINE_SEG 그룹에 맞춰 m:n 분배하는 것이다.

#1225에서 도입된 수식-only TAC 1:1 순서 매핑을 일반화한 성격이며, 편집/커서 공유 함수인 `control_text_positions()`는 변경하지 않고 렌더링 배정만 보정한다.

## 2. 반영 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/layout/paragraph_layout.rs` | `equation_only_tac_line_assignment()` 추가 |
| `src/renderer/layout/paragraph_layout.rs` | empty-runs TAC 수식 렌더 경로를 기존 `index_based_tac`에서 m:n 분배 매핑으로 교체 |
| `mydocs/plans/task_m100_1239*.md` | PR 작성자의 계획 문서 |
| `mydocs/working/task_m100_1239_stage*.md` | PR 작성자의 단계 기록 |
| `mydocs/tech/endnote_inline_eq_line_1239.md` | 미주 인라인 수식 줄 배정 조사 문서 |
| `mydocs/report/task_m100_1239_report.md` | PR 작성자의 완료 보고서 |

## 3. 자동 검증

| 항목 | 결과 |
|---|---|
| `git diff --check HEAD` | 통과 |
| `cargo fmt --all --check` | 통과 |
| `cargo test --test issue_table_vpos_01_page5_cell_hit_test` | 통과, 14 passed |
| `cargo test --test issue_1219_equation_line_hangul_advance` | 통과, 1 passed |
| `cargo test --test issue_1139_inline_picture_duplicate` | 통과, 41 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 4 passed |
| `cargo test --lib` | 통과, 1524 passed / 6 ignored |
| `cargo test --tests` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| `rhwp-studio npm run build` | 통과 |

`cargo test --tests`에서는 수식-only 셀 가드(`#1225` 계열), 미주/수식/페이지네이션 회귀 테스트, `svg_snapshot`이 모두 통과했다.

## 4. 시각 판정 자료

대상 샘플:

```text
samples/3-11월_실전_통합_2022.hwpx
```

한컴 페이지 기준 13쪽을 CLI 0-index `-p 12`로 내보냈다.

| 항목 | SVG | 판정 |
|---|---|---|
| 문20 S= 블록 | `output/poc/pr1241-equation-multiline/3-11월_실전_통합_2022_013.svg` | 통과 |
| 문20 S= 블록 debug/grid | `output/poc/pr1241-equation-multiline-debug/3-11월_실전_통합_2022_013.svg` | 통과 |

#1225 회귀 가드:

| 항목 | SVG | 판정 |
|---|---|---|
| `table-vpos-01.hwp` 5쪽 | `output/poc/pr1241-equation-multiline-guard/table-vpos-01_005.svg` | 통과 |
| `table-vpos-01.hwp` 5쪽 debug/grid | `output/poc/pr1241-equation-multiline-guard-debug/table-vpos-01_005.svg` | 통과 |

## 5. 관찰 사항

- PR #1241 base SHA와 현재 `local/devel`이 동일해 뒤처진 PR은 아니었다.
- 병합은 충돌 없이 성공했다.
- 자동 테스트와 빌드는 모두 통과했다.
- 이번 변경은 일반 텍스트 문단이 아니라 `runs.is_empty()`인 수식-only 줄 배정에만 걸린다.
- 메인테이너 시각 판정에서 PR #1241 대상 수식 다행 분리와 #1225 회귀 가드가 모두 통과했다.
- 시각 판정 중 `output/poc/tbox-v-flow-01/hwpx/tbox-v-flow-01.svg`에서 HWP는 글상자 세로쓰기가 구현되어 있으나 HWPX는 미구현인 별도 이슈를 확인했고, #1249로 등록했다.

## 6. 현재 판정

자동 검증, 빌드, 메인테이너 시각 판정을 모두 통과했다.

```text
2026-06-02 통과
```

## 7. 다음 절차

1. HWPX 글상자 세로쓰기 미구현 이슈 #1249 등록 완료
2. `local/devel`에 병합
3. 원격 `devel` push
4. PR #1241 및 이슈 #1239 종료 상태 확인
