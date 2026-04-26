# PR #235 — 구현 계획서

## PR 변경 내용 전체 파악

### 범위 재정의 — **2개 이슈 해결**

| 이슈 | 제목 | 상태 |
|------|------|------|
| [#229](https://github.com/edwardkim/rhwp/issues/229) | 표 셀 긴 숫자 텍스트 겹침/셀 폭 미사용 | open |
| [#240](https://github.com/edwardkim/rhwp/issues/240) | BMP 이미지 SVG 렌더링 불가 (브라우저 BMP data URI 미지원) | closed |

> PR 제목/본문에는 #229만 언급되어 있으나, 커밋 히스토리를 분석한 결과 **#240 수정(BMP→PNG 재인코딩, xmlns:xlink)도 포함**됨.

### 커밋 16개 구성

| 범주 | 커밋 수 | SHA |
|------|--------|-----|
| Task #229 본 수정 | 2 | fda2f6f, 686299a |
| Task #240 본 수정 | 2 | 6db0d4f, 1dc5a57 |
| Task #240 후속 (xmlns:xlink) | 2 | eeb934e, d7b031b |
| Merge/sync 커밋 | 10 | (정리 필요) |

### 소스 변경 (핵심)

| 파일 | 변경 | 내용 |
|------|------|------|
| `src/renderer/layout/text_measurement.rs` | +120 | 자간 보정 로직 (Task #229) |
| `src/renderer/layout/paragraph_layout.rs` | +76/-8 | 자간 분기 통합 |
| `src/renderer/layout/table_cell_content.rs` | +13/-6 | 셀 내 텍스트 처리 |
| `src/renderer/layout/table_layout.rs` | +62/-5 | 표 레이아웃 연동 |
| `src/renderer/layout/table_partial.rs` | +12/-5 | 부분 표 처리 |
| `src/renderer/layout/shape_layout.rs` | +12/-2 | 도형 내 텍스트 |
| `src/renderer/svg.rs` | +35/-4 | BMP→PNG 재인코딩 + xmlns:xlink (Task #240) |
| `src/renderer/svg/tests.rs` | +44 | 단위 테스트 |
| `Cargo.toml` | +1 | 의존성 추가 추정 (BMP→PNG 변환용) |

### 테스트 스냅샷

| 파일 | 변경 | 비고 |
|------|------|------|
| `tests/golden_svg/form-002/page-0.svg` | ±695 | 기존 골든 재생성 (자간 변경으로 좌표 이동) |
| `tests/golden_svg/table-text/page-0.svg` | +209 | 신규 골든 (Task #229 검증용) |

### 샘플 파일 추가

- `samples/bitmap.hwp` (BMP 이미지 포함)
- `samples/hwpx/table-text.hwpx` (표 긴 숫자 케이스)
- `samples/표-텍스트.hwpx`
- `samples/한셀OLE.hwp`

### 문서 추가

| 파일 | 내용 |
|------|------|
| `mydocs/orders/20260422.md` | 오늘 할일 |
| `mydocs/orders/20260422_issue_bmp_svg_render.md` | Task #240 상세 |
| `mydocs/plans/task_m100_229_fix.md` | #229 수행계획서 |
| `mydocs/plans/task_m100_229_fix_impl.md` | #229 구현계획서 |
| `mydocs/plans/task_bug_240.md` | #240 수행계획서 (파일명 규칙 비표준) |
| `mydocs/plans/task_bug_240_impl.md` | #240 구현계획서 (파일명 규칙 비표준) |
| `mydocs/working/task_m100_229_fix_report.md` | #229 최종 보고서 |
| `mydocs/working/task_bug_240_stage{1,2,3}.md` | #240 단계별 보고서 (파일명 규칙 비표준) |
| `mydocs/working/task_bug_240_report.md` | #240 최종 보고서 (파일명 규칙 비표준) |
| `mydocs/report/task229/*.png` | 비교 이미지 3장 |

## 검토 시 유의 사항

### 1. 파일명 규칙 비표준

- Task #240 관련 문서 파일명이 `task_bug_240*.md`로 되어 있음
- 프로젝트 규칙: `task_m100_{번호}.md`
- Task #229 관련도 `task_m100_229_fix.md`로 suffix `_fix` 포함 → 규칙은 `task_m100_229.md`

### 2. 범위 확장 (#240 포함)

- PR 제목/본문에는 #229만 명시
- 실제로는 #240도 수정함 → **PR 설명에 추가 언급 필요**하나, 기여자의 선의 작업으로 수용 가능

### 3. Merge/sync 커밋 다수

- 16개 중 10개가 merge/sync → squash merge로 정리 권장

## 단계 구성

### 1단계: 로컬에 PR 체크아웃 + 빌드·테스트 검증

```bash
gh pr checkout 235 --repo edwardkim/rhwp
cargo build
cargo test
cargo clippy -- -D warnings
```

- 893+ 테스트 통과 유지 확인
- Clippy 경고 0건 확인

### 2단계: E2E 테스트 + 샘플 렌더링 검증

- 관련 샘플 파일(`bitmap.hwp`, `table-text.hwpx`, `표-텍스트.hwpx`)로 SVG 내보내기
- 기존 문서 regression 없는지 확인 (KTX.hwp, biz_plan.hwp 등)
- 첨부된 `mydocs/report/task229/*.png` 비교 이미지 확인

### 3단계: PR merge (squash)

- GitHub에서 **squash merge** — 16개 커밋 → 1개로 정리
- merge 메시지에 #229, #240 둘 다 명시
- 메인 merge 후 이슈 자동 close 확인

### 4단계: 최종 보고서 + 오늘할일 갱신

- `mydocs/report/task_m100_pr235_report.md` 작성
- 파일명 규칙 비표준 건은 향후 컨트리뷰터 가이드 보강 아이템으로 기록

## 예상 단계: 4단계
