# PR #434 처리 보고서 — Task #430 그림 자동 크롭 (FitToSize+crop) 공식 교정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#434](https://github.com/edwardkim/rhwp/pull/434) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#430](https://github.com/edwardkim/rhwp/issues/430) (closes) |
| 처리 결정 | **cherry-pick 머지** (5 commits 모두 — 옵션 3) |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 + 작업지시자 옵션 3 결정

PR 의 5 commits 중 4 commits (Task #430) + 1 commit (테두리 inner padding 별도 fix) 모두 함께 흡수. 작업지시자 결정 정황:
- 5번째 commit 은 작성자가 별도 결함임을 명시했으나 본 브랜치에 함께 포함
- 코드 분리 영향 없음 (테두리 padding ≠ 이미지 crop)
- 함께 머지하여 작성자 의도 보존

### Stage 1: cherry-pick

`local/pr434` 브랜치 (`local/devel` 분기) 에서 5 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `060bd5f` (← `bdee30d`) | @planet6897 | Stage 1: 그림 crop 미적용 원인 정밀 조사 |
| `ead4a43` (← `a5541f9`) | @planet6897 | Stage 2: ImageNode.original_size_hu + svg.rs crop 공식 교정 |
| `09f5d0c` (← `401f3c5`) | @planet6897 | Stage 3: web_canvas.rs 동기화 + 회귀 검증 |
| `dfb685d` (← `97d18db`) | @planet6897 | Stage 4: 최종 보고서 + 오늘할일 갱신 |
| `add226a` (← `84c8a27`) | @planet6897 | fix: 테두리 문단 inner padding (별도 fix) |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (1062 → +4 신규 단위 테스트, 회귀 0건) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 44s, 4,182,395 bytes |

### Stage 3: 광범위 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교: **40 / 309 차이 발생 (의도된 정정 영향)**.

차이 분포: exam_kor 22, exam_eng 8, 2025년 기부 6, synam-001 3, k-water-rfp 1.

### Stage 4: 한컴 PDF 환경 의존성 II 발견 (작업지시자 추가 자료)

작업지시자가 검토 중 한컴 3종 PDF 자료 제공:
- `samples/2010-exam_kor.pdf` (한컴 2010, 4.57 MB)
- `samples/2020-exam_kor.pdf` (한컴 2020, 4.57 MB)
- `samples/hancomdocs-exam_kor.pdf` (한컴독스, 6.05 MB)

핵심 발견: 한컴 2010 ↔ 한컴 2020 ↔ 한컴독스 모두 다르게 조판 — 단일 한컴 정답지 가정의 한계 재확인. 발견 정황 I (PR #360) 의 "한컴 2010 + 2022 일관" 가정과 다른 정황.

작업지시자 코멘트:
> "이 시험지를 조판한 사람의 의도에 맞다는 생각이 들 정도입니다."

### Stage 5: 후속 조치 (본 검토 사이클 포함)

1. **3종 PDF samples/ commit** (devel `2714211`) — 모든 컨트리뷰터 공유
2. **위키 페이지 보강** — [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) 에 "발견 정황 II (PR #434 / 이슈 #430)" 섹션 추가
3. **README.md / README_EN.md 보강** (devel `a45c78b`) — Contributing 섹션 + 신규 "위키 자료 (Wiki Resources)" 서브섹션

### Stage 6: 시각 판정 (작업지시자 직접)

**비교 자료**: 한컴 3종 PDF (한컴 2010 / 2020 / 한컴독스) + 광범위 회귀 309 SVG vs PR #434 적용

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG + Canvas | ✅ 통과 (옵션 3 — 한컴 3종 PDF 비교 후 머지 결정) |

## 변경 요약

### Task #430 핵심 — 그림 자동 크롭 (FitToSize+crop) 공식 교정

| 파일 | 변경 |
|------|------|
| `src/renderer/render_tree.rs` | `ImageNode.original_size_hu: Option<(u32, u32)>` 필드 추가 |
| `src/renderer/layout/picture_footnote.rs` | `layout_picture` / `layout_body_picture` 의 `original_size_hu` 채움 |
| `src/renderer/layout.rs` | TAC + 텍스트 없는 문단 분기 동일 |
| `src/renderer/svg.rs` | `compute_image_crop_src` 헬퍼 추출 + 공식 교정 + 단위 테스트 4건 |
| `src/renderer/web_canvas.rs` | 시그니처 + 헬퍼 호출 동기화 (SVG/Canvas 단일 진실 원천 공유) |
| `src/main.rs` `dump` | 표 셀 그림 디버깅 출력 보강 |

### 별도 fix — 테두리 문단 inner padding (`84c8a27`)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | visible stroke 테두리 + `border_spacing` 좌/우 0 인 경우 paragraph margin 을 inner padding 으로 적용 |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 한컴 3종 PDF 직접 비교 후 통과 |
| `feedback_pdf_not_authoritative.md` | ✅ 본 PR 검토 중 새로운 발견 정황 II 정리 + 위키 페이지 보강 |
| `reference_authoritative_hancom.md` | ✅ 한컴 환경별 차이 자료 commit + 위키 안내 |
| `feedback_v076_regression_origin.md` | ✅ 광범위 변화에도 작업지시자 직접 시각 검증 게이트 |
| `feedback_hancom_compat_specific_over_general.md` | ✅ ImageNode.original_size_hu 명시 + page_path/lazy_path 분리 (일반화 회피) |
| output 폴더 가이드라인 | ✅ `output/svg/pr434-{visual,devel-baseline,regression-test,regression-baseline}/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr434` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr434` → `local/devel` → `devel` 머지 + push
3. PR #434 close + 작성자 댓글 (이슈 #430 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_434_review.md`
- PR: [#434](https://github.com/edwardkim/rhwp/pull/434)
- 이슈: [#430](https://github.com/edwardkim/rhwp/issues/430)
- 위키 보강: [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) (발견 정황 II)
- 한컴 정답지 3종: `samples/{2010,2020,hancomdocs}-exam_kor.pdf`
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415), [#424](https://github.com/edwardkim/rhwp/pull/424)
