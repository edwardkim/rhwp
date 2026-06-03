# Task #959 Stage 1 — 회귀 source 정확 식별

## 1. RHWP_DEBUG_TAC_CURSOR 추적 결과

시험지 page 1 단 1 (우측 단) 의 문8 (pi=64) ~ 문9 (pi=73) 사이 cursor:

```
TAC_CURSOR FullPara pi=64 y_in=550.0 y_out=570.0 dy=20.0  ← 문8 line
TAC_CURSOR FullPara pi=65 y_in=576.0 y_out=609.6 dy=33.6  ← ① ② ③
TAC_CURSOR FullPara pi=66 y_in=615.7 y_out=649.3 dy=33.6  ← ④ ⑤
TAC_CURSOR FullPara pi=67 y_in=655.3 y_out=673.3 dy=18.0  ← (빈)
TAC_CURSOR FullPara pi=68 y_in=673.3 y_out=691.4 dy=18.0  ← (빈)
TAC_CURSOR FullPara pi=69 y_in=691.4 y_out=709.4 dy=18.0  ← (빈)
TAC_CURSOR Shape pi=69 ci=0 y_in=709.4 y_out=983.4 dy=274.0 ⚠️ ← 회귀
TAC_CURSOR FullPara pi=70 y_in=983.4 y_out=1001.4 dy=18.0
TAC_CURSOR FullPara pi=71 y_in=1007.5 y_out=1025.5 dy=18.0
TAC_CURSOR FullPara pi=72 y_in=1025.5 y_out=1043.5 dy=18.0
TAC_CURSOR FullPara pi=73 y_in=1043.5 y_out=1095.2 dy=51.7  ← 문9 line
```

→ **Shape pi=69 ci=0 가 column 에 274px advance**. 후속 paragraph (pi=70~72 빈 줄 + pi=73 문9) 모두 그만큼 처짐.

## 2. pi=69 picture 상세 (dump)

```
[0] 그림: bin_id=38, common=16786×20400 (59.2×72.0mm),
    tac=false
[0]   crop=(0,0,90000,109380)
    크기: 59.2mm × 72.0mm (16786×20400 HU)
    위치: 가로=단 오프셋=79.5mm(22531) 정렬=Center
         세로=문단 오프셋=0.5mm(150) 정렬=Top
    배치: 위아래, 글자처럼=false, z=100
```

- **wrap = 위아래 (TopAndBottom)**
- **tac = false** (non-treat-as-char)
- **horz_rel_to = Column (단)**, 가로 offset = 79.5mm
- **vert_rel_to = Para (문단)**, 세로 offset = 0.5mm
- Picture height = 272 px

## 3. rhwp 실제 emit 위치

SVG image element:
```
<image x="769.6" y="693.4" width="223.8" height="272" .../>
```

- x = 769.6 → **body right (759.7) 외부!**
- 우측 단 column area 외부 (column 끝 x ~ 759)
- Picture 가 column 밖에 emit 됨에도 cursor 는 column 안에서 advance

## 4. 한컴 PDF 정합 검증

`pdf/3-11월_실전_통합_2022.pdf` page 1 시각 분석:
- 좌측 단: 문1~문5 정상
- 우측 단: 문6~문9 정상 (문7, 문8, 문9 사이 일관된 간격)
- **우측 단에 picture 표시 안 됨** (해당 영역에 visible 요소 없음)

→ 한컴 viewer 는 pi=69 picture 를 우측 단 column flow 에서 제외. rhwp 는 picture 를 column 에 reservation → 문9 처짐.

## 5. Root cause

`src/renderer/layout.rs:3500-3518` 의 non-TAC picture 처리:

```rust
} else {
    let comp = composed.get(para_index);
    ...
    let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
    let pic_container = LayoutRect {
        x: col_area.x, y: pic_y,
        width: col_area.width,
        height: col_area.height - (pic_y - col_area.y),
    };
    result_y = self.layout_body_picture(...);  // ← cursor advance
}
```

`is_paper_based` (line 3478): `vert_rel_to ∈ {Paper, Page} && horz_rel_to ∈ {Paper, Page}` — 단 (Column) 인 경우 false.

→ horz_rel_to=Column + vert_rel_to=Para 인 경우 paper_images 가 아닌 column 에 그려지고 cursor advance.

## 6. Fix 후보

### A. horz_rel_to=Column 인 picture 가 column 영역 외부 (offset+width > col_width) 일 때 cursor advance 제외

**위치**: `layout_shape_item` 라인 3500-3554 분기.

**변경**: picture 의 horz_rel_to=Column + offset 이 column 외부일 때 paper_images 처리 또는 advance skip.

- 위험: **중** (다른 Column 기반 picture sample 영향 가능)

### B. is_paper_based 조건 확장 (horz_rel_to=Column 포함)

**위치**: 라인 3478.

**변경**:
```rust
let is_paper_based = (pic.common.vert_rel_to == VertRelTo::Paper || pic.common.vert_rel_to == VertRelTo::Page)
    && (pic.common.horz_rel_to == HorzRelTo::Paper || pic.common.horz_rel_to == HorzRelTo::Page
        || pic.common.horz_rel_to == HorzRelTo::Column);
```

- 위험: **고** — Column 기반 picture 모두 paper_images 처리 (column flow 제외) — 광범위 영향

### C. Picture 위치 (x) 가 column area 외부일 때만 column flow advance 제외

**변경**: `layout_body_picture` 결과로 받은 picture 의 실제 x 가 col_area.x ~ col_area.x+col_area.width 범위 외이면 result_y = y_offset (advance 없음).

- 위험: **중** (정밀, 본 sample 한정 fix 가능)

## 7. 권장 fix

**Option C** — picture 의 실제 emit x 위치 기반 정밀 조건. 본 sample 처럼 column 외부에 위치한 picture 만 advance skip, column 내부 picture 는 정상 advance 유지.

## 8. 후속 (Stage 2)

- Stage 2 구현 계획 V2 — Fix C 의 안전한 구현 + 위험 평가
- Stage 3: 구현 + 시험지 page 1 단위 검증
- Stage 4: 다중 sample 회귀 검증 (특히 Column 기반 picture 보유 sample)
