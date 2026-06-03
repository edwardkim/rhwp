# Stage 5 완료보고서 - Task M100-1197

- 이슈: #1197
- 단계: Stage 5 - 통합 검증, 시각 판정 준비, 최종 보고
- 브랜치: `local/task1197`
- 기준 커밋: `58795584`
- 작성일: 2026-06-02

## 1. 진행 요약

Stage 1~4에서 구현한 HWPX 용지/페이지 기준 Picture/Table/Shape 공통 z-order 계약에 대해
최종 자동 검증을 수행하고, 작업지시자가 직접 확인할 수 있는 시각검증 산출물을 준비했다.

원본 재현 문서 `[2027] 온새미로 1 본교재.hwpx`와 PDF 정답지는 저장소에 없고,
#1197 이슈 본문에도 직접 첨부되어 있지 않다. 따라서 실제 원본 기반 SVG 산출은 보류하고,
자동 회귀 fixture와 #1167 실제 샘플을 시각검증 대상으로 제공한다.

## 2. 자동 검증 결과

### 포맷

```sh
cargo fmt --all --check
```

결과: 통과.

### 핵심 회귀

```sh
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
cargo test --test issue_1197_svg_object_zorder -- --nocapture
```

결과: 모두 통과.

비고: #1167 테스트는 기존과 같은 `LAYOUT_OVERFLOW` 진단 1건을 출력했지만 assertion은 통과했다.

### 전체 테스트

```sh
cargo test --tests
```

결과: 통과.

확인된 주요 구간:

- `src/lib.rs` 단위 테스트: `1525 passed; 0 failed; 6 ignored`
- integration tests: #1197, #1167, #1017, #516, #938 등 관련 회귀 포함 통과

### native Skia feature

```sh
cargo test --features native-skia --lib behind_text_layered_vector_replays_below_flow_across_tree_branches -- --nocapture
```

결과: `1 passed; 0 failed`.

## 3. 시각검증 산출물

산출물 위치:

- `output/poc/issue1197/visual_check.html`
- `output/poc/issue1197/synthetic/issue1197_synthetic_zorder.svg`
- `output/poc/issue1197/issue1167/복학원서.svg`

`output/`은 `.gitignore` 대상이므로 PR 커밋에는 포함하지 않는다.

### #1197 synthetic z-order 확인 기준

파일: `output/poc/issue1197/synthetic/issue1197_synthetic_zorder.svg`

확인 포인트:

- 빨간 `Z01_LOW_TABLE`은 파란 `Z11 IMAGE` 아래에 가려져야 한다.
- 초록 `Z12_FINAL_TABLE`은 파란 이미지 위에 보여야 한다.
- 노란 `01` 도형은 최상단에 보여야 한다.
- SVG 문서 순서는 `Z01_LOW_TABLE` → `Z11_IMAGE` → `Z12_FINAL_TABLE` → `Z69_FRONT_SHAPE`이다.

### #1167 실제 샘플 회귀 확인 기준

파일: `output/poc/issue1197/issue1167/복학원서.svg`

확인 포인트:

- 중앙 BehindText 워터마크가 본문 텍스트를 덮지 않아야 한다.

## 4. 원본 재현 문서 관련 상태

#1197 이슈 본문에는 기대/현재 스크린샷은 있으나 원본 HWPX/PDF는 직접 첨부되어 있지 않다.
본문에도 “원본 HWPX/PDF 파일은 이슈 본문에는 직접 첨부하지 않고, 후속 PR에서 재현용 테스트/검증 자료로 포함할 예정”이라고 되어 있다.

따라서 원본 기반 명령은 현재 실행하지 못했다.

```sh
target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -o output/poc/issue1197/page4 -p 3
```

원본 파일이 제공되면 위 명령으로 추가 시각검증을 수행할 수 있다.

## 5. 결론

코드 경로 자동 검증은 완료됐다.
작업지시자 시각검증은 준비된 `output/poc/issue1197/visual_check.html` 기준으로 진행하면 된다.

시각검증 승인 후 PR 준비 또는 push/PR 생성 단계를 진행한다.
