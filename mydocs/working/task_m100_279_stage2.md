# Task #279 Stage 2 — 빌드/테스트/clippy/wasm32 검증

## 결과 요약

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo build --release` | ✅ 26.04s | |
| `cargo test --lib` | ✅ 992 passed / 0 failed / 1 ignored | |
| `cargo test --test svg_snapshot` (1차) | ⚠️ 4 passed / **2 failed** | issue_267_ktx_toc_page (의도) + issue_147_aift_page3 (의도) |
| `cargo test --test svg_snapshot` (UPDATE_GOLDEN 후) | ✅ **6 passed / 0 failed** | |
| `cargo test --test issue_301` | ✅ 1 passed | z-table 회귀 가드 |
| `cargo clippy --lib -- -D warnings` | ✅ clean | |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean | |

## svg_snapshot 골든 영향 분석

본 task 가 의도한 두 변경 (leader dasharray + right tab 클램핑 제외) 이 다음 두 골든에 영향을 미쳤다:

### issue_267_ktx_toc_page

KTX.hwp 목차 페이지 — **본 task 의 정확한 목표 영역**.

**의도된 변경 1 (leader 표현)**:
```diff
- <line ... stroke-width="0.5" stroke-dasharray="1 2"/>
+ <line ... stroke-width="1.0" stroke-dasharray="0.1 3" stroke-linecap="round"/>
```

**의도된 변경 2 (right tab 정렬 — 핵심)**:
```diff
- <line ... x2="694.76" .../>           # leader 끝
- <text x="709.76" ...>3</text>          # 페이지번호 (소제목)

+ <line ... x2="712.47" .../>           # leader 끝 +17.7px
+ <text x="727.47" ...>3</text>          # 페이지번호 +17.7px (장제목과 동일 우측 edge)
```

소제목 페이지번호가 **이전 ~700px → 727.47px** 로 이동하여 장제목 (727.47±0.5) 과 동일 우측 edge 에 정렬. **#279 의 핵심 목표 달성**.

### issue_147_aift_page3

aift.hwp 3페이지 — 본 페이지는 표 안에 fill_type=3 leader 가 다수 있어 **dasharray 변경의 부수 영향**:

```diff
- <line ... stroke-width="0.5" stroke-dasharray="1 2"/>
+ <line ... stroke-width="1.0" stroke-dasharray="0.1 3" stroke-linecap="round"/>
```

**좌표는 모두 동일** (x1/x2/y 동일). dasharray/width/linecap 만 변경. 즉 **right tab 영향 없음**, leader 표현 통일 효과만 발생. 시각적으로는 leader 가 사각 대시 → 원형 점으로 일관 변경되는 의도된 개선.

### 처리

`UPDATE_GOLDEN=1 cargo test --test svg_snapshot` 으로 두 골든 갱신. 재실행 결과 6/6 통과.

갱신 파일:
- `tests/golden_svg/issue-267/ktx-toc-page.svg`
- `tests/golden_svg/issue-147/aift-page3.svg`

## 회귀 가드 통과 항목

- z-table (#301): `tests/issue_301.rs` 1 passed
- 다른 svg_snapshot 4건: table_text_page_0, issue_157_page_1, form_002_page_0, render_is_deterministic_within_process — 모두 무영향 / 통과
- lib unit tests 992 — 무영향

## Stage 2 완료 조건 점검

- [x] `cargo build --release` 통과
- [x] `cargo test --lib` 992/0/1ignored
- [x] `cargo test --test svg_snapshot` 6/0 (UPDATE_GOLDEN 후)
- [x] `cargo test --test issue_301` 통과
- [x] `cargo clippy --lib -D warnings` clean
- [x] `cargo check --target wasm32` clean
- [x] 골든 영향 의도성 검증 (KTX 목차 + aift 표 안 leader)

## 다음 단계

Stage 3 — KTX.hwp 좌표 측정 + 6 핵심 샘플 회귀 + WASM Docker 빌드 후 시각 검증.
