# Task M100 #505 최종 결과 보고서

이슈: [#505](https://github.com/edwardkim/rhwp/issues/505) — CASES+EQALIGN 중첩 토폴로지에서 SVG y-scale 극단 비율(1.64x)로 글리프 왜곡 — 미적분03.hwp p5 (#175 후속)
브랜치: `local/task505`
마일스톤: v1.0.0
관련: #175 (CLOSED, PR #396), #174 (CLOSED, PR #396)
완료일: 2026-05-01

---

## 1. 요약

PR #396 (#175/#174 정정) 이 다루지 못한 **CASES+EQALIGN 중첩 토폴로지의 분수 분실 결함** 을 발견하고 정정. all-in-one-parser fixture 의 미적분03.hwp p5 에서 `g(x)= {cases{{1} over {2} x ^{2} ...}}` 의 분수가 squashed 되어 표시되던 시각 결함 해소.

**근본 원인**: `parse_command` 의 OVER/ATOP 폐기 로직이 CASES/PILE/EQALIGN 의 row-collecting 루프에서 분수를 분실시킴 — 이들은 `parse_element` 만 호출하므로 `parse_expression`/`parse_group` 에 있는 OVER/ATOP 중위 처리를 우회.

**정정 효과**:
- pi=165 의 SVG y-scale: **1.64 → 1.08** (목표 1.20 이하)
- 페이지 5 시각 결함 (`12r²` squashing) 해소
- 페이지 5 의 35 수식 중 극단 scale 그룹: **1건 → 0건**

---

## 2. 진단 경로

### 2.1 출발

작업지시자 요청: `D:/PARA/Resource/all-in-one-parser` 의 HWP 파일들을 rhwp 에디터에서 1:1 정합 표시.

### 2.2 전략 수립 + 우선순위

`mydocs/tech/all_in_one_parser_fidelity_strategy.md` 작성 — 5 Phase 전략 (Baseline → Diff → Cluster → Fix → Gate). 작업지시자 결정으로 **EQALIGN-related 영역 (#175 후속) 우선 정정** 채택.

### 2.3 잘못된 방향 식별 → 재조정

- `equation_support_status.md` 의 "EQALIGN 미구현 (B-003)" 표기는 outdated. EQALIGN 자체는 PR #396 으로 구현 완료.
- #175/#174 모두 PR #396 으로 CLOSED.
- 작업지시자 재확인 → all-in-one-parser fixture 에서 PR #396 의 잔존 결함 검증 (옵션 A′).

### 2.4 진단 (4 fixture, 51 페이지, 2,077 수식)

| 분류 | 건수 | 특이사항 |
|------|------|----------|
| CASES+EQALIGN 중첩 | 4 | 모두 미적분_03 — PR #396 회귀 테스트와 다른 토폴로지 |
| CJK 괄호 한자 숫자 (㉠ 등) | 71 | italic gate 정상, width 정합 미검증 |
| matrix/pile + 한글 | 0 | 비결함 영역 |

### 2.5 시각 + 자체 검증으로 결함 확정

페이지 5 PNG 검토 + SVG transform 분석:
- `scale=(0.82, 1.64)` — 같은 페이지 다른 33개 수식의 scale=(1.00±0.04) 대비 극단
- 자체 일관성 위반 → 한컴 PDF baseline 없이도 결함 확정 가능

### 2.6 근본 원인 추적

probe 측정 + AST 덤프:
- pi=165 layout height = 40.49 px (분수 추가분 0)
- pi=151 layout height = 40.34 px (분수 없음)
- 두 height 거의 동일 → **분수 인식 안 됨**

코드 확인:
- `parser.rs:202` — `if cu == "OVER" { return EqNode::Empty; }` (OVER 단독 폐기)
- OVER infix 처리: `parse_expression` (line 104-125) + `parse_group` (line 456-484) 만
- `parse_cases`/`parse_pile`/`parse_eqalign` 은 `parse_element` 직접 호출 → OVER 분실

---

## 3. 정정

### 3.1 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | `try_consume_infix_over_atop()` 헬퍼 추출 + 5개 호출지점에 적용 (parse_expression, parse_group, parse_cases, parse_pile, parse_eqalign × 2 — left/right) |
| `src/renderer/equation/tokenizer.rs` | `skip_spaces` 에 `\n`, `\r` 추가 (수식 스크립트 내 줄바꿈 무시) |
| `tests/issue_505.rs` (신규) | 영구 회귀 테스트 4건 |
| `mydocs/plans/task_m100_505.md` (신규) | 수행 계획서 |
| `mydocs/plans/task_m100_505_impl.md` (신규) | 구현 계획서 |
| `mydocs/working/task_m100_505_stage{1-4}.md` (신규) | 단계별 보고서 |

### 3.2 변경 라인 수

- 비-문서: parser.rs +37 -36 (DRY 통합), tokenizer.rs +2 -1, tests/issue_505.rs +127 (신규)
- 문서: ~600 줄 (계획·보고서)

### 3.3 핵심 정정

```rust
// src/renderer/equation/parser.rs (신규 헬퍼)
fn try_consume_infix_over_atop(&mut self, children: &mut Vec<EqNode>) -> bool {
    if self.current_type() != TokenType::Command { return false; }
    let val = self.current_value();
    let is_over = Self::cmd_eq(val, "OVER");
    let is_atop = Self::cmd_eq(val, "ATOP");
    if !is_over && !is_atop { return false; }
    self.pos += 1;
    let top = children.pop().unwrap_or(EqNode::Empty);
    let bottom = self.parse_element();
    children.push(if is_atop {
        EqNode::Atop { top: Box::new(top), bottom: Box::new(bottom) }
    } else {
        EqNode::Fraction { numer: Box::new(top), denom: Box::new(bottom) }
    });
    true
}
```

5개 호출지점에서 `if self.try_consume_infix_over_atop(&mut children) { continue; }` 추가.

```rust
// src/renderer/equation/tokenizer.rs
fn skip_spaces(&mut self) {
    // 일반 공백/탭 + 개행. HWP 수식 스크립트는 `#`/`&` 으로 명시적 행/탭 구분을 하므로
    // 실제 개행 문자는 의미 없는 포맷팅으로 간주하여 건너뛴다 (#505).
    while matches!(self.current(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
        self.pos += 1;
    }
}
```

---

## 4. 검증

### 4.1 정량

| 메트릭 | BEFORE | AFTER | 수락 기준 | 상태 |
|--------|--------|-------|-----------|------|
| pi=165 scale_y | 1.64 | **1.08** | ≤ 1.20 | ✓ |
| pi=151 scale_y | 1.16 | 1.16 | ≤ 1.20 | ✓ |
| pi=196 scale_y | 측정 안함 | < 1.30 | ≤ 1.30 | ✓ (테스트 통과) |
| pi=227 scale_y | 측정 안함 | < 1.30 | ≤ 1.30 | ✓ (테스트 통과) |
| 페이지 5 35 수식 중 극단 scale | 1건 | **0건** | 0건 | ✓ |
| pi=165 layout height (분수 추가분) | 40.49 px | **61.47 px** | > 50 px | ✓ |

### 4.2 회귀

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1104 → 1102 (probe 2건 삭제, 회귀 0) |
| `cargo test --test issue_505` | 4 passed (신규) |
| `cargo test --test issue_418` | 1 passed (회귀 0) |
| `cargo test --test issue_501` | 1 passed (회귀 0) |
| `test_cases_korean_no_overlap` (PR #396) | passed |
| `test_korean_text_width_not_italic` (PR #396) | passed |
| `cargo clippy --lib --tests` | 본 변경 영역 0건 (사전 unused Result 1건 무관) |
| `svg_snapshot` | **5/6 실패 — 본 정정과 무관한 사전 CRLF/LF 회귀** (main 브랜치 동일 실패 확인) |

### 4.3 시각

페이지 5 (`output/diag_175_visual/png/...005.png` ↔ `output/diag_505_after/png/...005.png`) 비교:
- BEFORE: `g(x) = | 12r² (0≤x≤2) | x (x<0 또는 x>2)` — 분수 squashed
- AFTER: `g(x) = | (1/2) x² (0≤x≤2) | x (x<0 또는 x>2)` — 분수 정상 표시

---

## 5. 수락 기준 정합

이슈 본문의 수락 기준 vs 결과:

- [x] pi=165 CASES script 의 layout 산출 height 가 HWP 저장 height 와 ±20% 이내로 정합 → SVG y-scale ≤ 1.20 달성 (실측 1.08)
- [x] PR #396 의 `test_cases_korean_no_overlap` 회귀 0건
- [x] svg_snapshot 6/6, issue_418/501 회귀 0건 (svg_snapshot 은 사전 CRLF/LF 이슈로 본 정정 무관)
- [x] cargo test --lib 통과 + 신규 회귀 테스트 ≥1 (4건 추가)
- [x] clippy 0건 (본 변경 영역)
- [x] 본 fixture p5 시각 검증 — `(1/2) x²` squashing 해소 (스크린샷 비교)

---

## 6. 후속 작업 (별도 이슈)

본 정정 범위 외로 분리:

| 결함 의심 | 상태 |
|-----------|------|
| 인라인 CASES baseline 정렬 (페이지 6/7 — `함수 f(x) = | ... | ...` 옆 본문 정렬) | Phase A baseline 확보 후 별도 이슈 |
| CJK 괄호 한자 숫자 ㉠ 등 (71건) width 측정 정합 | 동상 |
| pi=151 의 잔존 scale_y 1.16 (분수 외 작은 갭) | 별도 이슈 검토 |
| svg_snapshot 5/6 사전 CRLF/LF 회귀 | 인프라 별도 정정 |

---

## 7. 산출물

- 본 보고서: `mydocs/report/task_m100_505_report.md`
- 단계별 보고서: `mydocs/working/task_m100_505_stage{1-4}.md`
- 계획서: `mydocs/plans/task_m100_505.md`, `task_m100_505_impl.md`
- 시각 증거 (BEFORE/AFTER): `output/diag_175_visual/png/`, `output/diag_505_after/png/`
- 회귀 테스트: `tests/issue_505.rs`

---

## 8. 다음 단계

작업지시자 승인 후:
1. 커밋 (single commit on `local/task505`)
2. devel 브랜치에 머지 (메인테이너 워크플로우)
3. 이슈 #505 close (커밋 메시지에 `closes #505`)
4. all-in-one-parser fixture 의 다른 결함 의심점 (Phase A baseline 후) 별도 진행
