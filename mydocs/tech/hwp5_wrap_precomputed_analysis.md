# HWP5/HWPX wrap_precomputed 미적용 결함 분석

## 발견 일시

2026-05-04, PR #589 (Task #511 v2 + #554) 시각 판정 중

## 결함 증상

`hwp3-sample5-hwp5.hwp` (HWP5 OLE 포맷, HWP3에서 변환된 변환본) page 4 렌더링:
- 그림(pi=74, 126.4×94.5mm 시스템 다이어그램) 위로 텍스트가 겹쳐서 그려짐
- "커널의 가장 밑바탕은..." (pi=75, 415자) 가 그림 우측 wrap zone 에 정상 배치되지 않음
- 다른 페이지(8, 16, 22, 27 등) 동일 패턴 추정

스크린샷 상태 확인 (rhwp viewer):
- 그림 위에 한국어 텍스트가 산재 (wrap zone 미적용)
- 좌측 정렬된 짧은 단어들이 그림 영역에 겹쳐서 표시
- 페이지 하단 (그림 아래) 에는 정상 본문 (pi=77 "커널이 독립적으로...") 이 full width 로 출력

## 결함 본질

### 데이터 분석 (page 4 LineSeg)

| pi | 텍스트 | LineSeg 수 | cs | sw | vpos | 본질 |
|---|---|---|---|---|---|---|
| 74 | (빈 문단, 그림 anchor) | 1 | 37164 | 13860 | 0 | wrap zone 시작 |
| 75 | "커널의 가장 밑바탕은..." (415자) | 13+ | 37164 | 13860 | 1440~ 누적 | 그림 옆 wrap text |
| 76 | (빈 문단, 전환) | 1 | **0** | 51024 | 31680 | full width 전환 |
| 77 | "커널이 독립적으로..." | 6 | **0** | 51024 | 33120~ | full width 본문 |

### HWP3 vs HWP5 LineSeg 인코딩 차이

| 포맷 | vpos 인코딩 | cs/sw 인코딩 |
|------|----------|------------|
| HWP3 | 모두 0 (문단 시작 기준) | wrap zone 직접 인코딩 |
| HWP5 | 누적 절대값 (1440, 2880, ...) | wrap zone 직접 인코딩 |
| HWPX | HWP5 동일 | HWP5 동일 |

`hwp3-sample5-hwp5.hwp` 의 LineSeg cs=37164 (495.5px) 는 그림 우측 끝과 정합.

### 흐름 분석 (현재 결함 상태)

```
1. typeset.rs:478~  pi=74 (그림 anchor): wrap_around_cs=37164/sw=13860 등록
2. typeset.rs:489~  pi=75 (wrap text, any_seg_matches): 매칭 성공
3. typeset.rs:496   wrap_precomputed=false → current_column_wrap_around_paras 흡수
                    (height 소비 없음, FullParagraph rendering skip)
4. typeset.rs:505   continue (다음 문단으로)
5. layout.rs:3341   Task #525 가 layout_wrap_around_paras 호출 모두 제거
6. ⚠️ 흡수된 wrap text 어디에서도 렌더링 안 됨
   → pi=75 화면 누락 (본문 내용 손실)
   → diff -396px (page 4 used 556 / hwp_used 952)
   → 그림 위에 다른 텍스트가 겹쳐 보임 (시각적 결함)
```

### 보완6 (HWP3) 와의 차이

본 세션 보완6 의 `wrap_precomputed` 검출:
```rust
// src/parser/hwp3/mod.rs
if para.line_segs.len() > 1
    && para.line_segs.iter().all(|s| s.vertical_pos == 0)  // ← HWP5 미충족
    && para.line_segs.iter().any(|s| s.column_start > 0)
{
    para.wrap_precomputed = true;
}
```

`all(vpos == 0)` 조건이 HWP3 전용 — HWP5/HWPX 변환본 + native 모두 vpos 누적값이라 미적용.

## 아키텍처 검토

### 파서 구조

| 영역 | 위치 | wrap_precomputed 설정 |
|------|------|---------------------|
| HWP5 파서 | `src/parser/body_text.rs:153` (`para.line_segs = parse_para_line_seg(...)`) | ❌ 미설정 |
| HWPX 파서 | `src/parser/hwpx/section.rs:309,360,478` | ❌ 미설정 |
| HWP3 파서 | `src/parser/hwp3/mod.rs:1568,1579` | ✅ 설정 (보완6/8) |

**프로젝트 디렉토리 특이점**: HWP5 전용 디렉토리 (`src/parser/hwp5/`) 없음. HWP5 파서는 `src/parser/` 루트에 위치.

### 렌더러 처리 path

| 영역 | 위치 | wrap_precomputed 처리 |
|------|------|-------------------|
| typeset (흡수) | `src/renderer/typeset.rs:496` | `if !para.wrap_precomputed` → 흡수, true → skip |
| layout (Task #525) | `src/renderer/layout.rs:3341` | `layout_wrap_around_paras` 호출 모두 제거 |
| paragraph_layout (Task #489) | `src/renderer/layout/paragraph_layout.rs:828` | `has_picture_shape_square_wrap` (anchor 문단) → ComposedLine cs/sw 사용 |
| paragraph_layout (보완6) | `src/renderer/layout/paragraph_layout.rs:862,883,1208` | `wrap_precomputed` (wrap text 문단) → LineSeg cs/sw 사용 |

**Task #525 가 HWP5 wrap text 렌더링 path 를 제거** + 보완6 가 HWP3 에만 wrap_precomputed 설정 → HWP5/HWPX 회귀 잔존.

## 옵션별 비교

| 옵션 | 처리 위치 | 본질 | CLAUDE.md 정합 | 위험 |
|------|---------|------|--------------|------|
| **A** | `body_text.rs` + `hwpx/section.rs` 에 wrap_precomputed 후처리 추가 | 각 파서가 자기 포맷의 LineSeg 패턴 검출 | ✅ 100% | 일반 HWP5/HWPX 회귀 검증 필요 |
| B | `apply_hwp3_origin_fixup` 에서 변환본만 wrap_precomputed 설정 | HWP3 변환본만 처리 | ⚠️ 위치 부적절 (parser/mod.rs) | 변환본 휴리스틱 false-negative 시 결함 |
| C | `typeset.rs` 흡수 조건 변경 (LineSeg 패턴 검사) | 렌더러 통합 처리 | ⚠️ 파서 책임 → 렌더러 이전 | 광범위 영향 |
| D | `layout.rs` Task #525 호출 복원 + 중복 회피 가드 | wrap_around_paras 렌더링 path 복원 | ❌ Task #525 회귀 재발 우려 | 7 샘플 37 페이지 광범위 결함 재현 |
| E | `paragraph_layout.rs` Task #489 확장 (anchor 외 wrap text 문단) | LineSeg cs/sw 사용 영역 확장 | ⚠️ 렌더러 부담 | typeset.rs 흡수 + paragraph_layout.rs 미처리 충돌 |

## 권장 본질 — 옵션 A

### 근거

1. **CLAUDE.md 정합**: HWP3 전용은 `hwp3/`, HWP5 전용은 `body_text.rs`, HWPX 전용은 `hwpx/`. 각 파서가 자기 책임 영역에서 처리.
2. **본질 일관성**: `wrap_precomputed` 는 IR 모델 필드 — 파서가 LineSeg 구조 분석 후 설정하는 게 자연스러움. 보완6 (HWP3) 와 동일 패턴.
3. **렌더러 무수정**: 본 세션의 wrap_precomputed 처리 로직 (typeset/paragraph_layout) 그대로 동작.
4. **회귀 안전성**: 일반 HWP5 문서에서 wrap zone 없는 문단은 cs=0/sw=0 → wrap_precomputed=false → 영향 없음. wrap zone 있는 문단은 현재 흡수 후 미렌더링 결함 상태이므로 정상화 방향.
5. **변환본 휴리스틱 무관**: HWP5 native + 변환본 모두 LineSeg cs/sw 패턴 동일 → Task #554 휴리스틱 의존 없음.

### 구체 변경 영역

| 파일 | 추가 LOC | 본질 |
|------|---------|------|
| `src/parser/body_text.rs` | +15 | HWP5 paragraph 후처리: `len()>1 && any(cs>0 && sw>0)` → wrap_precomputed=true |
| `src/parser/hwpx/section.rs` | +15 | HWPX paragraph 후처리: 동일 |
| `src/parser/hwp3/mod.rs` | (변경 없음) | 보완6/8 그대로 |
| `src/renderer/*` | (변경 없음) | 본 세션 보완6 처리 로직 그대로 |

### 검출 조건 (HWP5/HWPX)

```rust
// pseudo
for para in &mut paragraphs {
    let multi_seg_wrap = para.line_segs.len() > 1
        && para.line_segs.iter().any(|s| s.column_start > 0 && s.segment_width > 0);
    let single_seg_wrap = para.line_segs.len() == 1
        && para.line_segs[0].column_start > 0
        && para.line_segs[0].segment_width > 0
        && /* picture/page-start 조건 — 보완8 동일 */;
    if multi_seg_wrap || single_seg_wrap {
        para.wrap_precomputed = true;
    }
}
```

vpos 조건은 제외 (HWP5/HWPX 는 누적값이 정상).

### 검증 요구

- `cargo test --lib` 1124+ passed (회귀 없음)
- `cargo test --test issue_546` exam_science.hwp 4페이지 정합 (Task #546)
- `cargo test --test issue_554` 12 passed (HWP3 변환본 페이지네이션)
- HWP5 광범위 fixture sweep (form-002, issue-147, issue-157, issue-267, table-text 등 golden SVG 회귀 없음)
- HWP5 변환본 시각 판정:
  - `hwp3-sample5-hwp5.hwp` page 4: pi=75 wrap text 가 x≈552px 위치에 13개 라인 정합
  - page 8/16/22/27 동일 패턴 정합
  - diff 0px 도달 (page 4 used == hwp_used)

## 별도 후속 task 권고

### Task X — Task #525 본질 재검토

`Task #525 가 layout_wrap_around_paras 호출을 모두 제거한 본질 (중복 emit 결함, 7 샘플 37 페이지)` 이 wrap_precomputed 메커니즘 도입 후에도 유효한지 재검토:
- wrap_precomputed 가 흡수를 skip 시켜 FullParagraph path 로 일관 처리
- layout_wrap_around_paras 함수 자체가 사용처 0건이면 dead code 제거 가능
- 또는 Task #525 가드 (중복 회피) 를 wrap_precomputed 기반으로 재구현

## 참조

### 관련 task / PR
- **Task #460 보완6/8** (본 세션, PR #589) — HWP3 wrap_precomputed 도입
- **Task #525** — Picture Square wrap 호스트 텍스트 중복 emit 정정 (layout_wrap_around_paras 호출 제거)
- **Task #489** — Picture/Shape Square wrap LINE_SEG.cs/sw 적용 (anchor 문단)
- **Task #546** — exam_science.hwp 회귀 정정 (Task #460 보완5 revert)
- **Task #554** — HWP5/HWPX 페이지네이션 회귀 정정 (HWP3 변환본 식별 휴리스틱)

### 관련 문서
- `mydocs/troubleshootings/square_wrap_pic_bottom_double_advance.md`
- `CLAUDE.md` § HWP3 파서 규칙

### 검증 자료
- `output/svg/pr_v2_after/hwp3-sample5_{004,008,016,022,027}.svg` (HWP3 native, 정합)
- `samples/hwp3-sample5-hwp5.hwp` (HWP5 변환본, 결함)

## 결론

옵션 A 진행 시:
- HWP5 native wrap-around 영역의 잠재 결함 (Task #525 후 미렌더링) 정상화
- HWP5 변환본 (Task #554) 시각 판정 통과
- HWP3 (보완6/8) 영향 없음
- CLAUDE.md HWP3 파서 규칙 준수 (각 파서 자기 책임)
- Task #525 본질 재검토는 별도 후속 task 분리 권고
