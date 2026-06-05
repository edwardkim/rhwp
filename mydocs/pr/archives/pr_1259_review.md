# PR #1259 검토 — 미주 답안 제목 between-notes 7mm 갭 복원

- **작성일**: 2026-06-03
- **PR**: #1259
- **제목**: `Task #1256: 미주 답안 제목 between-notes(7mm) 갭 복원`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1256
- **base/head**: `devel` <- `task1256-between-notes-gap`
- **PR head**: `9b8d1d90c3467d7158fabe42ec97fd65f8bfda76`
- **PR 기준 base**: `af7476f3`
- **현재 devel**: `5137c07f`
- **검증 브랜치**: `local/pr1259-current`
- **검증 커밋**: `04e9165c`
- **규모**: 9 files, +553 / -7
- **GitHub mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1259는 미주 답안 영역의 `문N)` 제목 위 간격이 한컴보다 약 7mm 좁게 렌더링되는 문제를 수정한다.

핵심 주장은 다음이다.

- 문서의 `미주 사이` 값은 7mm, 즉 1984 HU다.
- typeset 단계는 직전 미주 separator line segment의 `line_spacing`에 이 값을 주입한다.
- 하지만 `height_cursor::vpos_adjust`의 `safe_vpos_backtrack` 계열 보정이 `y_offset`에 이미 포함된 7mm를 `end_y`로 되감아 덮어쓴다.
- 따라서 `문N)` 제목이 약 20.4px 위로 당겨진다.
- 단일 줄 prev + 주입된 between-notes + 미주 제목 + 실제 cram 상황에서만 `y_offset`을 유지하고, 후속 absolute-vpos desync 방지를 위해 active vpos base를 함께 이동한다.

## 2. 주요 변경 범위

| 파일 | 변경 |
|---|---|
| `src/renderer/height_cursor.rs` | 단일 줄 prev 미주 제목 경계에서 injected between-notes cram을 복원하고 vpos base를 이동 |
| `src/renderer/height_cursor.rs` | 신규 단위 테스트 2건 추가 |
| `tests/issue_1139_inline_picture_duplicate.rs` | 기존 #1209 테스트를 #1256 기준으로 갱신, 문12 y 범위 보정 |
| `mydocs/plans/task_m100_1256*.md` | 계획/구현 문서 추가 |
| `mydocs/working/task_m100_1256_stage*.md` | 단계별 분석/검증 기록 추가 |
| `mydocs/report/task_m100_1256_report.md` | 최종 보고서 추가 |

## 3. 타당한 부분

### 3.1 보정 조건이 충분히 좁다

수정 분기는 다음 조건을 동시에 요구한다.

```text
endnote_between_notes_hu > 0
seg.line_spacing >= endnote_between_notes_hu
compact_endnote_question_title
!vpos_rewind
!prev_is_multiline
stored_gap_px < -0.5
```

즉 일반 본문, 미주 제목이 아닌 문단, 다줄 prev의 기존 #1246 rescue 경로, 자연 trailing, 실제 cram이 아닌 케이스를 피한다. `height_cursor`가 민감한 영역인 점을 고려하면 안전하게 좁힌 편이다.

### 3.2 base-shift를 동반한다

단순히 `return y_offset`만 하면 다음 미주 항목이 stored vpos 기준으로 다시 되감기거나 겹칠 수 있다. PR은 `restored_hu`만큼 `vpos_page_base` 또는 `vpos_lazy_base`를 이동해 후속 항목의 기준도 함께 맞춘다.

이 접근은 기존 #1246/#1082 계열에서 반복적으로 문제가 되었던 "rendered y와 absolute-vpos base의 desync"를 피하는 방향이다.

### 3.3 회귀 테스트 갱신 사유가 명확하다

`issue_1209_2022_sep_page10_question12_uses_safe_vpos_backtrack`가 기존 cram 위치를 정답처럼 단언하고 있었고, PR은 한컴 PDF 기준으로 7mm 갭이 있는 위치가 맞다고 정정한다.

테스트 이름을 #1256 의미에 맞게 바꾸고, 수식 중앙 정렬/꼬리 간격 등 #1209의 원래 핵심 단언은 유지했다.

## 4. 위험 및 주의 사항

### 4.1 `height_cursor`는 페이지네이션 회귀 위험이 높은 파일이다

이번 변경은 매우 국소적이지만, 미주 absolute-vpos, page/lazy path, 다단/단 경계가 얽힌 곳이라 시각 판정이 필요하다.

특히 PR 본문이 언급한 `samples/3-09월_교육_통합_2022.hwp` / `.hwpx`의 9, 10, 13, 14, 18쪽은 메인테이너가 직접 확인하는 것이 좋다.

### 4.2 #1257로 분리한 잔여 케이스가 있다

PR 본문과 보고서는 컬럼 하단 제목, 일부 mid-column 부분 갭을 범위 밖으로 남겼다. 이는 본 PR의 결함이라기보다 과확장 회귀를 피하기 위한 범위 제한으로 보인다.

수용 시 #1257 또는 후속 이슈에서 잔여 미주 제목 간격을 별도 추적해야 한다.

### 4.3 PR base가 현재 devel보다 뒤처져 있다

PR base는 `af7476f3`이고 현재 `devel`은 `5137c07f`다. 검증 브랜치에서 현재 devel 위에 PR 커밋을 체리픽했으며 충돌은 없었다.

## 5. 자동 검증 결과

| 항목 | 명령 | 결과 |
|---|---|---|
| whitespace | `git diff --check devel..HEAD` | 통과 |
| Rust fmt | `cargo fmt --all --check` | 통과 |
| 신규 height_cursor 테스트 | `cargo test compact_endnote_between_notes --lib` | 통과, 2 passed |
| #1256 통합 테스트 | `cargo test issue_1256_2022_sep_page10_question12_keeps_between_notes_gap --test issue_1139_inline_picture_duplicate` | 통과, 1 passed |
| 전체 integration | `cargo test --tests` | 통과 |
| Clippy | `cargo clippy --all-targets -- -D warnings` | 통과 |

## 6. 시각 판정 권장 후보

| file | page | 확인 항목 |
|---|---:|---|
| `samples/3-09월_교육_통합_2022.hwp` | 9, 10, 13, 14, 18 | `문N)` 제목 위 7mm 미주 사이 갭 |
| `samples/3-09월_교육_통합_2022.hwpx` | 9, 10, 13, 14, 18 | HWPX에서도 동일한 미주 경계 정합 |
| `samples/3-11월_교육_통합_2022.hwp` | PR 본문 회귀 대상 | 컬럼 하단 오버플로우가 새로 생기지 않는지 |

판정 포인트:

- 페이지9 문6 -> 문7 구간이 한컴 PDF처럼 약 7mm 더 벌어지는지
- 문12 제목 위치가 기존 cram 위치보다 아래로 내려가되, 문13이 뒤로 밀려 페이지네이션이 깨지지 않는지
- 미주 다줄 prev 경계(#1246 경로)가 기존처럼 유지되는지

## 7. 권장 처리

권장안: **수용 후보로 진행**한다.

근거:

- 변경 조건이 충분히 좁다.
- PR이 주장한 자동 검증 축이 로컬 현재 `devel` 기반에서도 통과했다.
- `cargo test --tests`와 `cargo clippy --all-targets -- -D warnings`가 통과했다.
- 잔여 케이스를 후속 #1257로 분리한 범위 관리가 합리적이다.

다만 `height_cursor` 변경이므로, 최종 반영 전 메인테이너 시각 판정을 게이트로 두는 것을 권장한다.

## 8. 다음 승인 요청

권장 절차:

```text
1. 필요 시 WASM 빌드
2. samples/3-09월_교육_통합_2022.hwp/.hwpx 기준 메인테이너 시각 판정
3. 통과 시 devel에 검증 커밋 반영
4. PR #1259 종료 및 이슈 #1256 close 확인
```

## 9. 승인 후 진행 기록

2026-06-03 메인테이너 승인 후 다음을 추가 수행했다.

| 항목 | 명령/대상 | 결과 | 비고 |
|---|---|---|---|
| WASM build | `docker compose --env-file .env.docker run --rm wasm` | 통과 | `pkg/` 산출물 생성 |
| Studio public sync | `pkg/rhwp*.{js,wasm,d.ts}` -> `rhwp-studio/public/` | 통과 | JS/WASM 해시 일치 |
| Studio build | `npm run build --prefix rhwp-studio` | 통과 | Vite production build |
| HWP 시각 후보 SVG | `samples/3-09월_교육_통합_2022.hwp` p9/10/13/14/18 | 생성 성공 | 13쪽 기존 4px overflow 로그 1건 |
| HWPX 시각 후보 SVG | `samples/3-09월_교육_통합_2022.hwpx` p9/10/13/14/18 | 생성 성공 | 13쪽 기존 4px overflow 로그 1건 |

생성 파일:

```text
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_009.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_010.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_013.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_014.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_018.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_009.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_010.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_013.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_014.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_018.svg
```

WASM 동기화 후 Git diff는 `rhwp-studio/public/rhwp_bg.wasm.d.ts`에만 발생했다. `rhwp.js`,
`rhwp_bg.wasm`, `rhwp.d.ts`는 `pkg/`와 `rhwp-studio/public/`의 해시가 일치한다.

## 10. 메인테이너 시각 판정

메인테이너가 위 SVG와 rhwp-studio 기준으로 시각 판정을 수행했고 통과 판정했다.

| 대상 | 판정 | 비고 |
|---|---|---|
| HWP 시각 후보 SVG | 통과 | p9/10/13/14/18 |
| HWPX 시각 후보 SVG | 통과 | p9/10/13/14/18 |
| rhwp-studio WASM 빌드 산출물 | 통과 | Studio build 통과 후 확인 |

메인테이너 판정:

```text
2026-06-03 통과
```

## 11. 최종 처리 판단

자동 검증과 메인테이너 시각 판정이 모두 통과했으므로 PR #1259는 수용한다.

후속 절차:

```text
1. 검증 브랜치 변경 커밋
2. devel 병합
3. devel 기준 테스트 재확인
4. origin/devel push
5. PR #1259 및 이슈 #1256 종료 처리
```
