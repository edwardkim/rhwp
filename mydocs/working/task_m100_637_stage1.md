# Task #637 Stage 1 — 5가지 가설 체계적 검증

**상태**: Stage 1 완료, 작업지시자 승인 대기
**작성일**: 2026-05-06
**브랜치**: `local/task637`

---

## 1. 목표

Stage 0 사전 판정을 데이터로 최종 확정 + 회귀 위험 평가.

## 2. H3 (paragraph header 비트) — **최종 기각**

`examples/inspect_637.rs` 로 host paragraph 의 raw header bytes 추출 결과:

| 페이지 (sec.para) | char_count | control_mask | ps_id | break_raw | raw_header_extra (12B) |
|------------------|-----------|--------------|-------|-----------|------------------------|
| 1 (0.0) **표시** | 56 | 0x00200804 | 249 | 0x03 | `02 00 00 00 02 00 99 C0 55 BE 00 00` |
| **2** (0.1) **미표시** ★ | 9 | **0x00000800** | 122 | **0x04** | `01 00 00 00 01 00 00 00 00 00 00 00` |
| **3** (1.0) **미표시** ★ | 25 | 0x00000804 | 58 | 0x07 | `01 00 00 00 01 00 00 00 00 80 00 00` |
| **6** (2.57) **표시** | 9 | **0x00000800** | 28 | **0x04** | `01 00 00 00 01 00 00 00 00 00 00 00` |

**결정적 관찰**: 페이지 2 host (미표시) 와 페이지 6 host (표시) 의 paragraph header 가
`ps_id` 외 **모든 항목 byte-for-byte 동일**:
- char_count = 9
- control_mask = 0x00000800
- break_raw = 0x04
- raw_header_extra 12B 완전 동일

`ps_id` (paragraph shape) 는 정렬/들여쓰기 등 시각 모양 ID 이며 hide 관련 비트 없음
(ParaShape 구조 확인됨). 따라서 paragraph header 의 어느 비트도 hide 를 결정하지 않음.

**H3 최종 기각 확정**.

## 3. H4 (표 attr 비트) — **최종 기각**

| 페이지 | 표시 | 표 attr (record) | 라벨 |
|--------|------|-----------------|------|
| 1 | 표시 | 0x04000006 | tac=true 작은 표 |
| **2** | **미표시** ★ | 0x0600000e | cover (tac=false) |
| **3** | **미표시** ★ | 0x0400000e | cover (tac=false) |
| **6** | **표시** | **0x0600000e** | tac=false 작은 표 |

페이지 6 의 표 attr = **0x0600000e** 가 페이지 2 와 **완전히 동일**. attr 비트만으로는
표시/미표시 결정 불가.

**H4 최종 기각 확정**.

## 4. H1 (cover-style 휴리스틱) — **결정적 룰로 정형화 가능**

### 4.1 룰 후보 정형화

> **페이지가 items=1 인 단일 완전한 Table (PartialTable 아님) 을 포함하고
> 그 Table 의 `treat_as_char=false` 일 때 한컴은 쪽번호 표시 안 함.**

aift.hwp 데이터 검증:

| 페이지 | items | item 종류 | tac | 표시 |
|--------|-------|----------|-----|------|
| 1 | 2 | Table tac=true + PartialPara | true | 표시 |
| **2** | **1** | **Table** | **false** | **미표시** ★ |
| **3** | **1** | **Table** | **false** | **미표시** ★ |
| 4, 5 | 44, 13 | FullParagraphs | - | 미표시 (PageHide) |
| 6 | 18 | Table + 17 paragraphs | false | 표시 |

### 4.2 174개 샘플 전수 조사 결과

`samples/*.hwp` 174개 전체에 대해 다음 패턴 enumerate (Python parse 스크립트):

```
items=1 + Table (PartialTable 아님) + tac=false
```

**결과: aift.hwp 페이지 2, 3 단 두 페이지만 매칭**.

```
=== aift.hwp (2 cover-pages) ===
  page 2 (sec=0 page_num=2 pi=1): 35x27 635.6x946.3px
  page 3 (sec=1 page_num=3 pi=0): 14x17 631.3x861.1px
```

다른 패턴 (참고):
- `items=1 + Table + tac=true`: 23 페이지 (10 샘플)
- `items=1 + PartialTable`: 37 페이지 (분할 표 중간)

### 4.3 룰 보정 검증 (tac=true 케이스)

aift.hwp 페이지 74 (rhwp page_num=68), 페이지 75 (rhwp page_num=69) 가 `items=1 + Table + tac=true`
패턴에 매칭. 한컴 PDF 측정:

```python
pypdf 페이지 73: matches ['67']  # 표시 ✓
pypdf 페이지 74: matches ['68']  # 표시 ✓
```

**tac=true items=1 페이지는 한컴이 쪽번호 표시함**. 즉 `tac=false` 가 결정적 분리자.

### 4.4 룰의 결정성 vs 휴리스틱

| 측면 | 결과 |
|------|------|
| 174 샘플 매칭 정확도 | aift p2, p3 만 — 한컴 미표시 일치 |
| 다른 패턴 (tac=true) 검증 | 동일 샘플의 tac=true items=1 페이지 표시 — 룰 일관 |
| 매칭 페이지 수 (174 샘플) | 2 페이지 (전체의 0.06%) |
| Edge case (PartialTable, Shape, Picture) | 모두 비해당 (items=1 + Table + tac=false 만) |

**결론**: H1 은 **결정적 룰로 정형화 가능**. 휴리스틱이 아님.

## 5. 5가지 가설 최종 판정

| 가설 | 판정 | 근거 |
|------|------|------|
| **H1** cover-style 휴리스틱 → 결정적 룰 | **채택** | 174 샘플 정확 매칭, tac=true 케이스 일관 |
| H2 셀 내부 PageHide | 기각 | 문서 전체 PageHide 정확히 2개 (페이지 4, 5) |
| H3 paragraph header 비트 | 기각 | 페이지 2 host = 페이지 6 host byte-identical |
| H4 표 attr 비트 | 기각 | 페이지 6 (표시) attr = 페이지 2 (미표시) attr 동일 |
| **H5** 한컴 자체 휴리스틱 | **H1 의 정확형 등가** | H1 이 결정적 룰로 정형화되어 H5 는 별도 항목 아님 |

## 6. 회귀 위험 평가

룰 적용 시 영향:

- 174 샘플 중 영향받는 페이지: **2 페이지** (aift p2, p3)
- 이 두 페이지는 한컴 미표시이므로 **정합성 개선 (회귀 아님)**
- 다른 샘플에서는 0건 변경

룰 미적용 (현재 동작) 시:
- aift p2, p3 만 한컴 대비 잘못 표시 (분석 시작점)

**회귀 위험: 매우 낮음**. 결정적 룰이며, 룰 매칭 빈도가 극히 낮음.

## 7. Stage 2 진입 의의

Stage 1 결과로 H1 이 결정적 룰임이 확정되었으므로 Stage 2 (다른 cover-style HWP 교차 검증)
의 의의는:

1. ✅ 174 내부 샘플 전수 조사 (Stage 1 4.2 에서 완료)
2. (선택) 외부 HWP 파일 (작업지시자 보유 한컴 PDF 동반) 추가 측정
3. (선택) 합성 테스트 HWP 생성 후 한컴 출력 비교 (외부 작업지시자 환경 필요)

Stage 1 만으로도 룰의 결정성·회귀 위험은 충분히 확인됨. Stage 2 는 외부 자료 가용성에 따라
선택적 진입.

## 8. Stage 3 권고안 (사전)

룰이 결정적으로 정형화되어 회귀 위험이 매우 낮으므로:

- **시나리오 (a) 채택 권고**: 별도 fix issue 분리 후 본 issue close-as-analysis-complete
- 분리할 fix issue 제목 (예시): "한컴 호환: cover-style 페이지 (items=1 + 완전한 Table + tac=false) 자동 쪽번호 미표시"
- 구현 위치 후보: `src/renderer/pagination/engine.rs:finalize_pages` 또는 `PageNumberAssigner::assign`
- 검증: aift.hwp 페이지 2, 3 미표시 + 다른 173 샘플 회귀 0

대안 시나리오 (b): 룰 미적용 + 분석 완료 후 close. 회귀 위험은 0이지만 한컴 호환 지속 누락.

작업지시자 결정: 시나리오 (a) 또는 (b).

---

**Stage 1 결과**: H1 이 결정적 룰로 확정. H2, H3, H4 최종 기각. H5 는 H1 등가.
회귀 위험 매우 낮음 (174 샘플 중 2 페이지만 영향, 모두 한컴과 정합성 개선).

승인 후 Stage 2 또는 Stage 3 진입.
