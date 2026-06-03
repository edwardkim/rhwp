# Task #310 최종 보고서: LINE_SEG vpos 노출 + 검증 도구 추가

상위: Epic #309
브랜치: `task306`
커밋: `7e7d99a` (1단계), `621d0ba` (2단계), 본 보고서 커밋 (3단계)

## 결과 요약

목표 100% 달성. 코드 회귀 0. **2단계 작업 범위가 분석 결과로 크게 줄어듦** (전면 재설계 → 단일 조건 분기) — 본 분석의 최대 수확.

## 산출물 체크리스트

- [x] `dump-pages` 출력에 LINE_SEG vpos 컬럼 + `[vpos-reset@line{n}]` 마커
- [x] `--debug-overlay`에 vpos=0 리셋 위치 시각 표시 (앰버 가로 점선)
- [x] 4개 샘플 vpos 패턴 분석 보고서 (`mydocs/tech/line_seg_vpos_analysis.md`)
- [x] Epic #309 코멘트 게시 (다음 단계에서)
- [x] `cargo test`: 992 passed, 0 failed (회귀 0)

## 핵심 발견 (Epic 후속 작업의 기초)

| 샘플 | SVG 쪽 | vpos-reset | FullParagraph 내부 reset |
|------|--------|-----------|--------------------------|
| 21_언어 | 19 (+4) | 13 | **7** |
| exam_math | 20 ✓ | 0 | 0 |
| exam_kor | 25 | 8 | 0 |
| exam_eng | 11 ✓ | 0 | 0 |

21_언어의 +4쪽 과잉은 13개 vpos-reset 중 **7개가 FullParagraph 내부**에 있어 발생. 다른 3개 샘플은 이 패턴이 0건.

## 단계별 결과

### 1단계: dump-pages vpos 출력 (커밋 `7e7d99a`)
- `src/document_core/queries/rendering.rs` 수정
- 모든 PageItem(FullParagraph/PartialParagraph/Table/PartialTable/Shape)에 vpos 정보 출력
- vpos-reset 검출 로직: `line_index > 0 && vertical_pos == 0`

### 2단계: --debug-overlay vpos-reset 시각 표시 (커밋 `621d0ba`)
- `src/renderer/render_tree.rs` — TextLineNode에 `line_index`/`vpos` 필드 추가
- `src/renderer/layout/paragraph_layout.rs:819` — 본문 메인 루프에서 `with_para_vpos` 사용
- `src/renderer/svg.rs` — OverlayVposReset 구조체 + render_debug_overlay 마커 그리기
- 옵트인: `--debug-overlay` 미사용 시 SVG 출력 무변동

### 3단계: 4개 샘플 분석 보고서 (본 커밋)
- `mydocs/tech/line_seg_vpos_analysis.md`

## 권장 후속 작업

Epic #309의 다음 sub-issue로 **`페이지네이션에서 LINE_SEG vpos-reset을 단/페이지 경계로 강제`** 등록을 권장한다.

원래 분석 보고서(`task_m100_306_analysis.md`)는 "페이지네이션 엔진 LINE_SEG vpos 우선 모드 (전면 재설계)"를 권장했으나, 4개 샘플 데이터 결과 다음과 같이 좁은 범위로 충분할 가능성이 높다:

- **변경 범위**: `src/renderer/pagination/engine.rs`의 FullParagraph 처리 로직에서 line_segs vpos-reset 검출 시 PartialParagraph 분리 강제
- **회귀 위험**: 매우 낮음. 4개 샘플 중 21_언어만 동작 변경, 나머지 3개는 무변화 예상
- **단계적 도입**: 옵션 플래그(`--respect-vpos-reset`) 도입 후 4샘플 검증 → 기본 on 전환

## 회귀 검증

- `cargo test`: **992 passed; 0 failed** (1·2단계 모두)
- 기본 SVG 출력 무변화 (옵트인 옵션 미사용 시)
- 기본 dump-pages 라인 형식 유지 (vpos는 기존 필드 뒤에 추가)
- 4개 샘플 페이지 수 변화 0 (기존 동작 유지)

## 비범위 확인

본 타스크는 **검증 도구 추가만** 수행. 아래는 의도적으로 비범위:
- 페이지네이션 알고리즘 변경 (Epic #309 sub-issue #2 이후)
- vpos 기반 새 배치 로직
- 기본 SVG 렌더링 결과 변경

## 학습

- HWP 파일이 인코딩한 LINE_SEG vpos는 **모든 문서에 의미가 있는 것이 아니라**, 강제 단/페이지 분리 의도가 있는 문서에서만 등장한다 (exam_math/exam_eng 의 0개 케이스)
- 우연한 일치(exam_kor의 PartialParagraph 분리)는 매커니즘이 다르더라도 결과가 같으면 사용자 관점에서 문제 없음
- **분석 도구를 먼저 만들고 데이터를 본 결과, 원래 권장하던 전면 재설계가 불필요해짐** — 도구 우선 접근의 효용 입증

## 본 타스크 종료 절차

작업지시자 승인 시:
1. Epic #309에 분석 결과 코멘트 게시
2. Sub-issue #2 등록 (제안: `페이지네이션에서 LINE_SEG vpos-reset을 단/페이지 경계로 강제`)
3. `gh issue close 310`
