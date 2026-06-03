# Task #310 1단계 완료 보고서: dump-pages vpos 출력 추가

상위: 구현 계획서 `task_m100_310_impl.md`, Epic #309

## 변경 요약

`rhwp dump-pages` 출력에 각 항목의 LINE_SEG `vertical_pos` 정보를 노출. SVG 렌더링/페이지네이션 로직 무변경.

## 변경 파일

- `src/document_core/queries/rendering.rs`
  - `dump_page_items` 메서드의 각 `PageItem` 출력 라인에 `vpos=...` 정보 추가
  - 모듈 하단에 free 함수 `format_vpos_range(para, start_line, end_line) -> String` 신설

## 출력 형식

```
FullParagraph  pi=44  h=14.7 (...)  vpos=0  "..."
FullParagraph  pi=46  h=88.0 (...)  vpos=2476..11556  "..."
PartialParagraph  pi=50  lines=0..9  vpos=75116..0 [vpos-reset@line9]
Table          pi=45 ci=0  16x4 ...  vpos=...
Shape          pi=N ci=M  ...  vpos=...
```

`vpos-reset` 규칙:
- 줄 인덱스 > 0 인데 `vertical_pos == 0` 인 경우 마킹
- 문단 첫 줄(line 0)의 0 값은 자연 시작점이므로 제외
- 같은 문단 내 여러 리셋 발생 시 모두 표기

## 21_언어 페이지 3 검증

```
=== 페이지 3 (global_idx=2, section=0, page_num=3) ===
  단 0 (items=7)
    FullParagraph  pi=44 ... vpos=0
    ...
    PartialParagraph  pi=50  lines=0..9  vpos=75116..0 [vpos-reset@line9]   ← HWP 단 경계
  단 1 (items=21)
    PartialParagraph  pi=50  lines=9..12  vpos=0..3632 [vpos-reset@line9]   ← HWP 단 경계
    FullParagraph  pi=51 ... vpos=5448..29056
    ...
```

페이지 3 단 0/1 경계가 LINE_SEG `vpos=0` 리셋(pi=50, line 9)과 정확히 일치. 분석 보고서의 단서가 도구로 자동 확인됨.

## 회귀 검증

- `cargo build` 성공
- `cargo test`: **992 passed; 0 failed**
- 기존 출력 라인 형식 유지 (vpos 는 기존 필드 뒤에 추가됨)
- SVG 렌더링 결과 무변경

## 비범위 확인

- 페이지네이션 알고리즘 변경 없음
- `--debug-overlay` 변경 없음 (2단계 작업)
- 분석 보고서 미작성 (3단계 작업)

## 다음 단계

2단계: `--debug-overlay`에 vpos=0 리셋 위치 시각 표시
