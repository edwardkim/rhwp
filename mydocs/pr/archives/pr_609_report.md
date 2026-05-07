# PR #609 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #609 — Task #604 Document IR 표준 정합화 |
| 컨트리뷰터 | @jangster77 (Taesup Jang, tsjang@gmail.com) |
| 연결 이슈 | #604 (closed) |
| 처리 옵션 | 옵션 A — 11 commits 단계별 cherry-pick |
| 머지 commit | `29d1587` ~ `3228016` (linear, ff merge) |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

11 commits 단계별 보존 (author Taesup Jang, committer edward, rebase 으로 hash 재생성):

| Stage | 새 hash | 작업 |
|-------|---------|------|
| Stage 1 | `29d1587` | Document IR LineSeg 표준 정의 + `is_in_wrap_zone(col_w_hu)` helper |
| Stage 2 | `3761c4a` | typeset 출력 메타데이터 (`wrap_anchors`) 도입 |
| Stage 2b | `2a7f6d4` | `wrap_precomputed` 필드 제거 + HWP3 후처리 30 LOC 청산 |
| Stage 3 | `2718352` | HWP3 파서 wrap zone pgy 가드 단방향화 (Issue #604 정정) |
| Stage 4 | `c2b02a7` | 광범위 회귀 검증 + 시각 판정 자료 |
| Stage 5 B-2 | `0c93a53` | wrap zone 안 라인 line_spacing 100% 정합 (Stage A+D 에서 revert) |
| Stage 6 | `6bc949e` | paper_images z-order 정합 |
| Stage A+D | `dbc6a09` | HWP5 IR 표준 정밀 재진단 + HWP3 파서 정합 인코딩 |
| Stage D-2 | `a7cd61f` | HWP5 IR 표준 8 항목 정정 |
| Stage D-2 보완 | `9fc84eb` | paragraph 내 line wrap vpos reset |
| Stage D-2 문서 | `3228016` | 단계별/최종 보고서 + orders 갱신 |

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (PR #668 흡수 후 baseline) |
| `cargo test --test svg_snapshot --release` | ✅ 6/6 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ **12/12** |
| `cargo clippy --lib -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,598,892 bytes** (PR #601 baseline +10,869) |
| `rhwp-studio npm run build` | ✅ TypeScript 타입 체크 + dist 빌드 |

## 4. 광범위 페이지네이션 sweep

| 영역 | 결과 |
|------|------|
| BEFORE (rebase 전 devel) | 232 fixture |
| AFTER (local/pr609) | 234 fixture (신규 v2018/v2024 +2) |
| 페이지 수 차이 | **0** (231 fixture BEFORE/AFTER 일치, 회귀 0) |
| `hwp3-sample.hwp` (native) | 16 / 16 |
| `hwp3-sample4.hwp` (native) | 39 / 39 (변동 없음) |
| `hwp3-sample5.hwp` (native) | 64 / 64 |

> PR 본문 표의 "hwp3-sample4.hwp 정정 결과 36" 은 HWP5 변환본 (`hwp3-sample4-hwp5.hwp`) 의 페이지 수 영역 (issue_554 의 `hwp3_sample4_hwp5_36p` test 통과). native HWP3 자체는 BEFORE/AFTER 모두 39 페이지로 변동 없음.

## 5. 권위 자료 영구 보존

작업지시자 결정 (5/7) — 영구 보존:
- `samples/hwp3-sample5-hwp5-v2018.hwp` (296KB) — 한컴 v2018 변환본
- `samples/hwp3-sample5-hwp5-v2024.hwp` (285KB) — 한컴 v2024 변환본

직전 PR #611 의 3개 권위 샘플 영구 보존 패턴 일치 (samples/ 직속 git tracked).

## 6. 메인테이너 시각 판정

한컴 2010 + 한컴 2022 **편집기** 출력 권위 정답지 기준 (`reference_authoritative_hancom`):
- `hwp3-sample5.hwp` page 4 (Issue #604 권위 영역) ★ 통과
- `hwp3-sample5.hwp` page 8/16/22/27 + 43 ★ 통과
- `hwp3-sample4.hwp` 39 페이지 (native 영역) ★ 통과
- 잔존 영역 (별도 task scope) 까지 메인테이너 직접 확인 영역 통과

## 7. devel 머지 + push

### 머지 영역 처리
1. local/devel 에서 11 commits cherry-pick (충돌 0)
2. origin/devel 분기 발견 (`db07c3c` PR #668 처리 후속) → rebase 1차 진행
3. origin/devel 추가 분기 (`a86dcf7` PR #668 후속 갱신) → rebase 2차 진행
4. 최종 hash `29d1587` ~ `3228016` 11 commits linear 보존
5. devel ← local/devel ff merge → push 완료

### 분기 처리 메모리 룰 정합
- `feedback_release_sync_check` — 분기 발견 시 즉시 작업 중단 + 보고 패턴 일치. 다른 컴퓨터 push 와 본 환경 작업의 동시성 영역에서 rebase + ff-merge 정합 영역 처리.

## 8. PR / Issue close

- PR #609: 한글 댓글 등록 + close (`gh pr close 609`)
- Issue #604: 한글 댓글 등록 + close (`gh issue close 604`)

> `closes #604` 키워드는 cherry-pick merge 의 hash 재생성으로 자동 처리 안 됨 — 수동 close 진행. `feedback_close_issue_verify_merged` 정합.

## 9. 본 PR 의 주요 정정 영역

### Issue #604 결함 정정 (Stage 3)
- `src/parser/hwp3/mod.rs:1399-1407` 의 wrap zone pgy 가드 단방향화
- `pgy_start..pgy_end` 양방향 → `pgy < pgy_end` 만
- pi=75 첫 3 줄 cs/sw=0 → cs=35460/sw=15564 (그림 우측 wrap zone 정상 배치)

### IR 부채 청산 (Stage 2 + 2b)
- `Paragraph.wrap_precomputed` IR 플래그 제거
- HWP3 파서 후처리 30 LOC 청산
- `wrap_anchors` 메타데이터 채널 도입 — typeset 의 wrap_around state machine 매칭 결과를 layout 시점까지 전달

### HWP5 IR 표준 8 항목 정정 (Stage A+D+D-2)
1. `is_page_break` 영역 보강 — `prev_para_had_flags_break` + `first_pgy_here=0` 케이스
2. lh/ls HWP5 분리 인코딩 — `lh = th, ls = th * (ratio - 100) / 100`
3. `break_flag` → `tag` bit 누설 제거 — `tag = 0x00060000` 고정
4. pgy-based `column_type=Page` 설정 제거 — 자연 wrap 은 typeset 책임
5. wrap zone cs/sw 정합 인코딩 — anchor 의 `active_wrap_cs_sw` outer state + 후속 paragraph 정합 채움
6. paper-top anchor `acc_vpos` reset
7. `line_info.break_flag` 0x8001 → `column_type=Page` 변환
8. paragraph 내 line wrap vpos reset (`pgy[i] < pgy[i-1]` 시 acc_vpos = 0)

### Document IR 표준 명문화
- `mydocs/tech/document_ir_lineseg_standard.md` (+195) — LineSeg 필드별 단위/원점/0 의미 + HWP5/HWPX/HWP3 각 파서 인코딩 책임 명시
- `mydocs/tech/document_ir_parser_relationship_analysis.md` (+430) — IR ↔ 각 파서 관계
- `mydocs/tech/document_ir_wrap_zone_standard_review.md` (+209) — wrap zone 표준 review
- `mydocs/tech/hwp5_wrap_precomputed_analysis.md` (+185) — HWP5/HWPX 미적용 분석

## 10. 후속 task 영역 (PR 본문 §잔존)

작업지시자 결정 (5/7) — 한컴 편집기 시각 판정 후 결정:
1. HWP3 폰트 크기/줄간격 차이 — 본 환경 한컴 편집기 시각 판정 결과 결함 확인 시 후속 task 등록
2. HWP3 LineSeg vertical_pos 누적 계산 — 본 환경 광범위 sweep + 한컴 편집기 시각 판정으로 측정
3. Task #525 재검토 (`layout_wrap_around_paras` dead code 가능성) — 본 PR 머지 후 본 환경에서 dead code 검증 가능
4. 한컴 변환기 paragraph indent 흡수 휴리스틱 — sample4 pi=960 의 차이 영역, 본 환경 한컴 편집기 시각 판정 후 결정

> 본 시점 등록 영역 부재 — 메인테이너 시각 판정 ★ 통과 (잔존 영역까지 직접 확인) 결과로 별도 등록 미실행.

## 11. 메모리 룰 적용

- `reference_authoritative_hancom` — 한컴 2010 + 한컴 2022 편집기 권위 정답지. 한컴뷰어 / HWP5 v2024 변환본은 정답지 아님. PR 본문의 비교 표는 컨트리뷰터의 측정 기준.
- `feedback_pdf_not_authoritative` — 정답지 미입증 영역 비교 우선 회피.
- `feedback_close_issue_verify_merged` — Issue #604 close 시 본 PR 머지 검증 영역 (devel 머지 후 close 처리).
- `feedback_release_sync_check` — 분기 발견 시 즉시 작업 중단 + 보고 패턴 일치.
- `feedback_assign_issue_before_work` — Issue #604 assignee 미지정 영역 (외부 컨트리뷰터 자기 등록 사례).
- `feedback_visual_regression_grows` — 페이지 수 일치 + 시각 판정 ★ 결합.
- `project_hwpx_to_hwp_adapter_limit` — HWP3/HWP5/HWPX 3개 포맷 영역의 향후 정합 영역 진전.

## 12. 다음 사이클 영역

- v0.7.11 PATCH 릴리즈 후보 — 본 사이클 (5/7) 처리 누적 PR (#578/#629/#611/#620/#642/#601/#609 = 7건) 흡수 결과 묶음 가능성
- 본 PR 의 후속 영역 (Task #525 dead code 검증 등) 본 환경 직접 점검 영역
