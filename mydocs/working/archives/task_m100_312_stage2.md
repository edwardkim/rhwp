# Task #312 2단계 완료 보고서: 차이 origin 식별 (의외의 발견)

상위: 구현 계획서, Epic #309

## 핵심 발견

조사 도중 코드베이스에 **이미 존재하는** `TypesetEngine` 이 4샘플 모두에서 `Paginator` 보다 더 적은(더 정확한) 페이지 수를 산출함을 발견. 21_언어는 정확히 PDF와 일치(15쪽).

| 샘플 | Paginator | TypesetEngine | PDF | TypesetEngine = PDF? |
|------|-----------|---------------|-----|----------------------|
| 21_언어 | 19 | **15** | 15 | ✅ 정확히 일치 |
| exam_math | 20 | 20 | 20 | ✅ 동일 |
| exam_kor | 25 | 24 | (미보유) | typeset 1쪽 적음 |
| exam_eng | 11 | 9 | (미보유) | typeset 2쪽 적음 |

검증 출처: `dump-pages` 실행 시 stderr 로 출력되는 `TYPESET_VERIFY` 메시지.

## TypesetEngine 코드 위치

- `src/renderer/typeset.rs` — 단일 패스 조판 엔진. format() → fits() → place/split 흐름
- `src/document_core/queries/rendering.rs:837~` — Paginator 결과와 병렬 검증 (debug_assertions 한정, 페이지 수 차이 시 stderr 경고)
- 결과는 사용되지 않고 검증 비교용이므로 실제 렌더링은 여전히 Paginator 기반

## 페이지 7 단별 측정 데이터 (1단계 도구 사용)

| 페이지·단 | items | used | hwp_used | diff |
|-----------|-------|------|---------|------|
| P7 단 0 | 13 | 1062.5 | 1210.7 | -148.2 |
| P7 단 1 | 8 | 388.8 | 1030.1 | -641.3 |
| P1 단 0 | 12 | 1233.5 | 1147.7 | +85.8 |
| P1 단 1 | 2 | 1230.8 | 231.7 | +999.1 |
| P3 단 0 | 7 | 1219.5 | 1209.9 | +9.5 |
| P3 단 1 | 21 | 993.3 | 1209.9 | -216.6 |
| ... | | | | |

다양한 부호와 크기의 diff가 페이지/단별로 발생 — 단일 차이 origin 모델로는 설명 불가. 여러 미세 요인이 페이지/단마다 다르게 누적.

## 결론

본 sub-issue가 가정한 "단일 column 가용 공간 origin 식별 + 보정" 접근은 부정확. 대신 답은 다음에 있다:

**`TypesetEngine`을 main pagination으로 전환**

이미 검증된 코드가 존재하며 Paginator의 누적 오차 문제를 본질적으로 해결한다. 추가 사이드 효과:
- exam_kor, exam_eng 도 PDF 일치 가능성
- 단일 패스 설계로 measure → paginate → layout 3단계 불일치 제거

## 작업 범위 변경 제안

본 #312 범위 종료 (1단계 측정 도구 보존 + 본 발견 문서화). 새 sub-issue 등록:

**제목 (제안)**: `TypesetEngine을 main pagination으로 전환 (Paginator 대체)`

**범위**:
1. Paginator → TypesetEngine 전환 가이드 작성 (호환성 검토)
2. WASM API / DocumentCore 진입점 변경
3. 4샘플 검증: 21_언어 15쪽, exam_math 20쪽, exam_kor 24쪽, exam_eng 9쪽 (또는 PDF 일치)
4. 전체 cargo test 회귀 확인
5. Paginator 코드 정리/제거 또는 보존 결정

## 다음 단계

3단계: 본 발견을 최종 보고서로 정리 + Epic #309 코멘트 + 새 sub-issue 등록
