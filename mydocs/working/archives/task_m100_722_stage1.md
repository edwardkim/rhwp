# Task #722 Stage 1 단계별 보고서 — 본질 진단

## 진단 결과 요약

본 환경의 paragraph_layout 이 wrap_anchor 매칭 paragraph 의 **모든 줄에 LINE_SEG cs/sw 적용** 하지만, 한컴은 **그림 시작 전 영역의 첫 줄은 wrap 미적용** (cs=0, sw=full). PDF 권위 자료 일치.

## 1. HWP5 변환본 IR 비교 (한컴 인코딩 권위)

### 본 환경 HWP3 native pi=175

```
ls[0]: vpos=12960, cs=24560, sw=26464  (그림 우측 영역)
ls[1]: vpos=14400, cs=24560, sw=26464  (그림 우측 영역)
```

### HWP5 v2024 변환본 (한컴 권위)

```
ls[0]: vpos=12960, cs=26264, sw=24760  (그림 우측 영역)
ls[1]: vpos=14400, cs=26264, sw=24760  (그림 우측 영역)
```

**두 환경 모두 두 줄 모두 wrap zone 영역으로 인코딩** (cs+sw=51024 동일). cs/sw 값 차이 (1704 HU ≈ 6mm) 는 spacing 차이로 영향 미미.

→ IR cs/sw 인코딩은 정합. **본질은 layout 단의 처리 차이**.

## 2. PDF 권위 자료 (한컴 2022 변환본)

`pdf/hwp3-sample5-2022.pdf` 페이지 8:
- "Figure 3-1. ..." 줄
- "아래에 디렉토리 트리 각 부분의 역할에 대하여 설명하였다." (그림 위 영역, **단일 줄**)
- 그림 (좌측, 그 아래 시작)
- 본문 "루트 파일시스템은..." (그림 우측 흐름)

PDF 정답 = 한컴 viewer 정합. 본 환경과 다름.

## 3. HWP5 변환본 SVG 출력 (본 환경 layout)

본 환경 layout 으로 HWP5 v2024 변환본 SVG 출력:
- HWP3 native 와 **동일한 결함** ("부분의" / "역할에" / "대하여" / "설명하였" 분산)

→ HWP5 변환본도 본 환경 layout 으로는 동일 결함 발생. **layout 단의 결함 본질**.

## 4. paragraph_layout 본질 위치 (paragraph_layout.rs:915-924)

```rust
let (line_cs_offset, line_avail_w_override) = if wrap_anchor.is_some() {
    let seg = para.and_then(|p| p.line_segs.get(line_idx));
    let cs = seg.map(|s| s.column_start as i32).unwrap_or(0);
    let sw = seg.map(|s| s.segment_width as i32).unwrap_or(0);
    let cs_px = crate::renderer::hwpunit_to_px(cs, self.dpi);
    let sw_px = if sw > 0 { Some(crate::renderer::hwpunit_to_px(sw, self.dpi)) } else { None };
    (cs_px, sw_px)
} else {
    (0.0, None)
};
```

`wrap_anchor.is_some()` 일 때 **모든 줄에** LINE_SEG cs/sw 적용. **첫 줄의 vpos 가 그림 시작 vpos 보다 위에 있는 경우** 도 wrap zone 처리.

## 5. 그림 vertical_offset vs paragraph 첫 줄 vpos

paragraph 175 IR:
- LINE_SEG ls[0] vpos = 12960 HU = 46 mm (paragraph 첫 줄)
- 그림 vertical_offset = 18680 HU = 65.9 mm (그림 시작)
- **차이**: 18680 - 12960 - 900 (line_height) = 4820 HU = 17 mm

paragraph 175 첫 줄이 그림 시작 17 mm 위 영역에 있음. 한컴은 이 영역을 wrap 영향 없는 **자유 영역** 으로 처리, 본 환경은 wrap zone 적용.

## 6. WrapAnchorRef 구조 (pagination.rs:154)

```rust
pub struct WrapAnchorRef {
    pub anchor_para_index: usize,
    pub anchor_cs: i32,
    pub anchor_sw: i32,
}
```

**anchor 그림 vertical_offset 정보 부재** — paragraph_layout 이 각 줄의 vpos 와 그림 시작 vpos 비교 불가.

## 7. 본질 결함 메커니즘 (정리)

```
typeset.rs wrap_around state machine:
  paragraph 175 의 LINE_SEG cs/sw 가 그림 의 cs/sw 와 매칭
  → ColumnContent.wrap_anchors 에 등록 (WrapAnchorRef { anchor_cs, anchor_sw })
  ↓
paragraph_layout.rs:915:
  wrap_anchor.is_some() → 모든 줄에 LINE_SEG cs/sw 적용
  ↓
paragraph 175 첫 줄 (vpos=12960):
  그림 vertical_offset (18680) 보다 위 영역인데도 cs=24560 적용
  → 텍스트 첫 줄이 그림 우측 영역에 layout
  ↓
시각 결함: 첫 줄 텍스트가 그림 우측에서 wrap (다중 줄로 분할)
```

## 8. 정정 방향 (Stage 2 후보)

### 옵션 A: WrapAnchorRef 확장 + paragraph_layout 가드

1. `WrapAnchorRef` 에 `anchor_vpos: i32` 필드 추가
2. typeset.rs wrap_around state machine 에서 anchor 그림의 vertical_offset 또는 paragraph_layout 시작 vpos 추적
3. paragraph_layout.rs:915 에서 각 줄 `vpos < anchor_vpos` 일 때 cs=0, sw=col_area.width 사용

**장점**:
- 본질 정정 (한컴 동작 정합)
- 회귀 위험 좁힘 (가드 명시)
- WrapAnchorRef 데이터 모델 확장만

**단점**: typeset 단에서 anchor vpos 추출 영역 확인 필요

### 옵션 B: typeset.rs wrap_around state machine 정정

paragraph 첫 줄 vpos < anchor vpos 인 경우 매칭 제외 → wrap_anchors 미등록 → paragraph_layout 분기 미진입.

**단점**: 같은 paragraph 의 둘째 줄부터는 wrap zone 적용 필요 → 줄 단위 매칭 복잡도 증가

### 옵션 C: paragraph_layout 자체 보정 (LINE_SEG vpos 기반)

paragraph_layout 에서 LINE_SEG vpos 를 paragraph layout y_start 와 비교하여 그림 시작 영역 구분. WrapAnchorRef 확장 불필요.

**단점**: paragraph_layout 이 그림 vertical_offset 직접 알 수 없음

### 권고: **옵션 A** (WrapAnchorRef 확장)

회귀 위험 좁힘 + 본질 정정 + 데이터 모델 명시 영역.

## 9. Stage 1 산출물

본 단계별 보고서 — paragraph_layout.rs:915 의 wrap_anchor 처리가 모든 줄에 cs/sw 적용하는 본질 결함 식별.

## 10. Stage 2 진행 승인 요청

본 진단 결과 + Stage 2 정정 방향 (옵션 A: WrapAnchorRef 확장 + paragraph_layout 가드) 승인 요청.

특히 **정정 방향 옵션 A vs B vs C** 에 대한 작업지시자 결정 영역.

---

## 11. Stage 2 옵션 A 시도 결과 — 본질 미해결 (2026-05-08 갱신)

옵션 A (WrapAnchorRef 확장 + paragraph_layout 가드) + ComposedLine merge 분기 적용 후 native build SVG 출력 진단:

**동작 부분** (정상):
- ComposedLine 들이 단일 줄로 merge 됨 — y=258.57 에 "￼아래에디렉토리트리각부분의역할에대하여설명하였다." 단일 줄 출력 확인

**미해결 부분** (본질 결함):
- 단일 줄 y=258.57 ∈ image y=249 ~ 425 (image vertical_offset=18680 HU = ~249px) → **이미지 뒤에 가려짐**
- 한컴 권위 (PDF) 정합: paragraph 175 는 image **위 자유 영역** (y < 249) 에 배치되어야 함
- 본 환경 paragraph_layout 이 LINE_SEG.vpos = 12960 HU 를 paragraph 줄 절대 y 로 사용 → image 영역 침범

**HWP3·HWP5 공통 결함 확인**:
- HWP3 native 와 HWP5 v2024 변환본 모두 동일 IR 인코딩 (ls[0].vpos=12960, 그림 voff=18680)
- 동일 layout 결함 발생 → HWP3 파서 단 정정 (D') 으로 공통 해결 불가 → **layout 단 공통 정정 필요**

## 12. 본질 결함 정정 방향 — D'' (폐기, §13 으로 대체)

D'' (paragraph_layout 단 ComposedLine merge + 자연 흐름 y_start 강제) 는 "한컴은 paragraph 175 첫 줄을 image 위 자유 영역에 배치" 라는 잘못된 가설에 기반. PDF 권위 자료 시각 판정 결과 본 가설 폐기.

## 13. 본질 재진단 — PDF 권위 시각 판정 (2026-05-08)

`pdf/hwp3-sample5-2022.pdf` 페이지 8 정합 (PNG 변환 시각 판정):

```
[Figure 3-1. 유닉스 디렉토리 트리의 각 부분들. 점선은 각 파티션 영역의 경계를 나타낸다.]
                                                       ┌──────────────────────────────┐
[image: 디렉토리 트리 그림 좌측]              아래에 디렉토리 트리 각 부분의 역할에 ...│
                                                마다 고유한 것으로서(비록 루트 ...    │
                                                파일시스템이 램 디스크나 ...          │
```

paragraph 175 텍스트 "아래에 디렉토리 트리 각 부분의 역할에 대하여 설명하였다." 는 **image 위 자유 영역이 아니라 image 우측 wrap zone 의 첫 줄**.

paragraph 176 "마다 고유한 것으로서..." 는 image 우측 wrap zone 둘째 줄~. paragraph 175 LINE_SEG cs=24560, sw=26464 IR 인코딩이 그대로 정합.

## 14. 본질 결함 위치 — typeset.rs wrap_around state machine

paragraph 175 (anchor host paragraph) 자체가 `wrap_anchors` 에 미등록:

```
[T722_TS] register para_idx=176 anchor_table_para=175
[T722_TS] register para_idx=177 anchor_table_para=175
```

paragraph 176, 177 만 등록. paragraph 175 자체는 미등록 → paragraph_layout 진입 시 `wrap_anchor=None` → LINE_SEG cs/sw 미적용 → col_area 전체 폭 layout → image 영역 침범 → image z-order 후 그려져 텍스트 가려짐.

## 15. 새 정정안 E (D'' 폐기 후 신규)

**Stage 2 기존 정정 전부 rollback**:
- `paragraph_layout.rs` ComposedLine merge 분기 + host_pic_vertical_offset 가드 + wrap_anchor.anchor_vpos 가드
- `pagination.rs` WrapAnchorRef.anchor_vpos 필드
- `typeset.rs` anchor_vpos 추출 + DEBUG_TASK722 로그

**본질 정정** — `src/renderer/typeset.rs` wrap_around state machine:
- **anchor host paragraph (paragraph 175 자체) 도 wrap_anchors 에 등록**
- paragraph_layout 진입 시 host paragraph 도 wrap_anchor=Some → LINE_SEG cs/sw 적용 → image 우측 wrap zone 1~2줄 layout

**회귀 위험 좁힘**:
- typeset.rs 한 곳 수정 (한 줄~몇 줄 추가)
- 본 환경 paragraph_layout 의 wrap_anchor 처리 그대로 활용
- 광범위 페이지네이션 sweep 으로 fixture 영향 검증
- IR 무수정
