# PR #600 검토 보고서

**PR**: [#600 fix: Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정 (closes #513)](https://github.com/edwardkim/rhwp/pull/600)
**작성자**: @oksure (Hyunwoo Park)
**상태**: OPEN, **mergeable=MERGEABLE** (UI 표시)
**관련**: closes #513 (Supplementary PUA-A SVG 출력 누락)
**선행 컨트리뷰터 PR**: PR #581 / #582 / #583 (모두 cherry-pick 완료)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-06

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `convert_pua_enclosed_numbers` (composer) 가 F02B1~F02C4 를 CharOverlap (border_type=3) 으로 먼저 변환 → `map_pua_bullet_char` 의 ①~⑨ 매핑이 도달 못 함
2. **PR base skew** — UI mergeable=MERGEABLE 이지만 PR base 가 본 devel 보다 광범위 뒤처짐 (PR #571/#599 패턴). 그러나 본질 영역 (composer + paragraph_layout) 이 본 사이클 처리분과 충돌 가능성?
3. **Copilot review 응답 정합** — 컨트리뷰터가 commit `aafe85a` 로 응답 (⑩~⑳ 테스트 + 주석 + 폭 계산 정합)

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정 | 정합 |
| author | @oksure (Hyunwoo Park, 이메일 oksure@gmail.com) | PR #581/#582/#583 동일 컨트리뷰터 |
| changedFiles | 2 / +30 / -15 | 매우 작은 본질 변경 |
| 본질 변경 | `composer.rs` (CharOverlap 범위 제외) + `paragraph_layout.rs` (매핑 + 폭 계산) | 단일 본질 |
| **mergeable** | MERGEABLE (UI) | 그러나 PR base skew (PR #571/#599 패턴) |
| Issue | closes #513 | ✅ |

## 3. PR 의 2 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| **`77986135` 본질 정정** | `composer.rs` (border_type=3 범위 축소) + `paragraph_layout.rs` (⑩~⑳ 매핑) | ⭐ cherry-pick |
| **`aafe85a` 리뷰 응답** | Copilot review 3개 코멘트 응답 — ⑩~⑳ 테스트 / 주석 갱신 / 폭 계산 정합 (`map_pua_bullet_char` 후 `estimate_text_width`) | ⭐ cherry-pick |

→ **본질 cherry-pick 대상 = 2 commits 모두**.

## 4. 본질 변경 영역

### 4.1 결함 가설

PR 본문:
> Task #509 에서 `map_pua_bullet_char` 에 Supplementary PUA-A (U+F02B1~F02B9 → ①~⑨) 매핑을 추가했으나, `convert_pua_enclosed_numbers` (composer) 가 동일 범위를 CharOverlap(border_type=3, 사각형 안의 숫자) 으로 먼저 변환하여 매핑이 도달하지 못했습니다.
> 결과: SVG 에 ① 대신 "1" (사각형 박스) 이 렌더링됨.

→ 두 매핑 메커니즘 (CharOverlap 변환 vs 단순 매핑) 의 우선순위 충돌. CharOverlap 가 먼저 동작.

### 4.2 정정 (PR)

1. **`pua_enclosed_border_type`** (composer.rs): F02B1~F02C4 범위 제거 → CharOverlap 대상에서 제외
2. **`map_pua_bullet_char`** (paragraph_layout.rs): F02BA~F02C4 (⑩~⑳) 매핑 추가 → 전체 20자 완성
3. **반전 사각형** (F02CE~F02E1): CharOverlap 유지 (표준 Unicode 대응 문자 부재)

### 4.3 폭 계산 정합 (Copilot review 응답 영역)

```rust
// seg_widths 루프에서 map_pua_bullet_char 적용 후 estimate_text_width 호출
// PUA 원본 (메트릭 부재 → 0.5em 폴백) 대신
// 매핑된 문자 (① 등 전각) 기준으로 폭 계산
```

→ 레이아웃 정합 유지 (PUA 원본 → 매핑된 문자 기준).

### 4.4 영향 범위 (PR 본문)

| 샘플 | 사용 codepoint | 변경 전 | 변경 후 |
|------|----------------|---------|---------|
| mel-001 | F02B1~F02B3 | "1" "2" "3" 박스 | ① ② ③ |
| kps-ai | F02B1, F02B2 | "1" "2" 박스 | ① ② |
| KTX | F02EF | · (circle) | · (circle, 변경 없음) |

## 5. PR base skew 분석

### 5.1 GitHub UI vs 실제 base

GitHub UI 의 PR base diff 가 광범위 deletion 표시:
- PR #599 의 Skia 본질 모두 revert (`src/renderer/skia/` 전체)
- 메인테이너 후속 정정 모두 revert (export-png CLI / VLM 옵션)
- 본 사이클 cherry-pick 모두 revert (PR #561/#564/#567/#570/#575/#580/#584/#589/#592/#593)
- HWP3 변환본 fixture 5개 삭제
- 모든 보고서 / archives / orders 삭제

→ **PR #571/#599 와 동일한 base skew 패턴**. UI MERGEABLE 표시는 **명목상의 가시성만 보장하고 실제 머지 시 본 사이클 처리분 모두 revert**.

### 5.2 본질 commit 단위 cherry-pick 가능성 평가

본질 commits 2개 (`77986135` + `aafe85a`) 의 변경 영역만 보면:
- `src/renderer/composer.rs`: 매핑 범위 제외 영역
- `src/renderer/layout/paragraph_layout.rs`: ⑩~⑳ 매핑 추가 + 폭 계산 정합

본 사이클 cherry-pick 처리분 중 같은 파일에 변경이 있던 영역:
- **PR #592 (Task #588)**: `paragraph_layout.rs::map_pua_bullet_char` 에 SPUA-A 저영역 (`0xF0000..=0xF00CF`) 분기 신설
- **PR #570 (Task #568)**: `paragraph_layout.rs::layout_composed_paragraph` 분기 확장
- **PR #599 메인테이너 후속**: `paragraph_layout.rs` 영향 없음 (Skia 영역만)

본 PR 의 영역은 Supplementary PUA-A `F02B1~F02C4` (Task #509 본질 + Issue #513 정정) — PR #592 의 SPUA-A 저영역 (`F0000~F00CF`) 과 **다른 코드포인트 영역** → 충돌 가능성 낮음.

## 6. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr600-cherry-test` 임시 브랜치에서 2 commits 순차 cherry-pick:

| 단계 | 결과 |
|------|------|
| `77986135` cherry-pick | ✅ Auto-merging src/renderer/layout/paragraph_layout.rs (충돌 0) |
| `aafe85a` cherry-pick | ✅ Auto-merging src/renderer/layout/paragraph_layout.rs (충돌 0) |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |
| **PUA 테스트** (`pua_mapping_tests` + `pua_oldhangul`) | ✅ **12 passed** (`supplementary_pua_a_maps_middle_dot` 포함) |

→ **commit 단위 cherry-pick 정합 입증** + 본 사이클 처리분과 0 중첩.

## 7. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test` | 전체 통과 | ✅ 1134 passed |
| `cargo clippy -- -D warnings` | 경고 없음 | ✅ 0건 |
| `samples/mel-001.hwp` p2 | ①②③ `<text>` 정상 출력 | ⏳ 본격 검증 |
| `samples/pua-test.hwp` | ①~⑨ 9개 SVG text 확인 | ⏳ 본격 검증 |
| 작업지시자 시각 판정 | (미진행) | ⏳ 본 환경 시각 판정 게이트 |

## 8. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본질 cherry-pick 가능 (commit 단위)** — 2 commits 충돌 0
- ✅ **결정적 검증 정합** — cargo test --lib 1134 passed (회귀 0) / clippy 0 / PUA 테스트 12 passed
- ✅ **소형 정정** — `composer.rs` + `paragraph_layout.rs` 두 파일만 변경 (+30/-15)
- ✅ **케이스별 명시 가드** — F02B1~F02C4 (CharOverlap 제외) + F02CE~F02E1 (CharOverlap 유지) 명시 분리 (`feedback_hancom_compat_specific_over_general` 정합)
- ✅ **Copilot review 응답** — 3개 코멘트 모두 commit `aafe85a` 로 응답 (⑩~⑳ 테스트 / 주석 / 폭 계산 정합)
- ✅ **PR #571/#599 와 다른 처리** — base skew 동일하지만 본질 영역이 본 사이클 처리분과 0 중첩 → cherry-pick 가능
- ✅ **Task #509 본질 후속** — Task #509 의 매핑 추가가 도달 안 한 결함 정확히 식별
- ✅ **컨트리뷰터 협업** — @oksure 의 4번째 PR (PR #581/#582/#583 모두 cherry-pick 완료)

### 우려 영역
- ⚠️ **PR base skew (UI MERGEABLE 표시)** — 단순 머지 절대 금지, commit 단위 cherry-pick 필수
- ⚠️ **시각 판정 게이트** — PR 본문 미진행. 본 환경 cherry-pick 후 작업지시자 직접 시각 판정 필수
- ⚠️ **광범위 sweep** — PR #564/#570/#575/#580/#584/#592/#593 패턴 (164 fixture 1,614 페이지) 본 환경 자동 sweep 권장

## 9. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — 2 commits 충돌 0
- ✅ **결정적 검증** — 1134 passed / clippy 0 / PUA 12 passed
- ✅ **케이스별 명시 가드** — Task #509 본질 후속 + 명시 범위 분리
- ✅ **Copilot review 응답 정합**
- ⏳ **시각 판정 별도 진행 필요**
- ⏳ **광범위 sweep 본격 검증 필요**

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (commit 단위) (권장)
- 2 commits 순차 cherry-pick (단순 머지 절대 금지)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep
- 작업지시자 시각 판정 (★ 게이트) — mel-001 / kps-ai / KTX / pua-test 영향 영역
- 통과 시 devel merge + push + PR close 처리

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 — 본질 cherry-pick 깨끗 + 결정적 검증 통과 + 케이스별 명시 가드 + Copilot review 응답 정합.

## 10. 옵션 A 진행 결과 (작업지시자 승인 후)

### 10.1 핀셋 cherry-pick (commit 단위)

| 단계 | 결과 |
|------|------|
| `77986135` cherry-pick | ✅ 충돌 0 |
| `aafe85a` cherry-pick | ✅ 충돌 0 |
| local/devel commits | `34f8547` + `14f30e8` (author Hyunwoo Park 보존) |

### 10.2 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1134 passed** (회귀 0) |
| `cargo test --lib pua` | ✅ 12 passed (`supplementary_pua_a_maps_circled_digits` ⑩~⑳ GREEN) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546/554` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |

### 10.3 광범위 페이지네이션 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

### 10.4 SVG byte 차이 + 원문자 정량 측정

| Fixture | byte 차이 | 원문자 출현 (before/after) | 평가 |
|---|---|---|---|
| **pua-test** | 1/1 | **0 → 9** | ✅ PR 본문 명시 (9 SVG text) 100% 재현 |
| **mel-001** | 15/21 | 20 → 34 (+14) | ✅ PR 본문 정정 영역 |
| **kps-ai** | 1/80 | 32 → 34 (+2) | ✅ PR 본문 명시 정합 |
| **KTX** | 0/27 | (변경 없음) | ✅ PR 본문 명시 (변경 없음) 정합 |

→ Task #509 의 ⑩~⑳ 매핑이 CharOverlap 우선순위 결함 정정으로 SVG 에 정상 출력. 광범위 sweep 회귀 0 + 의도된 영역에만 변경 (PR 본문 100% 재현).

### 10.5 다음 단계

1. ✅ 본 1차 검토 보고서 작성
2. ✅ 본 환경 결정적 재검증
3. ✅ SVG 생성 (mel-001 / kps-ai / KTX / pua-test)
4. ✅ 광범위 페이지네이션 sweep
5. ⏳ **작업지시자 시각 판정** (★ 게이트, mel-001 ① 등 정상 출력 + KTX 회귀 0) — 본 단계 대기 중
6. ⏳ 통과 시 devel merge + push + PR close (한글 댓글)
7. ⏳ 처리 보고서 (`pr_600_report.md`) 작성 + archives 이동

## 11. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (F02B1~F02C4 vs F02CE~F02E1 분리)
- ✅ `feedback_rule_not_heuristic` — 코드포인트 범위 + 명시 매핑 (휴리스틱 아님)
- ✅ `feedback_pdf_not_authoritative` — 권위는 작업지시자 한컴 환경
- ✅ `feedback_per_task_pr_branch` — Issue #513 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/6) 누적 22번째 PR
- ✅ **PR #571/#599 패턴과 다름** — base skew 동일하지만 본질 영역 0 중첩으로 cherry-pick 가능

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
