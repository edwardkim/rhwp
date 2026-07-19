---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 페이지 레이아웃 컨트롤

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 분석 일자: 2026-02-09

## 1. 컨트롤 유형 요약

| 컨트롤 | ID | 리소스 이름 | 참조 수 | 플래그 |
|--------|------|-----------|---------|--------|
| SECDEF | `wt.Z.yfi` | IDS_CTRL_NAME_SECDEF | 83 | 0 |
| HEADER | `wt.Z.Ofi` | IDS_CTRL_NAME_HEADER | 59 | 0 |
| FOOTER | `wt.Z.xfi` | IDS_CTRL_NAME_FOOTER | 53 | 0 |
| FOOTNOTE | `wt.Z.Ifi` | IDS_CTRL_NAME_FOOTNOTE | 44 | 0 |
| ENDNOTE | `wt.Z.Cfi` | IDS_CTRL_NAME_ENDNOTE | 42 | 0 |
| COLDEF | `wt.Z._fi` | IDS_CTRL_NAME_COLDEF | 51 | `Vt.Z.ffi.wfi` |
| PAGE_NUM_POS | `wt.Z.Rfi` | IDS_CTRL_NAME_PAGE_NUM_POS | 20 | 0 |
| PAGE_NUM_CTRL | `wt.Z.Dfi` | IDS_CTRL_NAME_PAGE_NUM_CTRL | — | 0 |
| PAGE_HIDING | `wt.Z.Afi` | IDS_CTRL_NAME_PAGE_HIDING | 20 | 0 |

## 2. 섹션 정의 (SECDEF, wt.Z.yfi)

### 주요 속성/메서드

| 메서드 | 용도 |
|--------|------|
| `M0i()` | 섹션 모드/유형 (첫 페이지: `m0i`, 이후: `y0i`) |
| `ZVi()` | 페이지 유형 (첫 페이지/이후) |
| `w0i()` | 페이지 크기 정의 접근 |
| `hpi(ctrlType, ref)` | 중첩 컨트롤 검색 (SECDEF, COLDEF) |
| `Iki()` | 섹션 속성 초기화 |
| `Oqt()` | 용지 방향 조회 |
| `cbi()` | 가로 방향 여부 확인 |
| `pbi()` | 세로 방향 여부 확인 |

### 페이지 크기 데이터 구조

```javascript
{
    n8t: pageWidth,          // 용지 폭
    e8t: pageHeight,         // 용지 높이
    r8t: [left, right, top, bottom],  // 여백 배열
    s8t: totalWidth,         // 여백 포함 전체 폭: n8t + r8t[ACt] + r8t[RCt]
    h8t: totalHeight         // 여백 포함 전체 높이
}
```

### 여백 인덱스 상수

| 상수 | 방향 |
|------|------|
| `U.default.ACt` | 왼쪽 여백 |
| `U.default.RCt` | 오른쪽 여백 |
| `U.default.OCt` | 위쪽 여백 |
| `U.default.kCt` | 아래쪽 여백 |

### 페이지 크기 계산

```javascript
// 초기화
this.n8t = 0; this.e8t = 0;
this.r8t = [0, 0, 0, 0];
this.s8t = 0; this.h8t = 0;

// 크기 업데이트
i.n8t = s.width;
i.e8t = s.height;
i.s8t = i.n8t + i.r8t[U.default.ACt] + i.r8t[U.default.RCt];

// 방향 전환 시 DYi 함수로 치수 조정
var result = n.jPi().DYi(0, i.n8t, i.e8t);
i.n8t = result.width;
i.e8t = result.height;
```

### 컨트롤 처리 패턴

```javascript
switch(e.HOt()) {
    case wt.Z.yfi: this.f0i(e); break;   // 섹션
    case wt.Z.Ofi:
    case wt.Z.xfi: this.c0i(e); break;   // 머리말/꼬리말
    case wt.Z.Ifi: this.l0i(e); break;   // 각주
    case wt.Z.Dfi: this.d0i(e); break;   // 쪽 번호 제어
    case wt.Z.Afi: this.v0i(e); break;   // 감추기
    case wt.Z.Rfi: this._0i(e); break;   // 쪽 번호 위치
}
```

## 3. 머리말 (HEADER, wt.Z.Ofi)

### 주요 메서드

| 메서드 | 용도 |
|--------|------|
| `ZXi()` | 머리말 높이/치수 |
| `rnn()` | 머리말 영역 폭 (가로 방향) |
| `ftn()` | 머리말 여백/간격 |

### 영역 계산

```javascript
// 머리말 영역 크기
case wt.Z.Ofi:
    o = u.lbi.jPi().ZXi();     // 높이
    a = h.ftn();                // 여백
    // 가로 방향: 좌우 공간에 머리말 배치
    u.cbi() && u.pbi()
        ? e.right - e.left > o + a && (e.left = e.right - (o + a))
        : e.bottom - e.top > o + a;
```

### 첫 페이지/홀짝 페이지 구분

```javascript
// 첫 페이지 다른 머리말
Oqt() === U.default.b0i  // 첫 페이지
Oqt() === U.default.m0i  // 일반 페이지

// 홀짝 구분
YYi()  // true면 홀짝 머리말 활성
// 배열 접근
this.NQi.Q7i  // 머리말 배열
```

## 4. 꼬리말 (FOOTER, wt.Z.xfi)

### 주요 메서드

| 메서드 | 용도 |
|--------|------|
| `iqi()` | 꼬리말 높이/치수 |
| `snn()` | 꼬리말 영역 폭 (가로 방향) |
| `ctn()` | 꼬리말 여백/간격 |

### 영역 계산

```javascript
case wt.Z.xfi:
    o = u.lbi.jPi().iqi();     // 높이
    a = h.ctn();                // 여백
    // 머리말과 동일한 패턴
```

### CSS 클래스

```
hcwo_page_header       — 머리말 영역
hcwo_page_header_block — 머리말 블록
hcwo_page_footer       — 꼬리말 영역
hcwo_page_footer_block — 꼬리말 블록
```

## 5. 각주 (FOOTNOTE, wt.Z.Ifi)

### 주요 메서드

| 메서드 | 용도 |
|--------|------|
| `gzi()` | 각주 여부 확인 (`this.HOt() == wt.Z.Ifi`) |
| `S0i()` | 각주 번호 조회 |
| `Eqi()` | 각주 메타데이터 |
| `hne()` | 각주 번호 표시 (크기: `parseInt(3*t/4)`) |
| `$$i()` | 특수 서식 여부 |
| `V$i()` | 각주 앵커/마커 |

### 각주 수집 로직

```javascript
// 페이지 내 각주 수집
for (i = t.Ejn(); i; ) {
    var n = i.hgi();
    if (n == wt.Z.Ifi) {
        // 각주 → 페이지 하단에 수집
    }
}

// 각주 초기화
if ((this.hgi() == wt.Z.Ifi || this.hgi() == wt.Z.Cfi) && 0 == this.Svi()) {
    this.Dvi(e | Et.fOt);
}
```

### 직렬화 속성

```javascript
Sxn() {
    so: serialization,   // 직렬화 데이터
    li: link,            // 연결 정보
    ty: type,            // 유형
    pd: printDiff,       // 인쇄 차이
    pf: printFlag        // 인쇄 플래그
}
```

### CSS 클래스

```
hcwo_footnote_area — 각주 영역
hcwo_footnote_wrap — 각주 래퍼
```

## 6. 미주 (ENDNOTE, wt.Z.Cfi)

### 주요 메서드

| 메서드 | 용도 |
|--------|------|
| `vne()` | 미주 여부 확인 (`this.HOt() == wt.Z.Cfi`) |
| `S0i()` | 미주 번호 조회 |

### 각주와의 차이

```javascript
// 각주: 해당 페이지 하단
t.hgi() == wt.Z.Ifi && this.qjn(t, n, i)

// 미주: 섹션/문서 끝
t.hgi() == wt.Z.Cfi && this.Xjn(t, n, i)

// 참조 텍스트 선택
e.HOt() == wt.Z.Ifi ? n.fn : n.en
```

### CSS 클래스

```
hcwo_endnote_area — 미주 영역
hcwo_endnote_wrap — 미주 래퍼
```

## 7. 다단 (COLDEF, wt.Z._fi)

### 주요 속성

| 속성/메서드 | 용도 |
|------------|------|
| `VNt()` / `Tan()` / `aVi` | 단 수 |
| `$6t` | 단 사이 간격 |
| `fji[]` | 단별 폭 배열 |
| `dji()` | 단 수 또는 치수 정보 |

### 단 레이아웃 계산

```javascript
// 단 수: r.Tan() 또는 r.aVi
nCol = r.Tan();

// 단 간격: $6t 속성
gap = section.$6t;

// 단별 폭: fji[0], fji[1], ..., fji[n-1]
// 전체 폭 = 모든 단 폭 합 + 간격들

// 단 직렬화 (3개씩 묶어 저장)
n._Nt(3*r+0)  // 위치
n._Nt(3*r+1)  // 간격
n._Nt(3*r+2)  // 유형
```

### 플래그

```javascript
// wfi 플래그: 단 래핑 지원
hfi: Vt.Z.ffi.wfi

// 섹션에 단 정의가 있는지 확인
null != c.hpi(wt.Z._fi, r).result
    && (c.Jci |= U.default.Smi)  // Smi 플래그 설정
```

## 8. 쪽 번호 컨트롤

### PAGE_NUM_POS (wt.Z.Rfi)

쪽 번호 위치 제어:

```javascript
// JSON 임포트: t.UIn()
e.dUt = 0;           // 시작 번호 오프셋
e.lIn = e.cIn = e.dIn = 0;
e.WIn = 0;
e.GIn(r.po);         // 쪽 번호 위치
e.AMi(r.ft);         // 번호 형식
var s = r.sc;
e.WIn = s.charCodeAt(0);  // 구분자 문자
```

### PAGE_NUM_CTRL (wt.Z.Dfi)

쪽 번호 제어:

```javascript
// 속성
dUt: startOffset,    // 시작 쪽 번호
GIn(): format,       // 형식 (아라비아/로마 등)
kIn(): type,         // 유형 (연속/재시작)
```

### 번호 형식

| 속성 | 설명 |
|------|------|
| `dUt` | 시작 번호 오프셋 (0 기반 또는 1 기반) |
| `vt.Rft` | 형식 코드 (아라비아/로마/소문자 등) |
| `kIn(U.default.y0i)` | 표준 쪽 번호 매기기 설정 |

## 9. 감추기 (PAGE_HIDING, wt.Z.Afi)

### 감추기 플래그 비트마스크

```javascript
// 초기화
this.E0i = U.default.NIn | U.default.HIn | U.default.BIn;

// 플래그 의미
U.default.NIn  // 머리말 감추기
U.default.HIn  // 꼬리말 감추기
U.default.PIn  // 쪽 여백 감추기
U.default.LIn  // 하단 여백 감추기
U.default.MIn  // 인쇄 감추기
U.default.BIn  // 쪽 번호 감추기
```

### JSON 임포트 (xIn)

```javascript
t.xIn = function(i, n, e) {
    var r = 0;
    t.Pbn(i, n, e);
    var s = n.ph;
    if (null != s) {
        if (s.hh) r |= U.default.NIn;  // 머리말
        if (s.hf) r |= U.default.HIn;  // 꼬리말
        if (s.hm) r |= U.default.PIn;  // 여백
        if (s.hb) r |= U.default.LIn;  // 하단
        if (s.hi) r |= U.default.MIn;  // 인쇄
        if (s.hp) r |= U.default.BIn;  // 쪽 번호
    }
    e.FIn(r);
}
```

## 10. 치수/레이아웃 유틸리티

### 핵심 치수 메서드

| 메서드/속성 | 용도 |
|------------|------|
| `n8t` | 폭 (페이지/콘텐츠 폭) |
| `e8t` | 높이 (페이지/콘텐츠 높이) |
| `r8t[4]` | 여백 배열 `[좌, 우, 상, 하]` |
| `s8t` | 여백 포함 총 폭 |
| `h8t` | 여백 포함 총 높이 |
| `DYi(type, w, h)` | 방향 적용 치수 계산기 |

## 11. 페이지 컨트롤러 (전체 페이지 처리)

### 11.1 PageHandler (이벤트 디스패처)

webhwp에는 두 개의 `PageHandler` 클래스가 존재한다.

**VI — 문서 수준 PageHandler**

```javascript
t.e_refresh = "refresh"         // 페이지 새로고침
t.d_save = "save"               // 저장
t.d_download = "download"       // 다운로드
t.d_pdf_download = "pdfdownload" // PDF 다운로드
t.e_review = "review"           // 리뷰
t.d_save_as = "saveas"          // 다른 이름 저장
t.closeApp = "closeApp"         // 앱 닫기
```

**RE — 페이지 구조 PageHandler**

```javascript
t.e_delete_header_footer = "pagedeleteheaderfooter"
t.e_next_header_footer = "pagenextheaderfooter"
t.e_prev_header_footer = "pageprevheaderfooter"
t.p_page_break = "pagebreak"
t.p_column_break = "pagecolumnbreak"
t.p_column_one = "pagecolumnone"
t.p_column_two = "pagecolumntwo"
t.p_column_three = "pagecolumnthree"
t.p_column_left = "pagecolumnleft"
t.p_column_right = "pagecolumnright"
```

### 11.2 Actor/Updater 패턴

각 페이지 작업은 Actor(실행) + Updater(UI 갱신) 쌍으로 구현된다.

| Actor | 명령 코드 | 역할 |
|-------|----------|------|
| `pagebreakActor` | `t.mD` | 쪽 나누기 |
| `pagecolumnbreakActor` | `t.bD` | 단 나누기 |
| `pagecolumnoneActor` | `t.QD` | 1단 설정 |
| `pagecolumntwoActor` | `t.XD` | 2단 설정 |
| `pagecolumnthreeActor` | `t.qD` | 3단 설정 |
| `pagecolumnleftActor` | `t.JD` | 왼쪽 단 |
| `pagecolumnrightActor` | `t.ZD` | 오른쪽 단 |
| `pagebreaknewcolumnActor` | `t.CD` | 새 단에서 나누기 |
| `pagedeleteheaderfooterActor` | `t.Ez` | 머리말/꼬리말 삭제 |
| `pagenextheaderfooterActor` | `t.Iz` | 다음 머리말/꼬리말 |
| `pageprevheaderfooterActor` | `t.yz` | 이전 머리말/꼬리말 |
| `refreshActor` | `e_refresh` | 전체 페이지 새로고침 |

실행 패턴: `this.VPs.yNt(t.명령코드)` → 엔진에 명령 전달

### 11.3 페이지 조합 엔진 (Dr 클래스)

**핵심 메서드:**

| 메서드 | 역할 |
|--------|------|
| `SUt(ctrl, idx)` | 페이지 초기화 (컨트롤 유형별 분기) |
| `ehn(ctrl, idx)` | 페이지 상태 리셋 |
| **`Ehn(height, lastIdx)`** | **메인 페이지 빌더** — 콘텐츠 블록을 행/열로 배치 |
| `Zsn(flag)` | 페이지 완료 플래그 설정 |
| `ZXi()` | 페이지/영역 높이 |
| `Yhn()` | 페이지 간격 |
| `bhn()` | 콘텐츠 블록(행) 수 |
| `Rji(idx)` | 인덱스로 콘텐츠 블록 접근 |
| `M3i(idx)` | 서브 페이지 모델 접근 |
| `Chn(block)` | 블록에서 셀/컨트롤 접근 |
| `Vhi(size)` | 페이지 크기 계산 |
| `Awi(block, x, y)` | 블록 렌더링 |

**`Ehn()` 페이지 빌드 루프:**

```javascript
// 페이지네이션 메인 루프
Ehn(height, lastIdx) {
    var m = this.mhn();       // 마스터 핸들러
    var a = m.lVi();          // 전체 콘텐츠 항목 수
    // dhn[] = 행 배열 (콘텐츠 블록들)
    // 반환값: -1 (미완료), -2 (완료), 양수 (다음 항목 인덱스)
}

// 상위 호출부 (페이지 구성):
ZXi() && (h += ZXi() + Yhn());   // 머리말 높이 + 간격
l = -1;
for (;;) {
    a = t.Ehn(h, l);              // 콘텐츠 블록 배치
    if (-1 == a) { Zsn(false); break; }  // 페이지 미완료
    if (-2 == a) break;                   // 페이지 완료
}
```

**컨트롤별 SUt 초기화 분기:**

```javascript
// 표 (TABLE)
SUt(t, i) { t.HOt(), wt.Z.gfi; this.ehn(t, i); this.ahn = t; this.fhn = i; }

// 수식 (EQEDIT)
SUt(t, i) { t.HOt(), wt.Z.Tfi; this.ehn(t, i); this.NMe = t; this.fhn = i; }

// 차트 (CHART)
SUt(t, i) { t.HOt(), wt.Z.Vli; this.ehn(t, i); this.EMe = t; this.fhn = i; }
```

### 11.4 페이지 DOM 구조

```
페이지
├── hcwo_page_landscape             (가로 방향)
├── hcwo_page_opacity               (투명도)
├── hcwo_page_header                (머리말 영역)
│   ├── hcwo_page_header_block      (머리말 블록)
│   └── hcwo_page_header_overflow   (오버플로우)
├── hcwo_page_contents              (본문 영역)
│   └── hcwo_page_contents_block    (본문 블록)
├── hcwo_note_target                (주석 대상)
├── hcwo_footnote_area              (각주 영역)
│   └── hcwo_footnote_wrap          (각주 래퍼)
│       └── hcwo_footnote_block     (각주 블록)
├── hcwo_endnote_area               (미주 영역)
│   └── hcwo_endnote_wrap           (미주 래퍼)
│       └── hcwo_endnote_block      (미주 블록)
├── hcwo_page_footer                (꼬리말 영역)
│   ├── hcwo_page_footer_block      (꼬리말 블록)
│   └── hcwo_page_footer_overflow   (오버플로우)
└── hcwo_comment_wrap               (주석 영역)
    ├── hcwo_comment_header         (주석 헤더)
    └── hcwo_comment_block          (주석 블록)
```

### 11.5 마스터 페이지 유형

```javascript
IDS_PAGE_TYPE_BOTH          // 양면 페이지
IDS_PAGE_TYPE_EVEN          // 짝수 페이지
IDS_PAGE_TYPE_ODD           // 홀수 페이지
IDS_MASTERPAGE_BEGIN        // 마스터 페이지 시작 (첫 페이지)
IDS_MASTERPAGE_LAST         // 마스터 페이지 끝 (마지막 페이지)
IDS_MASTERPAGE_BEGIN_SHORT  // 짧은 이름
IDS_MASTERPAGE_LAST_SHORT   // 짧은 이름
```

### 11.6 리프레시 파이프라인

```javascript
refreshActor.irh()
  → i.PPt("e_refresh")
    → Dt (action handler)
      → 전체 페이지 재배치/재렌더링
```

### 11.7 기타 페이지 관련 CSS 클래스

```
hcwo_break_para            — 페이지 구분 문단
hcwo_header_footer_hidden  — 머리말/꼬리말 감춤
hcwo_bookmark              — 북마크
hcwo_bookmark_hidden       — 북마크 감춤
sectionBreak               — 섹션 구분
softBreak                  — 소프트 줄바꿈
pageBreak                  — 페이지 구분
```

## 12. rhwp와의 비교

| 항목 | webhwp | rhwp |
|------|--------|------|
| 페이지 크기 | `n8t/e8t` + `r8t[]` 여백 | `PageDef` + `PageAreas` |
| 머리말/꼬리말 | 전용 컨트롤 (Ofi/xfi) | 기본 지원 (렌더링) |
| 각주/미주 | 별도 수집 로직 + CSS | 미구현 |
| 다단 | `VNt()` + `fji[]` 폭 배열 | `ColumnDef` 기본 지원 |
| 쪽 번호 | 3가지 컨트롤 조합 | 미구현 |
| 감추기 | 비트마스크 플래그 | 미구현 |
| 방향 전환 | `DYi()` 치수 계산기 | `PageDef` 속성 직접 |
| 첫 페이지 구분 | `Oqt()` 반환값 분기 | 미구현 |
| 홀짝 페이지 | `YYi()` 메서드 | 미구현 |

---

*분석 일자: 2026-02-09*
