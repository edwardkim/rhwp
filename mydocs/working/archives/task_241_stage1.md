# Task 241 - 1단계 완료 보고서: FIELD_BOOKMARK 실태 조사 및 한컴 책갈피 출처 규명

## 핵심 발견

### 1. synam-001.hwp 바이너리 분석 결과

| 항목 | 수 |
|------|---|
| `%bmk` (FIELD_BOOKMARK) ctrl_id | **0개** |
| `bokm` (CTRL_BOOKMARK) in CTRL_HEADER | **0개** |
| `bokm` in PARA_TEXT (char code 22) | **10개** (20개 바이트 시퀀스 중 10개는 CTRL_DATA) |
| CTRL_HEADER 총 수 | 114개 (모두 $pic, $rec, $con 등) |

### 2. 책갈피 저장 메커니즘 규명

HWP 5.0에서 책갈피는 **두 가지 다른 방식**으로 저장됨:

#### 방식 A: CTRL_HEADER + CTRL_DATA (우리가 파싱하는 방식)
- CTRL_HEADER 레코드에 ctrl_id=bokm
- 뒤따르는 CTRL_DATA에 ParameterSet → 이름
- **synam-001.hwp에는 이 방식 0개**

#### 방식 B: PARA_TEXT inline (char code 22) + CTRL_DATA
- PARA_TEXT 안에 char code 0x0016 (22) = 16바이트 extended char
- additional 12바이트의 처음 4바이트 = ctrl_id (bokm)
- 별도 CTRL_HEADER 레코드 **없음**
- 같은 문단의 CTRL_DATA에 이름 저장
- **synam-001.hwp에는 이 방식 10개**

### 3. 우리 파서의 문제점

1. `is_extended_only_ctrl_char(22)` = true → `ctrl_idx += 1` 실행됨
2. 하지만 대응하는 CTRL_HEADER 레코드가 없음
3. ctrl_idx가 어긋나서 잘못된 CTRL_HEADER의 데이터로 Bookmark를 생성
4. 우연히 1개가 매칭된 것 (para=247은 본문 최상위 문단)

### 4. hwplib도 CTRL_HEADER 기반

hwplib의 `ForParagraph.control()`도 CTRL_HEADER 레코드 기반으로 파싱.
char code 22의 inline 전용 책갈피를 hwplib이 어떻게 처리하는지 추가 확인 필요.
→ hwplib도 CTRL_HEADER 없는 inline 책갈피를 별도 처리할 가능성 있음

### 5. FIELD_BOOKMARK(%bmk) 부재

synam-001.hwp에는 `%bmk` 시그니처 자체가 바이너리에 없음.
한컴 "찾아가기 > 책갈피" 목록은 모두 **char code 22 기반 CTRL_BOOKMARK(bokm)**에서 옴.

## 다음 단계에 필요한 작업

1. char code 22 전용 책갈피 파싱 경로 추가 (PARA_TEXT inline → Bookmark 컨트롤 생성)
2. 해당 문단의 CTRL_DATA에서 이름 추출
3. `ctrl_idx` 계산에서 char 22의 CTRL_HEADER 부재 보정
