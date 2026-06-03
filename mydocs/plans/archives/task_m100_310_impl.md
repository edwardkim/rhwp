# Task #310 구현 계획서

상위: 수행계획서 `task_m100_310.md`, Epic #309

## 단계 구성

3단계로 분할. 각 단계는 독립 커밋 가능하며, 단계 완료 시 `_stage{N}.md` 보고서 작성 후 승인 요청.

---

## 1단계 — `dump-pages` vpos 출력 추가

### 목표
`dump-pages` 출력에 각 문단의 LINE_SEG `vpos` 정보를 노출. SVG/렌더링 무관 — CLI 진단 도구만 변경.

### 변경 파일
- `src/document_core/queries/rendering.rs::dump_page_items` (1300번대)

### 변경 내용
1. `FullParagraph`/`PartialParagraph` 항목 출력 시 해당 파라그래프의 LINE_SEG 목록에서 `vpos` 추출
   - 첫 줄/마지막 줄 vpos 출력: `vpos=NNNN..MMMM`
   - 줄 중 `vpos=0` (line_index>0)이 있으면 `[vpos-reset@line{n}]` 마커 부착
   - LINE_SEG 접근 경로: `paragraphs[pi].line_segs` (기존 `dump` 명령에서 사용 중인 동일 필드)
2. `PartialParagraph`는 `start_line..end_line` 범위의 vpos만 표시
3. `Table`/`Shape`는 컨테이너 문단의 첫 줄 vpos만 표시 (참고용, 마커는 생략)

### 출력 예시 (변경 후)
```
=== 페이지 16 (global_idx=15, section=2, page_num=6) ===
  body_area: x=96.0 y=103.6 w=601.7 h=930.5
  단 0 (items=7)
    FullParagraph  pi=41  h=37.3 (sb=16.0 lines=21.3 sa=0.0)  vpos=15360..16360  "..."
    FullParagraph  pi=44  h=21.3 ...  vpos=75116..89644  "..."
    FullParagraph  pi=50  h=21.3 ...  vpos=0..1816 [vpos-reset@line0]  "..."
```

### 단위 테스트
- `21_언어_기출_편집가능본.hwp` 페이지 3 (global_idx=2): vpos-reset 마커가 적어도 1개 출력되는지 확인 (스냅샷 비교는 하지 않음 — assert는 substring 포함 여부만)
- `cargo test` 전체 통과

### 회귀 검증
- 기존 출력 라인 형식 유지 (vpos는 기존 필드 뒤에 추가)
- SVG 렌더링 무변경

### 산출
- 코드 수정
- `mydocs/working/task_m100_310_stage1.md` (커밋 동시)

---

## 2단계 — `--debug-overlay` vpos=0 리셋 표시

### 목표
SVG 디버그 오버레이에 LINE_SEG `vpos=0` 리셋 위치를 시각 표시. `--debug-overlay` 미사용 시 무영향.

### 변경 파일
- `src/renderer/render_tree.rs::TextLineNode` — 필드 추가
- `src/renderer/svg.rs` — `OverlayBounds` 옆에 vpos-reset 수집 구조 + `render_debug_overlay`
- TextLineNode 생성 지점 (text_layout 등) — 신규 필드 채움

### 변경 내용
1. `TextLineNode`에 `line_vpos: Option<i32>` 필드 추가 (LINE_SEG의 `vpos` 그대로)
   - `TextLineNode::with_para` 시그니처 확장 (또는 신규 생성자) — 기존 호출자는 `None`
2. TextLine 생성하는 지점에서 LINE_SEG 의 vpos 를 채워 전달
   - 위치: `src/renderer/text_layout.rs` 또는 page builder (grep으로 특정)
3. `svg.rs`에 `overlay_vpos_resets: Vec<VposResetMarker>` 추가
   - 수집: TextLine 처리 시 `line_vpos == Some(0)` 이며 line_index>0 인 경우 (line_index 추적 필요 → `line_index_in_para` 필드도 함께 추가하거나, 문단당 첫 TextLine을 기억하는 방식)
   - 단순화: para 별 첫 TextLine 후 등장하는 vpos=0 줄을 모두 리셋으로 간주
4. `render_debug_overlay`에서 노란 가로 점선 + 라벨 그리기
   - 색: `#FFB300` (앰버), 라벨 텍스트: `vpos-reset s{si}:pi={pi}`

### 단위 테스트
- 21_언어 페이지 3 SVG 생성 시 출력에 `vpos-reset` 문자열 포함 확인 (`--debug-overlay` 켰을 때)
- `--debug-overlay` 미사용 시 출력에 `vpos-reset` 미포함

### 회귀 검증
- `--debug-overlay` 미사용 시 SVG 출력 무변동 (기존 스냅샷 테스트 통과)
- 기존 디버그 오버레이 (문단/표 경계) 정상 동작

### 산출
- 코드 수정
- `mydocs/working/task_m100_310_stage2.md`

---

## 3단계 — 4개 샘플 vpos 패턴 분석 보고서

### 목표
1·2단계 도구를 사용하여 4개 샘플의 HWP 의도 vs 현 엔진 결과 차이를 데이터화. 코드 수정 없음.

### 작업
1. 각 샘플에 대해:
   - `rhwp dump-pages {sample.hwp}` 전체 출력 → vpos-reset 위치 목록 추출
   - `rhwp export-svg {sample.hwp} --debug-overlay` → 시각 확인
2. 데이터 표 구성 (보고서 구조):
   - 샘플별 총 페이지 수 (PDF / SVG)
   - 샘플별 vpos=0 리셋 총 개수
   - **HWP 의도 페이지 경계 목록**: vpos=0 리셋이 발생한 (섹션, pi, line) 위치
   - **현 엔진 페이지 경계 목록**: 페이지 첫 항목의 (섹션, pi)
   - **불일치 분석**: 21_언어에서 어긋남 시작 지점, exam_math에서 일치 패턴
3. 보고서: `mydocs/tech/line_seg_vpos_analysis.md`
4. Epic #309에 핵심 요약 코멘트 게시

### 산출
- `mydocs/tech/line_seg_vpos_analysis.md`
- `mydocs/working/task_m100_310_stage3.md`
- 최종 보고서 `mydocs/report/task_m100_310_report.md`

---

## 커밋/브랜치

- 브랜치: `task306` 유지
- 커밋 메시지: `Task #310: 1단계 - dump-pages vpos 출력`, `Task #310: 2단계 - debug-overlay vpos-reset 표시`, `Task #310: 3단계 - 4샘플 vpos 패턴 분석`
- 단계별 보고서는 해당 단계 코드 커밋과 함께 커밋

## 리스크 및 대응

| 리스크 | 대응 |
|--------|------|
| TextLineNode 필드 추가로 다수 호출 지점 변경 | 신규 setter 메서드로 옵트인. 기본은 None |
| 페이지 분할로 인해 vpos=0 가 자연 등장 (페이지 첫 줄) | line_index>0 조건으로 필터 |
| 4개 샘플 중 한컴 PDF 미보유 (exam_kor/eng) | 보고서에 "PDF 페이지 수 미상" 명시. SVG vs HWP 의도 비교에 집중 |

## 승인 요청

위 3단계 분할로 진행하는 것에 대한 승인. 승인 시 1단계 구현 시작.
