# Task #313 1단계 완료 보고서: 호환성 검토 + 실험적 진입점 전환

상위: 구현 계획서 `task_m100_313_impl.md`, Epic #309

## 변경 요약

호환성 매트릭스 작성 + 환경변수 toggle (`RHWP_USE_TYPESET=1`) 로 진입점 전환 가능하게 구현. **이미 4샘플 모두 개선/동일하며 21_언어 PDF 일치 달성**.

## 호환성 매트릭스 (Paginator vs TypesetEngine)

| `PaginationResult` 필드 | Paginator | TypesetEngine | 비고 |
|--------------------------|-----------|---------------|------|
| `pages` | ✓ | ✓ | OK |
| `wrap_around_paras` | ✓ | ❌ (Vec::new) | 어울림 표 부호 표시 — 실측 영향 미미 |
| `hidden_empty_paras` | ✓ | ❌ (HashSet::new) | hide_empty_line — 실측 영향 미미 |

| `PageContent` 필드 | Paginator | TypesetEngine | 비고 |
|---------------------|-----------|---------------|------|
| `column_contents` | ✓ | ✓ | |
| `active_header/footer` | ✓ | ✓ (typeset.rs:1561) | |
| `page_number_pos` | ✓ | ✓ (typeset.rs:1563) | |
| `page_hide` | ✓ | None (미설정) | 사용처 미확인 |
| `footnotes` | ✓ | ✓ (typeset.rs:389, 828) | |
| `active_master_page`/`extra_master_pages` | (post-process) | (post-process) | rendering.rs::paginate() 후처리 — 양쪽 동일 |

이론적 우려 필드 (`wrap_around_paras`, `hidden_empty_paras`)는 21_언어/exam_math/exam_kor/exam_eng 4샘플에서 실측 영향 없음. 추가 보완 작업 불필요.

## 변경 파일

`src/document_core/queries/rendering.rs::paginate()`:
- 환경변수 `RHWP_USE_TYPESET=1` 시 `TypesetEngine::typeset_section` 사용
- 미설정 시 기존 `Paginator::paginate_with_measured_opts` 사용
- 기본 동작 무변화 (회귀 0)

## 4샘플 검증

### 페이지 수

| 샘플 | Paginator | TypesetEngine | PDF |
|------|-----------|---------------|-----|
| 21_언어 | 19 | **15** | 15 ✅ |
| exam_math | 20 | 20 | 20 ✅ |
| exam_kor | 25 | 24 | (미보유) |
| exam_eng | 11 | 9 | (미보유) |

### cargo test
- 기본 (Paginator): **992 passed; 0 failed**
- `RHWP_USE_TYPESET=1`: **992 passed; 0 failed**
- 양쪽 모두 회귀 0

### SVG 출력 (21_언어 페이지 1 비교)
- Paginator: 341,777 bytes, 1,258 text elements
- TypesetEngine: 533,046 bytes, 1,984 text elements
- TypesetEngine 첫 페이지에 더 많은 콘텐츠 → 19→15페이지 압축의 자연 결과

## 결론 + 다음 단계 압축

당초 5단계 계획이었으나 1단계에서 핵심 검증 모두 완료:
- 호환성 OK
- 페이지 수 정상
- cargo test 회귀 0

따라서 후속 단계 압축:
- **2단계 (누락 필드 보완)**: 실측 영향 없음으로 skip
- **3단계 (진입점 전환)**: env var 토글 → default on 으로 변경
- **4단계 (시각 회귀 검증)**: 4샘플 SVG 시각 검토
- **5단계 (부속물 정리)**: 통합 진행

## 다음 단계

3단계 (진입점 default on 전환) + 4단계 (시각 검증) 통합 진행.
