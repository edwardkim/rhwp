# PR #609 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #609 |
| 제목 | Task #604: Document IR 표준 정합화 (HWP3/HWP5/HWPX 3개 포맷 동일 페이지 정합) |
| 컨트리뷰터 | @jangster77 (Taesup Jang, tsjang@gmail.com) |
| base / head | `devel` ← `jangster77:local/task604` |
| state / mergeable | OPEN / MERGEABLE / **BEHIND** (PR base 9 commits 뒤) |
| 변경 | 31 files, +3,348 / -104 |
| commits | 11 작업 commit + 11 merge commit = 22 (squash 또는 핀셋 cherry-pick 후보) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | Closes #604 (PR 본문 명시) |
| 작성일 / 갱신 | 2026-05-05 / 2026-05-06 |

### CI 상태 (모두 통과)
- Build & Test ✅
- Analyze (rust / python / javascript-typescript) ✅
- Canvas visual diff ✅
- CodeQL ✅
- WASM Build SKIPPED (PR 트리거 영역 외)

> 컨트리뷰터 5/5 댓글의 CI 결함 (CodeQL 인프라 일시 결함) 은 5/6 재실행 후 모두 통과. 현재 시점 `mergeStateStatus=BEHIND` 외 결함 없음.

---

## 2. Issue #604 요약

### 결함
`hwp3-sample5.hwp` page 4 (HWP3 native): pi=74 그림 (User level programs/Kernel/Hardware 다이어그램, 126.4×94.5mm, Square wrap, tac=false) 우측에 wrap text (pi=75, 415자) 가 정상 배치되지 않고 그림 좌측 + 그림 위 + 그림 아래 에 산재 (그림과 겹침).

### 데이터 (Issue #604 본문)
```
--- 문단 0.75 --- cc=415, text_len=415, controls=0
  ls[0]: ts=0,   vpos=0, cs=0, sw=0           ← wrap zone 미설정 (그림 좌측)
  ls[1]: ts=17,  vpos=0, cs=0, sw=0           ← 동일
  ls[2]: ts=33,  vpos=0, cs=0, sw=0           ← 동일
  ls[3]: ts=50,  vpos=0, cs=35460, sw=15564   ← 정상
  ls[4~]:               cs=35460, sw=15564    ← 정상
```

### 원인 추정 (Issue #604)
`src/parser/hwp3/mod.rs:1399-1407` 의 `current_zone` 추적 — `pgy >= pgy_start && pgy < pgy_end` 양방향 가드 → wrap text 문단의 첫 줄 pgy 가 anchor 의 pgy_start 미만 시 cs/sw=0 설정.

### 우선순위
시각 결함 (한컴뷰어와 불일치). milestone v1.0.0.

### Issue #604 assignee
- **assignee 미지정** — 컨트리뷰터 (@jangster77) 가 직접 자기 등록 후 작업 진입. `feedback_assign_issue_before_work` 메모리 룰 관점에서 일차 방어선 부재 사례.

---

## 3. 본 환경 상태 점검

### 본 환경 devel 의 직전 commits
```
ce77fa7 Merge local/devel: PR #589 처리 후속 (보고서 + archives + orders)
8f4d2c6 PR #589 처리 후속: 보고서 + archives 이동 + 5/5 orders
5ba7f2e Merge local/devel: PR #589 cherry-pick (Task #511 v2 + #554 — @jangster77 7 commits + 시각 판정 ★ 통과)
ff64387 Task #460 Stage 9 보완8: HWP3 single-LineSeg wrap zone 문단 감지 확장
bdb51a4 Task #460 Stage 9 보완6: HWP3 wrap zone x 위치 수정 (wrap_precomputed IR 플래그)
```

본 PR 은 PR #589 머지 후 시각 판정 중 발견된 잔존 결함의 정정 작업. 컨트리뷰터의 4/30~5/5 PR #478 → #585 → #589 → #609 사이클 누적.

### 본 환경 devel 현재 상태 (관찰)
- `src/model/paragraph.rs:55` — `wrap_precomputed: bool` 필드 잔존 (PR #589 의 보완6)
- `src/renderer/typeset.rs:492-496` — `wrap_precomputed=true` 분기 검사 잔존
- `src/parser/hwp3/mod.rs` — 1399-1407 의 `pgy_start..pgy_end` 양방향 가드 + 후처리 로직 잔존
- `mydocs/tech/document_ir_lineseg_standard.md` 등 — 부재 (본 PR 신규 추가)

본 PR 의 작업: PR #589 의 잔존 부채 (wrap_precomputed IR 플래그 + HWP3 후처리 30 LOC + pgy 양방향 가드) 청산 + Document IR 표준 명문화 + HWP3/HWP5/HWPX 3개 포맷 동일 페이지 일치.

---

## 4. PR diff 분석

### 변경 파일 분포 (31 files)

#### 소스 (14 files)
| 파일 | +/- | 변경 |
|------|-----|------|
| `src/parser/hwp3/mod.rs` | +200/-62 | 8 항목 정정 (is_page_break / lh-ls 분리 / break_flag 누설 제거 / pgy column_type=Page 제거 / wrap zone cs/sw 인코딩 / paper-top anchor reset / line_info.break_flag 0x8001 / paragraph 내 line wrap vpos reset) |
| `src/renderer/typeset.rs` | +39/-7 | wrap_anchors 메타데이터 채널 — anchor 종류 (Picture vs Table) 기반 분기 |
| `src/renderer/layout.rs` | +28/-10 | `ColumnItemCtx.wrap_anchors` 필드 + 7 호출처 정합 + paper_images z-order 정정 |
| `src/renderer/layout/paragraph_layout.rs` | +17/-11 | 3 시그니처에 `wrap_anchor` 인자 추가 + 3 검사 사이트 (BoundingBox.x / x_base / line_cs_offset) → `wrap_anchor.is_some()` 교체 |
| `src/renderer/pagination.rs` | +27/-0 | `WrapAnchorRef` struct + `ColumnContent.wrap_anchors` HashMap 필드 |
| `src/renderer/pagination/state.rs` | +6/-0 | `current_column_wrap_anchors` 필드 + flush 시 take |
| `src/model/paragraph.rs` | +28/-14 | LineSeg 필드 doc 정합 + `is_in_wrap_zone(col_w_hu)` helper 추가 + `wrap_precomputed` 필드 제거 |
| `src/renderer/layout/{picture_footnote,shape_layout,table_cell_content,table_layout,table_partial}.rs` | 각 1-2 LOC | 셀/도형/각주/캡션 컨텍스트 호출 7곳에 `None` 전달 |
| `src/renderer/page_number.rs` + `src/renderer/layout/tests.rs` | +1, +5 | 테스트 ColumnContent 정합 |

#### 문서 (12 files)
- `mydocs/plans/task_m100_604.md` (+278) / `task_m100_604_impl.md` (+552) — 수행/구현 계획서
- `mydocs/report/task_m100_604_report.md` (+325) — 최종 보고서
- `mydocs/working/task_m100_604_{stage1,stage2,stage2b,stage3,stage6,stageAD,stageD2}.md` (+810 누적) — 단계별 보고서 7개
- `mydocs/tech/document_ir_lineseg_standard.md` (+195) — IR 표준 명문화
- `mydocs/tech/document_ir_parser_relationship_analysis.md` (+430) — IR ↔ 각 파서 관계 분석
- `mydocs/tech/document_ir_wrap_zone_standard_review.md` (+209) — wrap zone 표준 review
- `mydocs/tech/hwp5_wrap_precomputed_analysis.md` (+185) — HWP5/HWPX 미적용 분석
- `mydocs/orders/20260505.md` (+6) — Task #604 entry

#### 샘플 (2 files)
- `samples/hwp3-sample5-hwp5-v2018.hwp` (295,936 bytes) — HWP5 변환본 (한컴 v2018)
- `samples/hwp3-sample5-hwp5-v2024.hwp` (285,184 bytes) — HWP5 변환본 (한컴 v2024)

> 본 환경 `samples/` 직속에 `hwp3-sample5-hwp5.hwp` 만 존재 — v2018/v2024 변환본은 본 PR 신규 추가. 영구 보존 가/부는 작업지시자 결정 (직전 PR #611 의 3개 권위 샘플 영구 보존 패턴 참고).

---

## 5. 컨트리뷰터의 단계별 작업 (Stage 1 ~ D-2)

11 commits 진행:

| Stage | commit | 변경 |
|-------|--------|------|
| Stage 1 | `4eaf52b` | Document IR LineSeg 표준 정의 + `is_in_wrap_zone(col_w_hu)` helper |
| Stage 2 | `7c15d43` | typeset 출력 메타데이터 (`wrap_anchors`) 도입 — wrap_around state machine 매칭 결과를 layout 시점까지 전달 |
| Stage 2b | `08cd66f` | `wrap_precomputed` 필드 제거 + HWP3 후처리 30 LOC 청산 (-23 net LOC) |
| Stage 3 | `e52d7eb` | HWP3 파서 wrap zone pgy 가드 단방향화 — `pgy_start..pgy_end` 양방향 → `pgy < pgy_end` 만 (Issue #604 결함 정정) |
| Stage 4 | `ca47825` | 광범위 회귀 검증 + 시각 판정 자료 + 최종 보고서 |
| Stage 5 (B-2) | `80fb84d` | wrap zone 안 라인 line_spacing 100% 일치 (HWP5 v2024 변환본 일치) |
| Stage 6 | `92e3aff` | paper_images z-order 정정 (body_node → paper_images, 그림이 top layer) |
| Stage A+D | `4ba0816` | HWP5 v2024 변환본 정밀 재진단 + HWP3 파서 정합 인코딩 (acc_section_vpos 누적). Stage 5 B-2 revert. |
| Stage D-2 | `ce9f6e1` | HWP5 IR 표준 정합 8 항목 정정 |
| Stage D-2 보완 | `67d6772` | paragraph 내 line wrap vpos reset (`pgy[i] < pgy[i-1]` 시 acc reset) |
| Stage D-2 문서 | `81e8855` | 단계별 보고서 + 최종 보고서 + orders 갱신 |

### PR 본문의 검증 결과

#### HWP3/HWP5/HWPX 3개 포맷 변환본 동일 페이지 수
| 파일 | baseline | 정정 결과 | 일치 |
|------|---------|-----------|------|
| `hwp3-sample.hwp` | 16 | 16 | ✅ |
| `hwp3-sample4.hwp` | HWP5 36 | 36 | ✅ + paragraph 시퀀스 일치 |
| `hwp3-sample5.hwp` | HWP5 64 | 64 | ✅ HWP5 완전 일치 |
| `exam_science.hwp` | 4 | 4 | ✅ Task #546 일치 |

#### LineSeg 일치 (sample5) — 컨트리뷰터의 비교 기준

> **주의**: 아래 표의 "HWP5 v2024" 컬럼은 컨트리뷰터의 비교 기준 (HWP5 변환본). 본 환경 권위 정답지 (한컴 2010 + 한컴 2022 편집기) 와는 별개. 한컴 편집기 시각 판정으로 본 환경 권위 영역 측정 필요.

| 항목 | 본 환경 (정정 후) | HWP5 v2024 (컨트리뷰터 측정) | 일치 |
|------|------------------|-----------|------|
| pi=74 picture page | 4 | 4 | ✅ |
| pi=74 ls[0] vpos | 0 | 0 | ✅ |
| pi=75 ls[0..18] cs/sw | 35460/15564 | 37164/13860 | ✅ wrap zone 안 |
| pi=75 ls[19..20] cs/sw | 0/0 | 0/51024 | ✅ wrap zone 끝 전환 |
| pi=1213 ls[0] vpos | 72000 | 72000 | ✅ |
| pi=1213 ls[1] vpos | 0 | 0 | ✅ paragraph 내 페이지 reset |
| pi=1213 ls[2..3] vpos | 1440, 2880 | 1440, 2880 | ✅ |

#### 결정적 검증
- `cargo build` ✅
- `cargo test --lib` ✅ 1131 passed
- `cargo test --test svg_snapshot` ✅ 6/6 (HWP5 native 회귀 0)
- `cargo test --test issue_546` ✅ Task #546 일치
- `cargo test --test issue_554` ✅ 12/12 (모든 fixture 일치)
- `cargo clippy --lib -- -D warnings` ✅ 0건

---

## 6. 옵션 분류

본 환경 임시 clone 에서 옵션별 cherry-pick simulation 진행 — 충돌/회귀 측정 결과 기록.

### Stage 의존 그래프

11 commits 의 의존 관계:

```
Stage 1 (4eaf52b) — IR 표준 명문화 + is_in_wrap_zone() helper
   │
   ▼
Stage 2 (7c15d43) — typeset wrap_anchors 메타데이터 채널
   │   • paragraph.rs 호출처 변경 0, 다음 stage 에서 일괄 적용
   │   • Paragraph.wrap_precomputed 검사 → wrap_anchor.is_some() 검사 교체
   ▼
Stage 2b (08cd66f) — wrap_precomputed 필드 제거 + HWP3 후처리 30 LOC 청산
   │   ⚠️ Stage 2 의존 (wrap_anchor 검사 부재 시 컴파일 실패)
   ▼
Stage 3 (e52d7eb) — HWP3 파서 wrap zone pgy 가드 단방향화 (Issue #604 정정)
   │   • src/parser/hwp3/mod.rs:1399 (단일 파일)
   │   ⚠️ Issue #604 결함 (page 4 그림 좌측 텍스트 겹침) 정정 commit
   │   ⚠️ 옵션 B (이 commit 미포함) 머지 시 결함 미해결 잔존
   ▼
Stage 4 (ca47825) — 광범위 회귀 검증 (소스 변경 0, 문서만)
   │
   ▼
Stage 5 B-2 (80fb84d) — 줄간격 100% 일치
   │   ⚠️ Stage A+D 에서 revert (Stage A+D 본문 명시)
   ▼
Stage 6 (92e3aff) — paper_images z-order 정정 (body_node → paper_images)
   │
   ▼
Stage A+D (4ba0816) — HWP5 IR 표준 재진단 + acc_section_vpos 누적 (Stage 5 B-2 revert)
   │   ⚠️ HWP3 native +3 페이지 회귀 발생 (sample5 64→67) — Stage D-2 에서 정정
   ▼
Stage D-2 (ce9f6e1) — HWP5 IR 표준 8 항목 정정 (sample5 65→64)
   │   ⚠️ 65 → expect 64 1 페이지 부풉, Stage D-2 보완 에서 정정
   ▼
Stage D-2 보완 (67d6772) — paragraph 내 line wrap vpos reset → sample5 64 완전 일치
   │
   ▼
Stage D-2 문서 (81e8855) — 단계별/최종 보고서 + orders 갱신
```

**의존 관계 요약**:
- Stage 2 → 2b 의존 (wrap_anchor 부재 시 컴파일 실패)
- Stage 5 B-2 → Stage A+D 에서 revert (단독 머지 시 잘못된 상태)
- Stage A+D → Stage D-2 → Stage D-2 보완 누적 회귀 정정 (중간 stage 만 머지 시 +1 또는 +3 페이지 회귀)

### 옵션 A — 전체 cherry-pick (11 commits, author 보존)

**진행 명령**:
```bash
git checkout devel && git checkout -b local/pr609_optionA
git cherry-pick 4eaf52b 7c15d43 08cd66f e52d7eb ca47825 \
                80fb84d 92e3aff 4ba0816 ce9f6e1 67d6772 81e8855
git checkout devel && git merge --no-ff local/pr609_optionA \
  -m "Merge local/devel: PR #609 cherry-pick (Task #604 — @jangster77 11 commits)"
```

**본 환경 cherry-pick simulation 결과** (임시 clone):
- 11 commits 모두 충돌 0 (auto-merge 통과: `paragraph.rs` / `layout.rs` / `paragraph_layout.rs`)
- `mydocs/orders/20260505.md` 6 LOC add/add 자동 정합

**장점**:
- PR #589 의 잔존 부채 (`Paragraph.wrap_precomputed` IR 플래그 + HWP3 후처리 30 LOC) 청산
- 11 commits 단계별 보존 → bisect + 회귀 origin 추적 (`feedback_close_issue_verify_merged` 관점)
- 컨트리뷰터 author 11 commits 모두 보존
- Issue #604 결함 정정 (Stage 3) 포함
- HWP5 IR 표준 8 항목 정정 (Stage D-2) 포함
- 누적 회귀 정정 (Stage A+D → D-2 → D-2 보완) 완전 일치
- `mydocs/tech/document_ir_lineseg_standard.md` 신규 자료 영구 보존
- 신규 회귀 차단 가드 (옵션): `samples/hwp3-sample5-hwp5-v2018.hwp` (296KB) + `-v2024.hwp` (285KB) 영구 보존 → HWP5 변환본 일치 영구 측정

**잠재 위험**:
- 31 files / +3,348 LOC / 1 파서 +200 LOC — 큰 묶음, 본 환경 광범위 sweep (162 fixture) 의무
- HWP3 파서 8 항목 동시 정정 → 다른 fixture 회귀 측정 필수
- 영구 보존 샘플 581KB 추가 → samples/ 영구 보존 가/부 결정 필요

### 옵션 A-2 — squash 머지 (1 단일 commit)

**진행 명령**:
```bash
git checkout devel && git checkout -b local/pr609_squash
git merge --squash local/pr609
git commit --author="Taesup Jang <tsjang@gmail.com>" -m "Task #604: ..."
```

**옵션 A 와의 차이**:
| 항목 | 옵션 A | 옵션 A-2 |
|------|--------|---------|
| commit 수 | 11 (각 Stage 보존) | 1 (단일 squash) |
| author 보존 | 11 commits 모두 jangster77 | 1 commit jangster77 |
| committer | edward (cherry-pick 메인테이너) | edward |
| bisect | Stage 별 분리 가능 | 단일 commit |
| devel 가독성 | Stage 분리 보존 | 단일 commit |
| diff | 31 files (Stage 별 분포) | 31 files (단일 묶음) |

**옵션 A-2 가 적합한 경우**:
- 본 PR 의 Stage 가 1 단위 (Document IR 표준 정합화) 라고 판단 시
- Stage 5 B-2 같은 revert 된 중간 stage 가 devel history 에 부담으로 인지될 시
- bisect 분해보다 devel 가독성이 우선되는 사이클 (예: v1.0.0 직전)

**옵션 A 가 적합한 경우**:
- Stage 별 분리 (Stage 1 IR 표준 / Stage 2 메타데이터 / Stage 3 HWP3 cs/sw / Stage A+D HWP5 정합 등) 를 bisect 에 보존 의지
- 본 PR 의 "11 commits 단계별 + 회귀 추적" 패턴 (`feedback_v076_regression_origin` 관점) 적용

**본 사이클의 컨트리뷰터별 패턴**:
- PR #589 (jangster77 직전 PR) — 옵션 A 7 commits 단계별 cherry-pick 머지 (`5ba7f2e`)
- PR #599 (seo-rii) — 옵션 A 9 commits 단계별 cherry-pick 머지 + 5 후속 정정 (`f7d5563`)
- PR #642 (postmelee) — 합본 squash 5 commits → 1 commit (`17434e9`)
- PR #601 (oksure) — 옵션 A-2 합본 squash 2 commits → 1 commit (`0059557`)

→ squash vs 단계별 분리 결정 = 컨트리뷰터의 단계 보존 vs PR 단위 정리 의 trade-off

### 옵션 B — 부분 cherry-pick (Stage 1~2b 만 우선 머지)

**진행 명령**:
```bash
git checkout devel && git checkout -b local/pr609_partial
git cherry-pick 4eaf52b 7c15d43 08cd66f
git checkout devel && git merge --no-ff local/pr609_partial \
  -m "Merge: Task #604 Stage 1~2b only (IR 부채 청산)"
# Stage 3~D-2 는 별도 PR
```

**본 환경 cherry-pick simulation 결과**:
- 3 commits 충돌 0 (`paragraph.rs` / `layout.rs` / `paragraph_layout.rs` auto-merge)
- `cargo test --lib --release` 1141 passed (회귀 0)
- `cargo test --test issue_554 --release` 12/12 passed
- `cargo test --test issue_546 --release` 1/1 passed

**측정 결과**:
- Stage 3 (HWP3 cs/sw 단방향 가드 정정 = Issue #604 결함 정정 commit) 미포함 상태에서도 issue_554 12/12 통과
- 즉 본 환경 통합 테스트 (svg_snapshot / issue_546 / issue_554) 는 Issue #604 결함 (page 4 그림 좌측 텍스트 겹침) 을 측정하지 않음
- Issue #604 결함은 시각 판정 ★ 단계 (`hwp3-sample5.hwp` page 4 의 graphical layout — cs=0/sw=0 lines 의 x 좌표) 에서만 발현

**옵션 B 의 범위**:
- IR 부채 청산 (wrap_precomputed 필드 제거 + HWP3 후처리 30 LOC 청산) 만 우선 진행
- HWP3 파서 cs/sw 가드 (Stage 3) + HWP5 IR 표준 8 항목 정정 (Stage A+D+D-2) 은 별도 PR

**장점**:
- 회귀 위험 좁힘 (Stage 1~2b = IR 인프라만, HWP3 파서 정정 분리)
- IR 표준 명문화 + 메타데이터 채널 도입 → 향후 모든 wrap zone 작업의 인프라

**잠재 위험 (본 환경 직접 검증)**:
- ⚠️ Stage 3 미포함 시 Issue #604 결함 (page 4 그림 좌측 텍스트 겹침) 미정정 잔존 — 시각 판정 ★ 에서 결함 그대로
- ⚠️ Stage A+D + D-2 의 HWP5 IR 표준 8 항목 정정 미포함 — HWP3/HWP5/HWPX 3개 포맷 동일 페이지 일치 미달성
- Stage 1~2b 만 머지 시 PR 의 핵심 (Issue #604 closes) 미달성 → `closes #604` 키워드 자동 처리 안 됨 → 컨트리뷰터에게 후속 PR 권유 필요
- 컨트리뷰터 부담 (이미 D-2 까지 작업한 11 commits 의 절반만 머지)

### 옵션 C — 컨트리뷰터에게 base 동기화 요청 후 재검토

**진행 명령**:
```bash
# 컨트리뷰터에게 댓글 — fork 의 local/task604 에 origin/devel 동기화 요청
# 컨트리뷰터: git checkout local/task604 && git merge devel && git push
```

**본 환경 BEHIND 측정**:
- PR base 시점: `780df3c` (5/5 11:13 UTC)
- 본 환경 devel 의 base 이후 commits: 9 (`a7087a2` PR #601 1차 검토 → `e7ae428` HWPX 가설 보고서)
- BEHIND 의 src 충돌: 0 (옵션 A simulation 에서 검증됨)

**옵션 C 가 적합한 경우**:
- BEHIND 에 src 충돌 발생 시
- 본 환경 정합 변경 필요 시

**결론**: 본 PR 은 BEHIND 측정에서 src 충돌 0. 본 환경 cherry-pick 으로 BEHIND 자체 처리 가능 → 옵션 C 미필요. (PR #571 같은 컨트리뷰터 fork 가 본 devel 보다 뒤 + src 회귀 발생 시의 옵션)

### 옵션 D — close 후 재구성 PR 권유

**진행 명령**:
컨트리뷰터에게 PR close + 본 PR 을 작은 단위 PR (Stage 1~2b / Stage 3 / Stage 5+6 / Stage A+D+D-2) 4 PR 로 분리해 재제출 권유.

**옵션 D 가 적합한 경우**:
- PR 이 단일 단위로 너무 큼 + 분리 가능 + 각 단위 독립 검증 가능 시
- 회귀 위험이 단위 별로 다름 시 (예: IR 인프라 vs HWP3 파서 vs HWP5 정밀)

**옵션 D 가 부적합한 경우 (본 PR)**:
- Stage 1~2b → Stage 3 → Stage A+D → Stage D-2 의 누적 의존 (Stage 5 B-2 가 Stage A+D 에서 revert 되는 등) — 분리 시 검증 부담 폭증
- 컨트리뷰터의 이미 완료된 11 commits 작업 분할 재제출 부담
- PR 의 결정적 검증 (cargo test 1131 / svg_snapshot 6/6 / issue_554 12/12 / clippy 0) 모두 통과 → close 사유 부재

**결론**: 본 PR 은 close 사유 부재 → 옵션 D 미적용.

### 권장 — 옵션 A 또는 옵션 A-2

#### 옵션 A 권장 사유 (단계별 보존)
1. Stage 별 분리가 명료 (Stage 1 IR 표준 / Stage 2 메타데이터 / Stage 3 HWP3 cs/sw / Stage A+D+D-2 HWP5 정합 / Stage 6 z-order)
2. Stage 5 B-2 revert history 보존 (향후 bisect 시 추적 가능)
3. 회귀 origin 추적 (`feedback_close_issue_verify_merged` 관점) — 향후 동일 결함 발생 시 Stage 별 bisect 가능
4. PR #589 (직전 jangster77 PR) 의 옵션 A 패턴 일관 적용

#### 옵션 A-2 권장 사유 (squash)
1. PR 단위가 "Document IR 표준 정합화" 단일 의도 — Stage 분리보다 PR 단위 정리 우선
2. Stage 5 B-2 revert 등 중간 stage 가 squash 에 자연 흡수
3. devel 정리 — 11 작업 commits + 11 merge commits = 22 commits 의 squash
4. PR #642 (postmelee 5 commits) + PR #601 (oksure 2 commits) 본 사이클 squash 패턴 일치

**작업지시자 결정**: 옵션 A (Stage 별 보존) vs 옵션 A-2 (devel 정리)

### 옵션 요약

| 옵션 | 진행 가능 | Issue #604 정정 | 결정적 검증 | author | devel 영향 | 권장 |
|------|----------|----------------|------------|--------|------------|------|
| A (11 commits) | ✅ 충돌 0 | ✅ Stage 3 + D-2 포함 | ✅ 1131/12/6 | 11 commits | Stage 별 보존 | ⭐ |
| A-2 (squash) | ✅ 충돌 0 | ✅ 합본 포함 | ✅ 1131/12/6 | 1 commit | 단일 commit | ⭐ |
| B (Stage 1~2b 만) | ✅ 충돌 0 | ⚠️ 미정정 잔존 | ✅ 통합 테스트 통과 (시각 결함 미발현) | 3 commits | 부분 적용 | ❌ |
| C (base 동기화 요청) | — | — | — | — | 컨트리뷰터 부담 | ❌ |
| D (close + 재제출) | — | — | — | — | close 사유 부재 | ❌ |

---

## 7. 잠정 결정

### 권장
- 옵션 A 또는 A-2 진행 + 본 환경 결정적 검증 + 작업지시자 시각 판정 ★
- 영구 보존 가/부: `samples/hwp3-sample5-hwp5-v2018.hwp` + `-v2024.hwp` (작업지시자 결정)
- 잔존 4 항목 별도 task 분리 (PR 본문 §잔존)

### 후속 task 후보 (PR 본문 §잔존)

> **주의**: 컨트리뷰터의 commit message 에서 후속 후보로 명시한 항목들. 본 환경 권위 정답지는 한컴 2010 + 한컴 2022 의 **편집기** 출력 (`reference_authoritative_hancom`). 한컴뷰어 / HWP5 v2024 변환본 / macOS 인쇄 / 외부 변환은 정답지 아님. 따라서 아래 항목은 **본 환경 기준 한컴 편집기 시각 판정 후 등록 가/부 결정**.

1. HWP3 폰트 크기/줄간격 차이 — 한컴 편집기 시각 판정 결과 결함 확인 시 후속 task 등록 후보 (컨트리뷰터의 "한컴뷰어 13pt vs HWP5 v2024 9pt" 표현은 권위 근거 아님, 본 환경에서 한컴 편집기 시각 판정 필요)
2. HWP3 LineSeg vertical_pos 누적 계산 — Stage A+D+D-2 의 누적 정정으로 sample5 64p 일치 도달, 잔존 영역의 발현 여부는 본 환경 광범위 sweep + 한컴 편집기 시각 판정으로 측정
3. Task #525 재검토 (wrap_anchors 도입 후 `layout_wrap_around_paras` dead code 가능성) — 본 PR 머지 후 본 환경에서 dead code 검증 가능 (코드 검증, 한컴 비교 영역 외)
4. 한컴 변환기 paragraph indent 흡수 휴리스틱 — sample4 pi=960 의 HWP3 native (TAB 4개 + indent 660 HU) vs HWP5 변환본 (TAB 2개 + indent 1320 HU) 차이. 본 환경에서 한컴 편집기 시각 판정 + 어느 쪽이 한컴 편집기 출력과 일치하는지 결정 필요

### 검증 항목 (옵션 A 진행 시 본 환경 직접 점검)
1. `cargo test --lib --release` 1131 passed
2. `cargo test --test svg_snapshot --release` 6/6
3. `cargo test --test issue_546` 통과
4. `cargo test --test issue_554` 12/12
5. `cargo clippy --lib -- -D warnings` 0
6. `cargo build --release`
7. Docker WASM 빌드 + byte 측정
8. `rhwp-studio npm run build` TypeScript 타입 체크 + dist 빌드
9. 광범위 페이지네이션 회귀 sweep — `samples/` 162 fixture 의 페이지 수 차이 측정
10. 시각 판정 ★ — `hwp3-sample5.hwp` page 4 + 8/16/22/27 작업지시자 판정

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- `feedback_assign_issue_before_work` — Issue #604 assignee 미지정 (외부 컨트리뷰터 자기 등록, 일차 방어선 부재 사례)
- `reference_authoritative_hancom` — 한컴 2010 + 한컴 2022 의 **편집기** 출력만 권위 정답지. 한컴뷰어 / HWP5 v2024 변환본 / macOS 인쇄 / 외부 변환은 정답지 아님. 본 PR 의 PR 본문 + commit message 에서 "HWP5 v2024 변환본 정합" / "한컴뷰어 13pt vs ..." 표현은 컨트리뷰터의 자기 환경 측정 기준이며 본 환경 권위 영역 아님.
- `feedback_pdf_not_authoritative` — PDF 출력은 환경별 차이로 정답지 미입증. 본 PR 의 PR 본문 표 ("HWP5 v2024 정합") 도 컨트리뷰터의 비교 기준이므로 본 환경 결정적 검증 + 한컴 편집기 시각 판정으로 재측정 필요.
- `feedback_rule_not_heuristic` (`feedback_hancom_compat_specific_over_general`) — wrap_precomputed 휴리스틱 청산 + 메타데이터 채널 (anchor 종류 기반)
- `feedback_close_issue_verify_merged` — Issue #604 close 시 본 PR 머지 검증 (closes #604 키워드 자동 처리 안 될 가능성)
- `feedback_visual_regression_grows` — 페이지 수 일치 + 시각 판정 ★ 결합
- `project_hwpx_to_hwp_adapter_limit` — HWP3/HWP5/HWPX 3개 포맷 일치 진전

---

## 9. 옵션 A 진행 결과 (본 환경 직접 검증)

### 작업지시자 결정 (5/7)
- **옵션 A** 진행 (11 commits cherry-pick, author 보존)
- **권위 자료 영구 보존** — `samples/hwp3-sample5-hwp5-v2018.hwp` + `-v2024.hwp` git tracked
- **후속 task 등록** — 한컴 편집기 시각 판정 후 결정 (현 시점 미등록)

### cherry-pick 결과
```
b3b1019 Task #604 Stage D-2: 단계별 보고서 + 최종 보고서 + orders 갱신
6c399c8 Task #604 Stage D-2 (보완): paragraph 내 line wrap vpos reset 정합
3503260 Task #604 Stage D-2: HWP5 IR 표준 정합 정정 본질 도달
376ee59 Task #604 Stage A + D: HWP5 IR 표준 정밀 재진단 + HWP3 파서 정합 인코딩
9ac613e Task #604 Stage 6: paper_images z-order 정합 (한컴 변환 시각 정합 완성)
578b574 Task #604 Stage 5 (옵션 B-2): wrap zone 안 라인의 line_spacing 100% 정합화
536c90d Task #604 Stage 4: 광범위 회귀 검증 + 시각 판정 + 최종 보고서
825f8df Task #604 Stage 3: HWP3 파서 cs/sw 인코딩 정정 (Issue #604 결함 본질 정정)
6abf4ae Task #604 Stage 2b: wrap_precomputed 필드 제거 + HWP3 후처리 청산 (IR 부채 마무리)
0173153 Task #604 Stage 2: typeset 출력 메타데이터 (wrap_anchors) 도입
fa4924f Task #604 Stage 1: Document IR LineSeg 표준 정의 + is_in_wrap_zone helper
```
11 commits 모두 충돌 0 통과 (auto-merge: `paragraph.rs` / `layout.rs` / `paragraph_layout.rs` / `mydocs/orders/20260505.md`).

### 결정적 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1141 passed** (PR 본문 1131 baseline 대비 +10, 본 환경 누적 정합) |
| `cargo test --test svg_snapshot --release` | ✅ 6/6 |
| `cargo test --test issue_546 --release` | ✅ 1/1 |
| `cargo test --test issue_554 --release` | ✅ **12/12** |
| `cargo clippy --lib -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,598,892 bytes** (PR #601 baseline 4,588,023 +10,869, Stage 1~D-2 누적 정합) |
| `rhwp-studio npm run build` | ✅ TypeScript 타입 체크 + dist 빌드 (`index-DMU370fc.js` 691,386 / `rhwp_bg-BMG1HwDs.wasm` 4,598,892) |

### 광범위 페이지네이션 sweep

| 영역 | 결과 |
|------|------|
| BEFORE (local/devel) | 232 fixture |
| AFTER (local/pr609) | 234 fixture (신규 v2018/v2024 추가) |
| 페이지 수 차이 | **0** (231 fixture BEFORE/AFTER 일치, 신규 2 파일 64 페이지 추가) |
| `hwp3-sample.hwp` | 16 / 16 (일치) |
| `hwp3-sample4.hwp` (native) | 39 / 39 (BEFORE/AFTER 변동 없음) |
| `hwp3-sample5.hwp` (native) | 64 / 64 (일치) |
| `hwp3-sample5-hwp5-v2018.hwp` | (부재) / 64 (신규 영구 보존) |
| `hwp3-sample5-hwp5-v2024.hwp` | (부재) / 64 (신규 영구 보존) |

### PR 본문 표와의 차이 — `hwp3-sample4.hwp`

PR 본문 표 ("`hwp3-sample4.hwp` baseline HWP5 36 / 정정 결과 36") 의 의미:
- **HWP5 변환본 (`hwp3-sample4-hwp5.hwp`) 의 페이지 수 영역** — `cargo test --test issue_554` 의 `hwp3_sample4_hwp5_36p` test 가 측정 (통과)
- **HWP3 native (`hwp3-sample4.hwp`) 자체의 페이지 수** — BEFORE/AFTER 모두 39 페이지 (변동 없음)

> PR 본문 표의 표기가 모호 — "baseline HWP5 36" 표기로 HWP5 변환본 영역임을 암시하지만 구분 명시 부재. 본 환경 직접 측정으로 native HWP3 자체는 39 페이지로 변동 없음 확인. 회귀 0.

### 시각 판정 자료

본 환경 SVG 출력 — 작업지시자가 한컴 2010 + 한컴 2022 **편집기** 출력과 비교 판정:

- `output/svg/pr609_after/hwp3-sample5/` (64 페이지 전체) — Issue #604 권위 영역
  - page 4 (`hwp3-sample5_004.svg`) — pi=74 그림 (User level programs 다이어그램) + pi=75 wrap text 위치 확인 영역
  - page 8 / 16 / 22 / 27 — 동일 패턴 정합 확인 영역
  - page 43 (`hwp3-sample5_043.svg`) — pi=1213 paragraph 내 페이지 reset 영역
- `output/svg/pr609_after/hwp3-sample4/` (39 페이지 전체) — sample4 native HWP3 영역

### 작업지시자 시각 판정 요청

`output/svg/pr609_after/hwp3-sample5/` 의 SVG (특히 page 4 + 8/16/22/27 + 43) 를 한컴 2010/2022 편집기 출력과 비교해서 시각 판정 ★ 부탁드립니다. 결과에 따라:
- 시각 판정 ★ 통과 → 옵션 A 머지 진행 + `pr_609_report.md` 작성 + Issue #604 close
- 시각 판정 영역 결함 발견 → 결함 보고 + 후속 영역 결정

### 다음 단계

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (완료) `pr_609_review.md` 작성 → 승인
3. (완료) 옵션 A cherry-pick + 결정적 검증 + WASM 빌드 + 광범위 sweep
4. (대기) 작업지시자 시각 판정 ★ → `pr_609_report.md` 작성
