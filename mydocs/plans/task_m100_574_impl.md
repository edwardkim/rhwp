# Task #574 구현 계획서

**브랜치**: `local/task574`
**이슈**: https://github.com/edwardkim/rhwp/issues/574
**Stage 0 본질 확정**: `is_heavy_display_face` (`src/renderer/style_resolver.rs:601-613`)
의 hardcoded list 에 **HY견명조** 가 잘못 포함되어 CharShape.bold=false 무시.

---

## 1. 본질 정정 범위

**제거 대상**:
- `"HY견명조"` ← 제거 (한컴 일반 두께 명조 — heavy 가 아님)

**보존**:
- `"HY견명조B"` ← 보존 (명시 Bold variant; "B" 접미는 Bold 의미)
- `"HY헤드라인M"`, `"HYHeadLine M"`, `"HYHeadLine Medium"` ← Task #146 v4 본질 케이스
- `"HY견고딕"` ← Heading 전용 굵은 고딕
- `"HY그래픽"`, `"HY그래픽M"` ← 그래픽 굵은 face

## 2. Task #146 v4 회귀 검증

| 샘플 | 사용 폰트 | 영향 평가 |
|------|----------|---------|
| `samples/text-align.hwp` (Task #146 base) | HY헤드라인M (17곳) | **영향 없음** — HY헤드라인M 보존 |
| `samples/exam_science.hwp` (Task #574 본 이슈) | HY견명조 (24곳/p1) | **fix 적용** — CharShape.bold 따라 분기 |

**골든 SVG 테스트** (`tests/golden_svg/`):
- `issue-267/ktx-toc-page.svg` — HY헤드라인M 사용 (영향 없음)
- HY견명조 사용 골든 SVG — **0건** (검색 결과)

## 3. 단계 분할

### Stage 2 — TDD 통합 테스트 추가 + RED 확인

1. `src/renderer/layout/tests.rs:938` 의 `test_is_heavy_display_face_matches_known_heavy_faces`
   를 **갱신**:
   - "HY견명조" 를 heavy 단언에서 제거
   - "HY견명조" 를 NOT heavy 단언에 추가
   - "HY견명조B" 는 heavy 단언에 유지 (명시 Bold variant)
2. `tests/integration_tests.rs` 또는 적절한 위치에 통합 테스트 추가:
   - `samples/exam_science.hwp` 페이지 1 SVG 에 `font-size="44"` + `font-weight="bold"`
     쪽번호 가 **없음** 단언 (fix 후 RED → GREEN)
3. RED 상태 확인 (현재 코드는 fix 전이므로 위 통합 테스트 fail)
4. Stage 2 보고서 + 커밋

### Stage 3 — Fix 적용

1. `src/renderer/style_resolver.rs:608-612` 의 `matches!` 패턴에서 `"HY견명조"` 제거:
   ```rust
   matches!(primary,
       "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
       | "HY견고딕" | "HY견명조B"
       | "HY그래픽" | "HY그래픽M"
   )
   ```
2. 단위 테스트 GREEN 확인:
   - `cargo test --release --lib test_is_heavy_display_face` 통과
   - Stage 2 통합 테스트 통과 (RED → GREEN)
3. Stage 3 보고서 + 커밋

### Stage 4 — 광범위 회귀 검증 (5개 샘플 sweep)

1. fix 전후 SVG diff:
   - `samples/exam_science.hwp` (4페이지) — 본 이슈 샘플
   - `samples/exam_kor.hwp`, `samples/exam_eng.hwp`, `samples/exam_math.hwp` — 기출 시리즈
   - `samples/synam-001.hwp`, `samples/복학원서.hwp` — 일반 샘플
   - `samples/text-align.hwp` — Task #146 base (회귀 없어야 함)
2. diff 분석:
   - HY견명조 사용 텍스트의 bold 변경 패턴
   - 변경 영역 ≈ CharShape.bold == false 인 HY견명조 텍스트 (의도된 정정)
   - HY견명조 외 폰트 변경 0건 확인 (회귀 없음 보장)
3. `cargo test --release --lib` 전체 통과 (1100+ tests)
4. clippy 회귀 검사 (`cargo clippy --all-targets`)
5. Stage 4 보고서 + 커밋

### Stage 5 — 한컴 PDF 시각 검증 + 최종 보고서

1. `samples/exam_science.pdf` 페이지 1 의 쪽번호 "1" 굵기 확인 (작업지시자 시각 판정)
2. fix 후 SVG 의 쪽번호 "1" 굵기 비교
3. 차이 시각 OK 면 close 진행
4. 최종 결과 보고서 (`mydocs/report/task_m100_574_report.md`) + 오늘할일 갱신
5. `closes #574` 커밋

## 4. 산출물 (예상)

| 파일 | 변경 |
|------|------|
| `src/renderer/style_resolver.rs` | line 610: `\| "HY견명조"` 제거 |
| `src/renderer/layout/tests.rs` | line 944: HY견명조 단언 위치 이동 (heavy → NOT heavy) |
| `tests/integration_tests.rs` (또는 적절한 위치) | exam_science 쪽번호 bold 검증 통합 테스트 신규 |
| `mydocs/working/task_m100_574_stage{2,3,4}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_574_report.md` | 최종 결과 보고서 |
| `mydocs/orders/{오늘}.md` | 본 타스크 상태 갱신 |

## 5. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| Task #146 v4 회귀 (HY헤드라인M heavy bold 미적용) | 큼 | text-align.hwp HY헤드라인M 보존 확인 — 영향 없음 |
| HY견명조 사용 시 PDF 와 시각 괴리 | 중간 | 한컴 PDF 시각 검증 (Stage 5) — 작업지시자 판정 |
| 다른 샘플의 HY견명조 + bold=false 텍스트 회귀 | 중간 | 5개 샘플 sweep + diff 분석 (Stage 4) |
| HY견명조B (명시 Bold variant) 영향 | 작음 | 보존 — 회귀 없음 |

## 6. 검증 명령

```bash
# Stage 2 RED 확인
cargo test --release --lib test_is_heavy_display_face -- --nocapture
# (현재 GREEN, fix 후 갱신된 테스트는 RED → fix 후 GREEN)

# Stage 3 GREEN 확인
cargo test --release --lib test_is_heavy_display_face

# Stage 4 sweep
for s in exam_science exam_kor exam_eng exam_math synam-001 복학원서 text-align; do
    ./target/release/rhwp export-svg samples/${s}.hwp -o /tmp/sweep574/${s}/
done
# fix 전후 diff (별도 head/working 비교)

# Stage 4 전체 테스트
cargo test --release --lib
cargo clippy --all-targets -- -D warnings
```

## 7. 커밋 단위

- Stage 2: "Task #574 Stage 2: TDD 통합 테스트 추가 + 단위 테스트 갱신 (HY견명조 → NOT heavy)"
- Stage 3: "Task #574 Stage 3: is_heavy_display_face HY견명조 제거 (CharShape.bold 권위 회복)"
- Stage 4: "Task #574 Stage 4: 광범위 회귀 sweep + 회귀 없음 확인"
- Stage 5: "Task #574 Stage 5: 최종 결과 보고서 + 오늘할일 갱신, closes #574"

## 8. 메모리 룰 준수

- **[feedback_essential_fix_regression_risk]**: HY견명조B 보존 + 광범위 sweep + 한컴 PDF
  시각 검증으로 본질 정정 회귀 위험 최소화.
- **[feedback_visual_regression_grows]**: 5개 샘플 + golden test sweep.
- **[feedback_pdf_not_authoritative]**: 한컴 PDF 보조 ref. 작업지시자 시각 판정 게이트
  (Stage 5).
- **[feedback_rule_not_heuristic]**: 화이트리스트 단일 룰 — heavy display 의미 face 만
  포함 (HY헤드라인M / HY견고딕 등). HY견명조 는 일반 두께 명조 → 룰 위배 → 제거.

---

승인 후 Stage 2 (TDD 통합 테스트 추가) 부터 시작합니다.

본 구현 계획서의 핵심 결정 사항:

1. **단일 줄 수정**: `"HY견명조"` 제거 (HY견명조B 보존)
2. **TDD 우선**: 통합 테스트 RED → fix → GREEN 순서
3. **광범위 sweep 필수**: 7개 샘플 (5개 핵심 + text-align + exam_science) diff 분석
4. **한컴 PDF 시각 검증**: 작업지시자 판정 게이트 (Stage 5)
