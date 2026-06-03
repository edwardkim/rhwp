# v0.7.12 PATCH 릴리즈 결과 보고서

- 일자: 2026-05-18
- 방식: 1 방식 + 패턴 A (v0.7.11 동일)
- 기준: v0.7.11 (`a9dcdee3`) → v0.7.12 (`1899ef9b`)

## 1. 결정

PATCH 릴리즈. 외부 기여자 PR 시리즈 흡수 + @jangster77 7-PR 시리즈 (#956~#968) 완결. 신규 API/모듈은 모두 opt-in·하위 호환 100% 이므로 `feedback_small_batch_release_strategy` 정합 — PATCH 로 빠른 회전.

## 2. 버전 동기화 (4 파일)

| 파일 | 0.7.11 → 0.7.12 |
|------|-----------------|
| `Cargo.toml` | ✓ |
| `rhwp-vscode/package.json` | ✓ |
| `npm/editor/package.json` | ✓ |
| `rhwp-studio/package.json` | ✓ |

## 3. CHANGELOG 3종

- `CHANGELOG.md` — `## [0.7.12] — 2026-05-18` 추가 (+ 누락된 `[0.7.11]` 소급 보강)
- `CHANGELOG_EN.md` — `## [0.7.12] — 2026-05-18` (영문) (+ `[0.7.11]` 소급)
- `rhwp-vscode/CHANGELOG.md` — `## [0.7.12] - 2026-05-18`

## 4. 흡수 범위 (v0.7.11..devel)

- 422 files / +64761 / -3328
- **@jangster77 7-PR 시리즈 (#956~#968)** — 원 Issue #952 (1 통합 → 5 분리 결함) 완결 + WMF #966 + HWP3 sample18 #968
  - #956 (#952 I1) 쪽 테두리 paper-based / #958 (#957) 빈 caption phantom / #961 (#959) Column picture advance / #963 (#960) inline TAC line 매핑 / #964 (#962) 글상자 inline equation duplicate / #966 (#965) WMF SetTextAlign vertical bits / #968 (#967) 빈 paragraph + 쪽나누기 단독 page
- PR #818 (#790) release LTO + codegen-units=1 + strip — 바이너리 -28% / WASM -6.5%
- rhwp-studio: 문서 비교·이력 (#799), searchAllText/rhwpDev.search (#814), F5 블록 선택 (#811), moveToDocumentEnd 다중 구역 (#808), 쪽 새 번호 (#809), Alt+Arrow 단어 이동 (#794), 표 셀 드래그 (#795) 등
- dependabot 7건 (vite 8 / skia-safe 0.97 / resvg 0.47 / puppeteer-core 24.43 등)

## 5. 자기 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release` (lib) | 1288 passed / 0 failed / 2 ignored |
| integration svg_snapshot | 8 passed |
| tab_cross_run | 1 passed |
| WASM 재빌드 (Docker) | 4.6 MB — rhwp-studio/public 동기화 |

## 6. 릴리즈 절차 (패턴 A)

1. local/devel 동기화 (origin/devel)
2. 버전 bump + CHANGELOG → `a0947774` (local/devel)
3. local/devel → devel no-ff merge → push (`974855f3..0ff97dec`)
4. `feedback_release_sync_check` — main 분기 점검: 4 main-only commit 전부 과거 릴리즈 PR merge commit, `git diff origin/devel...origin/main` 공집합 → 실제 content 분기 없음 (정상 패턴) 확인
5. devel→main **PR #970** 생성 → `--admin` merge (자기 PR 자기 승인 불가 → branch protection bypass merge, v0.7.11 동일 패턴)
6. main HEAD `1899ef9b` 에 **v0.7.12 태그** 생성 + push
7. local/devel ff → 정렬 (main/devel/local/devel 일관)

## 7. 잔존 / 후속

- HWPX sample18-hwp5.hwpx +7 inflate (별도 issue)
- `samples/hwp3-sample18.hwp` fixture 추가 권장 (#968 회귀 가드 — 본 릴리즈 미포함)
- hwpx2hwp 피드백 문서 2종 (작업지시자 지시 — 미커밋, 본 릴리즈 무관)
