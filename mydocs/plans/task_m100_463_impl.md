# Task #463 구현계획서

## 핵심 변경

`paragraph_layout.rs:2516` 의 `para_border_ranges.push` 호출을 **`cell_ctx.is_none()` 일 때만** 실행하도록 게이팅한다. 셀 안 단락의 외곽선은 이미 표 셀 렌더 경로(`border_rendering.rs` + cell border 출력) 에서 처리되므로 본문 외곽선 큐에 들어갈 이유가 없다.

## 구현 단계

### Stage 1: 수행계획서/구현계획서 작성 + 승인

- `mydocs/plans/task_m100_463.md` (수행계획서) ✅
- `mydocs/plans/task_m100_463_impl.md` (구현계획서) ✅
- 자동 승인 진행

### Stage 2: 셀 컨텍스트 게이팅 + 회귀 점검 코드 (단위 테스트 우선 추가)

**파일**: `src/renderer/layout/paragraph_layout.rs`

```rust
// 기존 (line 2516 부근)
if para_border_fill_id > 0 {
    let bg_height = y - bg_y_start;
    if bg_height > 0.0 {
        ...
        self.para_border_ranges.borrow_mut().push(...);
    }
}

// 변경 후
if para_border_fill_id > 0 && cell_ctx.is_none() {
    let bg_height = y - bg_y_start;
    if bg_height > 0.0 {
        ...
        self.para_border_ranges.borrow_mut().push(...);
    }
}
```

추가로 `layout_raw_paragraph` (fallback 경로) 도 동일 게이팅 적용 필요 여부 확인 후 일관 처리.

**검증**:
- `cargo build --release` 통과
- `cargo test --lib` 통과
- `cargo clippy --release -- -D warnings` 통과

### Stage 3: exam_kor 14p SVG 시각 검증 + 통합 테스트

**검증 명령**:
```bash
./target/release/rhwp export-svg samples/exam_kor.hwp -p 13 -o /tmp/p14_after
# SVG 의 stroke 있는 rect 개수, 위치, 크기 확인
grep -c 'stroke="#000000"' /tmp/p14_after/exam_kor_014.svg
```

**수동 시각 검증**:
- Chrome headless 로 SVG → PNG 렌더 후 PDF 와 비교
- 좌측 단 박스 개수 4 → 1 (또는 2 — 헤더 + 본문)
- 우측 단 (나) 편지 박스 통합 확인

**회귀**:
- 다른 샘플의 표/문단 외곽선 회귀 없음 확인
  - `samples/2010-01-06.hwp`, `samples/biz_plan.hwp`, `samples/aift.hwp` 등
- `cargo test --release --test integration_tests` 또는 동등 테스트 실행

### Stage 4: 최종 보고서 + 오늘할일 갱신

- `mydocs/working/task_m100_463_stage1.md` ~ `_stage3.md` 단계별 보고서 작성
- `mydocs/report/task_m100_463_report.md` 최종 보고서
- `mydocs/orders/{오늘날짜}.md` 갱신
- 모든 산출물 커밋 후 `local/devel` 머지 준비

## 변경 파일 목록 예상

| 파일 | 변경 내용 | 변경 라인 추정 |
|------|----------|--------------|
| `src/renderer/layout/paragraph_layout.rs` | `cell_ctx.is_none()` 게이팅 추가 | ~3 |
| `mydocs/plans/task_m100_463.md` | 수행계획서 신규 | 신규 |
| `mydocs/plans/task_m100_463_impl.md` | 구현계획서 신규 | 신규 |
| `mydocs/working/task_m100_463_stage{N}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_463_report.md` | 최종 보고서 | 신규 |

## 회귀 리스크 평가

- **낮음**: 본 변경은 "셀 단락의 외곽선 정보를 본문 큐에 넣지 않는다" 라는 단순 게이팅. 셀 외곽선은 이미 별도 경로에서 처리되므로 셀 외곽선 자체가 사라지지는 않는다.
- **확인 필요**: 만약 어떤 샘플에서 셀 외곽선이 본문 큐를 통해 그려지고 있었다면 회귀 발생. integration_tests 의 표 외곽선 케이스 통과 여부로 확인.

## 의존성

- Task #321 v6 (stroke signature merge) — 본 수정으로 입력 정상화. merge 로직 자체는 변경하지 않음.
- 향후 검토: stroke signature 기반 merge 가 cross-bf_id 병합을 허용하는 정책의 다른 부작용. 별도 이슈로 분리 권장.
