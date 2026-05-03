# Task #540 Stage 2 완료 보고서

**제목**: 빈 paragraph 음수 ls floor 적용 (가설 H2)
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/rhwp/issues/540

---

## 1. fix 내용

### 1.1 위치

`src/renderer/layout.rs` — column별 vpos correction 로직.

### 1.2 변경 요약

(1) 컬럼 스코프에 누적 보정값 추가:
```rust
let mut vpos_neg_ls_floor_total: i32 = 0;
```

(2) vpos correction 시 prev_pi 가 빈 paragraph + 음수 ls 인 경우 누적치에 합산하고
모든 후속 correction 의 `vpos_end` 에 누적치를 더해 floor 효과 재현:

```rust
let prev_neg_ls_floor: i32 = paragraphs.get(prev_pi)
    .map(|p| {
        if !p.text.is_empty() { return 0; }
        p.line_segs.iter()
            .map(|s| if s.line_spacing < 0 { -s.line_spacing } else { 0 })
            .sum::<i32>()
    })
    .unwrap_or(0);
vpos_neg_ls_floor_total += prev_neg_ls_floor;
let vpos_end = vpos_end_raw + vpos_neg_ls_floor_total;
```

### 1.3 핵심 설계 결정

**누적 보정 (단발 보정 아님)**: vpos correction 은 IR 절대 vpos 기반이라 단발만 보정하면
다음 paragraph 의 correction 이 IR 위치(시프트 미반영)로 되돌린다. 누적값을 모든 후속
correction 에 적용해야 한컴 동작(모든 후속 paragraph floor 분만큼 시프트) 과 일치.

**가드**: `text.is_empty()` — synam-001 의 음수 ls 57건 중 일반 paragraph (셀 내부 등) 의
음수 ls 는 보존하고 빈 paragraph 만 floor 적용. 본질 정정 회귀 위험 최소화.

## 2. 진단 결과 (수정 후)

```
VPOS_CORR pi=46 prev_pi=45 prev_ls=-440 vpos_end=2916 end_y=248.64  applied=true
VPOS_CORR pi=47 prev_pi=46 prev_ls=716  vpos_end=13812 end_y=393.92 applied=true
VPOS_CORR pi=48 prev_pi=47 prev_ls=716  vpos_end=24708 end_y=539.20 applied=true
VPOS_CORR pi=49 prev_pi=48 prev_ls=716  vpos_end=48316 end_y=853.97 applied=true
VPOS_CORR pi=50 prev_pi=49 prev_ls=716  vpos_end=75556 end_y=1217.17 applied=true
```

| pi | 수정 전 vpos_end | 수정 후 vpos_end | 차이 |
|----|----------------|----------------|------|
| 46 | 2476 | 2916 | +440 |
| 47 | 13372 | 13812 | +440 |
| 48 | 24268 | 24708 | +440 |
| 49 | 47876 | 48316 | +440 |
| 50 | 75116 | 75556 | +440 |

→ 누적 시프트 +440 HU (≈ 5.87 px) 가 pi=46 이후 모든 paragraph 에 일관 적용. ✓

## 3. 단위 테스트

### 3.1 Stage 1 TDD 테스트 통과

```
test test_540_empty_paragraph_negative_ls_floor ... ok
```

bracket(pi=44) → pi=46 gap = 38.88 px (기대값) ✓
- 수정 전: 33.01 px (IR 음수 ls=-440 그대로 반영)
- 수정 후: 38.88 px (음수 ls floor 적용)

### 3.2 전체 회귀

```
test result: ok. 1120 passed; 0 failed; 1 ignored
```

기존 1119 (Task #537/#539 포함) + Stage 1 신규 1 = 1120 모두 통과.
Task #537 trailing-ls 보정, Task #539 글박스 vpos 보정 모두 무회귀.

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | +20 LOC (vpos_neg_ls_floor_total 누적 + vpos_end 적용) |
| `mydocs/working/task_m100_540_stage2.md` | 본 보고서 |

## 5. 다음 단계 (Stage 3)

- `cargo test --release --lib` 전체 통과 (이미 통과 ✓).
- `synam-001.hwp` 광범위 회귀 검증 (음수 ls 57건 중 빈 paragraph 만 영향, 일반 paragraph 보존 검증).
- `exam_math`, `21_언어_기출` 등 광범위 샘플 회귀.
- 회귀 발견 시: 가드 정밀화 또는 본 task 정정 보류.

## 6. 승인 요청

Stage 2 완료. Stage 3 (광범위 회귀 검증) 진행 승인 요청.
