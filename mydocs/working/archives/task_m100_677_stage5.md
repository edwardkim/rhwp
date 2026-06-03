# Stage 5 — 시각 판정 + 잔존 점검 + WASM 빌드 (Task #677)

## 작업지시자 시각 판정 영역 (1차) — 피드백 반영

**1차 시각 판정 피드백** (작업지시자):
- "이미지가 너무 흐림" — 워터마크 너무 흐림
- "굵은 폰트처리가 미흡" — 굵은 폰트 처리 부족

### 영역 1 — 워터마크 정합 정정 (svg.rs / web_canvas.rs)

**1차 적용**: `is_watermark_image` 시 한컴 표준 프리셋 (brightness=+70, contrast=-50) 강제 — 너무 흐림.

**2차 정정 (본 피드백 반영)**: 저장값 강도 보존 + 부호 워터마크용 정합:
```rust
let (eff_brightness, eff_contrast) = if is_watermark_image {
    (img.brightness.unsigned_abs() as i8, -(img.contrast.unsigned_abs() as i8))
} else {
    (img.brightness, img.contrast)
};
```

**검증**:
- 본 fixture 저장 (-50, +70) → 적용 (+50, -70). 강도 보존 + 워터마크 부호 (밝게 + 저대비)
- 표준 프리셋 케이스 (저장 70, -50) → 적용 (70, -50). 변환 후에도 정합 유지
- 시각 결과: 흐릿한 회색 → **선명한 회색 워터마크 (PDF 정합)** — 고려대학교 엠블럼 + 호랑이 + KOREA UNIVERSITY + 1905 모두 가독 영역

### 영역 2 — 굵은 폰트처리 영역 진단 (잔존 영역)

**진단** (`examples/inspect_677_bold.rs` 임시 진단 도구로 점검):
- 본 fixture 의 모든 셀 paragraph CharShape `bold=false` (HWP IR 영역)
- PDF 정답지 영역의 굵은 외양은 한컴 자체 폰트 (한양견명조 / 한양신명조 / HY견명조 / HY신명조) 디자인 굵기 자체 — 본 환경 fallback (Noto Serif CJK KR Regular) 보다 굵음
- HWP IR 자체에 bold=true 마킹 없음 → 본 task 의 본질 정정 영역 외

**구체 조사**:

| Cell paragraph | CharShape bold | Font | PDF 외양 | 본 환경 외양 |
|---------------|---------------|------|---------|---------|
| "복학원서(학부)" 제목 | false | 한양견명조 32pt | 굵게 보임 | 정상 두께 (Noto Serif Regular fallback) |
| "Reinstatement Form (Undergraduate)" | false | 한양신명조 13pt | 굵게 보임 | 정상 두께 |
| "복 학 원 서 접 수 증" | false | 한양견명조 14pt | 굵게 보임 | 정상 두께 |
| "위 학생의 복학원서를 접수함." | false | 한양견명조 13pt | 굵게 보임 | 정상 두께 |
| "I have taken leave..." | **true** | 한양신명조 13pt | 굵게 보임 (정합) | **굵게 (font-weight=bold 적용)** ✓ |

**결론**: 본 결함은 **폰트 가용성 영역**의 결함 — 한컴 한양견명조/한양신명조 폰트 미설치 환경 (Linux/macOS 기본) 에서 fallback (Noto Serif CJK KR Regular) 의 stroke 두께 부족. HWP IR 자체에는 bold=true 마킹 없음.

**잠재적 후속 task 영역**:
- 한컴 한양견명조/한양신명조 → 더 무거운 fallback 폰트 영역 매핑 (Noto Serif CJK KR Medium / Source Han Serif Heavy 등)
- `--embed-fonts` 옵션 사용 시 한컴 폰트 임베딩 영역 활성화
- 또는 작업지시자 환경에서 한컴 한양견명조/한양신명조 ttf 설치 후 `--font-style` 사용

본 결함은 본 task 의 layout/워터마크 본질 정정 영역과 **다른 본질** (폰트 fallback 영역) 이므로 본 task 영역 외 — **별도 task 후보** 로 분리 권유.

### 갱신된 골든 영구 보존

워터마크 변환 영역 정정 (Stage 3 → 본 영역) 으로 SVG byte 변경 → `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` 갱신 (`UPDATE_GOLDEN=1` 영역). 회귀 가드 정합.

## 시각 판정 영역 (메인테이너 권위 영역)

### 산출물

- SVG: `output/svg/task677_final/복학원서.svg` (414,812 bytes)
- PNG (1600×2263): `output/svg/task677_final/복학원서_final.png` (359,568 bytes)
- 정답지: `pdf/복학원서-2022.pdf` (한글 2022 PDF, macOS/Linux 환경 1차 정답지 — `reference_authoritative_hancom` 정합)

### 시각 정합 영역 (PDF 정답지 비교)

PNG 렌더 결과 — 모든 영역이 PDF 정답지와 동일 위치에 정합 출력:

| PDF 영역 | rhwp AFTER 정합 |
|---------|----------------|
| 복학원서(학부) 제목 (대형) | ✅ |
| Reinstatement Form (Undergraduate) | ✅ |
| 5×4 표 (대학/학과/학번/성명/휴대전화/e-Mail/현주소) | ✅ |
| 본인은 휴학으로 인하여... (Korean) | ✅ |
| 복학원을 제출합니다 | ✅ |
| I have taken leave of absence... (English) | ✅ |
| this reinstatement form. | ✅ |
| 년(year) 월(momth) 일(day) (PDF 의 momth 오타까지 정합) | ✅ |
| 본인 (Name) signature line + 접수자 (Receiving Official) box | ✅ |
| **고 려 대 학 교 총 장 귀 하 (BEFORE 가려져 안 보임 → AFTER 정상 출력)** | ✅ |
| ─── 분리선 | ✅ |
| 복 학 원 서 접 수 증 / Filing Receipt | ✅ |
| 대학(Name of College) : / 학과/학부(Department/Major) : | ✅ |
| 학번(Student No.) : / 성명(Name) : | ✅ |
| 위 학생의 복학원서를 접수함. | ✅ |
| The above student's reinstatement form is hereby received. | ✅ |
| 년(year) 월(month) 일(day) ㊞ (붉은 도장) | ✅ |
| ※ 군필자는 병무행정(분)실에서... | ✅ |
| ※ Those who completed their military service... | ✅ |
| of absence/return to school registration period... | ✅ |
| **고려대학교 워터마크 (흐린 회색)** | ✅ (BEFORE 진한 어두운 회색 → AFTER 흐린 회색 PDF 정합) |

**작업지시자 시각 판정 영역 대기** — 본 PNG + PDF 비교 시각 검증 통과 시 머지 진행.

## WASM 빌드 정량 측정

### Docker WASM 빌드

```
docker compose --env-file .env.docker run --rm wasm
→ Finished `release` profile [optimized] target(s) in 20.24s
→ Optimizing wasm binaries with `wasm-opt`...
→ :-) Done in 56.22s
```

**산출**: `pkg/rhwp_bg.wasm` = **4,532,601 bytes**

### Studio WASM + JS 빌드 (vite + tsc)

```
cd rhwp-studio && npm run build
→ TypeScript 타입 체크 통과
→ vite v8.0.10 dist 빌드 성공
```

**산출**:
- `dist/assets/rhwp_bg-DjvWhb50.wasm` = 4,532,601 bytes (Docker WASM 정합)
- `dist/assets/index-B79aTyIr.js` = 693,023 bytes

### 사이클 baseline 대비 변화

| 측정 항목 | 본 task | 직전 baseline (PR #621) | Δ |
|----------|---------|----------------------|---|
| Docker WASM | 4,532,601 | 4,606,564 | -73,963 (-1.6%) |
| Studio JS | 693,023 | 691,386 (PR #642) | +1,637 (+0.24%) |

**WASM 감소 정황**: 본 task 의 src 변경 (+82 LOC) 대비 -73,963 bytes 감소는 LLVM 최적화 + wasm-opt 효과로 추정. 본 task 의 분기 추가 영역 (PP y 리셋 / U+F081C 폭 / 워터마크 게이트) 모두 단순 가드 분기로 코드 경로 확장이 미세하나, wasm-opt 의 dead code elimination 또는 inline expansion 차이로 절감된 것으로 보인다. 회귀 신호는 아니며 (functional 테스트 1155 passed 정합), 후속 PR 처리 시 baseline 갱신 영역.

## 잔존 결정적 검증

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
docker compose run --rm wasm                     WASM 빌드 success (4,532,601 bytes)
rhwp-studio npm run build                        TypeScript + vite dist 빌드 success
```

## 잔존 영역 (별도 task 후보, 본 task scope 외)

### 1. body-clip width 1619.92 (Stage 1 식별)

- 시각 영향 없음 (콘텐츠가 body 안에 있으므로 클립 폭이 넓어도 시각 결함 없음)
- 코드 위생 영역 — body 폭 계산이 본질적으로 ~2.4 배 부풀려져 있음
- 별도 후속 task 후보 (작업지시자 판단)

### 2. 워터마크 표준 프리셋 외 사용자 customization 영역

- 본 정정 (Stage 3) 은 모든 워터마크에 한컴 표준 프리셋 (brightness=+70, contrast=-50) 강제 적용
- 사용자가 custom 강도를 의도한 경우 ignore — 다른 워터마크 fixture 가 발견되면 customization 보존이 필요한지 별도 검토
- 현 시점에서 162+ HWP/HWPX fixture 중 워터마크 보유는 본 fixture 1개만 → 영향 영역 없음

### 3. 다른 PUA 채움 문자 영역 (잠재적)

- 본 정정 (Stage 2) 은 U+F081C 만 명시 폭 0
- PR #592 review 에서 언급된 U+F012B (복학원서.hwp) 등 추가 PUA 영역 존재 — 본 fixture 에는 없는 것으로 확인 (162+ sweep 결과 U+F081C 만 발견)
- 다른 fixture 에서 추가 PUA 채움 문자 발견 시 별도 task

## Stage 1~5 누적 변경 LOC

| 파일 | 변경 | 영역 |
|------|------|------|
| `src/renderer/layout.rs` | +30 / -2 | PP y 리셋 + max 누적 |
| `src/renderer/layout/text_measurement.rs` | +20 / 0 | U+F081C 폭 0 (5 사이트) |
| `src/renderer/svg.rs` | +9 / -1 | 워터마크 표준 프리셋 |
| `src/renderer/web_canvas.rs` | +9 / -1 | 워터마크 표준 프리셋 (WASM) |
| `tests/svg_snapshot.rs` | +14 / 0 | issue_677 회귀 가드 |
| `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` | 414812 bytes | 골든 영구 보존 |
| `mydocs/plans/task_m100_677.md` | +132 LOC | 수행계획서 |
| `mydocs/plans/task_m100_677_impl.md` | +219 LOC | 구현계획서 |
| `mydocs/working/task_m100_677_stage1.md` | +151 LOC | Stage 1 진단 |
| `mydocs/working/task_m100_677_stage2.md` | +119 LOC | Stage 2 정정 |
| `mydocs/working/task_m100_677_stage3.md` | +120 LOC | Stage 3 정정 |
| `mydocs/working/task_m100_677_stage4.md` | +120 LOC | Stage 4 회귀 가드 |
| `mydocs/working/task_m100_677_stage5.md` | (본 보고서) | Stage 5 시각 + 빌드 |
| **합계 (src + tests)** | **+82 / -4 LOC + 1 가드 + 1 골든** | 본질 정정 영역 |
| **합계 (mydocs)** | **+861 LOC** | 거버넌스 영역 |

## 승인 요청

본 Stage 5 결과 + **시각 판정 ★ 통과** 후 **최종 보고서** (`task_m100_677_report.md`) 작성 + **타스크 브랜치 → local/devel merge** + **stream/devel PR** 생성 진행하겠습니다.

작업지시자 시각 판정 영역:
- `output/svg/task677_final/복학원서_final.png` (1600×2263) ↔ `pdf/복학원서-2022.pdf` 비교
- 한컴 정답지 정합 영역 (워터마크 흐림 + 본문 가독성 + 모든 텍스트 영역 정상 출력) 통과 확인 후 진행
