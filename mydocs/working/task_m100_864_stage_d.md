# Task #864 Stage D HWP3 그림 caption 진단

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## 배경

페이지 2 의 WMF y 좌표 정정 (Stage A-C) 완료 후, 페이지 3 의 첫 BMP image (단독) 의 캡션 "Cut&Paste 할 영역" 이 화면에 보이지 않는 결함 추가 발견.

작업지시자 확인: 한컴오피스 편집기에서 해당 image 의 "캡션 넣기" 메뉴가 활성화 되어 있음 → image control 의 caption metadata 임 확인.

## 진단 절차

### D.1 WMF binary 내부 확인

```bash
RHWP_DEBUG_WMF=1 ./target/release/rhwp export-svg samples/hwp3-sample14.hwp -p 2 ...
```

페이지 3 첫 WMF (window org=(960, 3120), ext=(1997, 816)) 의 record 종류:
- WINDOW_ORG, WINDOW_EXT, STRETCHDIB (image) 만 존재
- **EXTTEXTOUT, POLYGON, RECTANGLE, TEXTOUT 모두 없음**

→ "Cut&Paste 할 영역" 텍스트는 **WMF binary 에 없음**.

### D.2 HWP3 IR caption 확인

`rhwp dump samples/hwp3-sample14.hwp -s 0 -p 29` 결과 + 임시 caption 출력 추가:

```
[0] 그림: bin_id=4, common=20040×8160 (70.7×28.8mm), tac=true
[0]   [image_attr] effect=RealPic external_path="E$$00003.WMF"
[0]   caption: dir=Bottom width=0 paras=1 text="Cut&Paste 할 영역"
[0]     cap_para[0] text_len=14 line_segs=1 char_shapes=1 char_offsets=14 ps_id=23
[0]       ls[0] vpos=0 lh=1000 cs=0 sw=42520
```

→ HWP3 parser 가 **caption 을 정확히 파싱**하고 있음 (text="Cut&Paste 할 영역", direction=Bottom).

본질은 parser 가 아닌 **renderer 의 caption 위치 계산 결함**.

### D.3 Renderer caption 호출 확인

`RHWP_DEBUG_CAP=1` 임시 추가하여 `layout_caption` 호출 trace:

```
LAYOUT_CAPTION inline: pi=29 ci=0 text="Cut&Paste 할 영역" dir=Bottom
LAYOUT_CAPTION_INNER: text="Cut&Paste 할 영역" x=263.27 w=267.20 y=241.07 paras=1
  LAYOUT_CAPTION composed[0]: lines=1 text_runs.first_text="ComposedTextRun {...}"
  LAYOUT_CAPTION after: parent.children 2 -> 3 (added 1)
```

**캡션이 render tree 에 정상 추가됨** (1 child added). SVG 검색 시 글자 단위 (`<text x="339.87" y="252.4">C</text>` 등) 로 실제 출력됨.

### D.4 본질 식별 — 위치 결함

**caption y_start = 241.07 px**.

페이지 3 pi=29 (image) 의 layout 정보:
- body_top y = 132.27 px
- pic_h (`pic.common.height` 변환) = 8160 HU = **108.8 px** → image y range 132.3 ~ 241.1
- LINE_SEG.line_height = 11716 HU = **156.2 px** → 실제 layout 진행 = 132.3 ~ 288.5
- result_y (다음 단락 시작) = pic_y + max(line_advance, pic_h) = 132.3 + 156.2 = 288.5

**기존 caption_y 계산** (`src/renderer/layout.rs:2984` 기존):
```rust
CaptionDirection::Bottom => pic_y + pic_h + caption_spacing,
//                          132.3 + 108.8 + 0 = 241.1 px
```

→ caption 이 **image 의 시각 영역 (132.3 ~ 288.5) 안쪽**에 그려져 BMP 안에 가려짐.

**올바른 위치**: caption 은 `pic_y + image_advance` (LINE_SEG 기반 실제 진행) 위치에 그려져야 함.

## Stage D 결론

본질: `src/renderer/layout.rs:2984` (treat_as_char inline picture caption y 계산) 가 `pic.common.height` 만 사용. LINE_SEG.line_height (실제 layout 진행) 보다 작은 경우 image 안에 caption 이 겹침.

정정: caption_y 계산 시 `pic_y + pic_h` → `pic_y + max(line_advance, pic_h)` 로 변경.

📋 **Stage D 완료. Stage E 정정 구현 진행합니다.**
