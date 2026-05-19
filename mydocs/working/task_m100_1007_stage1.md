# Task #1007 Stage 1 — 정밀 진단 보고서

이슈: [#1007](https://github.com/edwardkim/rhwp/issues/1007)
수행/구현 계획서: [`task_m100_1007.md`](../plans/task_m100_1007.md), [`task_m100_1007_impl.md`](../plans/task_m100_1007_impl.md)

## 1. 진단 도구

- `rhwp dump-pages -p N`
- `grep` 으로 vpos_reset 처리 코드 위치

## 2. 핵심 진단 결과

### 2-1. vpos 데이터 분석 (sample16-hwp5 page 3)

dump-pages 출력:

```
=== 페이지 3 (global_idx=2, section=0, page_num=1) ===
  단 0 (items=22, ...)
    FullParagraph  pi=69  vpos=852    "Ⅰ. 사업개요"
    FullParagraph  pi=70  vpos=4904   "1. 추진목적"
    FullParagraph  pi=71  vpos=7464   "(빈)" + Shape
    FullParagraph  pi=72  vpos=18576  "(빈)"
    FullParagraph  pi=73  vpos=19892  "2. 추진방향"
    FullParagraph  pi=74  vpos=23020..27052  "(1) 노후화된..."
    FullParagraph  pi=75  vpos=29636..33668  "(2) 주전산센터..."
    ... section 1 paragraphs ...
    FullParagraph  pi=86  vpos=68048  "󰏅 활용가능 기존장비..."
    FullParagraph  pi=87  vpos=70412  "(빈)"
    FullParagraph  pi=88  vpos=568    "(2) 주전산센터 서버통합(Consolidation) 체계 구축"  ← VPOS RESET
    FullParagraph  pi=89  vpos=3112   "󰏅 업무특성 및..."
```

**핵심**: pi=87 vpos=70412 → pi=88 vpos=**568**. vpos 가 갑자기 작은 값으로 **RESET** 됨.

### 2-2. vpos reset 의미

HWP3 또는 한컴 변환 시 page break 가 있던 위치에서 line_segs.vertical_pos 가 0 또는 작은 값으로 reset 됨. 이는 한컴이 인코딩한 page break 시그널.

rhwp 가 이 시그널을 인식하지 않고 paragraph 누적 진행 → pi=88 이 page 3 에 packed.

## 3. 기존 vpos_reset 처리 분석

`src/renderer/pagination/engine.rs:780` 의 `respect_vpos_reset` 옵션:
```rust
let forced_breaks: Vec<usize> = if respect_vpos_reset {
    para.line_segs.iter().enumerate()
        .filter(|(i, ls)| *i > 0 && ls.vertical_pos == 0)
        .map(|(i, _)| i).collect()
} else {
    Vec::new()
};
```

**한계**:
1. `respect_vpos_reset` default = false (`src/document_core/mod.rs:229`) — option 활성화 안 됨
2. **단일 paragraph 내** line_segs[i].vpos==0 만 catch (i>0 조건). **paragraph 간 vpos reset** 미감지

본 case 는 paragraph 간 (pi=87 → pi=88) vpos reset 이므로 기존 로직 미적용.

## 4. Root Cause

**파라그래프 간 vpos reset 미감지** — pi=87 last line vpos=70412 → pi=88 first line vpos=568 인 경우 page break 트리거 안 됨.

특히 변환본 (`is_hwp3_variant=true`) 에서 이 패턴이 자주 발생 — HWP3 의 page break 가 HWP5 변환본의 vpos 데이터에 그대로 인코딩.

## 5. Fix 방향 (Stage 2 후보)

### 시나리오 X (가장 유망) — Cross-paragraph vpos reset 감지

`pagination/engine.rs` 의 paragraph 추가 시점에:
- prev_para_last_line.vertical_pos > THRESHOLD (페이지 절반 등)
- AND curr_para_first_line.vertical_pos < THRESHOLD (페이지 시작 부근)
- → force page break before curr_para

THRESHOLD 결정 필요 (예: 페이지 절반 = body_area.height / 2 in HWPUNIT).

### 시나리오 Y — variant 시 respect_vpos_reset 자동 활성화

변환본일 때 (`is_hwp3_variant=true`) `respect_vpos_reset` 옵션 자동 활성화. 그러나 기존 로직은 paragraph 내만 catch 하므로 X 와 결합 필요.

### 시나리오 Z — empty paragraph 후 vpos reset 만 감지 (더 specific)

pi=87 이 (빈) paragraph 라는 특수 패턴 활용. variant + 빈 paragraph 후 vpos reset 시만 page break.

## 6. 격차 양 정량화

현재 page 3 used height = 958.7 px (page body height 971.3 의 99%). 거의 가득 참.
pi=88 첫 줄까지 포함되어 있음.

Page 3 의 마지막 paragraph (pi=87 "(빈)") 종료 위치 = vpos 70412 + lh + ls ≈ 71800 HUE ≈ 956 px (page body 거의 끝).

Fix 후: pi=88 부터 page 4 시작. page 3 used height ≈ 956 px (변동 없음, section 1 만 끝까지).

## 7. Stage 2 진입

다음 단계: Fix 후보 X / Y / Z 평가 + 선정 + 회귀 risk 분석.

권고 후보: **시나리오 X (Cross-paragraph vpos reset 감지) + 시나리오 Y (variant 시 자동 활성화)** 결합.
