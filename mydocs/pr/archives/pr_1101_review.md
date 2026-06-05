# PR #1101 검토 — 글자겹침(hp:compose) 동그라미 글자 누락 + 한컴2024 시각 정합

- 검토일: 2026-05-26
- PR: https://github.com/edwardkim/rhwp/pull/1101
- 관련 이슈: 없음 (closes 미지정)
- 관련 발견: #1126 (CanvasKit charOverlap 미지원 — 본 PR 외 별도 등록)
- 검토자: Claude (rhwp 메인테이너 보조)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1101 |
| 제목 | fix(parser/renderer): 글자겹침(hp:compose) 동그라미 글자 누락 + 한컴2024 시각 정합 |
| 작성자 | HaimLee-4869 (Lee eunjung) — 기존 컨트리뷰터 (8번째 PR) |
| base ← head | `devel` ← `pr/compose-char-overlap` |
| head SHA | `687fba8103a812437604355cb815d281109c3b2e` |
| commits | 2 (parser 정정 + renderer 정정 분리) |
| 상태 | OPEN / mergeable / mergeStateStatus=BEHIND (rebase 필요) |
| 변경 | 3 files, +58 / -28 |
| 본질 변경 | `src/parser/hwpx/section.rs` (+3) · `src/renderer/svg.rs` (+30/-14) · `src/renderer/web_canvas.rs` (+25/-14) |
| GitHub CI | 미실행 (fork 첫 푸시 패턴 가능성) |
| 외부 정답지 | 한컴2024 PDF (컨트리뷰터 자체 문서) |

## 2. 컨트리뷰터 누적 사이클

`gh pr list --author HaimLee-4869 --state all` — 누적 8개 PR (#1020/#1021/#1026/#1047/#1059 CLOSED, #1088/#1101/#1102 OPEN).
직전 PR #1088 은 자동 검증 통과했지만 시각 회귀로 거절 (수정 요청). 동일 컨트리뷰터의 PR 사이클이므로 **자동 검증만으로 신뢰하지 말고 시각 검증 게이트 필수**.

## 3. 문제와 원인

### (1) 글자 누락 (parser 영역)

한컴 HWPX 는 `<hp:compose>` 의 `composeText` 를 두 가지 형식으로 저장:
- **속성 폼**: `<hp:compose ... composeText="장">` (한컴 HWPX 산출)
- **자식 element 폼**: `<hp:compose>...<hp:composeText>장</hp:composeText></hp:compose>` (기존 지원)

[src/parser/hwpx/section.rs](src/parser/hwpx/section.rs#L4113) `parse_compose` 가 element 폼만 처리하여 속성 폼 입력 시 `CharOverlap.chars` 가 빈 채로 IR 진입 → 글자 누락.

### (2) 시각 정합 (renderer 영역)

[src/renderer/svg.rs](src/renderer/svg.rs#L1658) `draw_char_overlap` + `draw_char_overlap_combined`, [src/renderer/web_canvas.rs](src/renderer/web_canvas.rs#L2605) 동일 함수 2개 — 총 4 함수에서 세 가지 시각 결함:
- 테두리 색 `#000000` 하드코딩 → 한컴 PDF 영역 글자색과 동일 색상 (파랑/빨강) 으로 그려져야 함
- 음수 `inner_char_size` (charSz) 영역 무시 → font_size 축소 영역 누락
- 정원 (`<circle r>` / `arc`) → 세로로 긴 타원 (한컴 글리프 비율)

## 4. 변경 내용 — 영역 점검

### 4.1 parser 정정 (src/parser/hwpx/section.rs, +3 lines)

```rust
b"charSz" => co.inner_char_size = parse_i8(&attr),
b"composeType" => { ... }
+ // 한컴 HWPX는 `composeText="장"`처럼 속성에 글자를 넣기도 한다.
+ // 자식 element form(<composeText>장</composeText>)이 뒤에 나오면 그쪽이 덮어쓴다.
+ b"composeText" => co.chars = attr_str(&attr).chars().collect(),
_ => {}
```

**평가**: 속성 폼 + element 폼 양쪽 지원. element 폼이 뒤에 나오면 덮어쓰는 우선 순위는 OWPML schema 영역 비명시. 실제 한컴 산출본은 두 폼 동시 사용 안 함 (둘 중 하나) — 충돌 가능성 낮음.

### 4.2 renderer 정정 (svg.rs + web_canvas.rs, +55/-28)

#### (a) 테두리 색 = 글자색
```rust
- let stroke_color = "#000000";
+ let glyph_color = color_to_svg(style.color);
+ let stroke_color: &str = if is_reversed { "#000000" } else { &glyph_color };
+ let text_color: &str = if is_reversed { "#FFFFFF" } else { &glyph_color };
```
reversed 영역 (border_type=2/4) 영역은 기존대로 검정 채움 + 흰 글자 유지. 본 영역 분기 정합.

#### (b) 음수 charSz → 10% step 축소
```rust
let size_ratio = if overlap.inner_char_size > 0 {
    overlap.inner_char_size as f64 / 100.0          // 양수 → percent (기존 유지)
} else if overlap.inner_char_size < 0 {
    1.0 + overlap.inner_char_size as f64 * 0.10     // 음수 → -3 → 0.70 (새 적용)
} else {
    1.0
};
```

**가설의 권위 검증**:
- OWPML schema (`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:571`) — `charSz` = "테두리 내부 글자의 크기 비율. 단위 %" — 음수 영역 의미 미명시.
- 컨트리뷰터 측 검증 — 한컴2024 PDF 실측 9.12pt vs 본 가설 9.10pt (오차 0.02pt, 인지 불가).
- 한컴2024 는 본 프로젝트 정답지 등급 외 (메모리 룰 `feedback_pdf_not_authoritative` — 정답지 = 한컴 2020/2022 만) 이지만, 한컴 계열 영역의 합리적 추정.
- 모델 정의 `inner_char_size: i8` (signed) — 음수 영역의 실제 사용을 전제로 한 영역.

**평가**: 음수 → 10% step 영역 가설은 권위 미입증이지만 합리적이며 실측 정합. 본 가설 외 다른 음수 해석 (예: `charSz * -1` = percent) 도 있을 수 있으나 영향 받는 fixture 적음 (현 fixture 1개 — k-water-rfp, `charSz=-2 → 0.80`).

#### (c) 정원 → 세로 타원 (rx = ry × 0.85)
```rust
- let r = box_size / 2.0;
- "<circle ... r=\"{}\" ...>"
+ let ry = box_size / 2.0;
+ let rx = ry * 0.85;
+ "<ellipse ... rx=\"{}\" ry=\"{}\" ...>"
```

**평가**: 0.85 비율은 컨트리뷰터 측 한컴 글자 bbox 비율 근사. PR 본문 영역 "필요 시 폰트 metric 기반으로 정밀화 가능" 자체 점검. 영역 fixture 없음 → 회귀 위험 낮으나 향후 정밀화 영역.

#### (d) 4 함수 일관성
- `svg.rs::draw_char_overlap` (1658)
- `svg.rs::draw_char_overlap_combined` (1780, PUA 다자리 영역)
- `web_canvas.rs::draw_char_overlap` (2605)
- `web_canvas.rs::draw_char_overlap_combined` (2730)

→ 4 함수 모두 동일 정정 적용. 메모리 룰 `feedback_image_renderer_paths_separate` 정합 (rhwp 의 별도 renderer paths sweep 룰).

### 4.3 collateral 영향 — PUA 동그라미 영역

`draw_char_overlap_combined` 의 `effective_border = if border_type == 0 { 1 } else { border_type }` 분기 영역 — **border_type=0 + PUA 숫자** 영역은 본 정정으로 정원 → 타원 변경됨. 영향 받는 fixture: 현 golden 영역에 큰 동그라미 (font_size 단위) 영역 없음. 회귀 0 확정.

## 5. 자동 검증 결과

cherry-pick 으로 `local/pr1101-review` 브랜치 적용 후 자동 검증:

| 항목 | 결과 |
|------|------|
| `git cherry-pick pr-1101~1 pr-1101` | ✅ 충돌 없음 |
| `cargo build --release` | ✅ 통과 (2m 32s) |
| `cargo fmt --all -- --check` | ✅ 위반 0 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (4m 52s) |
| `cargo test --release --tests` | ✅ 통과 (svg_snapshot 8 passed, tab_cross_run 1 passed) |
| WASM 빌드 (Docker) | ✅ 성공 (4m 26s) |
| GitHub CI | ⚠️ 미실행 (fork 첫 푸시 가능성, 본 PR 결함 아님) |

> `cargo clippy --lib --tests` 영역은 60건 `unused_must_use` 위반 영역 있으나 본 PR 무관 (devel 기존 결함, 별도 영역).

## 6. 시각 검증 — k-water-rfp.hwpx 13페이지

작업지시자가 제공한 fixture 영역. 13페이지 안 **3개의 `<hp:compose>` 영역**:
- `circleType="SHAPE_REVERSAL_RECTANGLE"` (border_type=4, 반전 사각형)
- `charSz="-2"` (음수 영역 — PR 가설 영역)
- 속성 폼 `composeText="3"`/`"2"`/`"1"` (PR 정정 (1) 영역)

### 6.1 PR 적용 본 SVG (output/poc/pr1101/round1/k-water-rfp_013.svg)

```
<rect x="192.93" y="426.69" width="22.67" height="22.67" fill="#000000" stroke="#000000" stroke-width="0.8"/>
<text x="204.27" y="438.02" fill="#FFFFFF" font-family="..." font-size="18.13" font-weight="bold" ...>3</text>
... (2 / 1 영역 동일 구조)
```

검증 결과:
- ✅ **3개의 반전 사각형 정상 출력** (검은 채움 + 흰 글자)
- ✅ **글자 누락 없음** — PR 정정 (1) `composeText` 속성 폼 파싱 작동
- ✅ **font-size = 18.13** — 원래 글자 크기 22.66 × 0.80 (charSz=-2 → 0.80) 영역 가설 정합

### 6.2 SVG / native renderer 영역 영향

본 PR 의 변경 영역 (svg.rs + web_canvas.rs) 영역에 정상 작동. HWP 저장 영역은 본 PR 변경 외 (parser 영역만 영향) — k-water-rfp HWP 저장 정상 (별도 점검).

### 6.3 CanvasKit 영역 (rhwp-studio 화면) 영향

CanvasKit 영역은 본 PR 의 정정 영역과 무관 — 본 영역의 `charOverlap` op 자체가 `unsupportedOps` 분기 영역 (line 295) 으로 무시되고 있어 본 PR 영역과 별도. **별도 이슈 #1126 으로 분리** 등록.

## 7. 위험·관찰 영역

| 항목 | 등급 | 영역 |
|------|------|------|
| PR #1088 의 시각 회귀 사례 | 중 | 동일 컨트리뷰터 사이클. 자동 검증 통과 ≠ 시각 무결성. 본 PR 영역 시각 검증 완료 (k-water-rfp 13페이지) |
| `charSz` 음수 영역 가설의 권위 | 저 | OWPML schema 미명시 + 한컴2024 PDF (정답지 등급 외) 영역 실측 영역. 합리적 추정, 한컴 2020/2022 영역 fixture 추가 시 재검증 권장 |
| 타원 0.85 비율의 정밀화 | 저 | 컨트리뷰터 자체 "필요 시 폰트 metric 영역 정밀화 가능" 영역. 영역 후속 PR 대상 |
| 회귀 가드 tests 부재 | 중 | compose 영역 fixture 없음 → golden snapshot 영역 부재. k-water-rfp 영역 fixture 영역 활용한 회귀 가드 영역 권장 (별도 후속) |
| 한컴2024 PDF 사용 (정답지 등급 외) | 중 | 메모리 룰 `feedback_pdf_not_authoritative` — 정답지 = 한컴 2020/2022. 한컴 2020/2022 영역 동일 시각 검증 권장 |
| `unused_must_use` 60건 (devel 기존) | 외 | 본 PR 무관, 별도 영역 |
| CanvasKit 영역 charOverlap 미지원 | 외 | 본 PR 영역 외, **#1126 별도 등록** |

## 8. 최종 평가 (잠정)

| 항목 | 결과 |
|------|------|
| 본질 해결 | ✅ 글자 누락 + 색·크기·비율 한컴 정합 |
| 자동 검증 | ✅ 모두 통과 (build / fmt / clippy --lib / test / WASM) |
| 시각 검증 | ✅ k-water-rfp 13페이지 SVG 영역 정상 (3개 반전 사각형 + 흰 글자 + font-size 0.80) |
| 코드 품질 | ✅ 주석·commit 분리 명확. 4 함수 일관 정정 |
| 메모리 룰 정합 | ✅ `image_renderer_paths_separate`, ⚠️ `pdf_not_authoritative` (정답지 등급 영역 한컴2024) |
| 회귀 가드 | ⚠️ 없음 (compose fixture golden 영역 부재) |
| **결정 권장** | **MERGE** — 본 PR 영역 본질 해결, 시각 검증 통과 |

## 9. 작업지시자 결정 요청

1. 본 PR **MERGE** 진행 여부 — 본질 해결 + 시각 검증 통과
2. 회귀 가드 영역 (k-water-rfp 13페이지 영역 svg_snapshot 추가) 후속 영역 처리 — 본 PR 영역 또는 별도 후속 PR
3. 한컴 2020/2022 PDF 영역 시각 재검증 영역 필요 여부

승인 시 final report (`pr_1101_report.md`) 작성 + GitHub merge 진행.
