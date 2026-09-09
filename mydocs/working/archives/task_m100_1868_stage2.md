# Task M100 #1868 2단계 완료보고서 — 테스트 + 회귀

- 이슈: #1868 / 브랜치: `local/task1868`
- 작성일: 2026-07-04 / 단계: 2/3

## 신규 테스트 (`tests/issue_1868_export_hwpx_cli.rs`, 3건)

코어 경로(`from_bytes` → `export_hwpx_native`) 직접 검증 — ZIP 매직 + 재파싱 + 페이지 수 보존:

| 테스트 | 입력 | 결과 |
|---|---|---|
| `hwp5_to_hwpx_preserves_pages` | `samples/pr-1674.hwp` (CFB) | ✓ |
| `hwp3_to_hwpx_preserves_pages` | `samples/hwp3-sample.hwp` (HWP3 — 계획서의 "적합 샘플 부재 시 대체" 불필요, 실제 HWP3 fixture 발견·사용) | ✓ |
| `hwpx_reserialize_preserves_pages` | `samples/hwpx/pr-1674.hwpx` (re-serialize) | ✓ |

## 회귀

| 항목 | 결과 |
|---|---|
| 전체 `cargo test --tests` (release-test) | **FAILED 0** (2812 passed, 신규 3 포함) |
| fmt --check / clippy(main.rs·신규 테스트) | clean / 무경고 |
| **#1879 fidelity 하니스 스모크** | export-hwpx 산출물(pr-1674.hwpx) → `convert` 로 HWP 재변환 → dump-pages 대조 **SAME (35=35쪽)** — HWP→HWPX→HWP 왕복 전체 체인 정합 |

## 다음 단계

3단계: `cli_commands.md` 등재 + 최종 보고서 + 오늘할일 갱신 + 이슈 회신.
