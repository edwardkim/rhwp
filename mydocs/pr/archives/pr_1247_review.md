# PR #1247 검토 — 미주 between-notes margin min-gap

- **작성일**: 2026-06-02
- **PR**: #1247 (OPEN)
- **제목**: `Task #1246: 미주 between-notes margin min-gap (HeightCursor) — closes #1238, #1246`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1238, #1246
- **base/head**: `devel` ← `feature/issue-1246-endnote-vpos-anchor`
- **Head SHA**: `8e5493a096f3f28213c9cd69dbe1fd09237647b5`
- **PR 기준 base SHA**: `704f62d282e9f25569cc910cf08755c2b00343f3`
- **현재 local/devel**: `2601fc11`
- **규모**: 18 files, +1042 / -1, 1 commit
- **GitHub mergeable**: `CONFLICTING`
- **CI**: PR head 기준 `Build & Test`, `Render Diff`, `CodeQL` 통과. `WASM Build`는 skip.
- **PR 댓글**: 없음

## 1. PR 요약

PR #1247은 미주(답안) 영역에서 다줄 풀이로 끝나는 문제 다음 제목이 직전 줄에 붙는 문제를 처리한다.

컨트리뷰터의 분석은 다음과 같다.

```text
between-notes는 단순 가산이 아니라 최소 간격(min-gap / max) 모델이다.
stored vpos gap이 거의 0인 새 미주 제목에서만 HeightCursor가 직전 between-notes line_spacing만큼 끌어올린다.
```

기존 #1238의 render-only clamp 접근은 `#1209` safe-vpos-backtrack 및 `#1189` 계열과 충돌해 폐기했고, 이번 PR은 `HeightCursor.vpos_adjust()`에 좁은 min-gap 케이스를 추가하는 방향이다.

## 2. 주요 변경 범위

| 영역 | 변경 |
|---|---|
| `src/renderer/height_cursor.rs` | `endnote_between_notes_hu` 필드 추가, compact endnote min-gap 보정 추가, 단위 테스트 3개 추가 |
| `src/renderer/typeset.rs` | 섹션 미주 between-notes HU를 `PaginationResult`로 전달 |
| `src/renderer/pagination.rs` | `endnote_between_notes_hu` 필드 추가 |
| `src/renderer/pagination/engine.rs` | 기본 `PaginationResult` 초기화 보강 |
| `src/document_core/queries/rendering.rs` | layout engine에 섹션 미주 between-notes 전달 |
| `src/renderer/layout.rs` | 미주 흐름 column의 `HeightCursor`에 between-notes 주입 |
| `mydocs/*task_m100_1238*`, `mydocs/*task_m100_1246*` | PR 작성자의 계획/조사/보고 문서 |

## 3. 타당한 부분

### 3.1 #1238의 실패 접근을 폐기하고 더 좁은 위치로 이동했다

#1238 쪽 코멘트와 PR 문서에서 render-only clamp가 `issue_1189_2022_nov_pages10_12`, `issue_1209_2022_sep_page10_question12`를 깨뜨렸다는 사실을 명시하고 있다.

이번 PR은 문단 레이아웃 전체 진입부가 아니라 이미 compact endnote vpos 특례를 모아둔 `HeightCursor.vpos_adjust()` 내부에 min-gap 케이스를 추가한다. 수정 위치는 더 적절하다.

### 3.2 조건 게이트가 비교적 좁다

추가 보정은 다음 조건을 모두 요구한다.

```text
endnote_between_notes_hu > 0
compact_endnote_question_title
!vpos_rewind
prev 문단이 다줄
stored gap px가 [-0.5, 4.0)
```

단일줄 prev, 의도적 소-gap, backtrack/rewind는 제외하려는 의도가 분명하다.

### 3.3 기존 PR #1240과 의미가 직교한다

PR #1240은 같은 미주 내부의 다줄 문단 trailing 줄간격을 보존하는 문제였다.

PR #1247은 다른 미주 사이의 새 제목 min-gap 문제다. 개념상 병존해야 하며, 실제 merge-tree 충돌도 같은 위치에 메서드를 삽입해서 생긴 텍스트 충돌이다.

## 4. 주의 사항

### 4.1 현재 `devel`과 충돌한다

PR #1247의 base는 `704f62d2`이고, 현재 `local/devel`은 #1240/#1241 반영 후 `2601fc11`이다.

GitHub는 `CONFLICTING`으로 보고한다. 비파괴 `merge-tree` 확인 결과 핵심 충돌은 `src/renderer/layout.rs` 한 곳이다.

충돌 내용:

```text
our   : endnote_para_has_same_endnote_successor()   // PR #1240
their : set_endnote_between_notes_hu()              // PR #1247
```

해결 방향은 둘 중 하나를 버리는 것이 아니라 두 메서드를 모두 유지하는 것이다. `LayoutEngine` 필드 초기화에도 `endnote_between_notes_hu`를 추가해야 한다.

### 4.2 단순 `local/devel..pr/1247` diff는 착시가 있다

PR base가 뒤처져 있어서 단순 양끝 diff에서는 #1240/#1241 문서가 삭제되는 것처럼 보인다.

실제 PR 자체의 변경은 18 files, +1042 / -1이며, 3-way merge 관점에서는 새 문서 추가와 코드 보강으로 봐야 한다.

### 4.3 미주 좌표 경로는 고위험 영역이다

`HeightCursor`, `typeset`, `pagination`, `layout`을 모두 지나는 변경이다. 기존 미주 회귀군이 두껍고, 최근 #1209/#1236/#1241과 같은 영역을 연속으로 만지고 있다.

따라서 PR CI 통과만으로는 부족하고 현재 `local/devel` 기준 재검증이 필요하다.

## 5. 권장 검증

현재 `local/devel`에서 검증 브랜치를 만들고 PR #1247을 병합/충돌 해소한 뒤 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo test --test issue_1082_endnote_multicolumn_drift
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

미주 중심 시각 판정 자료:

```text
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 13 -o output/poc/pr1247-endnote-min-gap/page14
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 13 --debug-overlay --show-grid=3mm -o output/poc/pr1247-endnote-min-gap/page14-debug
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 9 -o output/poc/pr1247-endnote-min-gap/page10
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 16 -o output/poc/pr1247-endnote-min-gap/page17
```

회귀 가드 SVG:

```text
target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 9 -o output/poc/pr1247-endnote-min-gap/guard-sep-page10
target/debug/rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 10 -o output/poc/pr1247-endnote-min-gap/guard-oct-page11
```

## 6. 권장 처리

권장안: **수용 후보로 진행한다. 단, 현재 `local/devel` 기준 검증 브랜치에서 충돌을 수동 해소하고, 미주 회귀군 전체 테스트와 메인테이너 시각 판정을 게이트로 둔다.**

구현 방향은 #1238에서 반증된 render-clamp 접근보다 좁고, #1240과도 의미상 병존 가능하다.

다만 현재 PR은 base가 뒤처져 충돌 상태이며, 미주 좌표 핵심 경로를 건드리므로 직접 시각 판정 없이 push하면 안 된다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1247-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1247을 병합하고 `src/renderer/layout.rs` 충돌을 수동 해소
3. 미주 회귀 테스트와 전체 테스트 실행
4. WASM/Studio 빌드
5. 3-11월 page14 문22 및 3-09/3-10 회귀 가드 SVG 산출
6. 메인테이너 시각 판정 후 local/devel 반영
```
