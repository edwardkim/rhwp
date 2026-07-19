# PR #1262 검토 - 다줄 prev 미주 제목 between-notes 갭 보존

- **작성일**: 2026-06-03
- **PR**: #1262
- **제목**: `Task #1257: 다줄 prev 미주 제목 between-notes 갭 보존`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1257
- **base/head**: `devel` <- `task1257-multiline-gap`
- **PR head**: `28348d142b9688ba66cd47095b7c0cafacc2c231`
- **PR 기준 base**: `77c2efb4`
- **현재 devel**: `77c2efb4`
- **검증 브랜치**: `local/pr1262-current`
- **검증 커밋**: `c68b8109`
- **규모**: 7 files, +392 / -1
- **GitHub mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1262는 compact 미주 영역에서 직전 미주가 다줄/수식 같은 높은 줄로 끝날 때,
다음 `문N)` 제목 위의 `between-notes` 7mm 갭이 사라지는 문제를 좁게 수정한다.

기존 로직은 직전 `FullParagraph`의 마지막 `LINE_SEG`에서 주입된 `line_spacing > 1000`
값을 다음 제목 앞 gap 후보로 삼았지만, 동시에 `seg.line_height <= 1500` 조건을 요구했다.
이 때문에 마지막 줄이 수식 등으로 높아진 케이스는 `line_spacing=1984`를 가지고 있어도
gap 후보에서 탈락했다.

이번 PR은 해당 높이 제한만 제거한다.

```rust
.filter(|seg| seg.line_spacing > 1000)
```

PR 문서 기준 재현 케이스:

| 케이스 | 직전 마지막 seg | 기존 결과 | PR 결과 |
|---|---|---|---|
| 문26 | `line_spacing=1984`, `line_height=2070` | gap 0 | gap 보존 |
| 문29 | `line_spacing=1984`, `line_height=6897` | gap 0 | gap 보존 |

## 2. 주요 변경 범위

| 파일 | 변경 |
|---|---|
| `src/renderer/layout.rs` | `prev_endnote_title_gap_px` 산출 시 `line_height <= 1500` 필터 제거 |
| `mydocs/plans/task_m100_1257.md` | 작업 계획 |
| `mydocs/plans/task_m100_1257_impl.md` | 구현 계획 |
| `mydocs/tech/archive/endnote_seq_flow_redesign.md` | 미주 sequential flow 후속 설계 문서 |
| `mydocs/working/task_m100_1257_stage*.md` | 단계별 분석/검증 기록 |

## 3. 타당한 부분

### 3.1 변경 조건이 핵심 원인에 직접 대응한다

주입된 `between-notes` 마커는 `line_spacing`에 들어가며, 마지막 줄의 실제 높이와 독립적이다.
따라서 `line_height <= 1500` 조건은 문26/문29처럼 수식 또는 높은 줄로 끝난 미주에서
갭을 잘못 버리는 원인이 된다.

### 3.2 실제 적용은 기존 보존 게이트를 그대로 통과해야 한다

이번 PR은 gap 후보 산출만 넓힌다. 실제 이동은 기존 조건을 유지한다.

```text
current_is_endnote_question_title
prev_endnote_title_gap_px > 0
prev_endnote_title_gap_from_continued_partial || y_offset > y_before_vpos + 0.5
```

즉 일반 문단, 미주 제목이 아닌 문단, 후보 gap이 없는 경우에는 동작하지 않는다.

### 3.3 넓은 sequential-flow 재설계는 후속으로 분리했다

`mydocs/tech/archive/endnote_seq_flow_redesign.md`는 #1184/#1257 계열의 더 큰 구조 개선 방향을
정리하지만, 이번 PR 코드 변경은 국소 필터 수정 1건으로 제한한다.

## 4. 위험 및 주의 사항

### 4.1 `line_spacing > 1000` 휴리스틱은 여전히 SSOT가 아니다

`between-notes` 주입 여부를 명시 플래그로 전달하지 않고, `LINE_SEG.line_spacing` 값으로
추론한다. 이 방식은 현재 미주 compact flow의 기존 계약을 따르지만, 장기적으로는
trailing/gap SSOT 정리가 필요하다.

완화 요인:

- 조건이 `col_content.endnote_flow` 내부에만 있다.
- 다음 항목이 `문N)` 제목인지 다시 확인한다.
- 실제 y 이동은 기존 vpos 게이트를 통과해야 한다.

### 4.2 문27 계열 forward-suppress는 의도적으로 남겼다

PR Stage 3 문서는 `y_offset > y_before + 0.5` 조건 제거 시도에서 #1189/#1209/#1256 회귀가
발생했다고 기록한다. 따라서 문27 등 일부 잔여 케이스를 이번 PR 범위에서 제외한 판단은
안전한 범위 관리로 보인다.

## 5. 자동 검증 결과

현재 `devel` 위에 PR 커밋을 체리픽해 검증했다.

| 항목 | 명령 | 결과 |
|---|---|---|
| cherry-pick | PR commit 1건 -> `local/pr1262-current` | 통과, 충돌 없음 |
| whitespace | `git diff --check devel..HEAD` | 통과 |
| Rust fmt | `cargo fmt --all --check` | 통과 |
| compact endnote 단위 테스트 | `cargo test compact_endnote --lib` | 통과, 20 passed |
| 미주/페이지네이션 통합 테스트 | `cargo test --test issue_1139_inline_picture_duplicate` | 통과, 43 passed |

## 6. 시각 판정 권장 후보

| file | 확인 항목 |
|---|---|
| `samples/3-09월_교육_통합_2022.hwp` | 문26/문29 인근 미주 제목 위 7mm 갭 |
| `samples/3-09월_교육_통합_2022.hwpx` | HWPX 경로의 동일 갭 보존 |
| `samples/3-10월_교육_통합_2022.hwp` / `.hwpx` | PR 문서가 언급한 동류 케이스 |
| `samples/3-11월_실전_통합_2022.hwp` / `.hwpx` | 기존 미주 회귀 후보 |

판정 포인트:

- 직전 미주가 수식/높은 줄로 끝나도 다음 `문N)` 제목 앞 간격이 사라지지 않는지
- 기존 #1189/#1209/#1256 경계가 과하게 밀리지 않는지
- 페이지 개수와 주요 문항 시작 페이지가 기존 guard와 동일한지

## 7. 권장 처리

권장안: **수용 후보로 진행**한다.

근거:

- 코드 변경이 `layout.rs`의 한 필터 조건에 국한된다.
- 기존 보존 게이트는 유지되어 과적용 위험을 제한한다.
- 관련 단위/통합 테스트가 현재 `devel` 위에서 통과했다.
- PR 문서가 실패한 확장 시도와 범위 밖 잔여 케이스를 명확히 기록한다.

다만 미주 페이지네이션은 시각 회귀가 자주 발생하는 영역이므로, 최종 반영 전 메인테이너
시각 판정을 게이트로 두는 것을 권장한다.

## 8. 다음 승인 요청

권장 절차:

```text
1. 필요 시 WASM 빌드
2. 위 샘플 기준 메인테이너 시각 판정
3. 통과 시 검증 커밋 작성
4. devel 병합
5. devel 기준 간단 검증 후 push
6. PR #1262 및 이슈 #1257 종료 처리
```

## 9. 승인 후 진행 기록

2026-06-03 메인테이너 승인 후 다음을 추가 수행했다.

| 항목 | 명령/대상 | 결과 | 비고 |
|---|---|---|---|
| WASM build | `docker compose --env-file .env.docker run --rm wasm` | 통과 | `pkg/` 산출물 생성 |
| Studio public sync | `pkg/rhwp_bg.wasm` -> `rhwp-studio/public/rhwp_bg.wasm` | 통과 | 해시 일치 확인 |
| Studio build | `npm run build --prefix rhwp-studio` | 통과 | Vite production build |
| HWP 시각 후보 SVG | 2022 9월 p18/p19, 2022 10월 p11, 2022 11월 p17 | 생성 성공 | 10월 p11 overflow 로그 24.5px 1건 |
| HWPX 시각 후보 SVG | 2022 9월 p18/p19, 2022 10월 p11, 2022 11월 p17 | 생성 성공 | 10월 p11 overflow 로그 24.5px 1건 |

생성 파일:

```text
output/poc/pr1262-visual/hwp/3-09월_교육_통합_2022_018.svg
output/poc/pr1262-visual/hwp/3-09월_교육_통합_2022_019.svg
output/poc/pr1262-visual/hwpx/3-09월_교육_통합_2022_018.svg
output/poc/pr1262-visual/hwpx/3-09월_교육_통합_2022_019.svg
output/poc/pr1262-visual/hwp-oct/3-10월_교육_통합_2022_011.svg
output/poc/pr1262-visual/hwpx-oct/3-10월_교육_통합_2022_011.svg
output/poc/pr1262-visual/hwp-nov/3-11월_실전_통합_2022_017.svg
output/poc/pr1262-visual/hwpx-nov/3-11월_실전_통합_2022_017.svg
```

WASM/Studio 빌드 산출물은 추적 파일 변경을 만들지 않았다. 현재 커밋 대상은 본 리뷰 문서와
PR 체리픽 변경이며, 기존 untracked `mydocs/working/task_m100_1248_ssot_stage1.md`는 #1262 범위에서
제외한다.

## 10. 메인테이너 시각 판정

메인테이너가 위 SVG와 rhwp-studio 기준으로 시각 판정을 수행했고 통과 판정했다.

| 대상 | 판정 | 비고 |
|---|---|---|
| 2022 9월 HWP/HWPX p18/p19 | 통과 | 문26/문29 인근 미주 제목 gap 확인 |
| 2022 10월 HWP/HWPX p11 | 통과 | overflow 로그 24.5px 1건은 기존 후보 로그로 기록 |
| 2022 11월 HWP/HWPX p17 | 통과 | 기존 미주 회귀 후보 |
| rhwp-studio WASM 빌드 산출물 | 통과 | Studio build 통과 후 확인 |

메인테이너 판정:

```text
2026-06-03 통과
```

## 11. 최종 처리 판단

자동 검증과 메인테이너 시각 판정이 모두 통과했으므로 PR #1262는 수용한다.

후속 절차:

```text
1. 리뷰 문서 커밋
2. devel 병합
3. devel 기준 간단 검증
4. origin/devel push
5. PR #1262 및 이슈 #1257 종료 처리
```
