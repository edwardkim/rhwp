# Task #634 Stage 4 — Stage 2 fix revert + 가설 정정

**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/rhwp/issues/634
**진행 시점**: 2026-05-06

---

## 1. 배경

Stage 3 완료 후 작업지시자가 새 한컴 PDF (samples/aift.pdf, **1-up portrait** 으로 갱신)
검증을 통해 가설 H1'' 의 오류 발견.

**작업지시자 보고**: "1 페이지, 6 페이지는 쪽번호 보여져야 함."

새 한컴 PDF 정밀 측정:

| rhwp page | 한컴 footer text op (y<50) | 한컴 표시 |
|-----------|----------------------------|-----------|
| 1 (cover disclaimer "※ 동 사업...") | **3** | **표시** |
| 2 (사업계획서 표지, 35x27 표)         | 0    | 미표시   |
| 3 (요약문, 14x17 표)                | 0    | 미표시   |
| 4 (목차)                            | 0    | 미표시   |
| 5 (별첨 목차)                        | 0    | 미표시   |
| 6 (본문 시작)                        | **3** | **표시** |
| 7+ (NewNumber 발화 후)              | **3** | **표시** |

**Stage 2 fix (NewNumber 게이팅) 결과 vs 한컴**:
- rhwp 페이지 1~6 모두 미표시 (NewNumber 발화 전이라)
- 한컴 페이지 1, 6 표시 → **불일치**

→ **가설 H1'' 잘못됨**. Stage 2 fix revert.

## 2. revert 작업

### 2.1 revert 대상 (Stage 2 src/ 8 파일)

```bash
git checkout f939e84^ -- \
  src/document_core/queries/rendering.rs \
  src/renderer/layout.rs \
  src/renderer/layout/tests.rs \
  src/renderer/page_number.rs \
  src/renderer/pagination.rs \
  src/renderer/pagination/engine.rs \
  src/renderer/pagination/state.rs \
  src/renderer/typeset.rs
```

revert 후 모든 페이지 표시 (Stage 1 이전 동작) 으로 복귀.

### 2.2 Stage 1 통합 테스트 갱신

기존 5건 → 새 8건:

| 테스트 | 상태 | 검증 내용 |
|--------|------|----------|
| `test_634_aift_page1_shows_page_number` | **신규** | aift p1 (PageNumberPos 등록) 표시 |
| `test_634_aift_page4_pagehide_no_page_number` | **신규** | aift p4 PageHide 적용 미표시 |
| `test_634_aift_page5_pagehide_no_page_number` | **신규** | aift p5 PageHide 적용 미표시 |
| `test_634_aift_page6_shows_page_number` | **신규** | aift p6 표시 (회귀 방지) |
| `test_634_aift_page7_shows_page_number` | 유지 | aift p7 NewNumber 발화 후 표시 |
| `test_634_gukrip_page1_pagehide_no_page_number` | **신규** | 국립국어원 p1 PageHide 적용 미표시 |
| `test_634_gukrip_page3_shows_page_number` | 유지 | 국립국어원 p3 표시 |
| `test_634_no_newnumber_doc_shows_page_numbers_from_page1` | 유지 | hwp3-sample p1 표시 |

**제거 (잘못된 expectation):**
- `test_634_aift_page2_no_page_number_before_new_number` — 가설 H1'' 검증용
- `test_634_gukrip_page2_no_page_number_before_new_number` — 가설 H1'' 검증용

## 3. 검증

### 3.1 Task #634 통합 테스트 (8건)

```
test result: ok. 8 passed; 0 failed
```

### 3.2 전체 단위 테스트

```
test result: ok. 1127 passed; 0 failed; 1 ignored
```

기존 1119 + Task #634 신규 8 = 1127. **회귀 0**.

### 3.3 한컴 vs rhwp 일치 매트릭스 (revert 후)

| Page | 한컴 | rhwp | 일치 |
|------|------|------|------|
| 1 (PageNumberPos 등록) | 표시 | **표시** | ✓ |
| 2 (35x27 표 cover) | 미표시 | 표시 | **✗ 미해결** |
| 3 (14x17 표 요약문) | 미표시 | 표시 | **✗ 미해결** |
| 4 (목차) | 미표시 | 미표시 (PageHide) | ✓ |
| 5 (별첨 목차) | 미표시 | 미표시 (PageHide) | ✓ |
| 6 (본문 시작) | 표시 | 표시 | ✓ |
| 7+ (NewNumber 발화 후) | 표시 | 표시 | ✓ |

**페이지 2, 3 미해결 → 별도 issue 분리**.

## 4. 별도 issue 등록

`#637`: 한컴 호환 - aift.hwp 페이지 2, 3 (큰 표만 있는 cover-style) 쪽번호 미표시 메커니즘 분석

URL: https://github.com/edwardkim/rhwp/issues/637

가설 후보:
1. 표가 페이지 전체를 차지하는 cover-style 페이지 자동 미표시 (휴리스틱)
2. 표 셀 내부 paragraph 의 PageHide (rhwp dump 로 확인 시 0개)
3. paragraph header 의 우리가 못 본 비트
4. 표 컨트롤 attr 비트
5. 한컴 자체 휴리스틱

## 5. Task #634 의 결과

**범위 정정**: "한컴 호환 — 쪽번호 표시 동작 검증 + 회귀 방지 테스트". 가설 H1'' 잘못된
정정 시도였으나 revert 후 다음 효용:

1. **검증된 동작**: aift / 국립국어원 7개 케이스 모두 한컴 일치 (페이지 2, 3 제외)
2. **회귀 방지 8건 통합 테스트**: 페이지 1, 4, 5, 6, 7 / gukrip 1, 3 / hwp3 1 → 모두 GREEN
3. **별도 issue 분리**: 페이지 2, 3 미해결 메커니즘 → #637

**closes #634** — 최종 동작 검증 + 회귀 방지 테스트만으로 close.

## 6. 메모리 룰 준수

- **[feedback_pdf_not_authoritative]**: 첫 가설 (H1'') 은 PDF 만 보고 잘못 추정.
  새 PDF (1-up) 측정으로 가설 정정. PDF 측정도 정확한 page 매핑 + content stream 정밀 분석
  필요.
- **[feedback_rule_not_heuristic]**: 가설 H1'' 가 룰 같았으나 실제로는 한컴 동작 미일치.
  잘못된 룰 도출보다 회귀 방지 + 별도 분석이 안전.
- **[feedback_essential_fix_regression_risk]**: Stage 2 fix 가 페이지 1, 6 까지 미표시
  회귀를 만들었음. revert 로 회귀 0 복귀.

## 7. 최종 commit 단위

단일 commit (Stage 4):
```
Task #634 Stage 4: Stage 2 fix revert + 가설 정정 + 회귀 방지 테스트
```

**closes #634** 는 본 commit 메시지에 포함.
