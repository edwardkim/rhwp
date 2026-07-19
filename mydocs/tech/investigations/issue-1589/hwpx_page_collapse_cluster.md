---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1589/README.md
last_verified: 2026-07-16
---

# HWPX 페이지 붕괴 군집 조사 (#1589)

- 일자: 2026-06-27
- 바이너리: devel 0c72b210 (4 채택 누적)
- 도구: `tools/verify_hangul_pages.py` (한글 PageCount 오라클)

## 1. 군집 규모 (중대) — 확정

fidelity14 **PASS(IR diff=0) 파일** 한글 오라클 표본 측정:

| 표본 | 측정 | COLLAPSE | 붕괴율 | 95% CI |
|------|----:|----:|----:|----:|
| 무작위 A (seed 20260627) | 517 | 83 | 16.1% | ±3.2% |
| 무작위 B (seed 7) | 119 | 17 | 14.3% | ±6.3% |
| **무작위 합집합(비편향)** | **631** | **100** | **15.8%** | **±2.8%** |
| 참고: 알파벳순 first | 1879 | 347 | 18.5% | ±1.8% |

→ **IR 게이트 통과 파일의 ~16%(15.8±2.8%)가 한글에서 페이지 붕괴.** 단일 파일(#1589 최초
36384160)이 아닌 **대규모 군집**. IR diff=0 ≠ 시각 무손실의 가장 큰 잔존 갭.

> 알파벳순 표본(18.5%)이 무작위(15.8%)보다 높음 — 부서별 편차(초기 알파벳 부서의 실정보고 등
> 복합 템플릿이 붕괴 빈발) 시사. 비편향 추정은 **~16%**.

### 전수(10816) 미완 사유 — COM 환경 한계 (rhwp 무관)

한글 COM 자동화가 **~500–1900 오퍼레이션 후 사망**(com_error 다발). 도구 강화로 대응:
- 증분 기록 + `--resume`(크래시 재개), 주기적 `taskkill /F /IM Hwp.exe` + 재시작(누수 200+ 방지),
  시작 시 정리.
- 그럼에도 특정 부서(예 도로사업소 시설보수과 실정보고) 연속 처리 시 모달 다이얼로그/보안모듈
  추정 원인으로 회복 불가. 개별 파일은 클린 환경에서 정상 개방(진단 확인) → **rhwp/파일 무관,
  COM 자동화 환경 한계**. 무작위 표본으로 충분히 확정.

## 2. 붕괴 패턴

| 패턴 | 건수 |
|------|----:|
| 2→1 | 16 |
| 5→4 | 1 |

거의 전부 **마지막 1쪽 흡수**(2→1). 대상 전부 정부 "결재문서본문" 양식 — 체계적 단일 원인 시사.

## 3. IR-invisible 확인 (36389184, 2→1 대표)

orig↔rt 비교: **IR-비교 가능 메트릭 전부 동일**.
- 구조: hp:p 122, hp:run 110, hp:lineseg 218, hp:tbl 4, hp:tr 21, hp:tc 94 — 모두 일치.
- 수직: Σvertpos/vertsize/textheight/baseline/spacing 전부 일치.
- header: charPr 30(height 동일), paraPr 32(lineSpacing 동일), fontface 8, borderFill 10.

→ 한글이 reflow 에 쓰는 IR 값은 동일한데 페이지수만 다름.

## 4. 배제한 가설 (red herrings)

| 가설 | 검증 | 결론 |
|------|------|------|
| **탭 switch 래퍼 드롭**(48KB) | rhwp 가 `<hp:switch><hp:case ...HwpUnitChar><hh:tabItem unit="HWPUNIT"/></hp:case><hp:default>...</hp:default></hp:switch>` 를 plain `<hh:tabItem>` 로 방출(48KB 감소). **그러나 OK(비붕괴) 파일도 동일 드롭**(Δ=48628) | **무관**(보편적·양성). 단 pos=0 탭이라 레이아웃 영향 없음 |
| **#1592 빈문단 run 제거** | pre-#1592 rt(fidelity13)도 동일하게 2→1 붕괴 | **무관**(붕괴가 #1592 선행) |

## 5. 남은 구체 차이 (미규명)

- rt 가 **빈 `<hp:t></hp:t>` 58개 추가**(orig 0). close_run 규칙5(빈 run `<hp:t></hp:t>` 보존)
  유래. 단 빈 run 은 높이를 **더해** rt 를 길게 만들 텐데 붕괴는 rt 가 **짧음** → 방향 불일치,
  단순 원인 아님.
- rt 가 `Preview/PrvImage.png` 추가(썸네일, 레이아웃 무관).

## 5b. 근본원인 좁히기 — 통제 실험 (거의 동일 쌍)

**완벽한 비교쌍**: 붕괴 `36383351 [관악산] 산악구조대 구급의약품 폐기 계획`(2→1) vs OK
`36387726 [북한산] …`(동일 템플릿, 1문단 차). 두 파일의 **orig↔rt serialization 차이가 완전 동일**
(빈 hp:t +41/+39, 헤더 탭 −48KB, Preview) → **붕괴는 file-specific 아님, content 가 페이지 경계
근처인지에만 의존** 확정.

**하이브리드 bisection**(charPr id 매핑 동일=유효):

| 조합 | 페이지 |
|------|----:|
| orig-sec + orig-hdr | 2 |
| orig-sec + rt-hdr | 2 |
| rt-sec + orig-hdr | **1** |
| rt-sec + rt-hdr | **1** |

→ **rt-section0 이 원인**(header/탭 switch 확정 배제, header 무관).

**개별 차이 통제 배제**(orig 수정 or rt-sec revert, 한글 페이지수로 판정 — 전부 붕괴 불변):
빈 `<hp:t></hp:t>` 런·self-closed `<hp:run/>`·`<hp:t/>`·curSz(0↔5669)·noteLine(NONE↔SOLID)·
noteSpacing·fwSpace(전각공백→공백)·para id(0x80000000↔순차)·linesegarray(**96/96 바이트 동일**)·
outlineShapeIDRef(0↔1). **9+ 후보 전부 단일 원인 아님**.

→ **단일 변수로 재현 불가 = 누적/미세 상호작용 효과**(rt 의 빈런 표현·미세 spacing·shape 속성
차이가 합쳐져 경계-근처 문서를 tip). 정밀 규명은 한글 내부 레이아웃 디버깅 영역(정적 XML 분석 한계).

## 5c. 시각 추적 — 한글 페이지 브레이크 (PDF 렌더 비교)

`36383351 [관악산]` 을 한글로 PDF 내보내 페이지 레이아웃 직접 비교(pyhwpx save_as PDF →
PyMuPDF 렌더):

- **본문 최상위 문단별 페이지 매핑**(KeyIndicator prnpageno): orig 는 para 0–8 = 1쪽,
  **para 9(빈 문단, 문단나눔) = 2쪽**. rt 는 para 0–9 전부 1쪽.
- para 8 = `"붙임  구급 의약품 폐기 확인서(서식) 1부.  끝."`(마지막 본문, 양쪽 1쪽 동일).

**시각 확인(렌더 이미지)**:
- orig 1쪽: 본문이 ~60% 지점("붙임…끝.")에서 끝, 하단 40% 공백. **발신명의 footer 블록은 2쪽**.
- orig 2쪽: 거의 비고 하단에 **발신명의 블록만**(결재선 "1팀장 강한석…산악구조대장 이낙규" +
  "시행 산악구조대-2491" + "우 08825…" + "전화…").
- **rt 1쪽: 동일 본문 + 발신명의 블록이 1쪽 하단에 모두 수용**.

→ **붕괴의 시각적 실체 = 정부문서 표준 "발신명의" footer 블록(본문 뒤 하단 앵커)이 razor-thin
차이로 rt 에선 1쪽에 들어가고 orig 에선 2쪽으로 밀리는 것.** 본문은 시각적으로 완전 동일,
차이는 sub-line 누적 높이(§5b 9후보 배제와 정합 — 단일 원소 아닌 미세 누적이 경계를 tip).

> 함의: 붕괴는 "텍스트/내용 손실"이 아니라 **razor-thin 레이아웃 마진에서의 페이지 분할 변동**.
> 실문서 다수가 발신명의 블록을 본문 끝 직후 하단에 두는 동일 양식이라 14–16% 가 경계 근처.

## 5d. 근본원인 확정 — `holdAnchorAndSO` 직렬화 드롭 (실버그)

§5b 에서 단일 변수 분리 실패 후, **단락 단위 이진 탐색**(rt-section 문단을 orig 로 되돌리며
한글 페이지수 판정)으로 정확히 좁힘:

- 문단 5-9 revert → 해소 → 문단 9(발신명의 footer 표 포함) 단독 revert → **해소**.
- 문단 9 정규화 diff(id/빈런 제거) → **유일 차이 1건**:
  ```
  외곽 footer 표(페이지 하단 앵커, vertRelTo="PAGE" vertAlign="BOTTOM")의 <hp:pos>:
    orig: holdAnchorAndSO="1"   rt: holdAnchorAndSO="0"
  ```
- **결정 테스트**: orig 에서 `holdAnchorAndSO` 1→0 만 치환 → **2쪽→1쪽 붕괴 재현**. 확정.

### 코드 (실버그)

HWPX 직렬화기가 `holdAnchorAndSO` 를 **"0" 하드코딩**, 파싱된 IR 값 무시:
- `table.rs:146`, `picture.rs:407`, `shape.rs:899`, equation(`section.rs:1451`) = `("holdAnchorAndSO","0")`.
- 파서(`parser/hwpx/section.rs:1672`)는 정상 저장: `holdAnchorAndSO → common.prevent_page_break`(i32).
- **IR 비교가 `prevent_page_break` 미검사 → IR diff=0** (게이트 미검출, 시각만 붕괴).

### 군집 적용성

무작위 400 표본 orig 의 `holdAnchorAndSO="1"` 보유: **붕괴 53/63(84%)**, OK 278/337(82%).
→ 직렬화기가 전수 1→0 드롭. 페이지 경계 근처 문서(붕괴군)에서 발신명의 footer(페이지 하단 앵커)
위치가 바뀌어 붕괴. **`holdAnchorAndSO` 보존 수정 시 붕괴군 대다수 해소 예상**(별 통제 비교 필요).

> 결론: 페이지 붕괴는 "누적 미세차"가 아니라 **단일 속성 `holdAnchorAndSO` 직렬화 드롭**. §5b 의
> 9후보가 모두 음성이었던 이유 — 진짜 원인이 그 목록 밖(pos 의 boolean 속성)이었음.

## 5e. 잔여 붕괴(~8%) 좁히기 — 불완전 generic-shape 지오메트리

#1595(CLICK_HERE) 후 잔여 붕괴 표본 3건(36396457·36389684·36385226) 이진탐색·특성화:

| 파일 | 섹션 | 도형 | deciding |
|------|----:|------|----------|
| 36396457 | 3 | polygon×4 | section2 문단23 polygon |
| 36389684 | 4 | polygon×5,pic×2 | (polygon 추정) |
| 36385226 | 1 | ellipse×9,pic×2 | (ellipse 추정) |

**3건 전부 generic-shape(polygon/ellipse) 보유** → 공통 경로 `render_common_shape_xml`
(section.rs:1327)이 불완전:
- 태그에서 `numberingType="PICTURE"`·`dropcapstyle`·`href`·`groupLevel`·`instid` 드롭.
- `<hp:pos>` 에서 affectLSpacing·flowWithText·allowOverlap·holdAnchorAndSO 누락.
- **드로잉 지오메트리(lineShape·points·fillBrush) 드롭/축약**(rt −2614자) — **이것이 deciding**.

**통제 테스트(36396457 section2)**: polygon 태그속성 복원만으로는 미해소(page 4) →
**지오메트리 드롭이 원인**. 문단23(polygon 포함) 전체 revert 시에만 해소(page 11).

→ **잔여 붕괴의 근본 = generic-shape(polygon/ellipse/arc/curve) 지오메트리 직렬화 미완**
(shape.rs:11 "Arc/Polygon/Curve 확대 별도 분류" 갭). rect/picture 와 달리 실제 도형 데이터가
보존되지 않아 렌더 크기·레이아웃 변동 → 경계 근처 문서 붕괴. **별 타스크(도형 직렬화 완성)**
필요, 우선순위 낮음(잔여 ~8% = 군집의 long tail).

## 6. 결론 + 권고

붕괴는 **IR-identical content 인데 한글 reflow 결과만 다른 심층 레이아웃 충실도 결함**. 표면
XML 차이(탭 switch·빈 t·preview)로는 설명 안 됨 — 한글이 읽는 **비-IR 레이아웃 신호**(런 경계
분할, 문자 폭 미세, 줄바꿈 기회 등)의 차이로 추정.

**규모 큼(~14%)·난이도 높음**. 후속 권고:
1. **전수 오라클 배치**로 정확한 붕괴율·군집 규모 확정(현재 120 표본 추정).
2. 붕괴/비붕괴 파일 쌍의 **section0 정밀 바이트 diff**(런 경계·hp:t 분할 패턴 차이).
3. 한글에서 **페이지 브레이크 위치 시각 비교**(어느 줄/문단에서 갈리는지).
4. 가설: 런 경계 분할(charPr boundary)이 한글 줄바꿈 기회를 바꿔 더 촘촘히 패킹 → 행 수 감소.

근거: `output/poc/fidelity14/oracle_collapse_scan.tsv`, 메모리 [[hwp5-save-fidelity-gaps]].

## 7. [확정 #1598] ellipse/arc 전용 지오메트리가 잔여 long-tail 의 근본

§5e 의 "generic-shape 지오메트리 미완" 가설을 **ellipse 에 대해 통제 테스트로 확정**.

**근본**: HWPX 파서 `parse_shape_object` 가 ellipse/arc 자식 `<hc:center>/<hc:ax1>/<hc:ax2>/`
`<hc:start1>/<hc:end1>/<hc:start2>/<hc:end2>` 를 `_ => {}` 로 버리고(EllipseShape/ArcShape 를
`..Default::default()` 로 생성), 직렬화 `render_common_shape_xml` 도 미방출. IR diff 게이트는
ellipse 지오메트리를 비교하지 않아(IR-invisible) 미검출 — 한글 오라클만 검출.

**통제 테스트(36385226, ellipse×9, section0)**:

| 파일 | 한글 PageCount |
|------|---------------|
| orig | 3 |
| rt (지오메트리 드롭, 수정 전) | **2** ← 붕괴 |
| rt + orig 지오메트리 주입 | **3** ← 해소 |
| new-rt (#1598 파서+직렬화 수정) | **3** ← 해소 (end-to-end) |

→ ellipse 는 `treatAsChar=1` + `sz` 고정이라 bounding box 불변이지만, 한글은 center/축 없이는
타원을 다르게 렌더 → 누적 레이아웃 미세 변동 → 경계 근처 붕괴. **지오메트리 단독으로 해소**
(태그속성 intervalDirty/hasArcPr/arcType 는 불필요 — 보류).

**수정**: 파서 `parse_xy` 로 7개 점 적재(ellipse) / 3개(arc), 직렬화 `geom_tail` 로 shadow 직후
방출. polygon/curve points 는 #1067/#1200 으로 이미 정상.

**잔여**: 36389684 는 현재 바이너리에서 orig=rt=2 (붕괴 없음) — 군집 해소.
