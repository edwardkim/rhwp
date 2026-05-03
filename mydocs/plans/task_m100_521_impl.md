# Task #521 구현계획서: tac=true 표의 host_line_spacing 포함

## 변경 대상

`src/renderer/typeset.rs:1230-1237` — `is_tac` 분기 제거하여 `tac=true` 인라인 표도 host paragraph 의 line_spacing 을 `host_spacing.after` 에 포함시킴.

### 변경 전
```rust
let host_line_spacing = if !is_tac {
    para.line_segs.last()
        .filter(|seg| seg.line_spacing > 0)
        .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
        .unwrap_or(0.0)
} else {
    0.0
};
```

### 변경 후
```rust
let host_line_spacing = para.line_segs.last()
    .filter(|seg| seg.line_spacing > 0)
    .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
    .unwrap_or(0.0);
```

## 단계

### Stage 2: 수정 + 1차 회귀 검증

1. **베이스라인 회귀 측정** — 수정 전 `scripts/svg_regression_diff.sh` 실행, byte-identical 페이지 수 기록
2. **수정 적용** — typeset.rs 단일 분기 제거
3. **빌드 + 단위 테스트** — `cargo build --release`, `cargo test --lib`, `cargo test --test svg_snapshot`
4. **회귀 측정** — 수정 후 `svg_regression_diff.sh` 실행, 베이스라인과 비교
5. **본 케이스 검증** — exam_eng p2 18번 ① 위치 측정, PDF 와 비교

### Stage 3: 회귀 분석 + 최종 보고서

1. **회귀 차이 분석**:
   - byte-identical 가 줄어든 페이지를 모두 시각 확인 (PDF 대비 개선/악화)
   - 의도된 정정 (tac=true 표 다음 문단 정확도 향상) vs 회귀 (다른 케이스 변경) 분류
2. **clippy** 통과 확인
3. **최종 보고서** + orders 갱신
4. **본질 2 별도 task 등록** (`/2.0` spacing 가설 검증)

## 위험 평가

- **영향 범위**: `tac=true` 인라인 표 host paragraph 다음 문단 위치 (분포 조사상 exam_eng 9건 + 다른 샘플 가능)
- **회귀 위험**: **중간** — 인라인 표 케이스 모두 영향, 한컴 PDF 와 비교 검증 필수
- **롤백 가능성**: 단일 분기 제거이므로 단순 revert

## 검증 게이트

- [ ] cargo test --lib 통과
- [ ] cargo test --test svg_snapshot 통과
- [ ] cargo clippy --lib 무경고
- [ ] svg_regression_diff.sh: 변경 페이지 모두 시각 분석 + 의도된 정정 확인
- [ ] exam_eng p2 18번 ① 위치 PDF 와 4 px 이내 일치 (본질 2 미수정으로 완전 일치는 어려움)

## 산출물

- `mydocs/working/task_m100_521_stage2.md` — 수정 + 회귀 분석
- `mydocs/working/task_m100_521_stage3.md` — 최종 검증 + 보고서
- `mydocs/report/task_m100_521_report.md` — 최종 보고서
- 별도 GitHub 이슈 (본질 2: spacing /2.0 가설 검증 task)
