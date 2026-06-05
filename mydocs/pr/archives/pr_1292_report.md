# PR #1292 처리 보고서

## 1. 대상

- PR: https://github.com/edwardkim/rhwp/pull/1292
- 제목: `task 1284: #1274 후속 교육 통합 문항 흐름 drift 보정`
- 작성자: `jangster77`
- 처리일: 2026-06-05

## 2. 처리 내용

PR #1292의 실제 기능 커밋 15개를 최신 `local/devel` 위 통합 브랜치에 cherry-pick했다.

통합 브랜치:

```text
local/pr1292-integration
```

적용 방식:

```text
git fetch origin pull/1292/head:local/pr1292-upstream
git checkout -B local/pr1292-integration local/devel
git cherry-pick f3f4c0d7 a583c66c f2fa3632 b16e4e8f bc53f250 0495601b a83058e2 21b126f8 a8376127 b0462d67 28404d93 dcd99f59 54a214fb 7af89f67 b22c0a68
```

PR branch 전체를 merge하지 않은 이유:

- PR head는 최신 `local/devel`보다 3커밋 뒤였다.
- PR branch 전체를 merge하면 PR #1295의 Vite 업데이트와 archive 문서가 되돌아가는 diff가 발생했다.
- 따라서 PR branch의 merge commit 3개는 제외하고 실제 기능 커밋만 적용했다.

## 3. 변경 요약

### 3.1 compact 미주/문항 흐름 보정

`src/renderer/height_cursor.rs`, `src/renderer/layout.rs`, `src/renderer/typeset.rs`에 compact 미주, 수식-only tail, TAC 그림-only 문단, 문항 제목 tail 관련 페이지네이션 보정이 추가됐다.

주요 대상:

- `3-09월_교육_통합_2024-미주사이20.hwp`
- `3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
- `3-09월_교육_통합_2023.hwp`
- `3-09월_교육_통합_2022.hwp`
- `3-10월_교육_통합_2022.hwp`
- `3-11월_실전_통합_2022.hwp`

### 3.2 visual sweep 고도화

`scripts/task1274_visual_sweep.py`가 PDF bbox와 render tree bbox를 함께 비교하도록 확장됐다.

추가/보강 항목:

- question marker drift 비교
- frame tail overflow 후보 분리
- line band drift metric
- 작은 tail bleed suppression
- `question_flow.json` 출력

### 3.3 샘플 추가

검증용 샘플 3개가 추가됐다.

- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx`
- `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf`

### 3.4 회귀 테스트 추가

`tests/issue_1139_inline_picture_duplicate.rs`에 issue #1284 관련 페이지별 bbox 회귀 테스트가 추가됐다.

대표 검증:

- 2024-09 between20 page 13/18/19/21/22/23 문항 흐름
- 2023-09 page 14/16/19/20 문항 제목 tail
- 2022-09 page 17 문항 시작 위치
- 2022-10 page 11/14/15/17 수식/문항 tail
- 2022-11 practice page 11/19 tail frame

## 4. 검증

| 항목 | 결과 | 비고 |
|---|---|---|
| `cargo fmt --all -- --check` | 통과 |  |
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |  |
| `cargo build --verbose` | 통과 | Cargo global cache last-use DB readonly 경고만 발생 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |  |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 67 passed |
| `cargo test --features native-skia skia --lib --verbose` | 통과 | 39 passed |
| `python3 scripts/task1274_visual_sweep.py --target all` | 미실행 | 로컬 필수 도구 `rsvg-convert` 없음 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 | Done in 2m 54s |

WASM 산출물:

```text
pkg/rhwp_bg.wasm: 5.3M
```

WASM 동기화:

```text
cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts pkg/rhwp_bg.wasm.d.ts rhwp-studio/public/
```

해시 확인 결과:

- `pkg/rhwp_bg.wasm` == `rhwp-studio/public/rhwp_bg.wasm`
- `pkg/rhwp.js` == `rhwp-studio/public/rhwp.js`
- `pkg/rhwp.d.ts` == `rhwp-studio/public/rhwp.d.ts`
- `pkg/rhwp_bg.wasm.d.ts` == `rhwp-studio/public/rhwp_bg.wasm.d.ts`

## 5. GitHub Actions

PR head `1e49b1b6308039271873d79eb8ed8440fb4eee98` 기준:

| workflow | run id | conclusion |
|---|---:|---|
| CI | 26986440097 | success |
| Render Diff | 26986440093 | success |
| CodeQL | 26986440101 | success |

## 6. 메인테이너 시각 판정

메인테이너가 컨트리뷰터가 주장한 사항들을 SVG/웹 기준으로 직접 시각 판정했고 통과했다.

판정:

```text
2026-06-05 통과
```

## 7. 남은 절차

1. 완료 보고서 승인
2. `mydocs/pr/pr_1292_review.md`, `mydocs/pr/pr_1292_report.md` 포함 커밋
3. `local/devel` -> `devel` 로컬 merge
4. `devel` 최종 확인 후 `origin/devel` push
5. PR #1292 메인테이너 코멘트 등록
6. PR #1292 close
7. 연결 이슈 #1284 close 여부 확인 및 처리

## 8. 판정

자동 검증, WASM 빌드, 메인테이너 시각 판정을 모두 통과했다.

PR #1292는 수용한다.
