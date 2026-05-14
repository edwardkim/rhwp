# Task #894 구현 계획서 — 잔존 통합 (Stage 1~3)

**이슈**: [edwardkim/rhwp#894](https://github.com/edwardkim/rhwp/issues/894)
**수행 계획서**: [task_m100_894.md](task_m100_894.md)
**진행 순서**: Stage 1 (C') → Stage 2 (B) → Stage 3 (A)
**Scope 변경**: 항목 C (HWP5 변환본 inflate) 는 #877 진행 중 이미 해결 → **항목 C' 로 대체**: HWPX 변환본 페이지 수 정합 (72 → 62)
**Scope 추가**: 항목 D (CLAUDE.md c2955b5 컨트리뷰터 워크플로우 보강) — PR #890 미포함, task894 PR 로 메인테이너 전달. base 자동 포함, 별도 stage 없음

## Stage 1 — 항목 C' : HWPX 변환본 페이지 수 inflate (72 → 62)

### 1.0 사전 진단 결과 (이미 수행)

```
샘플                          rhwp 페이지  한컴 viewer  차이
hwp3-sample16-hwp5.hwp        62           62          0 ✅
hwp3-sample16-hwp5.hwpx       72           62          +10 ❌
hwp3-sample16.hwp             64           64          0 ✅
```

`ir-diff` 카테고리 요약 (HWPX vs HWP5):

| 항목 | 건수 | 영향 |
|------|------|------|
| char_shapes count | 604 | 라인 구성 영향 가능 |
| line_segs count | 59 | **직접적 pagination 영향** |
| cc (char count) | 26 | paragraph 길이 |
| text | 13 | paragraph 텍스트 |
| controls count | 1 | |

### 1.1 정밀 진단 (Step 1)

- [ ] `ir-diff` 의 line_segs count 차이가 발생하는 paragraph 위치 파악 (`--max-lines` 또는 grep 으로 lines 차이만 추출)
- [ ] 차이 paragraph 의 char_shape segmentation 차이 분석 (rhwp 가 HWPX char_shape 를 과다 분할 가정)
- [ ] HWPX `section0.xml` 원본 char shape 정보 직접 확인 (`unzip -p hwp3-sample16-hwp5.hwpx Contents/section0.xml | head`)
- [ ] HWPX 파서의 char_shape parsing 코드 위치 식별 (`src/parser/hwpx/section.rs` 등)

### 1.2 가설 후보

| 가설 | 검증 방법 |
|------|----------|
| H1: HWPX char_shape 가 line_seg 마다 fragmenting 됨 (HWP5 는 paragraph 전체 1개) | char_shape count 604 / line_segs 59 의 비율 분석 |
| H2: HWPX paragraph 의 line wrap 계산이 다른 폰트 metric 사용 → 페이지 라인 부족 | 동일 paragraph 의 line_seg 폭 / height 비교 |
| H3: HWPX charPr 의 size unit 단위 변환 오류 | char_shape size 직접 비교 |

### 1.3 수정 (Step 2)

가설 검증 후 root cause 에 따라 분기. 잠정 작업:
- [ ] HWPX 파서 (`src/parser/hwpx/`) 의 char_shape / line_seg 변환 로직 수정
- [ ] 변환 결과 ir-diff 재실행 → line_segs count 차이 0 또는 최소화

### 1.4 검증 (Step 3)

- [ ] `cargo run --release -- dump-pages samples/hwp3-sample16-hwp5.hwpx` → 62 페이지 정합
- [ ] `cargo test --release` 전체 통과
- [ ] **HWPX 회귀 점검**: 모든 HWPX 샘플 페이지 수 회귀 없음
  - samples 디렉토리 내 `.hwpx` 파일 전체 `dump-pages` 페이지 수 before/after 비교 스크립트 실행
- [ ] golden SVG 회귀 (`tests/golden_svg/`)

### 1.5 위험

- HWPX 파서 변경은 **모든 HWPX 샘플에 영향** — 회귀 위험 최고. 1.4 단계에서 회귀 발견 시 fix 범위를 sample16-hwp5.hwpx 의 특정 패턴으로 좁히는 방향 전환.

---

## Stage 2 — 항목 B : paragraph multi-line picture SVG 중복 emit

### 2.1 진단 (Step 1)

- [ ] `cargo run --release -- dump samples/hwp3-sample16.hwp -s 0 -p 394` → controls 정밀 구조
- [ ] `cargo run --release -- export-svg samples/hwp3-sample16.hwp -p 17` → SVG `<image>` 개수 / href 동일성
- [ ] 다른 샘플에서 inline picture (`￼`) multi-line 패턴 검색 (`rg -l "ls_count"` 등)
- [ ] picture emit 코드 위치: `src/renderer/typeset/` 또는 `src/renderer/svg/` (`grep -rn "image" src/renderer/svg/`)

### 2.2 가설 후보

| 가설 | 검증 방법 |
|------|----------|
| H1: picture control 이 line_seg 단위로 emit → ls_count=3 마다 3번 발생 | typeset.rs 의 picture placement 코드 inspection |
| H2: text 의 `￼` marker 가 3개 있어 marker 마다 image 1개 emit | text 분석 + control index ↔ marker index 매핑 검증 |
| H3: paragraph emit 시 controls iterate × line_seg iterate 이중 loop | 코드 trace |

### 2.3 수정 (Step 2)

- [ ] root cause 에 따라 picture emit 위치 1회 emission 으로 dedupe
- [ ] treat_as_char picture 와 그렇지 않은 picture 의 emit 분기 보존

### 2.4 검증 (Step 3)

- [ ] sample16 페이지 18 SVG → `<image>` 1개 (paragraph 394 [1])
- [ ] `cargo test --release` + golden SVG 회귀
- [ ] 다른 picture 샘플 회귀 점검 (treat_as_char picture 정상 emit)

### 2.5 위험

- 렌더러 typeset 변경은 광범위 영향. treat_as_char picture / float picture 모두 회귀 점검 필수.

---

## Stage 3 — 항목 A : HWP3 페이지 외곽선 좌표 기준 정합

### 3.1 진단 (Step 1)

- [ ] `cargo run --release -- dump samples/hwp3-sample16.hwp -s 0 -p 0` → page_border_fill attr / spacing 값 확인
- [ ] `cargo run --release -- export-svg samples/hwp3-sample16.hwp --debug-overlay -p 1` → 페이지 2 외곽선 + 텍스트 좌표
- [ ] `pdf/hwp3-sample16-hwp5-2022.pdf` 페이지 2 → 한컴 정답 좌표
- [ ] HWP5 변환본 IR (`rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0`) → page_border_fill attr 비교

### 3.2 가설 후보

| 가설 | 검증 방법 |
|------|----------|
| H1: HWP3 → IR 변환 시 `attr & 0x01` paper_based 가 잘못 설정 (false 가 정답인데 true 또는 반대) | HWP5 변환본 attr 와 대조 |
| H2: `border_margin*` → `spacing_*` 변환 값 부정확 (5mm 가 아닌 다른 단위) | 한컴 spec 참조 + HWP5 변환본 spacing 값 대조 |
| H3: 본문 paragraph 의 right margin 이 body_area 초과 — page border 가 아닌 paragraph 측 문제 | paragraph 의 right offset 측정 |

### 3.3 수정 (Step 2)

- [ ] HWP3 page_border_fill IR 변환 (`src/parser/hwp3/mod.rs`) 수정
- [ ] paper_based / spacing 정합

### 3.4 검증 (Step 3)

- [ ] sample16 페이지 2 SVG → 페이지 번호 외곽선 박스 안
- [ ] HWP3 sample 6종 (sample/4/5/10/13/14) 페이지 외곽선 회귀 점검
- [ ] `cargo test --release` + golden SVG 회귀

### 3.5 위험

- HWP3 다른 샘플의 page border 회귀. 회귀 발견 시 sample16 특정 조건 (attr / margin 값) 으로 분기 처리.

---

## 통합 검증 (모든 Stage 완료 후)

- [ ] `cargo test --release` 전체 통과 (현재 1381 passed 기준)
- [ ] HWP3 6종 + HWPX 주요 샘플 dump-pages 회귀 없음
- [ ] sample16 (HWP3 / HWP5 / HWPX) 페이지 수 정합 64 / 62 / 62
- [ ] golden SVG 회귀 없음

## 산출물

| 단계 | 파일 |
|------|------|
| Stage 1 보고서 | `mydocs/working/task_m100_894_stage1.md` |
| Stage 2 보고서 | `mydocs/working/task_m100_894_stage2.md` |
| Stage 3 보고서 | `mydocs/working/task_m100_894_stage3.md` |
| 최종 보고서 | `mydocs/report/task_m100_894_report.md` |

## 의사결정 요청

본 구현 계획서 자체 승인. 승인 시 Stage 1 (HWPX 변환본 페이지 수 정합) 진단부터 진행.
