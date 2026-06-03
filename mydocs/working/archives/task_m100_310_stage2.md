# Task #310 2단계 완료 보고서: --debug-overlay vpos=0 리셋 표시

상위: 구현 계획서 `task_m100_310_impl.md`, Epic #309

## 변경 요약

`--debug-overlay` 옵션 사용 시, SVG에 LINE_SEG `vertical_pos == 0` (문단 첫 줄 제외) 위치를 앰버 가로 점선 + 라벨로 시각 표시. 미사용 시 출력 무변동.

## 변경 파일

- `src/renderer/render_tree.rs`
  - `TextLineNode`에 `line_index: Option<u32>`, `vpos: Option<i32>` 필드 추가
  - 새 생성자 `with_para_vpos(line_height, baseline, section_index, para_index, line_index, vpos)`
  - 기존 `new`/`with_para`는 유지하며 신규 필드는 `None` (호출자 변경 없음)

- `src/renderer/layout/paragraph_layout.rs:819`
  - 본문 문단 메인 루프에서 `with_para_vpos` 사용. `para.line_segs[line_idx].vertical_pos` 전달
  - 기타 호출 지점(셀 내부, 그리기 객체 내부 등)은 기존 `new`/`with_para` 유지

- `src/renderer/svg.rs`
  - `OverlayVposReset` 구조체 신설 (section_index, para_index, line_index, x/y/width)
  - `overlay_vpos_resets: Vec<OverlayVposReset>` 필드 추가
  - TextLine 처리 블록에서 `line_index > 0 && vpos == 0` 시 마커 수집
  - `render_debug_overlay`에 마커 그리기 (앰버 `#FFB300` 가로 점선 + 라벨 `vpos-reset s{si}:pi={pi}:line={n}`)
  - `begin_page`에서 `overlay_vpos_resets.clear()`

## 검증

### vpos-reset 마커 출력
21_언어 페이지 3 (`global_idx=2`):
```bash
$ rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 2 --debug-overlay -o /tmp/
$ grep -c "vpos-reset" /tmp/*_003.svg
1
```
1단계 dump-pages 출력에서 확인된 `pi=50 line=9` vpos 리셋이 SVG에 가로선으로 표시됨.

### `--debug-overlay` 미사용 시
```bash
$ rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 2 -o /tmp/
$ grep -c "vpos-reset" /tmp/*_003.svg
0
```
옵트인 정상 동작.

### 회귀 테스트
- `cargo build` 성공
- `cargo test`: **992 passed; 0 failed**
- 기존 디버그 오버레이 (문단/표 경계) 정상 동작
- 본문 외 영역 (머리말/꼬리말/바탕쪽/표 셀 내부) 마커 미표시 — `overlay_skip_depth` 가드 활용

## 비범위 확인

- 페이지네이션 알고리즘 변경 없음
- 기본 SVG 출력 (옵트인 미사용) 변경 없음

## 다음 단계

3단계: 4개 샘플(21_언어/exam_math/exam_kor/exam_eng) vpos 패턴 분석 보고서 작성
