---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1584/README.md
last_verified: 2026-07-16
---

# HWPX 잔존 IR_DIFF 10건 분석 (fidelity10, #1584 이후)

- 일자: 2026-06-27
- 바이너리: `local/task1584` HEAD (#1584 ColumnDef 수정 반영)
- 말뭉치: hwpdocs 9660 hwpx → IR_DIFF 10 (PARSE_FAIL 12 별개)
- **검증**: 10건 전부 #1584 수정 전/후 diff 성격 **완전 동일** → ColumnDef 수정과 무관한 선존 잔여.

## 분류 요약

| 클래스 | 건수 | 성격 | 시각 영향 | 수정 난이도 |
|--------|----:|------|----------|-----------|
| **A. Ruby 직렬화기 부재** | 3 | 실버그 | **있음**(루비 주음 소실) | 중 |
| **B. write_line shapeComment 누락** | 3 | 경미 | 거의 없음(설명 메타데이터) | **하**(1줄) |
| **C. para0 char_shape 경계 오정렬** | 3 | 조사 필요 | 가능성 있음 | 미상 |
| **D. 빈/공백 문단 spurious (0,0)** | 1 | 경미 | 없음 | 하~중 |

---

## A. Ruby 컨트롤 드롭 (3건: 36384160, 36399208, 36389301)

**근본원인**: `render_control_slot`(section.rs)에 **`Control::Ruby` arm 부재**.
Ruby 는 `is_hwpx_inline_slot`(line 749)에 등록돼 슬롯으로 인식되지만, 방출 dispatch 에서
대응 arm 이 없어 `_ => {}` 로 빠져 **XML 미방출 → 드롭**. (ColumnDef 와 동형 — 인식되나 미방출.)

```
36384160 sec2: paragraph[88]cell·[118]·[179](ruby×2)·[181]  ruby 드롭
36399208 sec2: paragraph[72]  ruby 드롭
36389301 sec0: paragraph[6]  ruby 드롭 → char_shapes (51,8)→(43,8) −8 시프트(하위 증상)
```

- **36389301 의 char_shape −8 시프트는 ruby 드롭의 하위 증상** (8유닛 슬롯 소실 → 후속 경계 −8).
  ColumnDef 와 동일 패턴 — 컨트롤 드롭이 인덱스/오프셋을 밀어 char_shape 를 변위.
- 영향: 루비(한자 독음·위첨자) 텍스트가 사라짐 → 실제 시각 차이.
- 수정 방향: `write_ruby` 직렬화기 + `render_control_slot` 에 `Control::Ruby` arm 추가.
  파서(`parse_hwpx`)의 ruby 역매핑 확인 필요.

## B. write_line shapeComment 누락 (3건: 36389418, 36392900, 36391302)

**근본원인**: `shape.rs`의 `write_line`(line 121)이 **`write_shape_comment` 미호출**.
`write_rect`(line 110)·`write_container_close`(line 235)는 호출하나 선 도형만 누락.
→ 선 도형 설명("선입니다.")이 저장 시 드롭.

```
36389418 sec0 p18 cell shape ×6,  36392900 sec0 p19 cell shape ×11,  36391302 ×2
```

- 영향: 도형 설명(접근성/메타데이터, 화면 비표시) 소실 — 시각 영향 거의 없음.
- 수정 방향: `write_line` 의 caption 방출 뒤 `write_shape_comment(w, c)?;` 1줄 추가
  (#1392/#1403 도형 설명 보존과 정합). **가장 간단**.

## C. para0 char_shape 경계 오정렬 (3건: 36384689, 36385445, 36388711)

필드(fieldBegin/End)·루비와 **무관**. 섹션 첫 문단(secPr+colPr+표 영역)의 char_shape 경계가
control-slot 공간에서 어긋남.

```
36384689 p0 "문서번호": (24,10)→(32,10)  +8   [ctrl,tbl,tbl]
36385445 p0 "문서번호": (24,15)→(32,15)  +8   [ctrl,tbl,tbl]   ← 동일 패턴(체계적)
36388711 p0 (로고/표): (52,8)(81,9)→(36,8)(73,9)  −16/−8  [secPr,colPr,tbl, line×3]
```

### 후속 규명 (#1591/#1593 조사 결과)

| 파일 | 진짜 근본 | 처리 |
|------|----------|------|
| 36384689·36385445 (C1) | **first-para mismatch-path 위치추정**: para0 가 #1584 ColumnDef 템플릿 흡수로 `slot_count`(cc기반)≠`slots.len()`→mismatch 경로가 char_shape +8 오배치. 표면 증상이던 북마크 hoist 는 무관(수정해도 +8 불변) | #1591 재범위, **미해결(F3급)** |
| 36388711 (C2) | **same-para fieldEnd 드롭(cc −8) + 북마크 hoist 결합**. fieldBegin/End 1/1→1/0. F3(#1561 cross-para) 와 다른 same-para 변종 | #1593, **보류** |

→ **잔여 3건 모두 first-para mismatch-path 슬롯 위치추정으로 수렴**. F3(#1561, 2회 실패)·
#1591(순효과0 롤백)과 동질의 고위험 영역. 개별 파일 수정 대신 **mismatch-path 슬롯 위치추정
통합 리팩터**로 묶어 처리 권고(광역 통제비교 필수, 악화 즉시 롤백).

## D. 빈/공백 문단 spurious char_shape (1건: 36386761)

```
36386761 sec0 p5 " 수신 ": expected=[] actual=[(0,0)]
```

- 원본에 char_shape 없는 공백 문단에 저장→재파싱 시 기본 char_shape (0,0) 가 생성됨.
- 영향: 없음(기본 글자모양). 경미.
- **해결(#1592, 채택)**: render_runs 가 완전 빈 문단(char_shapes=[])에 run 미방출. 통제비교
  개선1/회귀0. (RunSplitter::new 규칙3 + close_run 규칙5 가 빈 run 을 가공하던 것을 차단.)

---

## 처리 결과 (2026-06-27)

| Class | 이슈 | 결과 |
|-------|------|------|
| A (Ruby 드롭) | #1587 | **채택** (개선3) |
| B (선 도형 shapeComment) | #1588 | **채택** (개선3) |
| C1 (para0 mismatch-path) | #1591 | **불채택**(북마크 hoist 순효과0) → mismatch-path 재범위, 미해결 |
| C2 (fieldEnd 드롭+북마크 결합) | #1593 | **보류** (통합 리팩터 권고) |
| D (spurious 0,0) | #1592 | **채택** (개선1) |

**누적: HWPX 실문서 IR_DIFF 59→3** (#1584 ColumnDef 포함 4 채택). 잔여 3건(C1 2 + C2 1)은
모두 **first-para mismatch-path 슬롯 위치추정**으로 수렴 → F3(#1561)·#1591 동질 고위험.
**통합 리팩터 권고**(개별 수정 금지, 광역 통제비교 필수).

> PARSE_FAIL 12건 = "ZIP EOCD 없음" 손상 다운로드(수집기 아티팩트, rhwp 무관).
