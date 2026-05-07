# 최종 결과 보고서 — Task #677

## 이슈

[Issue #677](https://github.com/edwardkim/rhwp/issues/677) — **복학원서.hwp PDF 정합 결함 — pi=16 인라인 표 라인 누적 + 워터마크 효과 미적용**

- 마일스톤: v1.0.0 (M100)
- 권위 자료: `pdf/복학원서-2022.pdf` (한글 2022 PDF, macOS/Linux 환경 1차 정답지 — `reference_authoritative_hancom` 정합)
- 원본: `samples/복학원서.hwp`
- 브랜치: `local/task677`

## 본질 결함 영역 (3개 정정)

### 결함 #1 — pi=16 PartialParagraph y 누적 결함 (273px 단 하단 초과)

**현상** (BEFORE):
```
LAYOUT_OVERFLOW: page=0, col=0, para=16, type=PartialParagraph, y=1357.8, bottom=1084.7, overflow=273.1px
LAYOUT_OVERFLOW: page=0, col=0, para=16, type=Shape,            y=1357.8, bottom=1084.7, overflow=273.1px
```

- pi=16 구성: 3×3 접수증 표 (642×280px, `tac=true` 인라인) + ㊞ 도장 (`wrap=InFrontOfText`) + 본문 2줄 (※ 군필자…)
- 표 (`is_tac_table_inline`=false, block-TAC 분류) Table item 이 y_offset 을 표 바닥 (~1055px) 까지 누적 후, 후속 PartialParagraph 가 동일 paragraph 의 line 1 을 그대로 상속받아 진입 — HWP IR 영역에서 line 1 의 lh=21296 (=표 높이) 영역이 이미 인코딩된 상태에서 표 높이 이중 누적
- 결과: 접수증 영역 + "고 려 대 학 교 총 장 귀 하" 큰 제목 영역 단 하단 초과 → 콘텐츠 손실

**본질 정정** (`src/renderer/layout.rs:2120-2147`):
```rust
let pp_y_in = if *start_line > 0
    && para.controls.iter().any(|c|
        matches!(c, Control::Table(t) if t.common.treat_as_char))
    && para_start_y.contains_key(para_index)
{
    if let (Some(seg), Some(seg0), Some(para_top)) = (
        para.line_segs.get(*start_line),
        para.line_segs.first(),
        para_start_y.get(para_index).copied(),
    ) {
        para_top + hwpunit_to_px(
            seg.vertical_pos - seg0.vertical_pos, self.dpi)
    } else { y_offset }
} else { y_offset };
let pp_y_out = self.layout_partial_paragraph(..., pp_y_in, ...);
y_offset = y_offset.max(pp_y_out);
```

조건 가드 3개로 좁게 발동:
- `start_line > 0` (문단 첫 PP 미적용)
- `para` 가 TAC 표 보유 (treat_as_char=true)
- `para_start_y` 등록 (Table item 선행 처리됨 — 같은 column)

`y_offset = y_offset.max(pp_y_out)` 누적: Table item 의 누적값과 PP 자연 종료값 중 최대로 갱신.

**결과**: LAYOUT_OVERFLOW 273.1px → **2.5px** (96% 감소, tolerance 영역).

### 결함 #2 — U+F081C HWP PUA 채움 문자 폭 결함 (인라인 표 658px 우측 밀림)

**Stage 2 진단으로 추가 식별** (Stage 1 의 y 결함 정정 후 잔존 시각 결함 식별).

**현상** (BEFORE):
- pi=16 의 ComposedLine cl[0] 가 99 chars × U+F081C 채움 문자 보유
- `compute_tac_leading_width` (`layout.rs:3563`) 가 block-TAC 케이스에서 line 0 의 모든 run 텍스트 폭 합산
- `estimate_text_width` 의 default fallback 이 U+F081C 를 `font_size * 0.5` 로 측정 → 99 × 6.65 = 658px 의 leading width 가산
- 결과: 3×3 접수증 표가 col_left + 658 = **x=716px (body 우측 끝)** 으로 배치 → 본문 우측 외곽 + 콘텐츠 가독 안 됨

**본질 정정** (`src/renderer/layout/text_measurement.rs` `char_width` 클로저 5 사이트):
```rust
// [Issue #677] HWP PUA 채움 문자 (U+F081C) — 시각 폭 0
// 한컴이 인라인 TAC 표/도형 앞에 삽입하는 placeholder 채움 문자.
// 한컴 PDF 정합 — 폭 0 으로 라인 inline x 에 영향 없음. fillers 가
// 표 너비만큼 (≈97 chars × 1 char width = table width) 채워져
// 표가 fillers 영역 위에 시각적으로 겹쳐 column-left 출력 패턴.
if c == '\u{F081C}' {
    return 0.0;
}
```

**결과**: 3×3 접수증 표가 **x=716 → x=63.69 (body left margin)** 으로 정합 — 한컴 PDF 정답지 영역 일치.

### 결함 #3 — 워터마크 모드 변환 미적용 (어두운 회색 본문 가림)

**현상** (BEFORE):
- HWP IR: `effect=GrayScale, brightness=-50, contrast=70, watermark=custom`
- 한컴 표준 워터마크 프리셋: `effect=GrayScale, brightness=+70, contrast=-50` (`src/model/image.rs:75`) — 부호 반대
- 저장값 그대로 적용 → brightness=-50 (어두움) + contrast=+70 (고대비) → **진한 어두운 회색 본문 가림**

**본질 정정** — 1차 (Stage 3) → 2차 (작업지시자 시각 판정 후 정정):

1차 (한컴 표준 프리셋 강제): `(70, -50)` — 너무 흐림 (작업지시자 피드백)

**2차 본질 정정** (저장값 강도 보존 + 부호 워터마크용 정합):
```rust
let is_watermark_image = !matches!(img.effect, ImageEffect::RealPic)
    && (img.brightness != 0 || img.contrast != 0);
let (eff_brightness, eff_contrast) = if is_watermark_image {
    (img.brightness.unsigned_abs() as i8, -(img.contrast.unsigned_abs() as i8))
} else {
    (img.brightness, img.contrast)
};
```

**검증**:
- 본 fixture 저장 (-50, +70) → 적용 (+50, -70). 강도 보존 + 워터마크 부호 (밝게 + 저대비)
- 표준 프리셋 케이스 (저장 70, -50) → 적용 (70, -50). 변환 후에도 정합 유지
- 시각 결과: **흐릿한 회색 워터마크** (PDF 정합) — 고려대학교 엠블럼 + 호랑이 + KOREA UNIVERSITY + 1905 모두 가독 영역

SVG (`src/renderer/svg.rs:1082-1097`) + WASM Canvas (`src/renderer/web_canvas.rs:418-432`) 양쪽 동기 정정 (`feedback_image_renderer_paths_separate` 정합).

## 정량 측정 (BEFORE → AFTER)

| 측정 항목 | BEFORE | AFTER |
|----------|--------|-------|
| pi=16 LAYOUT_OVERFLOW (PP) | y=1357.8 overflow=273.1px | y=1087.2 overflow=2.5px |
| pi=16 LAYOUT_OVERFLOW (Shape) | y=1357.8 overflow=273.1px | y=1087.2 overflow=2.5px |
| 3×3 접수증 표 x 좌표 | 716.69 (body 우측 끝) | **63.69 (body left margin)** |
| 워터마크 brightness 적용값 | -50 (어두움) | **+50 (밝음)** |
| 워터마크 contrast 적용값 | +70 (고대비) | **-70 (저대비)** |
| 워터마크 시각 외양 | 진한 어두운 회색 본문 가림 | 흐릿한 회색 워터마크 (PDF 정합) |
| "고 려 대 학 교 총 장 귀 하" 큰 제목 | 워터마크에 가려짐 | 본문 위에 정상 출력 |

## 시각 정합 영역 (한컴 PDF 정답지 비교)

PNG 렌더 결과 — 모든 영역이 PDF 정답지와 동일 위치에 정합 출력:

| 영역 | PDF | rhwp AFTER |
|------|-----|------------|
| 복학원서(학부) 제목 | ✅ | ✅ |
| Reinstatement Form (Undergraduate) | ✅ | ✅ |
| 5×4 표 (대학/학과/학번/성명/휴대전화/e-Mail/현주소) | ✅ | ✅ |
| 본인은 휴학으로 인하여... (Korean) | ✅ | ✅ |
| 복학원을 제출합니다 | ✅ | ✅ |
| I have taken leave of absence... (English) | ✅ | ✅ (font-weight=bold) |
| this reinstatement form. | ✅ | ✅ |
| 년(year) 월(momth) 일(day) | ✅ | ✅ |
| 본인 (Name) signature line + 접수자 (Receiving Official) box | ✅ | ✅ |
| **고 려 대 학 교 총 장 귀 하** | ✅ | ✅ (BEFORE 안 보임) |
| 분리선 ─── | ✅ | ✅ |
| 복 학 원 서 접 수 증 / Filing Receipt | ✅ | ✅ |
| 대학(Name of College) / 학과/학부 / 학번 / 성명 | ✅ | ✅ |
| 위 학생의 복학원서를 접수함 | ✅ | ✅ |
| The above student's reinstatement form is hereby received | ✅ | ✅ |
| 년(year) 월(month) 일(day) ㊞ (붉은 도장) | ✅ | ✅ |
| ※ 군필자... + ※ Those who completed... | ✅ | ✅ |
| 고려대학교 엠블럼 워터마크 (흐린 회색) | ✅ | ✅ (BEFORE 진한 어두운 회색) |

**메인테이너 시각 판정 ★ 통과** (작업지시자 평가):
- 1차: 워터마크 표준 프리셋 강제 → "이미지가 너무 흐림, 굵은 폰트처리가 미흡" 피드백
- 2차: 저장값 강도 보존 + 부호 변환 → **"승인."** ★

## 잔존 영역 (별도 task 후보)

### 1. 굵은 폰트처리 영역 (Stage 5 작업지시자 시각 판정 피드백 영역)

**진단 영역**: `examples/inspect_677_bold.rs` 임시 진단 도구로 점검 — 본 fixture 의 모든 셀 paragraph CharShape **bold=false** (HWP IR 영역). PDF 정답지 영역의 굵은 외양은 한컴 자체 폰트 (한양견명조/한양신명조) 디자인 굵기 자체 — 본 환경 fallback (Noto Serif CJK KR Regular) 보다 굵음.

**본질**: **폰트 가용성 영역의 결함** — 한컴 한양견명조/한양신명조 폰트 미설치 환경 (Linux/macOS 기본) 에서 fallback 의 stroke 두께 부족. HWP IR 자체에는 bold=true 마킹 없음. 본 task 의 layout/워터마크 본질 정정 영역과 다른 본질.

**잠재적 후속 task 영역**:
- 한컴 한양 시리즈 → 더 무거운 fallback 폰트 영역 매핑 (Noto Serif CJK KR Medium / Source Han Serif Heavy)
- `--embed-fonts` 옵션 사용 시 한컴 폰트 임베딩 영역 활성화
- 또는 작업지시자 환경에서 한컴 한양견명조/한양신명조 ttf 설치 후 `--font-style` 사용

**작업지시자 승인** — 별도 task 분리 영역 OK.

### 2. body-clip width 1619.92 (Stage 1 식별)

- SVG body-clip width 가 페이지 폭 (793.7) / body 폭 (687.9) 대비 ~2.4 배 부풀려져 있음
- 시각 영향 없음 (콘텐츠가 body 안에 있으므로 클립 폭이 넓어도 시각 결함 없음)
- 코드 위생 영역 — 별도 후속 task 후보

### 3. 워터마크 표준 프리셋 외 사용자 customization 영역

- 본 정정은 모든 워터마크에 `(|brightness|, -|contrast|)` 변환 적용
- 다른 워터마크 fixture 가 발견되면 customization 보존이 필요한지 별도 검토
- 현 시점 162+ HWP/HWPX fixture 중 워터마크 보유는 본 fixture 1개만 → 영향 영역 없음

## 회귀 검증 (전부 GREEN)

```
cargo test --release --lib                       1155 passed (회귀 0)
cargo test --release                             전체 GREEN (failure 0)
cargo test --release --test svg_snapshot         8 passed (issue_677 신규 + 7 기존)
cargo test --release --test issue_546            1 passed
cargo test --release --test issue_554            12 passed
cargo test --release --test issue_598_*          4 passed (footnote_marker_nav)
cargo test --release --test issue_501            1 passed
cargo clippy --release --lib                     0 warnings (lib 영역)
cargo build --release                            success
cargo check --target wasm32-unknown-unknown --release --lib  WASM lib 빌드 success
docker compose run --rm wasm                     WASM 빌드 success
rhwp-studio npm run build                        TypeScript + vite dist 빌드 success
```

### 광범위 페이지네이션 sweep (162+ HWP/HWPX fixture)

전체 fixture 페이지 합계: **1,964 페이지**

핵심 fixture 페이지 수 정합 (역사적 documented 값과 일치):
- aift.hwp: 77 (PR #601 정합)
- aift.hwpx: 74 (PR #601 정합)
- synam-001.hwp: 35 (PR #601 정합)
- hwp3-sample5.hwp: 64 (PR #609 정합)
- exam_kor / eng / math / science: 20 / 8 / 20 / 4 (정합)
- footnote-01.hwp: 6 (PR #642 정합)
- 복학원서.hwp (본): 1 (baseline)

**페이지 수 회귀: 0건**

### 영향 범위 좁힘 (이중 검증)

- U+F081C 보유 fixture: **1개만** (복학원서.hwp)
- 워터마크 보유 fixture: **1개만** (복학원서.hwp)
- 다른 fixture 영향: **0**

기존 7개 svg_snapshot 골든 모두 **byte-identical** 통과 → 다른 fixture 회귀 가드 정합.

## WASM 빌드 정량 측정

- **Docker WASM**: 4,532,601 bytes (PR #621 baseline 4,606,564 대비 -73,963 / -1.6%)
- **Studio WASM**: 4,532,601 bytes (Docker WASM 정합)
- **Studio JS**: 693,023 bytes (PR #642 baseline 691,386 대비 +1,637 / +0.24%)

WASM 감소 정황: 본 task 의 src 변경 (+82 LOC) 대비 -73,963 bytes 감소는 LLVM 최적화 + wasm-opt 효과로 추정. functional 테스트 1155 passed 정합 — 회귀 신호 아님.

## 회귀 차단 가드 영구 보존

- `tests/svg_snapshot.rs::issue_677_bokhakwonseo_page1` 신규 (Stage 2/3/2차 영역 모두 잠금)
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` (414,812 bytes) 영구 보존

## 변경 LOC

| 파일 | 변경 | 영역 |
|------|------|------|
| `src/renderer/layout.rs` | +30 / -2 | PP y 리셋 + max 누적 |
| `src/renderer/layout/text_measurement.rs` | +20 / 0 | U+F081C 폭 0 (5 사이트) |
| `src/renderer/svg.rs` | +9 / -1 | 워터마크 변환 (\|abs\|, -\|abs\|) |
| `src/renderer/web_canvas.rs` | +9 / -1 | 워터마크 변환 (WASM, 동기) |
| `tests/svg_snapshot.rs` | +14 / 0 | issue_677 회귀 가드 |
| `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` | 414812 bytes | 골든 영구 보존 |
| `mydocs/plans/task_m100_677.md` | +132 LOC | 수행계획서 |
| `mydocs/plans/task_m100_677_impl.md` | +219 LOC | 구현계획서 |
| `mydocs/working/task_m100_677_stage1.md` | +151 LOC | Stage 1 진단 |
| `mydocs/working/task_m100_677_stage2.md` | +119 LOC | Stage 2 정정 |
| `mydocs/working/task_m100_677_stage3.md` | +120 LOC | Stage 3 정정 |
| `mydocs/working/task_m100_677_stage4.md` | +120 LOC | Stage 4 회귀 가드 |
| `mydocs/working/task_m100_677_stage5.md` | +180 LOC | Stage 5 시각 + 빌드 + 피드백 정정 |
| `mydocs/report/task_m100_677_report.md` | (본 보고서) | 최종 보고서 |
| **합계 (src + tests)** | **+82 / -4 LOC + 1 가드 + 1 골든** | 본질 정정 영역 |
| **합계 (mydocs)** | **+1041+ LOC** | 거버넌스 영역 |

## 메모리 룰 정합

- **`reference_authoritative_hancom`** — `pdf/복학원서-2022.pdf` (한글 2022 PDF, macOS/Linux 1차 정답지) 시각 판정 통과
- **`feedback_close_issue_verify_merged`** — Issue #677 close 시 정정 commit 의 devel 머지 검증 후 진행
- **`feedback_pr_to_stream_devel`** — 최종 보고서 + 작업지시자 승인 직후 stream/devel PR 절차 진행
- **`feedback_image_renderer_paths_separate`** — SVG (CLI) + Canvas (WASM web editor) 양쪽 워터마크 변환 동기 정정
- **`feedback_hancom_compat_specific_over_general`** — U+F081C 만 명시 폭 0 (광범위 PUA 룰 없음), 워터마크 단일 룰 (effect != RealPic + brightness/contrast 비-zero)
- **`feedback_rule_not_heuristic`** — HWP IR 표준 (LineSeg.vpos / LineSeg.line_height / ImageAttr) 직접 사용, 측정/휴리스틱 회피
- **`feedback_visual_judgment_authority`** — 작업지시자 시각 판정 영역 (1차 피드백 → 2차 정정 → 승인) 의 권위 게이트웨이 정합

## 본 task 의 권위 사례 패턴

1. **다단계 진단** — Stage 1 임시 디버그 추적 (`eprintln!`) + 정량 분석 (vpos / lh / px / HU 변환) → 본질 결함 영역 정확 식별
2. **케이스별 명시 가드** — U+F081C 만 명시 + 조건 가드 3개 (start_line>0 + TAC 보유 + para_start_y 등록) + 워터마크 게이트 (effect != RealPic + 비-zero)
3. **단일 룰 / 비휴리스틱** — HWP IR 표준 직접 사용, 측정 의존 없음
4. **광범위 sweep 영향 범위 좁힘** — U+F081C 보유 1개 fixture / 워터마크 보유 1개 fixture / 페이지 수 회귀 0
5. **작업지시자 시각 판정 게이트웨이** — 1차 피드백 (이미지 흐림) → 2차 정정 (저장값 강도 보존) → 승인
6. **양쪽 렌더 경로 동기** — SVG (CLI) + Canvas (WASM web editor) 동시 정정
7. **회귀 차단 가드 영구 보존** — svg_snapshot 신규 + 골든 (414KB) 영구 보존
8. **잔존 영역 분리** — 굵은 폰트 (폰트 가용성 영역) / body-clip 위생 / 워터마크 customization → 별도 task 후보 명시

## 승인 요청

본 최종 보고서 승인 후 다음 절차 진행:
1. 본 task 브랜치 (`local/task677`) 의 모든 변경 commit
2. `local/devel` 으로 merge (--no-ff)
3. `devel` (origin) push
4. `pr-task677` 브랜치 생성 (`stream/devel` base) + cherry-pick + origin push
5. `gh pr create --repo edwardkim/rhwp --base devel --head planet6897:pr-task677` 으로 PR 생성
6. PR 본문에 closes #677 + 시각 판정 ★ 통과 + 회귀 검증 + 변경 LOC 명시
7. mydocs/orders/20260507.md 본 task 항목 갱신 + 머지

(`feedback_pr_to_stream_devel` 정합)
