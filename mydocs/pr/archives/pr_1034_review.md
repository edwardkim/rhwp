# PR #1034 검토 — Task #1008: HWP3 sample16 Shape/Text 정합 격차 종합 정정

- PR: [#1034](https://github.com/edwardkim/rhwp/pull/1034)
- 작성자: @jangster77 (Taesup Jang, 25+번째 PR — paper_based outline 시리즈 #1011/#1015/#1031 마무리 후 신규)
- closes #1008 (M100, v1.0.0, 메인테이너 작성 이슈, assignee 미지정)
- base: devel (PR base = `5263f53d` = PR #1032 머지 후, 현재 origin/devel = `bbd38e85` = PR #1033 머지 후)
- head: local/task1008 (10 commits — Stage 2~6 + merge 2 + fmt fix 1)
- mergeable: MERGEABLE, CI 전체 통과 (Build & Test / CodeQL / Render Diff)
- 변경 규모: +1751/-5, 12 파일 (2 코드 + 1 test + 9 문서)
- 일시: 2026-05-20

## 1. 컨트리뷰터 사이클 + 시리즈 위치 (`feedback_contributor_cycle_check`)

@jangster77 25+ PR 누적. paper_based outline 시리즈 (#997~#1011→#1015→#1031) 완료 후 신규 영역 진입:

- PR #1031 (closes #1029) — HWP3 외곽선 paper-edge 회귀 정정 (머지)
- **PR #1034 (closes #1008)** — HWP3 sample16 Shape/Text 4 격차 (본 PR)
- PR #1036 (closes #1035) — HWP3 vs HWP5 변환본 페이지 alignment fix 37.5%→93.75% (OPEN, 본 PR 후속)

HWP3 sample16 정합 시리즈로 진입 — PR #1034 가 Shape/Text 격차, PR #1036 이 페이지 alignment.

## 2. 이슈 #1008 배경

`samples/hwp3-sample16.hwp` 의 한컴 한글 정답지 비교에서 4 격차:

| Aspect | rhwp HWP3 | rhwp HWP5 변환본 | 한컴 한글 정답 (HWP3) | 격차 |
|--------|-----------|-----------------|-----------------|------|
| Box border style | dashed | dashed | **solid** | HWP3+HWP5 |
| Box fill | **plain** | gradient (purple) ✓ | gradient (purple) | **HWP3 한정** |
| HEAD 번호 라벨 | **"■1.추진목적■"** | "1. 추진목적" ✓ | "1. 추진목적" | **HWP3 한정** |
| 한글 단어 공백 | **"세계3대물서비스기업"** | "세계 3대 물 서비스 기업" ✓ | "세계 3대 물 서비스 기업" | **HWP3 한정** |

**중요**: 원본 이슈 가설 (HWP5 변환본 gradient strip) 이 한컴 한글 정답지 시각 검증 결과 **정반대로 판명** (gradient 있음 정답, HWP3 회귀). issue body + 수행/구현계획서 v2 재작성 — 매우 신중한 진단.

## 3. 본질 — 4 격차 정정 (src/parser/hwp3/ 격리)

### 3.1 격차 A — HWP3 Shape gradient IR 매핑 (`drawing.rs:792~830`)

HWP3 raw `Hwp3DrawingObjectGradientAttr` (drawing.rs:149~170) 가 이미 파싱되었으나 Fill IR 구축에서 `fill_type=Solid, gradient=None` 하드코딩으로 데이터 무시.

**Fix**: HWP5 매핑 contract (doc_info.rs:404) 와 동일 IR 주입:
```rust
let (fill_type, gradient) = if let Some(g) = header.gradient_attr.as_ref() {
    let grad = GradientFill {
        gradient_type: g.kind as i16,
        angle: g.angle as i16,
        center_x: g.center_x as i16, center_y: g.center_y as i16,
        blur: g.step as i16, step_center: 0,
        colors: vec![g.start_color, g.end_color],
        positions: vec![],
    };
    (FillType::Gradient, Some(grad))
} else { (FillType::Solid, None) };
```

격리도: HWP5/HWPX 무수정 (HWP5 는 이미 동일 contract).

### 3.2 격차 B — HWP3 Shape border LineType 2~7 → Solid normalize (`drawing.rs:758~785`)

HWP3 raw `style=0x0002` (LineType=2 Dash per spec) 가 점선 렌더되나 한컴 viewer 는 실선. HWP3 LineType 변형 spec 정의 vs 한컴 실제 동작 차이.

**Fix**: LineType 2~7 을 1 (Solid) 로 normalize:
```rust
let line_type = raw_attr & 0x3F;
if (2..=7).contains(&line_type) {
    (raw_attr & !0x3F) | 0x01
}
```

격리도: HWP3 sample 분포 sweep 명시 — `line_style=2` 는 sample16 한정 (다른 fixture 0/1 만) → narrow fix, 회귀 risk 0. HWP5/HWPX 무영향.

### 3.3 격차 C — HWP3 heading decoration 휴리스틱 strip (`mod.rs:2870~2960`)

HWP3 raw paragraph 가 `"═════■ 1.추진목적 ■═════"` 형태 decoration text 를 plain text 로 저장 (sample16 pi=70: 52자). 한컴 변환기/viewer 모두 decoration strip → 7자 ("1. 추진목적").

**Fix**: `fixup_hwp3_heading_decoration` post-processing pass + `strip_heading_decoration` 휴리스틱:
- Leading `═{5+}` + Trailing `═{5+}` + 중간에 `■...■` marker 패턴 detection
- 두 `■` 사이의 텍스트 추출

**한계 (PR 본문 명시)**: HWP3 spec 미참조 휴리스틱 — 의도된 `═══...■...■═══` typography 회귀 위험 있음. 25 fixture sweep 회귀 0 으로 1차 입증.

### 3.4 격차 D — HWP3 한글 spacing (CharShape dedupe + 폰트명 매핑) (`mod.rs:1869~1900, 2570~2585, 2908~2924`)

**Root cause 1**: rep CharShape + inline shape change 가 같은 `pos=0` 으로 양쪽 push (pi=4: rep id=57 base_size=1000 + inline id=58 base_size=1400). 한컴 변환기 dedupe → mod.rs char_shapes 빌드 후 dedupe loop 추가:
```rust
let mut deduped: Vec<CharShapeRef> = Vec::with_capacity(char_shapes.len());
for cs in char_shapes {
    if let Some(last) = deduped.last_mut() {
        if last.start_pos == cs.start_pos {
            *last = cs;  // inline override 가 의미적으로 정확
            continue;
        }
    }
    deduped.push(cs);
}
```

**Root cause 2**: HWP3 "신명조" vs HWP5 변환본 "HY신명조" 폰트명 mismatch → 폰트 metric 측정 차이로 char-by-char advance drift. 한컴 변환기 mimic 으로 5 legacy 매핑:
```rust
fn hwp3_font_name_to_hwp5(name: &str) -> String {
    match name.trim() {
        "신명조" | "신명" => "HY신명조".to_string(),
        "고딕" => "HY고딕".to_string(),
        "중고딕" => "HY중고딕".to_string(),
        "견고딕" => "HY견고딕".to_string(),
        "그래픽" => "HY그래픽".to_string(),
        _ => name.to_string(),
    }
}
```
원본 명칭은 `font.alt_name` 에 보존 (트레이싱).

**효과 (PR 본문 명시)**: HWP3 SVG 좌표가 HWP5 변환본과 byte-for-byte 일치 (`"` @131.69 → `세` @137.69 → ... → `한` @408.37).

**한계 (PR 본문 명시)**: 5 legacy 명칭만 매핑. 다른 명칭 발견 시 확장 필요.

## 4. cherry-pick dry-run 검증 — PR #1033 영역 보존 점검

PR base = `5263f53d` (PR #1032 머지 후 docs commit) — PR #1033 (`01a8c75e`) 이전. 본 환경 origin/devel = `bbd38e85` (PR #1033 머지 후).

**핵심 검증**: 본 PR 의 변경이 PR #1033 의 `advance_row_block_cut` / `is_block_split` 영역을 보존하는지.

### 4.1 PR head merge commit `f32644d6` 흡수 범위

```
f32644d6 Merge branch 'devel' into local/task1008
  └─ merged: 5263f53d (PR #1032 docs)
9a5bf1a6 Task #1008: fix CI cargo fmt
737df553 Merge branch 'devel' into local/task1008
  └─ merged: earlier devel
... Task #1008 Stage commits
```

PR 의 마지막 devel merge 가 `5263f53d` (PR #1032 후 docs commit) 까지만 흡수 — **PR #1033 (`01a8c75e`) 미흡수**. PR 작성 시점 PR #1033 도 OPEN 상태였을 가능성.

### 4.2 본질 파일만 적용 (cherry-pick 대체)

PR head 의 src/parser/hwp3/{drawing.rs, mod.rs} + tests/issue_1008_gradient.rs + 문서 9 파일만 선별 적용 (총 11 파일):

```bash
git checkout pr1034-review -- \
    src/parser/hwp3/drawing.rs \
    src/parser/hwp3/mod.rs \
    tests/issue_1008_gradient.rs \
    mydocs/{plans,report,working}/task_m100_1008*.md
```

이 방식으로 **PR #1033 의 src/renderer/* 영역은 자동 보존**.

### 4.3 본 환경 검증 결과 (dry-run, pr1034-squash 브랜치)

- 빌드 성공 (cargo build --lib)
- cargo test --release --lib **1319 passed** (PR #1033 후 1319 + PR #1034 tests 별도)
- cargo test --release --test issue_1008_gradient **4/4 passed**:
  - `hwp3_sample16_business_box_has_gradient` ✅
  - `hwp3_sample16_business_box_border_solid` ✅
  - `hwp3_sample16_heading_decoration_stripped` ✅
  - `hwp3_sample16_font_name_mapped_to_hwp5_convention` ✅

## 5. 코드 품질 평가

### 5.1 강점

- **HWP3 격리 엄격**: CLAUDE.md 의 "HWP3 전용 로직은 반드시 src/parser/hwp3/ 안에서만" 규칙 정합. HWP5/HWPX/renderer 무수정
- **HWP5 contract 참조**: 격차 A 의 gradient IR 매핑이 HWP5 doc_info.rs:404 와 동일 contract → IR 일관성 보장
- **case-specific 가드**: 격차 B 의 LineType 2~7 normalize 는 sample16 한정 (다른 fixture sweep 0/1 만 분포) — narrow fix
- **회귀 가드 4 테스트**: 각 격차에 대응하는 단위 테스트 — 영구화
- **본문 가설 정반대 발견 신중한 처리**: 원본 이슈 가설이 한컴 시각 검증으로 정반대 판명 시 issue body + plans v2 재작성 (`feedback_visual_judgment_authority` 정합)
- **alt_name 보존**: 폰트 매핑 시 원본 명칭 보존 → 트레이싱/디버깅 가능
- **단계별 commit 분리**: Stage 2~6 + fmt fix → 각 격차의 독립 책임 명확

### 5.2 우려

- **휴리스틱 (격차 C)**: HWP3 spec 미참조 패턴 detection — 의도된 `═══...■...■═══` typography 회귀 위험 (PR 본문 명시). 25 fixture sweep 회귀 0 으로 1차 입증, 광범위 sweep 필요
- **PR base 차이**: PR head 가 PR #1033 머지 이전 base → cherry-pick 직접 머지는 부적합 (PR #1033 영역 revert 위험). 본질 파일 선별 적용 필요
- **폰트 매핑 5 legacy 한정**: 다른 HWP3 fixture 에서 미매핑 폰트명 발견 시 확장 필요 (PR 본문 명시)
- **격차 C 공백 cosmetic 비범위**: rhwp "1.추진목적" vs 한컴 "1. 추진목적" period 뒤 공백 — HWP3 raw 부재, 한컴 자동 삽입 — 본 PR 범위 외

## 6. 검증 계획 (옵션 A 진행 시)

1. **본질 파일만 선별 적용** (cherry-pick 대체):
   - `src/parser/hwp3/{drawing.rs, mod.rs}` + `tests/issue_1008_gradient.rs` + `mydocs/{plans,report,working}/task_m100_1008*.md` (11 파일)
   - **PR #1033 영역 자동 보존** 확인

2. **CI 패턴 검증**:
   - cargo test --release --lib (1319 + 통합 0 failed expected)
   - cargo test --release --test issue_1008_gradient (4/4 expected)
   - clippy + fmt --all --check

3. **공개 fixture 광범위 sweep**:
   - HWP3 sample 11종 (hwp3-sample, sample10, sample11, sample13, sample14, sample16) + HWP5/HWPX 변환본 + 일반 fixture (exam_kor/math, aift, biz_plan, KTX)
   - 페이지 수 + LAYOUT_OVERFLOW + svg_snapshot
   - **격차 C 휴리스틱 회귀 가드**: 의도된 `═` 사용 사례 회귀 0 입증

4. **PR #1031 + #1032 + #1033 회귀 부재**:
   - HWP3 외곽선 paper-edge (sample16-hwp3 cover, PR #1031)
   - Task #1027 노트 정합 (sample16-hwp5, PR #1032)
   - 분할 표 (form-01/02, PR #1033 무영향)
   - issue_852 5/5

5. **WASM 빌드**: ~4.89MB 정합

6. **작업지시자 시각 판정** (`feedback_visual_judgment_authority`):
   - HWP3 sample16 cover RFP 박스 (gradient + solid border) — 한컴 한글 정답지 정합
   - 사업개요 "1. 추진목적" heading (decoration strip) + 한글 spacing 정합
   - 5p Ⅱ.제안일반사항 영역

## 7. 옵션 권고

| 옵션 | 설명 | 위험 | 권고 |
|------|------|------|------|
| **A. 본질 파일 선별 적용 + sweep + 시각 판정** | PR head 의 11 파일만 적용 (PR #1033 영역 자동 보존) → sweep + WASM + 작업지시자 시각 판정 → 머지 | 낮음 — dry-run 검증 완료 (1319 + issue_1008 4/4), HWP3 격리 엄격, 4 회귀 가드 영구화 | **권고** |
| B. supersede 요청 (rebase 후 재제출) | 컨트리뷰터에게 PR base 갱신 (origin/devel `bbd38e85`) 요청 후 재제출 | 매우 낮음 — 명시적 base 정합 | 검토 추가 절차 — 본 PR 본질 차이 미발생이므로 불필요 |
| C. cherry-pick (충돌 처리) | PR head 단일 squash → 충돌 영역 (PR #1033) 영역 origin/devel 정합 영역 유지 (`-X theirs` + 수동 처리) | 중간 — PR #1033 영역 처리 수동 결정 필요 | 옵션 A 보다 복잡 |

## 8. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — 본 PR 검증의 본질이 한컴 한글 정답지 시각 대조. 본 환경 sweep + 작업지시자 시각 판정 필수
- ✅ `feedback_visual_judgment_authority` — 본문 가설 정반대 발견 (gradient 가 회귀 아닌 정답) → issue body + plans 재작성. 모범 사례
- ✅ `feedback_diagnosis_layer_attribution` — 4 격차 각각의 root cause 정확 분리 (gradient IR 매핑 / LineType normalize / heading decoration / CharShape dedupe + 폰트명)
- ✅ `feedback_hancom_compat_specific_over_general` — 격차 B (sample16 한정 LineType 2~7), 격차 D-2 (5 legacy 폰트명만) 모두 case-specific
- ✅ `feedback_pr_supersede_chain` — @jangster77 25+ PR 누적. paper_based outline 시리즈 완료 후 HWP3 sample16 시리즈 진입 (#1008 + #1035)
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt 전체 CI 패턴
- ✅ `feedback_close_issue_verify_merged` — PR #1031/#1032/#1033 머지 검증 + 본 PR 머지 시 회귀 부재 필수
- ✅ `feedback_contributor_cycle_check` — @jangster77 시리즈 위치 명시

## 9. 작업지시자 결정 요청

| 결정 | 옵션 |
|------|------|
| 진행 여부 | A (본질 파일 선별 적용 + sweep + 시각 판정) / B (supersede) / C (cherry-pick 충돌 처리) |
| sweep 검증 범위 | HWP3 11종 + HWP5/HWPX 변환본 + 일반 fixture / 광범위 |
| 시각 판정 | 본 환경 정량 입증 + 작업지시자 시각 판정 / 정량만 |
