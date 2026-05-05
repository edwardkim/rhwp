# PR #592 검토 보고서

**PR**: [#592 [m100] exam_eng.hwp p7 #40 글상자 사이 화살표 누락 — PUA U+F003B → ↓ 매핑 추가 (closes #588)](https://github.com/edwardkim/rhwp/pull/592)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**상태**: OPEN, **mergeable=MERGEABLE** (본 사이클 첫 케이스 — CONFLICTING 아님)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-05

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — `samples/exam_eng.hwp` 7페이지 #40 요약형 문항 글상자 사이 ↓ 화살표가 SPUA-A 저영역 (U+F003B) 미매핑으로 두부 표시되는 결함이 본 환경에서도 재현되는가?
2. **글리프 매핑 근거 정합** — PR 본문이 명시한 한컴 PDF 임베디드 폰트 (`AAAAAL+HCRBatang`) 의 `uF003B` 글리프 외곽 분석 (1 contour 7 pts → ↓ 형태) 의 정합성?
3. **디스조인트 설계** — 신설 분기 (`0xF0000..=0xF00CF`) 가 기존 분기 (Task #528 / #509 / Wingdings) 와 무중첩한가?
4. **MERGEABLE 표시** — 본 사이클 처음으로 CONFLICTING 아님 — 본 환경 cherry-pick 시 충돌 영역 무인가?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | Task #588 PUA U+F003B → ↓ 매핑 추가 | 정합 |
| author (PR 등록) | @planet6897 | — |
| commit author | Jaeook Ryu (= @jangster77) | 컨트리뷰터간 협업 흐름 (PR #561~#584 동일 패턴) |
| changedFiles | 6 / +665 / -0 | 본질 코드 +32 + 보고서 다수 |
| 본질 변경 | `src/renderer/layout/paragraph_layout.rs` +32 | 단일 파일 |
| **mergeable** | **MERGEABLE** | 본 사이클 첫 케이스 (이전 PR 들 모두 CONFLICTING) |
| Issue | closes #588 | ✅ |

## 3. PR 의 4 commits 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `1baa36c5` Stage 1 — PDF 글리프 분석 + 광범위 통계 | 컨트리뷰터 fork plans/working | 무관 |
| **`2788fea8` Stage 2 — 본질 정정** | `paragraph_layout.rs` +32 + working stage2 | ⭐ **cherry-pick 대상** |
| `c1e27ec6` Stage 3 — 광범위 회귀 점검 | 컨트리뷰터 fork working | 무관 |
| `9b78108a` Stage 4 — 최종 보고서 | 컨트리뷰터 fork report | 무관 |

→ **본질 cherry-pick 대상 = `2788fea8` 단독**. PR #561~#580 의 단일 본질 commit 패턴.

## 4. 본질 변경 영역

### 4.1 결함 가설

`samples/exam_eng.hwp` p7 pi=278 의 단일 문자 **U+F003B** (UTF-8 `f3 b0 80 bb`) 가 SPUA-A 저영역 (`0xF0000~0xF00CF`) 으로 `map_pua_bullet_char` 매핑 표 밖. 글리프 미보유 폰트에서 두부(□) 표시.

### 4.2 정정 (단일 분기 신설)

```rust
if (0xF0000..=0xF00CF).contains(&code) {
    return match code {
        0xF003B => '\u{2193}', // ↓ DOWNWARDS ARROW
        _ => ch,
    };
}
```

### 4.3 매핑 글리프 확정 근거 (Stage 1 PDF 분석)

PR 본문:
> `samples/exam_eng.pdf` 7쪽의 임베디드 폰트 `AAAAAL+HCRBatang` 의 `uF003B` 글리프 외곽 분석 — 1 contour 7 pts (stem 35% × arrowhead 100% × solid filled) → ↓ 형태 확정.

→ 한컴 PDF 권위 자료의 폰트 임베드 글리프 직접 분석으로 매핑 결정. 메모리 룰 `feedback_pdf_not_authoritative` (PDF 정답지 아님) 의 잠재 우려는 있지만, **본 케이스는 PDF 임베드 폰트의 글리프 외곽 자체** 가 권위 (한컴이 PDF 에 임베드한 글리프 = 한컴이 의도한 시각). 시각 판정 게이트로 보정 필요.

### 4.4 디스조인트 설계 (회귀 차단)

| 영역 | 범위 | 출처 |
|------|------|------|
| **본 PR (신규)** | `0xF0000..=0xF00CF` | Task #588 |
| Task #528 | `0xF00D0..=0xF09FF` | 책괄호/예시 |
| Task #509 | `0xF02B0..=0xF02FF` | 원문자 |
| Wingdings | `0xF020..=0xF0FF` | BMP PUA |

→ **신설 분기는 기존 영역과 완전 디스조인트** (`0xF0000..=0xF00CF` 와 `0xF02B0..=0xF02FF` 는 SPUA-A vs BMP PUA 다른 plane). 회귀 위험 영역 0.

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr592-cherry-test` 임시 브랜치에서 `2788fea8` 단독 cherry-pick:

| 단계 | 결과 |
|------|------|
| `2788fea8` cherry-pick (no-commit) | ✅ Auto-merging src/renderer/layout/paragraph_layout.rs (충돌 0) |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (이전 baseline 1132 + 신규 단위 테스트 +2) |
| `cargo clippy --release --lib` | ✅ 0건 |

→ **MERGEABLE 표시 정합** — 본질 commit (`2788fea8`) 단독 cherry-pick 시 본 환경 devel 에 깨끗하게 적용 + 충돌 0.

## 6. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test --lib` | 1126 passed (3 ignored, 0 failed) | ✅ 1134 passed (본 환경 baseline 정합) |
| `cargo test --test svg_snapshot` | 6/6 GREEN | ⏳ 본격 검증 |
| 통합 테스트 (issue_418/501/505/514/516/530/546/exam_eng_multicolumn) | 전부 통과 | ⏳ 본격 검증 |
| `cargo clippy --lib -- -D warnings` | 0 신규 | ✅ 0건 |
| WASM 빌드 | 4,529,640 bytes | ⏳ 본격 검증 (본 환경 4,581,498 baseline 대비 -51,858 차이는 PR fork 시점 차이) |
| 13 fixture 광범위 byte sweep | 582/583 byte-identical (1건 = exam_eng/p7 의도된 변경) | ⏳ 본 환경 sweep 권장 |
| 작업지시자 시각 판정 | (Stage 4 시각 검증 후 close 예정) | ⏳ 본 환경 시각 판정 게이트 |

## 7. 메인테이너 정합성 평가

### 정합 영역
- ✅ **MERGEABLE 표시** — 본 사이클 첫 케이스 (PR #561~#584 모두 CONFLICTING). PR base 가 본 devel 과 정합한 시점인 것으로 추정
- ✅ **본질 cherry-pick 깨끗** — 충돌 0 (auto-merge)
- ✅ **결정적 검증 정합** — cargo test --lib 1134 passed (본 환경 baseline +2 단위 테스트) / clippy 0
- ✅ **단일 분기 + 디스조인트 설계** — 기존 영역과 완전 무중첩 (Task #528/#509/Wingdings) 으로 회귀 위험 0
- ✅ **PDF 글리프 외곽 직접 분석** — 한컴 PDF 임베드 폰트 (`AAAAAL+HCRBatang`) 의 `uF003B` 1 contour 7 pts 분석으로 매핑 결정 (`feedback_v076_regression_origin` 정밀 진단 정합)
- ✅ **단위 테스트 +2** — `supplementary_pua_a_low_range_maps_down_arrow` + `supplementary_pua_a_low_range_unmapped_returns_original`. RED → GREEN 전환은 아니지만 신규 분기의 명시 검증
- ✅ **시각 변경 정량 측정** — `exam_eng_007.svg:4162` 정확한 1행 변경 (`󰀻` → `↓`, x/y/font/size 동일)
- ✅ **하이퍼-워터폴 흐름** — Stage 1 분석/통계 → Stage 2 본질 → Stage 3 sweep → Stage 4 보고. 본 환경 워크플로우 정합
- ✅ **잔존 영역 명시** — Stage 1 광범위 통계로 동일 영역 미매핑 코드포인트 식별 (U+F0090, U+F012B/F081C, U+F02BA 등) — 별도 이슈 후보

### 우려 영역
- ⚠️ **PDF 권위 자료 사용** — `feedback_pdf_not_authoritative` 메모리 룰 (한컴 PDF 출력은 환경별로 다름) 잠재 우려. 그러나 본 케이스는 **임베드 폰트 글리프 외곽** (한컴이 PDF 에 임베드한 데이터 자체) 으로 환경 의존도 낮음. 작업지시자 시각 판정으로 보정 필요
- ⚠️ **작업지시자 시각 판정 게이트** — PR 본문 "Stage 4 시각 검증 후 close 예정" 명시. 본 환경 cherry-pick 후 직접 시각 판정 필수
- ⚠️ **광범위 sweep "582/583 byte-identical"** — PR 본문 측정값이 본 환경에서도 재현되는지 본격 검증 권장

## 8. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능 + 충돌 0** — `2788fea8` 단독 + MERGEABLE 표시 정합
- ✅ **결정적 검증** — 1134 passed / clippy 0
- ✅ **디스조인트 설계** — 기존 영역과 완전 무중첩 (회귀 위험 0)
- ✅ **PDF 글리프 외곽 직접 분석** — 정밀 진단의 우수 사례
- ✅ **단위 테스트 +2** — 신규 분기 명시 검증
- ⏳ **시각 판정 별도 진행 필요** — 본 환경 cherry-pick 후 작업지시자 직접 시각 판정 필수
- ⏳ **광범위 sweep 본격 검증 필요** — 본 환경 자동 sweep (PR #564/#570/#575/#580/#584 패턴)

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- `2788fea8` 단독 cherry-pick (Stage 1/3/4 의 plans/working/report 는 컨트리뷰터 fork 정합 — 본 환경 자체 처리 보고서)
- 본 환경 결정적 재검증 + 광범위 페이지네이션 sweep + WASM
- 작업지시자 시각 판정 (★ 게이트) — exam_eng p7 #40 화살표 정합
- 통과 시 devel merge + push + PR close 처리

#### 옵션 B — 추가 정정 요청
- 시각 판정에서 잔존 결함 발견 시 컨트리뷰터에게 추가 정정 요청 (잔존 영역 미매핑 코드포인트 추가 매핑 등)

#### 옵션 C — close + 본 환경 직접 처리
- 시각 판정 다수 결함 발견 시 본 환경에서 직접 처리

→ **작업지시자 결정 대기**. 옵션 A 권장 — 본질 cherry-pick 깨끗 + MERGEABLE + 결정적 검증 통과 + 디스조인트 설계 + PDF 글리프 직접 분석 + 단위 테스트 +2.

## 9. 옵션 A 진행 결과 (작업지시자 승인 후)

### 9.1 핀셋 cherry-pick

| 단계 | 결과 |
|------|------|
| 본질 commit cherry-pick (`2788fea8`) | ✅ 충돌 0, author Jaeook Ryu 보존 |
| local/devel cherry-pick commit | `dd568ed` |

### 9.2 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (단위 테스트 +2) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |
| Docker WASM 빌드 | ✅ **4,581,527 bytes** (1m 34s, PR #584 baseline +29 bytes — paragraph_layout.rs +32 LOC 정합) |

### 9.3 광범위 페이지네이션 sweep (페이지 수 회귀 자동 검출)

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ U+F003B → ↓ 매핑이 페이지네이션에 영향 없음.

### 9.4 SVG 정량 측정 — U+F003B 두부 → ↓ 화살표 변경

| Fixture | U+F003B before | U+F003B after | ↓ after | 평가 |
|---|---|---|---|---|
| **exam_eng** | **1** | **0** | **1** | ✅ PR 본문 권위 영역 (page 7 #40 화살표 정정) |
| exam_kor | 0 | 0 | 0 | 회귀 영역 무 |
| exam_math | 0 | 0 | 0 | 회귀 영역 무 |
| exam_science | 0 | 0 | 0 | 회귀 영역 무 |

→ exam_eng 단독 영향 (page 7 의 1건 두부 → ↓ 정확 변환). 다른 fixture 무영향.

### 9.5 페이지 7 권위 케이스 검증 (PR 본문 명시 영역)

`exam_eng_007.svg:4162` 정확한 1행 변경:

```diff
- <text transform="translate(796.7266666666668,1218.2666666666667) scale(1.3000,1)" font-family="HY신명조,..." font-size="15.333333333333334" fill="#000000">󰀻</text>
+ <text transform="translate(796.7266666666668,1218.2666666666667) scale(1.3000,1)" font-family="HY신명조,..." font-size="15.333333333333334" fill="#000000">↓</text>
```

→ x/y/font/size/scale/fill 모두 동일, U+F003B (`󰀻`) → U+2193 (`↓`) 만 변경. PR 본문 100% 재현.

### 9.6 다음 단계

1. ✅ 본 1차 검토 보고서 작성 (현재 문서)
2. ✅ 본 환경 결정적 재검증
3. ✅ SVG 생성 — `output/svg/pr592_before/exam_eng/` + `output/svg/pr592_after/exam_eng/` (page 7 만 의도된 차이)
4. ✅ Docker WASM 빌드 완료 (4,581,527 bytes)
5. ✅ 광범위 페이지네이션 sweep — 164 fixture 1,614 페이지 / 페이지 수 회귀 0
6. ⏳ **작업지시자 시각 판정** (★ 게이트, exam_eng page 7 #40 화살표) — 본 단계 대기 중
7. ⏳ 통과 시 devel merge + push + PR close
8. ⏳ 처리 보고서 (`pr_592_report.md`) 작성 + archives 이동

## 10. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영
- ✅ `feedback_v076_regression_origin` — Stage 1 PDF 임베드 폰트 글리프 외곽 분석 (1 contour 7 pts → ↓ 형태) 정밀 진단 정합
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (SPUA-A 저영역 단독 분기, 디스조인트 설계)
- ✅ `feedback_rule_not_heuristic` — 코드포인트 범위 + 명시 매핑 (휴리스틱 아닌 규칙)
- ⚠️ `feedback_pdf_not_authoritative` — PDF 임베드 글리프 외곽은 환경 의존도 낮지만 작업지시자 시각 판정으로 보정 필요
- ✅ `feedback_per_task_pr_branch` — Task #588 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — 본 PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 누적 19번째 PR

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
