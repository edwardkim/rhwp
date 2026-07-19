---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1151/README.md
last_verified: 2026-07-16
---

# 한컴 picture "글자처럼 취급" 토글 동작 — 산출물 분석

대상 task: [Task #1151 v2 후속](../../../plans/archives/task_m100_1151_v2.md) · 정합 산출물: `samples/pic-in-table-with-toggle.hwp` (사용자 추가 2026-05-29)

## 1. 사용자 직접 검증 (2026-05-29)

> "글자처럼 취급하면 바로 표에 들어가지 않고, 셀 바깥을 나오는 것을 확인. 이미지를 표 내부에 넣고 글자처럼 취급은 바깥에서 추가 후 이동해야 되는것을 확인."

핵심: 한컴은 셀 위 floating picture 의 "글자처럼 취급" 토글 시 picture 를 **셀 안 inline 으로 자동 이동시키지 않는다**. picture 가 셀 바깥으로 나옴.

## 2. 산출 파일 `samples/pic-in-table-with-toggle.hwp` dump 분석

`./target/debug/rhwp dump samples/pic-in-table-with-toggle.hwp`:

```
=== 구역 0 ===
--- 문단 0.0 --- cc=33, text_len=0, controls=4 [구역나누기]
  ls[0]: ts=0, vpos=39354, lh=1154, th=1154, bl=981, ls=600, cs=0, sw=42520
  [0] 구역정의
  [1] 단정의
  [2] 표: 2행×2열, treat_as_char=false, wrap=위아래
        size=41954×38788, valign=Top
        셀[0] r=0,c=0  paras=1
          p[0] ctrls=1 ls[0] vpos=0 lh=1000
            ctrl[0] 그림: bin_id=2, w=12257 h=7880, tac=false, wrap=Square,
                          vert=Para(off=5365), horz=Para(off=3463)
        셀[1,2,3] r=0,c=1 / r=1,* — paras=1, ctrls=0 (비어있음)
  [3] 그림: bin_id=1, common=16942×1154, tac=true (z=1)
          위치: 가로=문단 오프셋=0, 세로=문단 오프셋=0, 글자처럼=true

--- 문단 0.1 --- cc=9, text_len=0, controls=1
  ls[0]: ts=0, vpos=41108, lh=7880, th=7880, bl=6698, ls=600, cs=0, sw=42520
  [0] 그림: bin_id=2, common=12257×7880, tac=true (z=2)
          위치: 가로=문단 오프셋=0, 세로=문단 오프셋=0, 글자처럼=true

=== 완료: 1 구역, 2 문단 ===
```

### 위치 / 속성 매트릭스

| 위치 | bin_id | tac | 크기 (HU) | 부모 LINE_SEG.lh | wrap | offsets | rel_to |
|------|--------|-----|-----------|-------------------|------|---------|---------|
| 문단 0.0 / 표 (controls[2]) / 셀[0] 내부 paragraph | 2 | **false** (floating) | 12257×7880 | 셀 paragraph lh=1000 | Square | h=3463, v=5365 | Para, Para |
| 문단 0.0 / sibling control (controls[3]) | 1 | **true** (inline) | 16942×1154 | 문단 0.0 ls[0].lh=**1154** | 어울림 (의미 없음) | 0, 0 | Para, Para |
| 문단 0.1 / 단독 control (controls[0]) | 2 | **true** (inline) | 12257×7880 | 문단 0.1 ls[0].lh=**7880** | 어울림 | 0, 0 | Para, Para |

## 3. 추출된 패턴 (잠정)

### 패턴 P1 — Inline picture 와 paragraph LINE_SEG.line_height 일치

Inline picture 가 어느 paragraph 의 sibling control 이든, 그 paragraph 의 **LINE_SEG[0].line_height 가 picture 의 height 와 정확히 일치**한다.

- 문단 0.0 (lh=1154) ↔ controls[3] picture height=1154 ✓
- 문단 0.1 (lh=7880) ↔ controls[0] picture height=7880 ✓

이는 task plan 의 분기 (B) 가정 (`LINE_SEG.line_height = picture height`) 의 직접 증거.

### 패턴 P2 — Inline picture 가 단독 paragraph 에 위치하는 경우 존재

문단 0.1 은 cc=9, text_len=0, controls=1 인 **picture 전용 paragraph**. picture 가 표 sibling 으로 같은 paragraph 에 머무는 케이스 (controls[3]) 와는 다른 패턴.

→ 한컴이 토글 시 picture 를 **새 paragraph 로 분리하는 path** 가 존재할 가능성. 단 본 dump 만으로는 어느 입력 조건에서 그렇게 분리하는지 확정 불가.

### 패턴 P3 — Floating picture 와 inline picture 의 공존

셀[0] 내부 floating picture (bin_id=2) 와 문단 0.1 의 inline picture (bin_id=2) 가 **같은 bin_id, 같은 크기**로 공존. 어떤 작업의 결과인지 본 dump 만으로는 단정 불가.

가능한 해석 후보:
- (i) 사용자가 picture 두 개를 별도 작업으로 만들었음 (한 개 셀 안 floating, 한 개 별도 inline). 같은 bin_id 는 한컴이 같은 이미지 데이터를 재사용한 결과.
- (ii) 사용자가 셀 안 floating picture 를 토글한 결과, 한컴이 picture 를 셀 안에서 제거하지 않고 **사본을 새 paragraph 에 inline 으로 추가** 한 결과.
- (iii) 사용자가 셀 안 floating + 셀 외 별도 picture + 추가 토글 작업 등 복합 작업의 산출물.

## 4. 가설 검증 (Scenario A~D) — H1 확정

[hancom_picture_tac_toggle_verification.md](hancom_picture_tac_toggle_verification.md) 의 protocol 에 따라 사용자가 `samples/tac-verify/scenario-{a,b,c,d}-{before,after}.hwp` 8개를 한컴 2022 에서 직접 작업·저장 (2026-05-29). rhwp 가 dump 비교:

```bash
for s in a b c d; do
  ./target/debug/rhwp dump samples/tac-verify/scenario-${s}-before.hwp > /tmp/${s}-before.txt
  ./target/debug/rhwp dump samples/tac-verify/scenario-${s}-after.hwp  > /tmp/${s}-after.txt
  diff /tmp/${s}-before.txt /tmp/${s}-after.txt
done
```

### 시나리오별 diff 핵심

| 시나리오 | picture height | 부모 ls[0].lh (before→after) | text_len before/after | paragraph count before/after | 변화 매트릭스 |
|---------|----------------|--------------------------------|------------------------|--------------------------------|----------------|
| A — 1×1 작은 picture | 5331 | 1000 → **5331** | 0 / 0 | 2 / 2 | tac=false→true, Paper→Para, offset (10983,13500)→(0,0) |
| B — 1×1 큰 picture | 16038 | 1000 → **16038** | 0 / 0 | 2 / 2 | 위 동일 + before 의 ls[0].cs=17083, sw=25437 → after 의 cs=0, sw=42520 (큰 floating picture 의 폭 reservation 해제) |
| C — 3×3 중앙 셀 picture | 4847 | 1000 → **4847** | 0 / 0 | 2 / 2 | tac=false→true, Paper→Para, offset (26402,21923)→(0,0). 셀 위치 (중앙) 무관. |
| D — 본문 floating (셀 없음) | 19019 | 1000 → **19019** | 0 / 0 | 1 / 1 | tac=false→true, Paper→Para, offset (13428,13568)→(0,0). 본문/표 셀 무관. |

### 가설 확정 — **H1 (B-i)**

네 시나리오 모두 동일 패턴:
- ✓ **picture 위치 불변** — 같은 paragraph 의 같은 control index 에 남음. paragraph count 변화 없음. 셀 내부 ctrls 변화 없음.
- ✓ **부모 paragraph 의 LINE_SEG[0].line_height = picture height 정확 일치** (네 시나리오 모두).
- ✓ **tac=false→true, horz/vert_rel_to: Paper→Para, h/v_offset: floating 좌표→0**.
- ✗ **sentinel char (\u{FFFC}) 추가 없음** — paragraph.text_len 불변. char_offsets / char_shapes 도 변화 없음.
- ✗ **새 paragraph 분리 없음** — H2 (B-ii) 기각.
- ✗ **셀 안 이동 없음** — H3 (A) 기각.

추가로 발견된 B 시나리오 단독 항목 (큰 floating picture 의 LINE_SEG.cs 폭 reservation):
- before: cs=17083 (≈ picture width 17593), sw=25437 (= paper_width − cs ≈ 42520 − 17083). 큰 floating picture 가 본문 영역을 가려 paragraph 의 가용 폭이 줄어든 상태.
- after: cs=0, sw=42520 (전체 폭). picture 가 inline 글리프 화 되어 폭 reservation 불요.

본 cs/sw 갱신은 paginate 단계에서 자동으로 재계산될 가능성이 높다 (`paginate_if_needed` 호출 시). 본 fix 의 직접 수정 대상에 포함할지 Stage 1-c 단위 테스트에서 검증.

### 한컴 정합 fix 정책 (최종 확정)

`set_picture_properties_native` 의 tac false→true 마이그레이션:

```rust
// 1. picture 속성 갱신 (apply_picture_props_inner 가 처리하는 부분)
pic.common.treat_as_char = true;
pic.common.attr |= 0x01;
pic.common.horz_rel_to = HorzRelTo::Para;
pic.common.vert_rel_to = VertRelTo::Para;
pic.common.horizontal_offset = 0;
pic.common.vertical_offset = 0;

// 2. 부모 paragraph 의 LINE_SEG[0].line_height 갱신
let picture_height_hu = pic.common.height as i32;
if let Some(seg) = parent_para.line_segs.first_mut() {
    seg.line_height = seg.line_height.max(picture_height_hu);
    seg.text_height = seg.text_height.max(picture_height_hu);
    // bl (baseline) 도 picture 의 비율로 갱신: bl ≈ lh × 0.85 (관찰값 4531/5331)
    seg.baseline = (seg.line_height as f64 * 0.85) as i32;
} else {
    // line_segs 가 비어있는 경우 (드물지만) 신설
    parent_para.line_segs.push(LineSeg {
        line_height: picture_height_hu,
        text_height: picture_height_hu,
        baseline: (picture_height_hu as f64 * 0.85) as i32,
        line_spacing: 600,  // 한컴 기본
        ..Default::default()
    });
}

// 3. paragraph.text / char_offsets / char_shapes — 변경 없음 (한컴 정합)
// 4. picture 의 paragraph 위치 — 변경 없음
// 5. 표 sibling 인 경우 Table.dirty = true
// 6. mark_section_dirty + paginate_if_needed
```

`baseline ≈ lh × 0.85` 관찰값:
- A: bl=4531, lh=5331 → 4531/5331 ≈ 0.850
- B: bl=13632, lh=16038 → 13632/16038 ≈ 0.850
- C: bl=4120, lh=4847 → 4120/4847 ≈ 0.850
- D: bl=16166, lh=19019 → 16166/19019 ≈ 0.850

정확한 비율 (0.85) 은 한컴의 baseline 정책. picture 의 경우 ascent 가 line height 의 85% (남은 15% 가 descent) 로 보임. plan §3-2 의 `attach_picture_inline` helper 에 명시.
