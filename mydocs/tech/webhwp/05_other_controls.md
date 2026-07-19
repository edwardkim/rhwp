---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 기타 컨트롤 (수식, 하이퍼링크, OLE 등)

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 분석 일자: 2026-02-09

## 1. 수식 (EQEDIT, wt.Z.Tfi)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Tfi` |
| 리소스 이름 | `IDS_CTRL_NAME_EQEDIT` |
| 참조 횟수 | 62 |
| 요소 유형 | `U.default.hri` (컨테이너) |
| 플래그 | 0 |

### JSON 임포트 (Qbn)

```javascript
// 수식 속성
e.Xbn(h.sc)          // 크기 설정
e.qbn = h.lm ? 1 : 0 // 줄 모드 토글
e.Jbn(h.bl)          // 베이스라인 설정
e.pSi(h.tc)          // 텍스트 색상
e.Zbn(h.bu)          // 버퍼/내용
```

### 데이터 구조

```javascript
{
    sc: scale,       // 크기 비율
    lm: lineMode,    // 줄 모드 (boolean)
    bl: baseline,    // 베이스라인
    tc: textColor,   // 텍스트 색상
    bu: buffer,      // 수식 내용 버퍼
    ve: version      // 버전
}
```

### 플러그인 통합

- 외부 수식 편집기 플러그인 기반 아키텍처
- LaTeX 스타일 구문 파싱 (`Plugin.kGr` 모듈)
- Canvas/SVG 렌더링 지원

## 2. 북마크 (BOOKMARK, wt.Z.Dli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Dli` |
| 리소스 이름 | `IDS_CTRL_NAME_BOOKMARK` |
| 참조 횟수 | 25 |
| 요소 유형 | `U.default.Ali` (하이퍼링크) |
| 플래그 | `Vt.Z.ffi.Qfi` (북마크 전용) |

### JSON 임포트 (nEn)

```javascript
t.Pbn(i, n, e);                // 공통 속성 초기화
e.bKi = gt.YHt(vt.g4);        // 북마크 마커 할당
```

### 렌더링 스타일

```css
/* CSS 클래스 */
.hcwo_hyperlink          /* 하이퍼링크 스타일 */
.hcwo_hand_pointer        /* 손가락 커서 */
.hcwo_disable_open        /* 열기 비활성화 */
```

- 밑줄 + 파란색 텍스트 (표준 하이퍼링크)
- 클릭 시 문서 내 북마크 대상으로 네비게이션

## 3. 필드 (FIELD_END, wt.Z.Wfi)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Wfi` |
| 리소스 이름 | `IDS_CTRL_NAME_FIELD_END` |
| 참조 횟수 | 25 |
| 요소 유형 | `U.default.SOt` (필드) |
| 플래그 | 0 |

### 필드 속성

```javascript
{
    cc: fieldCode,     // 필드 코드
    ci: controlId,     // 컨트롤 ID
    fo: fieldOffset,   // 필드 오프셋
    zo: zeroOffset,    // 제로 오프셋
    ed: editMode,      // 편집 모드
    fi: fieldId,       // 필드 ID
    fb: fieldBegin,    // 필드 시작
    fk: fieldKey,      // 필드 키
    sk: searchKey      // 검색 키
}
```

### 필드 코드 ↔ 결과 토글

- 필드 코드 표시와 필드 결과 표시 전환 지원
- 괄호 표기: `"[" + u + o + "]"` 형태로 표시
- 문서 상태에 따라 동적 업데이트

## 4. 자동 번호 (AUTO_NUM, wt.Z.POt)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.POt` |
| 리소스 이름 | `IDS_CTRL_NAME_AUTO_NUM` |
| 참조 횟수 | 37 |
| 요소 유형 | `U.default.NOt` (필드) |

### JSON 임포트 (SIn)

```javascript
e.dUt = 0;
e.lIn = e.cIn = e.dIn = 0;
t.Pbn(i, n, e);

var s = n.an;
if (null != s) {
    e.A0i(s.nu);           // 번호 단위
    e.DIn(s.nt);           // 번호 유형
    e.AMi(s.at);           // 자동 유형
    e.lIn = s.au.charCodeAt(0);  // 대문자 마커
    e.cIn = s.ap.charCodeAt(0);  // 접두사 문자
    e.dIn = s.ac.charCodeAt(0);  // 닫기 문자
    e.KTi(s.as);           // 구분자
}
```

### 번호 매기기 속성

| JSON 키 | 속성 | 설명 |
|---------|------|------|
| `nu` | 번호 단위 | 열거 유형 |
| `nt` | 번호 유형 | 로마/아라비아/사용자 정의 |
| `at` | 자동 유형 | 글머리 기호/번호 |
| `au` | 대문자 마커 | 접미 마커 |
| `ap` | 접두사 | 접두 문자 |
| `ac` | 닫기 문자 | 닫기 문자 |
| `as` | 구분자 | 구분 문자 |

### 저장 구조

```javascript
wqt[7]  // 7단계 번호 매기기 형식 저장 배열
// wqt[0]~wqt[6]: 각 단계별 형식
// RSi(t): 모든 단계에 동일 값 설정
```

## 5. 새 번호 (NEW_NUM, wt.Z.UOt)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.UOt` |
| 리소스 이름 | `IDS_CTRL_NAME_NEW_NUM` |
| 참조 횟수 | 28 |
| 요소 유형 | `U.default.FOt` (필드) |

### JSON 임포트 (AIn)

```javascript
e.dUt = 0;
t.Pbn(i, n, e);
var r = n.nn;
if (null != r) {
    e.RIn(r.nu);   // 재시작 번호 값
    e.DIn(r.nt);   // 번호 유형
}
```

### AUTO_NUM과의 차이

| 항목 | AUTO_NUM | NEW_NUM |
|------|----------|---------|
| 속성 수 | 많음 (접두사/접미사/구분자 포함) | 최소 (번호 값 + 유형만) |
| 용도 | 자동 번호 매기기 | 번호 재시작 지점 |
| 접두사/접미사 | 지원 | 미지원 |
| 목록 계층 | 유지 | 리셋 |

## 6. 덧말 (DUTMAL, wt.Z.Nli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Nli` |
| 리소스 이름 | `IDS_CTRL_NAME_DUTMAL` |
| 참조 횟수 | 26 |
| 요소 유형 | `U.default.kli` (주석) |

### JSON 임포트 (cEn)

```javascript
e.dEn = e.vEn = e._En = 0;
e.pEn(r.pt);    // 위치 (위/아래)
e.wEn(r.sr);    // 크기 비율 (루비/본문 비)
e.mEn(r.op);    // 투명도
```

### 덧말 속성

| JSON 키 | 메서드 | 설명 |
|---------|--------|------|
| `pt` | `pEn()` | 위치 (위/아래) |
| `sr` | `wEn()` | 크기 비율 |
| `op` | `mEn()` | 투명도 |
| `si` | `bEn()` | 문자 스타일 참조 |
| `al` | `yEn()` | 수평 정렬 (좌/중/우) |
| `mt` | `IEn` | 위쪽 여백 |
| `st` | `EEn` | 아래쪽 여백 |

### 타이포그래피

- 본문 위/아래에 작은 주석 텍스트 (후리가나 스타일)
- 연결된 스타일 ID에서 문자 스타일 상속
- 여백 조절로 간격 조정
- 투명도 제어 가능

## 7. 주석 (COMMENT, wt.Z.xli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.xli` |
| 리소스 이름 | `IDS_CTRL_NAME_COMMENT` |
| 참조 횟수 | 18 |
| 요소 유형 | `U.default.kli` (주석) |

### 렌더링

- Canvas 기반 렌더링 (`W7i()`)
- 범위/텍스트 선택 하이라이팅
- 여백 노트 스타일 표시

### CSS 클래스

```
hcwo_comment_wrap   — 주석 래퍼
hcwo_comment_block  — 주석 블록
```

## 8. 찾아보기 표시 (INDEXMARK, wt.Z.Rli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Rli` |
| 리소스 이름 | `IDS_CTRL_NAME_INDEXMARK` |

### JSON 임포트 (CEn)

```javascript
e.gEn.YAt(r.fk);   // 1차 키 (주 항목)
e.TEn.YAt(r.sk);   // 2차 키 (하위 항목)
```

- 2단계 계층적 색인 지원 (주 항목 + 하위 항목)
- 렌더링된 문서에서는 보이지 않음 (마커 전용)
- 문서 색인/목차 자동 생성용

## 9. 글자 겹침 (COMPOSE, wt.Z.Oli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Oli` |
| 리소스 이름 | `IDS_CTRL_NAME_COMPOSE` |
| 참조 횟수 | 20 |
| 요소 유형 | `U.default.kli` (주석) |

### JSON 임포트 (sEn)

```javascript
e.hEn = u.c1;       // 첫 번째 구성 문자
e.uEn = u.cs;       // 문자 세트/변형 참조
e.oEn = u.c2;       // 두 번째 구성 문자
e.aEn.YAt(u.c3);    // 세 번째 구성 문자 (선택)
```

### 속성

| JSON 키 | 메서드 | 설명 |
|---------|--------|------|
| `c1` | `hEn` | 첫 번째 구성 문자 |
| `cs` | `uEn` | 문자 세트 |
| `c2` | `oEn` | 두 번째 구성 문자 |
| `c3` | `aEn` | 세 번째 구성 문자 |
| `cp` | `fEn` | 렌더링 경로 데이터 |
| `pc` | — | 구성 도형 수 |

- 여러 글리프를 하나의 렌더링 단위로 합성
- 구성 글리프별 경로(path) 기반 렌더링
- `Fsi()` 도형/경로 렌더링 사용

## 10. 차트 (CHART, wt.Z.Vli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Vli` |
| 리소스 이름 | `IDS_CTRL_NAME_CHART` |
| 참조 횟수 | 37 |
| 요소 유형 | `U.default.hri` (컨테이너) |
| 플래그 | `Vt.Z.ffi.wfi` |

### JSON 임포트 (jbn)

```javascript
e.SJt.jAt(u, i.pmn().bmn(o), false);
```

### 차트 속성

```javascript
{
    ch: chartData,      // 차트 설정 전체
    bi: bindingData,    // 데이터 바인딩 소스 참조
    js: jsonSpec        // 차트 사양 객체
}
```

### 렌더링

```javascript
// Canvas 기반 차트 렌더링
var canvas = document.createElement("canvas");
var ctx = canvas.getContext("2d");
// 다중 반복 렌더링 (품질 = a, a -= 0.2)
// toDataURL()로 이미지 내보내기
```

- Canvas 기반 차트 렌더링
- 이미지 내보내기 (`toDataURL`)
- 그리기 객체 첨부 (`Gbn`)
- 도형 데이터 임베딩 (`Bln`)

## 11. OLE (wt.Z.Gli)

### 컨트롤 정의

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.Gli` |
| 리소스 이름 | `IDS_CTRL_NAME_OLE` |
| 참조 횟수 | 26 |
| 플래그 | `Vt.Z.ffi.wfi | Vt.Z.ffi.afi` |

- 임베디드 OLE 객체 렌더링
- 외부 애플리케이션 데이터 표시

## 12. 공통 초기화 패턴 (Pbn)

모든 컨트롤은 `t.Pbn(i, n, e)` 함수로 공통 속성을 초기화한다:

```javascript
// 공통 속성 로딩
t.Pbn = function(i, n, e) {
    // n.ps → 문단 스타일
    // n.ro → 읽기 전용 속성
    // 기본 ID, 위치, 크기 속성 설정
}
```

### JSON 속성 약어

```javascript
// 공통 약어
id: controlId,   // 컨트롤 ID
zo: zeroOffset,  // 제로 오프셋
nt: numberType,  // 번호 유형
tw: textWidth,   // 텍스트 폭
tf: textFormat,  // 텍스트 형식
lo: layout,      // 레이아웃
li: linkInfo,    // 연결 정보
ty: type,        // 유형
```

## 13. 전체 컨트롤 요약

| 컨트롤 | 유형 | 요소 유형 | 참조 수 | 렌더러 | 핵심 기능 |
|--------|------|----------|---------|--------|----------|
| EQEDIT | 수식 | hri (컨테이너) | 62 | Qbn | 플러그인 기반 LaTeX 파싱 |
| BOOKMARK | 네비게이션 | Ali (하이퍼링크) | 25 | nEn | 파란 밑줄 + 클릭 내비 |
| FIELD_END | 필드 | SOt (필드) | 25 | SEn/Fmn | 코드/결과 토글 |
| AUTO_NUM | 번호 매기기 | NOt (필드) | 37 | SIn | 7단계 다중 레벨 형식 |
| NEW_NUM | 번호 매기기 | FOt (필드) | 28 | AIn | 번호 재시작 제어 |
| DUTMAL | 덧말 | kli (주석) | 26 | cEn | 위/아래 음성 텍스트 |
| COMMENT | 주석 | kli (주석) | 18 | eEn | Canvas/범위 기반 표시 |
| INDEXMARK | 마커 | Ali (하이퍼링크) | — | CEn | 2단계 색인 항목 |
| COMPOSE | 글자 겹침 | kli (주석) | 20 | sEn | 다중 글리프 합성 |
| CHART | 차트 | hri (컨테이너) | 37 | jbn | Canvas 렌더링 + 이미지 내보내기 |
| OLE | 임베디드 | hri (컨테이너) | 26 | — | 외부 객체 렌더링 |

## 14. rhwp와의 비교

| 항목 | webhwp | rhwp |
|------|--------|------|
| 수식 | 플러그인 기반 렌더링 | 미구현 |
| 북마크 | 하이퍼링크 스타일 + 내비게이션 | 미구현 |
| 필드 | 코드/결과 토글 | 미구현 |
| 자동 번호 | 7단계 다중 레벨 | 미구현 |
| 덧말 | 위/아래 주석 텍스트 | 미구현 |
| 주석 | Canvas + DOM 표시 | 미구현 |
| 차트 | Canvas 렌더링 | 미구현 |
| OLE | 임베디드 렌더링 | 미구현 |
| 글자 겹침 | 경로 기반 합성 | 미구현 |

---

*분석 일자: 2026-02-09*
