# Task #311 구현 계획서

상위: `task_m100_311.md`, Epic #309
브랜치: `task311`

## 단계 구성

3단계로 분할. 각 단계는 독립 커밋. 단계 완료 시 `_stage{N}.md` 보고서 후 다음 단계 진행.

---

## 1단계 — `PaginationOpts` 구조체 도입 (리팩토링, 회귀 0)

### 목표
`paginate_with_measured_opts`의 bool 플래그 인자를 구조체로 전환. 기능 변경 없음.

### 변경 파일
- `src/renderer/pagination/engine.rs` — API + 시그니처
- `src/renderer/pagination.rs` (기존 호출자)
- `src/document_core/queries/rendering.rs:726, 824` (호출자 2곳)

### 변경 내용
1. 신규 구조체:
```rust
#[derive(Debug, Clone, Default)]
pub struct PaginationOpts {
    pub hide_empty_line: bool,
    pub respect_vpos_reset: bool,  // 1단계에서는 미사용 필드, 2단계에서 활용
}
```
2. `paginate_with_measured_opts` 시그니처를 `(..., opts: PaginationOpts)` 로 변경
3. `paginate_with_measured` 는 `PaginationOpts::default()` 전달로 단순화
4. 호출자 2곳 마이그레이션

### 검증
- `cargo build` / `cargo test`: 992 passed 유지
- 4개 샘플 페이지 수 무변화

---

## 2단계 — vpos-reset 강제 분리 로직 + `--respect-vpos-reset` 플래그

### 목표
옵션 on 시 LINE_SEG vpos-reset 위치에서 PartialParagraph 강제 분리. 옵션 off 시 기존 동작 100% 동일.

### 변경 파일
- `src/renderer/pagination/engine.rs::paginate_text_lines` — 핵심 로직
- `src/main.rs` — CLI `--respect-vpos-reset` 플래그
- `src/document_core/queries/rendering.rs` — 옵션 전달 (필요 시)

### 알고리즘 (paginate_text_lines 진입 시)

```rust
if opts.respect_vpos_reset {
    let forced_breaks: Vec<usize> = para.line_segs.iter().enumerate()
        .filter(|(i, ls)| *i > 0 && ls.vertical_pos == 0)
        .map(|(i, _)| i)
        .collect();
    if !forced_breaks.is_empty() {
        return self.paginate_with_forced_breaks(st, para_idx, para, measured, &forced_breaks);
    }
}
// 기존 로직 (변경 없음)
```

새 메서드 `paginate_with_forced_breaks`:
- forced_breaks를 줄 분할 cursor 운영의 강제 break 지점으로 사용
- 각 break 직전까지를 PartialParagraph 로 push, `advance_column_or_new_page()` 호출
- 마지막 segment는 line_count까지
- 가용 공간 부족 시 기존 줄 분할 로직과 결합 (forced_break 우선, 그 외엔 자연 분할)

### CLI 통합
- `src/main.rs` `export-svg` 분기에 `--respect-vpos-reset` 파싱
- `DocumentCore` 또는 렌더 호출 경로로 전달
- 옵션을 `PaginationOpts.respect_vpos_reset` 으로 채움

### 옵션 off 가드
함수 진입 시 `if !opts.respect_vpos_reset { /* 기존 로직 */ }` early-branch — 옵션 off 경로는 기존과 비트레벨 동일.

### 검증
- 옵션 off (기본값): 4샘플 페이지 수 무변화 + `cargo test` 통과
- 옵션 on:
  - 21_언어: **15쪽** (PDF 일치)
  - exam_math: 20 유지
  - exam_kor: 25 유지
  - exam_eng: 11 유지
- `dump-pages` 출력에서 vpos-reset 줄이 새 단/페이지 첫 항목으로 이동 확인

---

## 3단계 — 4샘플 검증 + 기본 on 전환 + 보고서

### 목표
2단계 옵션을 기본 on 전환. 보고서 작성.

### 변경
- `PaginationOpts::default()` 의 `respect_vpos_reset: true`
- `--respect-vpos-reset` 플래그를 `--no-respect-vpos-reset` 으로 토글 (또는 디버그용으로 유지)
- 회귀 테스트(`cargo test`) 4개 샘플 페이지 수 검증 추가 (선택)

### 검증
- 기본 동작에서 21_언어 15쪽
- `cargo test` 992 passed 유지
- 분석 보고서 갱신: `mydocs/tech/line_seg_vpos_analysis.md` 에 결과 추가

### 산출물
- 단계 보고서 `task_m100_311_stage3.md`
- 최종 보고서 `task_m100_311_report.md`
- 오늘할일 갱신
- Epic #309 클로즈 가능 여부 평가 코멘트

---

## 회귀 검증 명령

```bash
# 페이지 수 자동 비교
for f in samples/{21_언어_기출_편집가능본,exam_math,exam_kor,exam_eng}.hwp; do
  pages=$(cargo run --bin rhwp -q -- dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  echo "$(basename $f): $pages 쪽"
done
```

기대값:
- 21_언어: 15
- exam_math: 20
- exam_kor: 25
- exam_eng: 11

## 커밋/브랜치

- 브랜치: `task311` 유지
- 커밋: `Task #311: 1단계 - PaginationOpts 구조체 도입`, `Task #311: 2단계 - vpos-reset 강제 분리`, `Task #311: 3단계 - 기본 on 전환 + 보고서`
- 단계 보고서는 해당 단계 코드 커밋과 함께 커밋

## 승인 요청

위 분할로 진행. 승인 시 1단계 시작.
