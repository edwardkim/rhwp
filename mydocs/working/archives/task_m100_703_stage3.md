# Task #703 Stage 3 — 광범위 회귀 검증 완료 보고서

**Issue**: #703
**브랜치**: `local/task703`
**작성일**: 2026-05-08
**구현계획서**: `mydocs/plans/task_m100_703_impl.md`
**선행**: Stage 1 (`e0b8fd0e`), Stage 2 (`5c4cc190`)

---

## 1. 검증 항목

| 검증 | 결과 |
|------|------|
| 196 샘플 SVG/PDF 페이지 수 비교 | **회귀 0, 정합 +2** |
| `cargo test --lib --release` (1158 tests) | 0 fail |
| `cargo test --release --tests` (전체 통합) | 0 fail (issue_703 1 pass + 2 ignored) |
| `cargo test --release --test svg_snapshot` (7 tests) | 0 fail |
| `cargo clippy --release -- -D warnings` | 0 warning |
| `cargo build --release` | OK |

## 2. 196 샘플 SVG vs 한글2022 PDF 페이지 수 비교

**baseline**: `mydocs/report/svg_vs_pdf_diff_20260508.tsv` (Stage 1 RED 시점)
**after**: `mydocs/report/svg_vs_pdf_diff_20260508_after.tsv` (Stage 2 GREEN 시점)

### 변동 (실 SVG 카운트)

| 파일 | baseline | after | 변동 |
|------|----------|-------|------|
| `basic/calendar_year` | 2 | **1** | -1 (PDF=1 정합) |
| `table-ipc` | 13 | **10** | -3 (PDF=10 정합) |

**기타 195 파일 SVG 카운트 변동 0건** (회귀 0).

> baseline TSV 의 `basic/shortcut` PDF 카운트가 1→7 로 보이는 것은 작업지시자가 컨버세이션 중간에 `pdf/basic/shortcut-2022.pdf` 를 업데이트한 결과 (본 task 무관).

### `table-ipc.hwp` 의 추가 정합 — Stage 2 정정의 광범위 효과

`grep "wrap=" target/release/rhwp dump samples/table-ipc.hwp` 결과 11 개 표가 `wrap=글앞으로` (InFrontOfText). Stage 2 의 BehindText/InFrontOfText 가드가 이 표들의 본문 흐름 누적을 모두 제외 → 13 페이지 → 10 페이지 (한글2022 PDF 와 완전 정합).

**의도하지 않은 추가 정합** — 본질 정정이 더 넓은 케이스를 자동 해소함을 확인. Stage 2 정정의 정당성 강화.

### Stage 1 commit (RED) 와 Stage 2 commit (GREEN) 비교 검증

```bash
$ git stash
$ git checkout e0b8fd0e -- src/renderer/typeset.rs && cargo build --release
$ target/release/rhwp dump-pages samples/table-ipc.hwp | grep "^문서"
문서 로드: samples/table-ipc.hwp (13페이지)
$ git checkout 5c4cc190 -- src/renderer/typeset.rs && cargo build --release
$ target/release/rhwp dump-pages samples/table-ipc.hwp | grep "^문서"
문서 로드: samples/table-ipc.hwp (10페이지)
```

직접 확인: Stage 2 정정이 13 → 10 페이지 변환의 직접 원인.

## 3. 라이브러리/통합 테스트

```
$ cargo test --lib --release
test result: ok. 1158 passed; 0 failed; 2 ignored

$ cargo test --release --test issue_703
test issue_703_calendar_year_single_page ... ok
test issue_703_tonghap_2010_11_single_page ... ignored (Issue #704)
test issue_703_tonghap_2011_10_single_page ... ignored (Issue #704)
test result: ok. 1 passed; 0 failed; 2 ignored

$ cargo test --release --test svg_snapshot
test result: ok. 7 passed; 0 failed
```

**라이브러리 1158 + 통합 92건 (issue_703 1 + 기타 91) = 1250+ tests, 회귀 0**.

## 4. clippy / build

```
$ cargo clippy --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 11.56s
# 0 신규 경고

$ cargo build --release
    Finished `release` profile [optimized] target(s) in 13.87s
# 정상 빌드
```

## 5. 결론

Stage 2 정정 (`typeset_table_paragraph` 의 Control::Table 분기에 BehindText/InFrontOfText 가드 추가, +13 줄) 은:

- **본 task 정정 대상** `basic/calendar_year.hwp` 한글2022 PDF 정합 달성 (2→1 페이지)
- **의도하지 않은 추가 정합** `table-ipc.hwp` 도 PDF 완전 정합 (13→10 페이지)
- **회귀 0** (196 샘플 페이지 수 변동 0건 추가, lib/통합 테스트 0 fail)
- **clippy 0 신규 경고**

분리된 Issue #704 (통합재정통계 borderline) 는 본 task 와 본질이 다른 결함 — 별도 단계에서 처리.

## 6. 다음 단계

최종 보고서 (`mydocs/report/task_m100_703_report.md`) 작성 + 작업지시자 승인 후:
- `git push origin local/task703` (선택)
- `local/task703` → `local/devel` (또는 `devel`) merge
- `gh issue close 703` (또는 commit 메시지 `closes #703`)
