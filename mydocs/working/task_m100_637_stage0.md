# Task #637 Stage 0 — 사전 데이터 수집

**상태**: Stage 0 완료, 작업지시자 승인 대기
**작성일**: 2026-05-06
**브랜치**: `local/task637`

---

## 1. 목표

5가지 가설 (H1 cover-style 휴리스틱 / H2 셀 내부 PageHide / H3 paragraph header 비트 /
H4 표 attr 비트 / H5 한컴 휴리스틱) 검증에 필요한 raw 데이터 수집.

## 2. 페이지 구조 측정 (`rhwp dump-pages`)

| rhwp 페이지 | 섹션 | page_num | items | body 사용 (px) | 페이지 구성 | 한컴 PDF |
|------------|------|----------|-------|--------------|------------|---------|
| 1 | 0 | 1 | **2** | 221.3 / 971.4 (23%) | Table 1×1 tac=true (205.8px) + PartialParagraph pi=0 line 1..2 | 표시 "- 1 -" ✓ |
| **2** | 0 | 2 | **1** | 954.7 / 971.4 (98%) | **Table 35×27 tac=false (946.3px) 단독** | **미표시** ✗ |
| **3** | 1 | 3 | **1** | 866.6 / 971.4 (89%) | **Table 14×17 tac=false (861.1px) 단독** | **미표시** ✗ |
| 4 | 2 | 1 | 44 | 929.5 | 목차 paragraphs (PageHide on 2.34) | 미표시 (PageHide) |
| 5 | 2 | 2 | 13 | 261.2 | 별첨 목차 (PageHide on 2.54) | 미표시 (PageHide) |
| 6 | 2 | 3 | **18** | 823.4 / 971.4 | Table 6×6 (82.7px=8%) + 17 FullParagraph | 표시 "- 4 -" ✓ |

**핵심 차이**: 페이지 2, 3 은 **items=1, 표가 본문 영역의 89~98% 차지**. 페이지 6 은
items=18, 표는 8% 만 차지하고 나머지는 paragraph.

## 3. host paragraph 비교 (`rhwp dump`)

### 3.1 페이지 1 (sec0 p0, **표시**)

```
--- 문단 0.0 --- cc=56, text_len=23, controls=4 [구역나누기]
  텍스트: "  * 사업계획서 제출 시 상기 문구 삭제"
  [PS] ps_id=249 align=Justify line=145/Percent
  [0] 구역정의
  [1] 단정의
  [2] 표: 1×1, attr=0x04000006, tac=true, wrap=위아래, size=168×54mm
       raw=[11, 22, 2A, 08, ...]
  [3] 쪽번호위치: format=0, pos=5
```

### 3.2 페이지 2 (sec0 p1, **미표시**)

```
--- 문단 0.1 --- cc=9, text_len=0, controls=1 [쪽나누기]
  텍스트: (빈 문단)
  [PS] ps_id=122 align=Center line=145/Percent
  [0] 표: 35×27, attr=0x0600000e, tac=false, wrap=위아래, size=168×250mm (수직 거의 가득)
       raw=[10, 23, 2A, 08, ...]
```

### 3.3 페이지 3 (sec1 p0, **미표시**)

```
--- 문단 1.0 --- cc=25, text_len=0, controls=3 [쪽나누기]
  텍스트: (빈 문단)
  [PS] ps_id=58 align=Justify line=140/Percent
  [0] 단정의
  [1] 구역정의
  [2] 표: 14×17, attr=0x0400000e, tac=false, wrap=어울림, size=167×228mm
       raw=[10, 23, 0A, 08, ...]
```

### 3.4 페이지 6 (sec2 p57, **표시**)

```
--- 문단 2.57 --- cc=9, text_len=0, controls=1 [쪽나누기]
  텍스트: (빈 문단)
  [PS] ps_id=28 align=Center line=110/Percent
  [0] 표: 6×6, attr=0x0600000e, tac=false, wrap=위아래, size=168×22mm (수직 8%)
       raw=??
```

## 4. PageHide 컨트롤 (전체 문서)

```bash
$ ./target/release/rhwp dump samples/aift.hwp 2>&1 | grep "감추기"
[0] 감추기: header=false, footer=false, master=false, border=false, fill=false, page_num=true   # para 2.34 (페이지 4)
[0] 감추기: header=false, footer=false, master=false, border=false, fill=false, page_num=true   # para 2.54 (페이지 5)
```

**문서 전체에 PageHide 는 정확히 2개**. 페이지 2, 3 에는 PageHide 없음.

## 5. 셀 내부 paragraph 컨트롤 스캔 (페이지 2, 3 표)

페이지 2 표 (35×27, 168 셀) 의 모든 셀 paragraphs scanning 결과: 셀 내부 paragraph 중
`ctrls=1` 인 것이 2개 발견. 그러나 dump 형식상 cell-내부 control 의 종류가 직접 보이지 않음.
**전체 PageHide 가 정확히 2개 (4, 5 페이지) 임이 별도로 확인되었으므로 셀 내부 control 은
확실히 다른 종류** (NewLine, FieldStart 등 inline char control 가능성).

## 6. 한컴 PDF 측정 (pypdf)

```python
페이지 1: page-number-pattern 매치 = ['1']     # 표시
페이지 2: page-number-pattern 매치 = []        # 미표시 ✓
페이지 3: page-number-pattern 매치 = []        # 미표시 ✓
페이지 4: page-number-pattern 매치 = []        # 미표시 (PageHide)
페이지 5: page-number-pattern 매치 = []        # 미표시 (PageHide)
페이지 6: page-number-pattern 매치 = ['4']     # 표시
페이지 7: page-number-pattern 매치 = ['1']     # 표시 (NewNumber on para 2.79)
```

페이지 2, 3 미표시 확정 + 페이지 1, 6, 7 표시 확정.

## 7. 표 attr 비트 비교 매트릭스

| 페이지 | 표시 | 표 attr (table record) | 표 raw common 첫 4B | wrap | tac |
|--------|------|----------------------|--------------------|------|-----|
| 1 | 표시 | **0x04000006** | 0x082A2211 | TopAndBottom | **true** |
| 2 | **미표시** | **0x0600000e** | 0x082A2310 | TopAndBottom | false |
| 3 | **미표시** | **0x0400000e** | 0x080A2310 | Square | false |
| 6 | 표시 | **0x0600000e** | (raw 미수집) | TopAndBottom | false |

**결정적 관찰**: 페이지 6 (**표시**) 의 표 attr = 0x0600000e 가 페이지 2 (**미표시**) 와
**완전히 동일**. 그리고 페이지 2 (0x0600000e) vs 페이지 3 (0x0400000e) 은 **둘 다 미표시**
임에도 attr 가 다름. 즉 attr 비트만으로는 표시/미표시를 구분할 수 없음.

## 8. paragraph header 비교 (page 2 vs page 6)

| 항목 | 페이지 2 (sec0 p1) 미표시 | 페이지 6 (sec2 p57) 표시 |
|------|------------------------|------------------------|
| char_count (cc) | 9 | 9 |
| text_len | 0 | 0 |
| controls 수 | 1 | 1 |
| break_type | 쪽나누기 | 쪽나누기 |
| para_shape_id | 122 | 28 |
| 컨트롤 종류 | 표 | 표 |
| 표 wrap | 위아래 | 위아래 |
| 표 tac | false | false |

**결론**: paragraph header 자체는 페이지 2 와 페이지 6 이 **거의 동일** (ps_id 만 다름).
ps_id 는 들여쓰기/정렬 등 단순 모양 ID 라 hide 와 무관. paragraph header 에 hide 비트가
있다면 페이지 2 와 페이지 6 의 cc, control_mask 등이 달라야 하나 동일.

## 9. 가설 사전 판정 (Stage 0 시점)

| 가설 | 판정 (잠정) | 근거 |
|------|-----------|------|
| H1 cover-style 휴리스틱 (단일 큰 표 = 표지) | **유력** | 페이지 2/3 만 items=1 + 표 89~98% |
| H2 셀 내부 PageHide | **기각** | 문서 전체 PageHide 2개 (페이지 4, 5) |
| H3 paragraph header 비트 | **기각** | 페이지 2 host 와 페이지 6 host 거의 동일 |
| H4 표 attr 비트 | **기각** | 페이지 6 표 attr = 페이지 2 표 attr (0x0600000e) |
| H5 한컴 자체 휴리스틱 | H1 의 변형 | 만약 H1 이 결정적 룰이면 H5 와 등가 |

## 10. Stage 1 진입 시 추가 검증 항목

- 페이지 6 표의 raw common 4B 와 페이지 2/3 의 raw common 4B 정확 비교
- 페이지 6 paragraph 2.57 의 paragraph header raw 바이트 확인 (hide 비트 가능성 최종 확인)
- "items=1 + 표 비율 X%" 의 정확한 임계값 결정 (cover-style 룰의 결정성 평가)
- aift.hwp 외 다른 HWP 에서 같은 패턴 (큰 표 단독 페이지 + 한컴 미표시) 발견 가능성 (Stage 2)
- **회귀 위험 평가**: rhwp 다른 샘플 (synam, exam_*) 에서 cover-style 룰 적용 시 잘못 미표시되는
  케이스가 있는지

---

**Stage 0 결과**: H2, H3, H4 사전 기각. **H1 유력, H5 (휴리스틱) 가능성**.
Stage 1 에서 H1 의 결정적 룰 가능성 정밀 측정 + 회귀 위험 평가.

승인 후 Stage 1 진입.
