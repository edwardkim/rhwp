# PR #1153 완료 보고서 — TAC 표 intra-paragraph vpos-reset 가드 + RowBreak 분할 회귀 보강

## 1. 처리 요약

| 항목 | 결과 |
|---|---|
| PR | #1153 |
| 작성자 | planet6897 |
| 연결 이슈 | #1152 |
| 판단 | 수용 권장 |
| 적용 방식 | 현재 `devel` 기준 전용 review branch cherry-pick |
| 검토 브랜치 | `local/pr1153-review` |
| cherry-pick 결과 | 충돌 없음 |

## 2. 수용 사유

컨트리뷰터가 제시한 문제는 현재 rhwp-studio에서도 사실 확인됐다.

문제의 본질은 `samples/2022년 국립국어원 업무계획.hwp`의 pi=586 호스트 문단에서
두 번째 TAC 표에 매핑되는 `line_seg[1].vertical_pos == 0`을 같은 문단 내부의 page reset
신호로 처리하지 못한 점이다.

PR의 수정은 `typeset_tac_table()` 진입부에서 다음 조건을 모두 만족할 때만 강제 advance한다.

```text
current_items가 비어 있지 않음
ctrl_idx > 0
호스트 문단 text가 비어 있음
line_segs.len() == controls.len()
line_segs[ctrl_idx].vertical_pos == 0
```

조건이 좁고, 문제 샘플의 HWP line_seg contract와 직접 대응한다.

## 3. 검증 결과

현재 `devel` 기준 cherry-pick 후 검증:

```text
cargo check
cargo test --test issue_1152_intra_para_vpos_reset
cargo test --test issue_1145 --test issue_1070_tac_table_post_text_overflow --test issue_1073_nested_table_split --test issue_1086
cargo test --test svg_snapshot
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown --lib
```

결과:

```text
all pass
```

세부:

```text
issue_1152_intra_para_vpos_reset: 1 passed
issue_1070_tac_table_post_text_overflow: 3 passed
issue_1073_nested_table_split: 3 passed
issue_1086: 4 passed
issue_1145: 1 passed
svg_snapshot: 8 passed
wasm32 target lib check: pass
fmt check: pass
```

추가 메인테이너 시각 판정 중 `samples/kps-ai.hwp` 37쪽과 `samples/synam-001.hwp` 5~16쪽의
분할표 페이지네이션 회귀가 발견되어 PR 수용 패치와 함께 보강했다.

추가 수정:

```text
1. kps-ai.hwp page 37
   - RowBreak 표의 intra-row slice가 전체 fragment 높이를 초과하면 해당 row slice를 다음 쪽으로 defer
   - page 37은 rows=0..16까지만 출력, page 38에서 rows=16..32 continuation

2. synam-001.hwp page 5~7
   - 큰 rowspan 블록 내부의 RowBreak 일반 내용 행은 한컴처럼 셀 내부 분할 허용
   - page 5에 6쪽 첫 셀 일부를 end_cut=[2,2]로 출력
   - page 6은 start_cut=[2,2]부터 이어지고 pi=72 본문까지 포함
   - 전체 페이지 수 35쪽으로 한컴 PDF와 일치

3. synam-001.hwp page 13~15
   - 반복 제목행 continuation에서 row-area budget 비교 시 header height 이중 차감 제거
   - page 13에 row 6 첫 조각을 포함하고 page 14에서 continuation
```

추가 검증:

```text
cargo test --test issue_1156_rowbreak_fragment_fit
cargo test --test issue_1073_nested_table_split --test issue_1145 --test issue_713
cargo test --test issue_554 --test issue_1035_alignment --test issue_1100_exam_social_hwpx_header --test issue_nested_table_border
cargo test --test exam_eng_multicolumn --test issue_986 --test issue_676_trailing_empty_para --test issue_702 --test issue_712 --test issue_713
cargo fmt --all -- --check
cargo check
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
all pass
WASM build pass
maintainer visual check pass
```

GitHub PR branch CI:

```text
CodeQL: success
CI: success
Render Diff: success
```

## 4. 남은 위험

페이지네이션 핵심 경로와 RowBreak 분할표 경로가 함께 변경되었다. kps-ai, synam-001, issue_1145,
issue_1073, issue_713, page-count guard 계열 자동 테스트와 WASM/시각 판정을 통과했다.

## 5. 결론

PR #1153은 현재 `devel` 기준으로 수용 가능하다. PR 수용 패치와 메인테이너 보강 패치를 함께 반영한다.

승인 후 절차:

```text
1. 검토/보고 문서 커밋
2. local/pr1153-review -> local/devel 반영
3. local/devel -> devel 반영
4. 필요한 최종 검증 후 origin/devel push
5. PR #1153 / Issue #1152 close 처리
```
