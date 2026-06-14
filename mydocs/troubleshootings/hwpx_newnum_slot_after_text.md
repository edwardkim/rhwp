# HWPX serializer — newNum 슬롯이 텍스트 뒤가 아니라 앞으로 방출 (Task #1407)

## 증상

`143E433F503322BD33.hwpx` roundtrip 후 ir-diff:

```
--- 문단 0.14 --- "김영훈 기자(jcomm@sanggongnews.com)"
  [차이] char_offsets[3]: A=27 vs B=35
```

dump 로는 컨트롤 3개(머리말·Hyperlink 필드·새번호)의 종류·순서·char_shapes 가
원본·RT 동일 → 차이는 **컨트롤의 inline 위치(char_offsets)** 에만 있다.

## 근본 원인 — 메인 루프가 fieldEnd 자리를 newNum 으로 가로챔

문단 0.14 원본 IR:

- text = "김영훈 기자(jcomm@…)" (30 chars)
- char_offsets = `[16,17,18, 27,28,…,53]`
- controls = `[Header(0), Field/Hyperlink(1), NewNumber(2)]`
- field_ranges = `[start=0 end=3 control_idx=1]` (하이퍼링크가 "김영훈" 3글자 래핑)

위치 해석:
- 0~15: Header 슬롯(8) + fieldBegin(8) = 16유닛 → "김영훈" 16,17,18
- "김영훈"(18) 다음 갭 19→27 = **8유닛 = fieldEnd**(field_ranges 유래, controls 에 없음)
- 27부터 " 기자(…)" 텍스트
- newNum 은 원본 XML 상 **텍스트 끝**(pos 54) — `<hp:t>…</hp:t><hp:ctrl><hp:newNum/></hp:ctrl>`

`render_runs` 메인 루프(`section.rs:431~`) 슬롯 방출 조건:

```rust
while slot_idx < slots.len() && char_pos >= expected_utf16_pos.saturating_add(8) {
    render_control_slot(... slots[slot_idx] ...);  // slot_idx 순서대로 소비
}
```

idx=3(" ") 에서 char_pos=27 ≥ expected(16)+8 → **slots[2]=NewNumber 를 방출**.
그러나 그 27 자리에 와야 할 것은 **fieldEnd**(별도 처리, line 525)다. 슬롯 방출(437)
이 fieldEnd 방출(525)보다 먼저라 newNum 이 fieldEnd 자리(27)를 가로채고, 이후 모든
텍스트가 +8 밀려 char_offsets[3] 27→35.

핵심: 메인 루프는 "다음 8유닛 갭이 보이면 controls 순서대로 다음 슬롯 방출" 방식이라,
**텍스트 끝에 위치한 슬롯(newNum)** 과 **텍스트 중간의 fieldEnd 갭**을 구분하지 못한다.
autoNum(#1382)은 placeholder 공백으로 슬롯 위치가 char_offsets 에 명시돼 정확히 잡히나,
newNum 은 placeholder 가 없어(파서 `section.rs:3891` 주석: newNum 은 text/offsets 미 push)
위치 정보가 char_offsets 에 남지 않는다 → controls 배열 순서 + 8유닛 갭 추론에만 의존.

## 해소 (증상 ①) — post-char fieldEnd 에 expected+=8

실측 추적 결과 가로채기는 1단계 가설(pre-char 가드)보다 한 idx 앞이었다. post-char
fieldEnd 방출(`section.rs:525~`)이 `expected_utf16_pos` 를 +8 진행하지 않아, 다음 idx
에서 텍스트-끝 슬롯(newNum)이 그 8유닛 갭을 차지했다. **fieldEnd 방출 직후 `expected
+= 8`** 1줄로 정정 → newNum 은 남은 슬롯 일괄 방출로 텍스트 끝 배치. 143E ir-diff 0.

## RT 페이지 수 1→2 (증상 ②) — 별개 원인: 본문 colPr 템플릿 하드코딩

슬롯 정정 후에도 페이지 수는 1→2 그대로. `dump-pages` 대조로 **2단(컬럼) 손실**이
원인임을 확정:

- 원본: `colCount="2" sameGap="2268"`(2단) / RT: `colCount="1" sameGap="0"`(단일 단)
- 본문(depth 0) ColumnDef 가 `render_runs` 인라인 슬롯에서 제외되고(#1379), 템플릿
  `empty_section0.xml` 의 하드코딩 colPr(colCount=1)만 방출 → 2단→1단 → 페이지 넘침.
- **#1388(secPr 여백 템플릿 하드코딩) 동형**. newNum 슬롯과 별개 결함.

해소: `write_section` 에서 첫 문단 ColumnDef IR 을 템플릿 colPr anchor 에 치환
(`render_col_pr_ctrl` 재사용). RT colCount=2 복원, 페이지 1→1.

## 게이트 사각

143E 는 ir-diff·baseline IR diff 가 0인데도 페이지 수가 달랐다. 본문 colPr **방출 손실**
은 재파싱 시 colCount=1 로 읽혀 양쪽 IR 이 같아 보이는 사각이었다(IR→XML 손실). 방출
정정으로 표면화 종결. 다른 본문 템플릿 하드코딩(secPr=#1388, colPr=#1407) 잔여 점검 필요.

## 재발 방지 체크리스트

- [x] placeholder 없는 inline 슬롯(newNum 등)이 **텍스트 끝**에 올 때, 메인 루프가
      텍스트 중간 갭(fieldEnd)으로 가로채지 않는지 — post-char fieldEnd expected+=8 로 봉인
- [x] field_ranges(fieldEnd) 갭과 controls 슬롯의 우선순위 — fieldEnd 가 expected 를
      진행시켜 슬롯 가로채기 차단
- [x] 본문(depth 0) 섹션 colPr 이 IR ColumnDef 로 방출되는지 (템플릿 하드코딩 금지) —
      #1388(secPr)·#1407(colPr) 동형 패턴, 향후 본문 템플릿 고정값 추가 시 IR 치환 점검

## 관련

- Task #1407 (본 건), #1382(autoNum placeholder 해소 — 별개 계열), #1379/#1380(RT 인프라)
- `mydocs/report/task_m100_1382_report.md` 4절 (시대별 RT 대조 실증)
