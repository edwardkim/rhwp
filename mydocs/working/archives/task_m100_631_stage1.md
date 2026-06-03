# Task #631 Stage 1: 정밀 진단 (코드 무수정)

> **이슈**: [#631](https://github.com/edwardkim/rhwp/issues/631)
> **브랜치**: `local/task631`
> **작성일**: 2026-05-06

---

## 핵심 발견

**+15.4px 누적은 부수 현상이고, 진짜 원인은 별개다.**

`pi=222` 자체가 HWP 측에서 **페이지 경계 정보(LINE_SEG vpos-reset)를 보유**하고 있는데, 우리 렌더러가 이를 무시하고 자체 누적 높이로 페이지 분할을 결정한 결과 1줄 짧게 자르고 있다.

## 증거

### 1. pi=222 LINE_SEG 구조 (HWP 정답값)

```
ls[0]: vpos=67980, lh=1200, ls=720   ← page 18 (line 0)
ls[1]: vpos=69900, lh=1200, ls=720   ← page 18 (line 1)
ls[2]: vpos=0,     lh=1200, ls=720   ← page 19 시작 (vpos-reset)
ls[3]: vpos=1920,  lh=1200, ls=720   ← page 19 (line 1)
```

`ls[2].vpos=0` 은 HWP 한컴 엔진이 직접 계산한 **"이 줄에서 페이지가 새로 시작된다"** 표식이다.
즉 한컴 의도는 page 18에 line 0~1 (2줄), page 19에 line 2~3 (2줄). PDF 출력과도 일치.

### 2. 우리 렌더러 결과 (default)

```
page 18: PartialParagraph pi=222 lines=0..1 vpos=67980..69900   ← 1줄만
page 19: PartialParagraph pi=222 lines=1..4 vpos=69900..1920 [vpos-reset@line2]
```

### 3. `--respect-vpos-reset` 플래그 동작 검증

`src/renderer/pagination/engine.rs:621` 에 vpos-reset 강제 분리 로직이 이미 존재:

```rust
let forced_breaks: Vec<usize> = if respect_vpos_reset {
    para.line_segs.iter().enumerate()
        .filter(|(i, ls)| *i > 0 && ls.vertical_pos == 0)
        .map(|(i, _)| i)
        .collect()
} else { Vec::new() };
```

그러나 **`respect_vpos_reset` 기본값 = `false`** (`src/document_core/mod.rs:213`,
`src/document_core/commands/document.rs:90`). CLI 플래그 `--respect-vpos-reset` 로만 켤 수 있음.

**검증**: `--respect-vpos-reset` 플래그 켜고 재실행 → **결과 동일** (lines=0..1):

```
$ rhwp dump-pages samples/aift.hwp -p 17 --respect-vpos-reset
PartialParagraph  pi=222  lines=0..1  vpos=67980..69900
$ rhwp dump-pages samples/aift.hwp -p 17
PartialParagraph  pi=222  lines=0..1  vpos=67980..69900   # 동일
```

→ 플래그가 켜져도 결과 변화 없음. 즉 forced_breaks 경로가 실효적으로 작동하지 않거나, 
   기본 분할 경로가 동일 결과를 산출하고 있음.

### 4. 페이지 18 단락별 y 좌표 (debug-overlay)

| pi | y (px) | Δ |
|----|--------|----|
| 209~213 | 75.6 → 178.0 | 25.6 × 5 (5줄 텍스트) |
| 214 표 | 203.6 | 표 스팬 |
| 215 | 426.5 | 표+간격 222.9 |
| 216 (그림 단락) | 554.5 | 128.0 |
| 217~221 | 859.8 → 962.2 | 25.6 × 5 |
| 222 line 0 | 987.8 | 25.6 |

body 하단 = 75.6 + 971.4 = **1047.0**. pi=222 line 0 끝 = 1013.4. 
**line 1 추가 시 끝 = 1039.0 < 1047.0** → 수학적으로 line 1 도 페이지 내에 들어감.

### 5. 페이지 누적 산식 (engine.rs:686~715)

`paginate_text_lines` 의 line-by-line 분할 루프:

```rust
let page_avail = (base_available_height - ... - st.current_height - ...).max(0.0);
let avail_for_lines = (page_avail - sp_b).max(0.0);
let mut cumulative = 0.0;
let mut end_line = cursor_line;
for li in cursor_line..line_count {
    let content_h = mp.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        break;
    }
    cumulative += mp.line_advance(li);  // line_height + line_spacing
    end_line = li + 1;
}
```

수학적 시뮬레이션 (cursor_line=0 시점, current_height=912.2):
- avail_for_lines = 971.4 − 912.2 = **59.2px**
- li=0: 0+16 = 16 ≤ 59.2 → cumulative = 25.6, end_line = 1
- li=1: 25.6+16 = 41.6 ≤ 59.2 → cumulative = 51.2, **end_line = 2**
- li=2: 51.2+16 = 67.2 > 59.2 → break

→ 시뮬레이션에 따르면 **end_line=2 가 나와야 함**. 실제로는 end_line=1.

## 결론 — 진짜 원인 후보

### 가설 A (가장 유력): pagination 의 `st.current_height` 가 렌더링 y(912.2)보다 큼

페이지 18 안의 표/그림 단락 처리 후 vpos 보정(engine.rs:246~286)이나 표 control 처리에서
pagination 측 `current_height` 가 렌더링 y보다 추가로 누적되어, pi=222 진입 시점에 
`avail_for_lines < 51.2px` 가 됨. 결과 line 1 이 안 들어간다고 판정.

`+15.4px` 의 정체 = `pagination current_height` 와 `렌더링 y` 사이의 누적 drift.

### 가설 B: forced_breaks 경로가 paginate_text_lines 로 도달하지 않음

`pi=222` 가 다른 경로(예: 표/도형 control 처리 후 후속 paragraph 처리)로 우회하여 
`forced_breaks` 가 평가되지 않음. 그래서 `--respect-vpos-reset` 플래그가 무효.

## 영향 범위

- `respect_vpos_reset = false` 기본값으로 인해 **현재 모든 사용자가 이 버그에 노출**됨
- HWP가 pre-computed 한 LINE_SEG vpos-reset 정보가 **전혀 활용되지 않음**
- 다른 문서에서도 페이지 끝 1~2줄 밀림이 동일 원인일 가능성 큼

## 다음 단계 (Stage 2 = 구현계획서)

1. **포인트 진단**: `eprintln!` 임시 삽입으로 pi=222 진입 시점의 `st.current_height`, 
   `forced_breaks`, `avail_for_lines`, `end_line` 실제값 측정 → 가설 A vs B 확정
2. 가설 A 확정 시: vpos drift 보정 로직 점검 (engine.rs:246~286 의 page_has_block_table 가드)
3. 가설 B 확정 시: paginate_text_lines 호출 경로 점검
4. **부수 검토**: `respect_vpos_reset` 기본값을 `true` 로 전환해야 하는가?

## 산출물

- 본 보고서: `mydocs/working/task_m100_631_stage1.md`
- 진단용 SVG: `/tmp/task631/aift_018.svg`, `/tmp/task631_vpos/aift_018.svg`
