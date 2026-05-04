# Task #588 Stage 3 — 광범위 회귀 점검

## 목표

Stage 2 의 매핑 구현이 다른 fixture 에 회귀를 일으키지 않음을 결정적으로 검증.

## 검증 결과

### 1. 단위 테스트

```
cargo test --lib
test result: ok. 1126 passed; 0 failed; 3 ignored; 0 measured
```

baseline (1126) 회귀 0 (Stage 2 이전 1124 + 신규 2 = 1126).

### 2. SVG snapshot 테스트

```
cargo test --test svg_snapshot
test result: ok. 6 passed; 0 failed
```

6/6 GREEN — table-text / issue-147 / issue-157 / issue-267 / form-002 / 결정성 테스트 모두 통과.

### 3. 통합 테스트 (issue_*)

| 테스트 | 결과 |
|--------|------|
| issue_418 (1) | ok |
| issue_501 (1) | ok |
| issue_505 (1) | ok |
| issue_514 (9) | ok |
| issue_516 (8) | ok |
| issue_530 (1) | ok |
| issue_546 (1) | ok |
| exam_eng_multicolumn (3) | ok |

### 4. Clippy

```
cargo clippy --lib -- -D warnings
Finished `dev` profile (warnings: 0 신규)
```

신규 warning 0건.

### 5. WASM 빌드

```
docker compose --env-file .env.docker run --rm wasm
Finished `release` profile [optimized]
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

WASM pkg: `pkg/rhwp_bg.wasm` 4,529,640 bytes — 정합 빌드.

### 6. 14 fixture 광범위 byte sweep

13 fixture (samples 14건 중 mathpresso/synap-001/kshs-001 미존재) 광범위 비교:

| fixture | SVG 수 | 변경 | 회귀 |
|---------|-------|------|------|
| exam_kor.hwp | 20 | 0 | 0 |
| exam_science.hwp | 4 | 0 | 0 |
| **exam_eng.hwp** | 8 | **1 (p7)** | **0 (의도된 변경)** |
| 21_언어_기출_편집가능본.hwp | 15 | 0 | 0 |
| synam-001.hwp | 35 | 0 | 0 |
| k-water-rfp.hwp | 27 | 0 | 0 |
| aift.hwp | 77 | 0 | 0 |
| mel-001.hwp | 21 | 0 | 0 |
| hwpspec.hwp | 177 | 0 | 0 |
| hwp3-sample.hwp | 16 | 0 | 0 |
| hwp3-sample4.hwp | 39 | 0 | 0 |
| hwp3-sample5.hwp | 64 | 0 | 0 |
| kps-ai.hwp | 80 | 0 | 0 |
| **합계** | **583** | **1** | **0** |

**582/583 byte-identical** — 회귀 0.

### 7. 변경 detail (exam_eng.hwp p7)

`exam_eng_007.svg:4162` 정확한 1행 변경:

```diff
< <text transform="translate(796.73,1218.27) scale(1.3,1)" font-family="HY신명조,..." 
-   font-size="15.33" fill="#000000">󰀻</text>
> <text transform="translate(796.73,1218.27) scale(1.3,1)" font-family="HY신명조,..." 
+   font-size="15.33" fill="#000000">↓</text>
```

x/y/font/size/scale 동일 — 글자 본체만 U+F003B → U+2193 (↓).

## 산출물

- `/tmp/svg_before/` (13 fixture, 583 SVG) — Stage 1 (pre-mapping) baseline
- `/tmp/svg_after/` (13 fixture, 583 SVG) — Stage 2 mapping 적용 후
- `pkg/rhwp_bg.wasm` (4,529,640 bytes) — WASM 빌드 정합 확인
- `mydocs/working/task_m100_588_stage3.md` — 본 보고서

## 다음 단계

Stage 4 (작업지시자 시각 검증 + 최종 보고서) 진행 승인 요청.

## 메모리 룰 정합

- `feedback_visual_regression_grows` — 광범위 byte sweep + 의도된 변경 외 회귀 0 결정적 검증
- `feedback_essential_fix_regression_risk` — 분기 디스조인트 설계 검증 (582/583 byte-identical)
