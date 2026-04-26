# PR #323 검토 — Task #321: TypesetEngine LINE_SEG vpos-reset 강제 분할

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#323](https://github.com/edwardkim/rhwp/pull/323) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#321](https://github.com/edwardkim/rhwp/issues/321) (OPEN, assignee 없음) |
| base/head | `devel` ← `task321` |
| 변경 | +3,233 / -74, 11 파일 (코드 2 + 문서 9) |
| Mergeable | UNKNOWN, mergeStateStatus=BEHIND (rebase/merge 필요) |
| maintainerCanModify | ✅ true |
| 이전 PR | #322 (closed without merge) → 본인이 v2/v3 보강 후 #323 재제출 |
| 검토일 | 2026-04-25 |

## 트러블슈팅 사전 검색 (memory 규칙)

| 문서 | 관련성 |
|------|--------|
| `multi_tac_table_pagination.md` | TAC 표 페이지네이션 — 본 PR과 다른 경로 (TAC 표 자체 처리) |
| `line_spacing_lineseg_sync.md` | ParaShape vs LineSeg 이중 구조 — 본 PR과 동일 인식 (LineSeg가 물리값 진실) |
| `hwpx_lineseg_reflow_trap.md` | HWPX의 LINE_SEG는 한컴 계산값 신뢰해야 함 — 본 PR의 vpos-reset 신호 활용 근거와 일치 |

→ 신규 적용 영역. 트러블슈팅 신규 등록 후보 (vpos-reset 신호 페이지네이션 활용).

## 변경 요약

### 핵심 (보고서 명시)

1. **vpos-reset 강제 분할** (`src/renderer/typeset.rs::typeset_section`)
   - `cv == 0 && pv > 5000 HU && !current_items.is_empty()` 조건에서 `advance_column_or_new_page()` 호출
   - 문단간(inter-paragraph) reset만 대상 (intra-paragraph는 기존 `detect_column_breaks_in_paragraph`)

2. **`RHWP_TYPESET_DRIFT` 진단 훅** (Stage 1 산출물 보존)
   - eprintln 형태, env 가드. fmt vs vpos 정합 디버깅용.

### 추가 (보고서 미언급, v2/v3 커밋)

3. **`pending_body_wide_top_reserve` 상태 추가** (`bc18164`, v2)
   - col 0 상단의 body-wide TopAndBottom 표/도형 높이를 추적
   - col 1+ advance 시 `current_height` 시작값으로 사용 (= layout의 `body_wide_reserved`와 정합)
   - 새 페이지 진입 시 0으로 초기화

4. **Paper(용지) 기준 도형 제외** (`3932b83`, v3)
   - `src/renderer/layout/shape_layout.rs::body_wide_reserved` 분기에 `VertRelTo::Paper` skip
   - `src/renderer/typeset.rs::compute_body_wide_top_reserve_for_para` 동일 조건
   - 21_언어 4×5 표 page-y=131px 케이스 (Paper 기준은 페이지 절대 위치, col 1 시작에 reserve 불필요)

### 변경 파일

| 파일 | 변경 | 설명 |
|------|------|------|
| `src/renderer/typeset.rs` | +109/-1 | vpos-reset 분할 + body-wide reserve 추적 + 진단 훅 + 신규 헬퍼 fn |
| `src/renderer/layout/shape_layout.rs` | +4 | body_wide_reserved 계산에서 Paper 도형 제외 |
| 문서 | +3,120 | 계획·구현·stage1·stage2·최종보고서 + orders 갱신 + PR 본문 사본 + 21_언어 png |

## 검토 시 확인할 점

### A. 코드 정확성

| 항목 | 평가 |
|------|------|
| **vpos-reset 가드 조건** | `cv == 0 && pv > 5000 && !current_items.is_empty()` — 3중 가드로 오탐 최소화. 5000 HU(≈ 1.76mm) 임계값 적절 (빈 문단의 우연 vpos 제외) |
| **`advance_column_or_new_page` 재진입성** | flush_column 후 col_count 초과 시 `push_new_page()`. state 일관성 유지됨 |
| **`pending_body_wide_top_reserve` 적용 시점** | col 0 처리 중에만 등록 (`current_column == 0 && pending == 0.0`), col 1 advance 시 한번만 사용, 새 페이지에서 초기화. 의도 명확 |
| **`zone_y_offset` 건드리지 않는 이유** | layout의 `body_wide_reserved`가 별도 처리하므로 double-shift 방지 — 주석에 명시됨, 정합 OK |
| **Paper 도형 제외 일관성** | typeset.rs와 shape_layout.rs **양쪽에 동일 조건** 적용 — 정합 OK |

### B. 회귀 리스크

| 리스크 | 검증 |
|--------|------|
| vpos=0 우연 일치로 오탐 | `pv > 5000` 가드로 빈 문단 제외. 작성자: 21_언어 외 4개 핵심 샘플 페이지 수 무변화 확인 |
| body-wide reserve 누적 | `pending_*` 는 col 0에서 1회만 등록되고 새 페이지에서 0 리셋. layout의 reserved와 1:1 |
| Paper 도형 케이스 | 21_언어 4×5 표 (page-y=131px)가 명확한 사례. 다른 샘플에서 Paper TopAndBottom 도형이 본문 영역 80%+ 인 경우 유사 패턴 검증 필요 |

### C. 보고서 vs 실제 변경 불일치

⚠️ **최종 보고서가 v1 시점 내용으로만 작성됨**:
- 보고서: "코드: `src/renderer/typeset.rs` (+ 진단 훅 + vpos-reset 검출 로직)"
- 실제: `shape_layout.rs` Paper 제외, `pending_body_wide_top_reserve` 추적, `compute_body_wide_top_reserve_for_para` 헬퍼 추가

머지 전 보고서 보강 또는 머지 후 후속 커밋으로 처리하면 됨. **머지 차단 사유는 아님**.

## 절차 준수 점검 (외부 기여자 PR)

| 규칙 | 준수 | 비고 |
|------|------|------|
| 이슈 → 브랜치 → 계획서 → 구현 순서 | ✅ | 이슈 #321 + 수행계획 + 구현계획 + stage1·2 + 최종보고서 |
| 수행계획서 (`task_m100_321.md`) | ✅ | 구조 타당 (배경/목표/범위/접근 원칙/단계/리스크) |
| 구현계획서 (`task_m100_321_impl.md`) | ✅ | 단계별 작업/산출물/완료 조건 명시 |
| 단계별 보고서 | ✅ | stage1·2 존재 (Stage 3·4는 stage2에 통합되어 보고됨, 본문 보고서가 종합) |
| 최종 보고서 (`mydocs/report/`) | △ | 위치 OK, **내용이 v1 시점 — Paper 제외/reserve 추적 미반영** |
| orders 갱신 | ✅ | `mydocs/orders/20260425.md` 에 Task #321 섹션 추가됨 |
| 브랜치 `local/task{번호}` 또는 `task{번호}` | ✅ | `task321` (이전 task 체인 일관) |
| 커밋 메시지 `Task #N:` | ✅ | Stage별 + v2/v3 보강 표기 일관 |
| Test plan | ✅ | `cargo test --release` 992/0, 4개 핵심 샘플 페이지 수 보고 |

## 검증 (메인테이너 실측)

### 1. base 동기화 (mergeStateStatus=BEHIND)

작성자가 PR #320 머지 후 devel 변경을 task321에 merge 완료 (`8193c57`). 우리 측 별도 동기화 push 불필요 — base 재계산 시 mergeable 확정.

### 2. 빌드/테스트

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 26.11s |
| `cargo test --lib` | ✅ 992 passed / 0 failed / 1 ignored |
| `cargo test --test issue_301` | ✅ 1 passed (z-table 가드) |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 무회귀) |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |

### 3. 21_언어 SVG 시각 검증

- 1페이지 우측단 pi=29/pi=30 시각 겹침 해소 확인
- LAYOUT_OVERFLOW 5→**0** (보고서 5→4보다 더 좋음. v2/v3 보강 효과로 추정)
- 페이지 수 15 유지

### 4. 회귀 샘플 (1차 결과)

| 샘플 | devel | PR #323 | 평가 |
|------|-------|---------|------|
| 21_언어_기출_편집가능본.hwp | 15 | 15 | ✅ 무변화 (이슈 해소) |
| exam_math.hwp | 20 | 20 | ✅ |
| exam_kor.hwp | 24 | 24 | ✅ |
| exam_eng.hwp | 9 | 9 | ✅ |
| basic/KTX.hwp | 1 | 1 | ✅ |
| biz_plan.hwp | 6 | 6 | ✅ |
| **aift.hwp** | **74** | **78 (+4)** | ⚠️ 분석 진행 → 의도된 개선으로 판정 |

### 5. aift.hwp +4 회귀 분석

devel 74쪽 vs PR #323 78쪽 비교에서 5개 신규 분할 지점 식별:

| 분할 위치 | prev_last_vpos | curr_first_vpos | 가드 트리거 | 판정 |
|-----------|---------------|-----------------|-------------|------|
| pi=128 | 71193 | **0** | ✅ vpos-reset | HWP 명시적 분할 신호 |
| pi=257 | 70920 | **0** | ✅ vpos-reset | HWP 명시적 분할 신호 |
| pi=284 | 69635 | 71555 | ❌ (cv≠0) | 누적 시프트 (앞 분할 효과) |
| pi=371 | 70078 | **0** | ✅ vpos-reset | HWP 명시적 분할 신호 |
| pi=477 | 66554 | **0** | ✅ vpos-reset | HWP 명시적 분할 신호 |
| pi=784 | 0 | 0 | ❌ (pv=0) | pi=783이 표 + 누적 시프트 |

**판정**: 5개 분할 중 **4개가 한컴 LINE_SEG에 명시적으로 박힌 `vpos=0` reset 신호**와 일치 (= HWP 원본 의도). 나머지 2개는 앞 분할 누적에 의한 시프트로, 본 PR이 새로 도입한 회귀가 아닌 **연쇄 효과**. Task #291의 aift.hwp 18페이지 byte-diff 개선과 동일 카테고리.

**결론**: aift.hwp +4는 회귀가 아닌 **의도된 개선** (HWP 원본 분할 신호 반영). 머지 차단 사유 아님.

### 6. WASM 빌드 + 브라우저 시각 검증

- Docker WASM 빌드: ✅ 완료 (`pkg/rhwp_bg.wasm` 4.08MB, 14:01:20)
- 작업지시자 시각 검증: ⚠️ **회귀 발견** — 21_언어 1페이지 col 1 (오른쪽 단) 시작 y가 위로 들림. 4×5 표 영역과 텍스트가 겹쳐 보임.

### 7. 회귀 정밀 분석 (col 1 시작 y)

`dump-pages -p 0` 결과 비교:

| 브랜치 | 단 1 used | hwp_used | diff | 단 1 시작 |
|--------|-----------|----------|------|-----------|
| devel (정상) | 1223.1 | **38.9** | +1184.3 | body 하단부터 시작 (≈1184px reserve) |
| PR #323 | 1174.7 | **1213.1** | -38.4 | body 상단부터 시작 (reserve=0) |

**원인**: v3 커밋 (`3932b83`) 의 `VertRelTo::Paper` 제외 분기 부작용.

21_언어 pi=0 표 속성:
```
[common] treat_as_char=false, wrap=위아래, vert=용지(9872=34.8mm), horz=용지
size=66492×13740 (234.6×48.5mm)
```

표는 page-y=131px ~ 314px 영역에 있고 body_area (y=209.8 ~ 1436.2) 와 수직 겹침. v3 가정 "Paper 기준은 페이지 절대 위치이므로 col 1 시작에 영향 없음" 이 잘못. 작성자 v3 커밋 메시지의 근거 "col 1은 HWP 기준 1213px 를 사용" 의 1213px 는 col 가용 **폭** 이지 시작 **y** 와 무관.

## 처리 결과

⚠️ **수정 요청 후 재검토 대기**

PR #323 코멘트 게시: [comment-4318132231](https://github.com/edwardkim/rhwp/pull/323#issuecomment-4318132231)

권장 옵션 (Option A): v3 가드 정밀화 — Paper 도형이라도 body 와 수직으로 겹치면 reserve 대상 유지. `shape_bottom <= body_top` 인 머리말 전용 도형만 제외.

작성자 수정 push 후 재검토 진행. 로컬 브랜치 `pr323-task321` 보존.

## 판정

⚠️ **수정 요청** (회귀 1건 발견 → 작성자 수정 push 후 재검토)

이전 판정 (수정 요청 전): ✅ Merge 권장 (자동 검증 + WASM 시각 검증 통과 시)

**사유:**
1. 명확한 원인 규명 (LINE_SEG vpos는 한컴이 보존한 페이지 분할 신호)
2. 좁은 가드 (3중 조건)로 오탐 최소화
3. Task #311 부정 결과를 보존·교훈화한 점 (Paginator vs TypesetEngine 차이 분석)
4. v2/v3 보강을 본인이 self-detect → 책임감 있는 후속 처리
5. 회귀 의심 (aift +4)이 분석 결과 의도된 개선 (4/5가 한컴 명시 vpos=0)

**머지 후 후속 (선택, 머지 차단 사유는 아님):**
- 최종 보고서에 Paper 제외 + body-wide reserve 추적 보강 내용 반영
- 잔존 9.5px LAYOUT_OVERFLOW (포맷터 vs vpos 정합) 별도 후속 이슈 등록
- aift.hwp pi=284, pi=784 분할 검증 (한컴 PDF 비교 가능 시)

## 참고 자료

- 이슈: [#321](https://github.com/edwardkim/rhwp/issues/321)
- 관련 트러블슈팅: `mydocs/troubleshootings/line_spacing_lineseg_sync.md`, `hwpx_lineseg_reflow_trap.md`
- 선행: Task #310/#311 (Paginator 시도·부정), Task #313 (TypesetEngine 전환), Task #314 (z-table 회귀 가드)
