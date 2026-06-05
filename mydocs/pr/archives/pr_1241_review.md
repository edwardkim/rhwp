# PR #1241 검토 — 미주 연속 인라인 수식 다행 병합 해소

- **작성일**: 2026-06-02
- **PR**: #1241 (OPEN)
- **제목**: `미주 연속 인라인 수식 다행 병합 해소 (closes #1239)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1239
- **base/head**: `devel` ← `feature/issue-1239-equation-multiline-merge`
- **Head SHA**: `3af90e8d42b9a570cc4c3b6dc389bcd4c8ca43a0`
- **PR 기준 base SHA**: `47e1151ef60e8ea44615c7dffac550e32efce86d`
- **현재 local/devel**: `47e1151ef60e8ea44615c7dffac550e32efce86d`
- **규모**: 7 files, +367 / -13, 1 commit
- **mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1241은 #1239에서 보고된 `samples/3-11월_실전_통합_2022.hwpx` 13쪽 문20 풀이의 `S = ...` 다행 수식 병합 문제를 수정한다.

한컴/PDF에서는 미주 풀이의 S 블록이 여러 줄로 분리되지만, rhwp에서는 텍스트가 없는 연속 인라인 수식(treat-as-char, U+FFFC)이 같은 char position으로 복원되어 두 수식이 같은 줄에 배정되고 직전 줄은 비는 문제가 있었다.

컨트리뷰터의 수정 방향은 다음과 같다.

```text
모든 줄이 빈 runs이고 char_start가 비구분/degenerate인 수식-only 문단에서
같은 char_start 그룹의 줄들에 같은 position의 연속 TAC를 순서대로 분배한다.
```

이는 #1225의 `줄수 == tac수` 1:1 특수 처리를 m:n 분배로 일반화하는 성격이다.

## 2. 주요 변경 범위

| 영역 | 변경 |
|---|---|
| `src/renderer/layout/paragraph_layout.rs` | `equation_only_tac_line_assignment()` 추가 |
| `src/renderer/layout/paragraph_layout.rs` | empty-runs TAC 수식 렌더 경로에서 기존 `index_based_tac`를 m:n 분배 매핑으로 교체 |
| `mydocs/plans/task_m100_1239*.md` | 수행/구현 계획 문서 추가 |
| `mydocs/tech/endnote_inline_eq_line_1239.md` | 원인 조사 문서 추가 |
| `mydocs/working/task_m100_1239_stage*.md` | 단계별 작업 기록 추가 |
| `mydocs/report/task_m100_1239_report.md` | 최종 보고서 추가 |

## 3. 타당한 부분

### 3.1 문제 원인과 수정 위치가 맞다

증상은 수식 파서 내부의 다행 구조 문제가 아니라, 한 문단 안에 텍스트 없이 연속 배치된 인라인 수식들이 어느 LINE_SEG 줄에 들어갈지 정하는 배정 문제다.

PR은 편집/커서 의미에 영향을 주는 `control_text_positions()`는 유지하고, 렌더링 단계의 줄 배정만 보정한다. 이 방향은 공유 모델을 흔들지 않아 적절하다.

### 3.2 #1225의 자연스러운 일반화

기존 #1225는 수식-only 셀에서 `lines.len() == tac_offsets_px.len()`일 때만 1:1 순서 매핑을 적용했다.

이번 PR은 한 줄에 TAC가 2개 이상 들어갈 수 있는 경우를 포함해, 같은 char_start 그룹 안에서 TAC를 줄 수에 맞게 분배한다.

문20 S 블록처럼 첫 줄에 `S`와 첫 수식이 함께 들어가고 이후 줄이 이어지는 패턴에는 1:1보다 m:n 매핑이 더 맞다.

### 3.3 게이트가 일반 텍스트 문단을 피한다

새 helper는 다음 조건이 모두 맞는 경우에만 동작한다.

```text
1. 여러 줄
2. TAC 존재
3. 모든 composed line의 runs가 비어 있음
4. char_start가 뒤 줄에서 같거나 감소하는 degenerate 구조
```

따라서 일반 텍스트+수식 문단, 텍스트가 포함된 미주 문단, 일반 본문 수식은 기존 char-position 기반 배정을 유지한다.

## 4. 확인 필요 사항

### 4.1 수식-only 빈 runs 경로는 공통 렌더 경로다

변경은 미주만이 아니라 `paragraph_layout.rs`의 empty-runs TAC 렌더 경로 전체에 걸린다.

따라서 #1239 대상 미주 S 블록뿐 아니라 다음 가드도 확인해야 한다.

```text
- #1221 / PR #1225 z-표 수식-only 셀 행 분리 유지
- issue_table_vpos_01_page5_cell_hit_test
- issue_1219_equation_line_hangul_advance
- svg_snapshot
```

### 4.2 m:n 분배의 잔여 리스크

같은 char_start의 TAC 수가 같은 char_start의 줄 수보다 많으면 나머지를 마지막 줄에 모으는 정책이다.

이 정책은 문20 첫 줄처럼 한 줄에 여러 TAC가 필요한 경우에 합리적이다. 다만 다른 수식-only 문단에서 한컴이 다른 분배를 기대하는 사례가 있을 수 있으므로 시각 판정과 기존 수식-only 표 회귀가 필요하다.

### 4.3 이슈 본문과 PR 진단의 차이

이슈 #1239 초기 본문은 수식 레이아웃 서브시스템 문제로 추정했지만, PR 조사 결과는 인라인 수식 줄 배정 문제로 확정했다.

이 차이는 문제되지 않는다. 오히려 구현 단계에서 root cause가 더 좁혀진 것으로 보면 된다.

## 5. 권장 검증

현재 PR base와 `local/devel`이 동일하므로 검증 브랜치에서 병합 후 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_table_vpos_01_page5_cell_hit_test
cargo test --test issue_1219_equation_line_hangul_advance
cargo test --test issue_1139_inline_picture_duplicate
cargo test --test issue_1082_endnote_multicolumn_drift
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

메인테이너 시각 판정용 SVG:

```text
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 12 -o output/poc/pr1241-equation-multiline
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 12 --debug-overlay --show-grid=3mm -o output/poc/pr1241-equation-multiline-debug
```

필요하면 #1225 회귀 확인용으로 다음도 함께 산출한다.

```text
target/debug/rhwp export-svg samples/table-vpos-01.hwp -p 4 -o output/poc/pr1241-equation-multiline-guard
```

## 6. 권장 처리

권장안: **수용 후보로 진행한다. 단, 현재 `local/devel` 기준 검증 브랜치에서 병합하고, 수식-only 줄 배정 회귀 테스트와 메인테이너 시각 판정을 게이트로 둔다.**

변경은 문제 원인에 잘 맞고, 편집 모델이 아니라 렌더 줄 배정에만 적용되어 방향이 좋다.
다만 #1225의 보수적 1:1 조건을 m:n으로 일반화하므로, 수식-only 셀/미주/표 계열 회귀 확인이 필수다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1241-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1241을 병합 시뮬레이션
3. 수식-only 줄 배정 중심 테스트와 전체 테스트 실행
4. WASM/Studio 빌드
5. 13쪽 문20 SVG 및 웹 canvas 시각 판정 후 local/devel 반영
```
