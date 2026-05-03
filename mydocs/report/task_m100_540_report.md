# Task #540 최종 결과 보고서

**제목**: 21_언어_기출 페이지 2 빈 paragraph 직후 줄간격 좁음 정정 (가설 H2)
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/rhwp/issues/540
**Milestone**: M100 (v1.0.0)

---

## 1. 요약

빈 paragraph (text=∅, controls=∅) 의 음수 line_spacing 을 한컴이 floor (무시)
하여 advance = line_height 만큼만 진행하는 동작을 재현. 컬럼별 누적 floor 값을
모든 후속 vpos correction 의 `vpos_end` 에 적용하여 일관 시프트.

**핵심 측정값** (21_언어_기출 페이지 2 col 1 — `[4~6]` → 지문 첫 line):
- 수정 전 gap: 33.01 px (IR 음수 ls=-440 그대로 반영)
- 수정 후 gap: 38.88 px (가설 H2 floor 적용)
- 한컴 측정값: 38.88 px (작업지시자 보고)

## 2. 변경 사항

### 2.1 `src/renderer/layout.rs`

(1) 컬럼 스코프에 누적 보정값 도입
```rust
let mut vpos_neg_ls_floor_total: i32 = 0;
```

(2) vpos correction 시 prev_pi 가 진짜 빈 paragraph + 음수 ls 인 경우 floor
값을 누적하고 `vpos_end` 에 적용
```rust
let prev_neg_ls_floor: i32 = paragraphs.get(prev_pi)
    .map(|p| {
        if !p.text.is_empty() || !p.controls.is_empty() { return 0; }
        p.line_segs.iter()
            .map(|s| if s.line_spacing < 0 { -s.line_spacing } else { 0 })
            .sum::<i32>()
    })
    .unwrap_or(0);
vpos_neg_ls_floor_total += prev_neg_ls_floor;
let vpos_end = vpos_end_raw + vpos_neg_ls_floor_total;
```

(3) lazy_base 산출 시 누적 보정 가산 (회귀 방지)
```rust
let lazy_base = prev_vpos_end - y_delta_hu + vpos_neg_ls_floor_total;
```

### 2.2 `src/renderer/layout/integration_tests.rs`

`test_540_empty_paragraph_negative_ls_floor` 통합 테스트 추가 — 페이지 2 col 1
의 `[` (pi=44) 와 다음 본문 line (pi=46) baseline gap = 38.88 px 검증.

## 3. 핵심 설계 결정

### 3.1 누적 보정 (단발 보정 아님)

vpos correction 은 IR 절대 vpos 기반이라 단발만 보정하면 다음 paragraph 의
correction 이 IR 위치 (시프트 미반영) 로 되돌린다. 누적값을 모든 후속
correction 에 적용해야 한컴 동작 (모든 후속 paragraph 가 floor 분만큼 시프트)
와 일치.

### 3.2 가드: `text.is_empty() && controls.is_empty()`

**목적**: synam-001 의 음수 ls 57건 중 일반 paragraph (셀 내부, 콘텐츠 줄바꿈
조절 등) 의 음수 ls 는 **보존**. 진짜 빈 paragraph (paragraph terminator 만
존재) 만 floor 적용.

**`controls.is_empty()` 추가 근거**: exam_science s0.p0 (cc=57, controls=7,
ls=-1348) 같은 section-setup paragraph (구역나누기/머리말/표 컨테이너) 는 advance
≈ 0 의도이므로 floor 하면 본문이 1348 HU 만큼 잘못 시프트됨 (Stage 3 회귀
발견).

### 3.3 lazy_base 보정

`vpos_neg_ls_floor_total` 가 누적된 만큼 sequential y_offset 은 IR vpos 보다
앞서나간다. lazy_base 는 IR 절대 vpos 좌표 기준이므로 누적 floor 분만큼 보정
하지 않으면 `prev_vpos_end - y_delta_hu` 가 음수로 흘러 fallback 경로 진입 →
vpos correction 미적용 → 후속 paragraph 가 IR 위치로 되돌아감 (Stage 3 회귀
발견).

## 4. 검증 결과

### 4.1 단위 테스트

```
test result: ok. 1120 passed; 0 failed; 1 ignored
test_540_empty_paragraph_negative_ls_floor ... ok
```

기존 1119 (Task #537/#539 포함) + Stage 1 신규 1 = 1120 모두 통과. 무회귀.

### 4.2 광범위 샘플 회귀 검증

| 샘플 | 페이지 수 | 차분 페이지 | 음수 시프트 |
|------|----------|-----------|------------|
| `synam-001.hwp` | 35 | **0** | 0 |
| `21_언어_기출_편집가능본.hwp` | 15 | 3 (target) | 0 |
| `exam_math.hwp` | 20 | 6 | 0 |
| `exam_eng.hwp` | 8 | 0 | 0 |
| `exam_kor.hwp` | 20 | 0 | 0 |
| `exam_science.hwp` | 6 | 0 (controls 가드로 보존) | 0 |

모든 차분이 양의 시프트 (의도된 floor 효과). 음수 시프트 (회귀) 0건.

### 4.3 21_언어_기출 페이지 2 (target)

|  | gap (px) | 일치 |
|--|---------|-----|
| rhwp 수정 전 | 33.01 | ❌ |
| rhwp 수정 후 | 38.88 | ✅ |
| 한컴 (작업지시자 측정) | 38.88 | — |

페이지 2 col 1 9곳 빈 paragraph 음수 ls 케이스 모두 통일된 floor 적용:
- 60% 빈 paragraph (ls=-440): +5.87 px 시프트
- 95% 빈 paragraph (ls=-56): +0.75 px 시프트
- 65% 빈 paragraph (ls=-384): +5.12 px 시프트

## 5. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| synam-001 음수 ls 57건 회귀 | 매우 큼 | text+controls 가드. 0 차분 확인 |
| Task #537/#539 회귀 | 큼 | 1120 단위 테스트 통과 |
| section-setup paragraph 잘못 floor | 큼 | controls.is_empty() 가드 (Stage 3 발견) |
| lazy_base fallback 회귀 | 큼 | lazy_base 에 누적 보정 가산 (Stage 3 발견) |

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | +30 LOC (누적 보정, 가드, lazy_base 보정) |
| `src/renderer/layout/integration_tests.rs` | +76 LOC (TDD 테스트) |
| `mydocs/plans/task_m100_540.md` | 수행 계획서 |
| `mydocs/plans/task_m100_540_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_540_stage{0,1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_540_report.md` | 본 보고서 |

## 7. 커밋 이력

- `851e01bd` — Stage 0: 사전 분석 + 한컴 환경 검증 입력 대기
- `7c8509a9` — Stage 1: 가설 H2 TDD 단위테스트 + 진단
- `8d27ad44` — Stage 2: 빈 paragraph 음수 ls floor 적용 (가설 H2)
- (Stage 3) — Stage 3: 가드/lazy_base 정밀화 + 광범위 회귀 검증

## 8. 회고

Stage 1 진단에서 `vpos correction` 의 단순한 vpos_end 보정으로 충분할 것으로
판단했으나, Stage 2 실측에서 단발 보정이 다음 correction 에 의해 되돌려지는
문제 확인 → 누적 방식으로 변경. Stage 3 광범위 검증에서 (1) section-setup
paragraph 잘못 floor (2) lazy_base 음수 흐름 회귀 추가 발견 → 가드 강화 +
lazy_base 보정으로 완전 해소.

「룰과 휴리스틱 구분」[feedback_rule_not_heuristic] 관점에서 본 정정은 "빈
paragraph 의 음수 ls floor" 라는 한컴 표준 동작의 룰 적용이며, `text + controls`
가드는 룰 적용 범위를 정확히 한정 (휴리스틱이 아닌 명세적 분류). lazy_base
보정은 누적 시프트와 IR 좌표계 사이의 정합성 유지를 위한 수학적 도출.

closes #540
