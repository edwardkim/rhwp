---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 트러블슈팅: HWP 확장 제어 문자 바이트 구조

## 문제 상황

HWP 5.0 스펙 문서의 "표 6: 문단 텍스트" 제어 문자 정의를 해석하는 과정에서 바이트 크기 혼란 발생.

### 스펙 문서 내용 (표 6)

| 문자 코드 | 종류 | 크기 | 설명 |
|-----------|------|------|------|
| 0-1 | 문자 컨트롤 | 1 | 사용 안함, 예약 |
| 2-3 | 확장 컨트롤 | 8 | 확장 컨트롤 |
| 4-9 | 인라인 컨트롤 | 8 | |
| 10 | 확장 컨트롤 | 8 | |
| 11-12 | 확장 컨트롤 | 8 | 표, 그리기 객체 |
| ... | ... | ... | ... |

**문제**: 스펙에서 "크기 = 8"이라고 명시되어 있어 8바이트로 해석했으나, 실제 파싱 시 오류 발생.

---

## 분석 과정

### 1. hwplib (Java) 코드 분석

[ForParaText.java](https://github.com/neolord0/hwplib/blob/main/src/main/java/kr/dogfoot/hwplib/reader/bodytext/paragraph/ForParaText.java):

```java
case ControlExtend:
    extendChar(paraText.addNewExtendControlChar(), sr);
    return 16;  // ← 16바이트 반환!

private static void extendChar(HWPCharControlExtend extendChar,
                               StreamReader sr) throws Exception {
    byte[] addition = new byte[12];  // ← 12바이트 addition
    sr.readBytes(addition);
    extendChar.setAddition(addition);
    extendChar.setCode(sr.readSInt2());  // ← 2바이트 코드
}
```

### 2. HWPCharControlExtend.java 분석

```java
public void setAddition(byte[] addition) throws Exception {
    if (addition.length != 12) {  // ← 반드시 12바이트
        throw new Exception("addition's length must be 12");
    }
    this.addition = addition;
}
```

### 3. 실제 HWP 파일 Hex Dump 분석

통합재정통계(2010.1월-6월).hwp 파일의 PARA_TEXT 레코드:

```
0b00 206c627400000000000000000b00 0d00
└─2B─┘└───────── 12B ──────────┘└2B┘└다음┘
코드    addition (ctrl_type+extra) 코드  문자
```

분해:
- `0b 00` = 0x000B (코드 11, 표/그리기 객체)
- `20 6c 62 74` = ' lbt' (little-endian → 'tbl ': 표)
- `00 00 00 00 00 00 00 00` = 8바이트 추가 정보
- `0b 00` = 0x000B (종료 코드)
- `0d 00` = 0x000D (다음 문자: 문단 끝)

---

## 결론: 스펙 해석

### 스펙의 "크기 = 8"의 의미

**8 WCHAR (Wide Character) 단위 = 8 × 2바이트 = 16바이트**

HWP 5.0은 유니코드 기반으로 모든 문자가 2바이트(WCHAR) 단위.
따라서 스펙의 "크기" 컬럼은 바이트가 아닌 WCHAR 단위.

### 확장 제어 문자 실제 구조 (16바이트)

| 오프셋 | 크기 | 내용 |
|--------|------|------|
| 0-1 | 2바이트 | 제어 문자 코드 |
| 2-5 | 4바이트 | 컨트롤 타입 ('tbl ', 'gso ', 'eqed' 등) |
| 6-13 | 8바이트 | 추가 정보 (instance id 등) |
| 14-15 | 2바이트 | 제어 문자 코드 (반복) |

### 구현 코드 (수정 후)

작성 당시 외부 로컬 분석 스크립트 `hwp_semantic/record_parser.py`:

```python
class CtrlChar:
    """
    HWP 제어 문자

    확장 제어 문자 구조 (16 bytes):
    - 코드 (2 bytes)
    - addition (12 bytes): ctrl_type[0:4] + extra[4:12]
    - 코드 (2 bytes)

    처음 코드 2바이트를 읽은 후 추가로 14바이트 스킵 필요
    """
    EXTENDED_EXTRA_BYTES = 14     # addition(12) + 마지막 코드(2)
```

---

## 컨트롤 타입 식별자

addition[0:4]에 저장된 4바이트 ASCII 문자열 (little-endian):

| 저장값 | 실제 의미 | 코드 |
|--------|-----------|------|
| ' lbt' | 'tbl ' | 표 (Table) |
| ' osg' | 'gso ' | 그리기 객체 (GSO) |
| 'deqe' | 'eqed' | 수식 (Equation) |
| 'mrof' | 'form' | 폼 컨트롤 |
| 'dces' | 'secd' | 구역 정의 |
| 'dloc' | 'cold' | 단 정의 |
| 'klh%' | '%hlk' | 하이퍼링크 |

---

## 교훈

1. HWP 스펙의 "크기"는 **WCHAR 단위** (바이트가 아님)
2. 확장 제어 문자 = 16바이트 = 8 WCHAR
3. hwplib Java 코드가 실제 구현의 정답 (16바이트 반환)
4. 스펙 문서만으로는 바이트 레벨 해석이 어려움 → 실제 파일 hex dump 확인 필수

---

## 관련 파일

- [HWP 5.0 스펙](../tech/한글문서파일형식_5.0_revision1.3.md) - 표 6: 문단 텍스트
- [hwplib ForParaText.java](https://github.com/neolord0/hwplib/blob/main/src/main/java/kr/dogfoot/hwplib/reader/bodytext/paragraph/ForParaText.java)
- [hwplib HWPCharControlExtend.java](https://github.com/neolord0/hwplib/blob/main/src/main/java/kr/dogfoot/hwplib/object/bodytext/paragraph/text/HWPCharControlExtend.java)
- 작성 당시 외부 로컬 `hwp_semantic/record_parser.py` - CtrlChar 클래스

---

**작성일**: 2026-01-06
**관련 타스크**: Task 56 - 렌더링 순서 기반 문서 트리 구현
