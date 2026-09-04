# 최종 결과보고서 — Task M100 #1891

## 이슈

[#1891 규제영향분석서 hwpx 계열 라운드트립 쪽수 팽창 (표본 173중 14 비-PASS, 최대 149→173쪽) — 비표준 ZIP(EOCD) 단서](https://github.com/edwardkim/rhwp/issues/1891)

## 요약

이슈의 "비표준 ZIP 컨테이너" 가설은 기각. 비-PASS 14건은 **확장자 위장
HWP5 10건 + 진짜 hwpx 4건**으로 분해되며, 진짜 hwpx 4건 중 3건은 #1893 merge
로 이미 PASS, 잔여 1건(73504)의 **외부 참조(BinData Link) 그림 왕복 소실**을
본 타스크에서 수정했다 (STRUCT 167px → PASS 0.00, hwpx-roundtrip diff 0·r2 0).
HWP5 위장 10건(쪽수 팽창 전건)은 HWP5→HWPX **변환** 축으로, 한글 2022 오라클
판정까지 완료해 후속 설계 사안으로 분리했다.

## 판별 — 이슈 전제의 재구성

- 대표 80250 의 파일 시그니처 = `D0 CF 11 E0` (**OLE/HWP5**). `.hwpx` 확장자로
  업로드된 부처 시스템 산출물이다. `hwpx-roundtrip` 의 "ZIP EOCD" PARSE_FAIL 은
  비-ZIP 파일에 대한 정상 거절이고, `render-diff --via hwpx` 는 `parse_document`
  자동 감지로 HWP5 파스에 성공한 뒤 **HWP5→HWPX 변환 왕복**을 측정한 것.
  도구 관용도 불일치의 실체는 "확장자 신뢰(hwpx-roundtrip) vs 시그니처
  자동감지(parse_document)" 차이다.
- 14건 분해: OLE(HWP5) 10건 — PAGE_MISMATCH 7건 전부 + STRUCT 3건 /
  진짜 ZIP hwpx 4건 — STRUCT 370px ×3 (#1893 merge 로 PASS 전환 확인) +
  73504 (STRUCT 167px, 본 수정).

## 축 B 수정 — 외부 참조(BinData Link) 그림 왕복 (73504)

### 근본 원인 (3중 결함 체인)

manifest 의 `<opf:item id="image1" href="D:\다운로드\" media-type="image/"
isEmbeded="0"/>` (작성자 로컬 경로 잔재, 콘텐츠 없음)를 본문 `<hp:pic>` 4곳이 참조.

1. **직렬화기 bin_data_map 임베디드 전용** — Link 참조 pic 의 `resolve_bin_id`
   실패 → pic 컨트롤 통째 드롭 → 레이아웃 앵커 소실 (render tree Image 4→1,
   TextRun 재배치 167px).
2. **manifest 순번 명명의 숫자 불변식 위반** — HWPX 파서는
   `binaryItemIDRef="imageN"` 의 숫자 N 을 그대로 bin_data_id 로 파싱한다.
   순번(i+1) 명명은 Link 항목으로 id 에 구멍이 있으면 이름↔id 가 어긋나
   재파스 그림 참조 전체가 시프트된다.
3. **파서 bin_data_items 필터의 media-type 의존** — `image/*` 또는 `BinData/`
   경로만 수집해, octet-stream 외부 참조가 누락되면 후속 항목 인덱스
   (=bin_data_id)가 전부 밀린다 (2-round 불안정의 원인).

### 수정

- `serializer/hwpx/context.rs`: `BinDataEntry.is_embedded` 신설, Link 항목 등록
  (storage_id=0 은 #1567 placeholder 센티널이라 제외), 명명을 숫자 불변식
  `image{bin_data_id}` 로 통일.
- `serializer/hwpx/mod.rs`: ZIP 쓰기·3-way 단언을 임베디드 한정.
- `serializer/hwpx/content.rs`: `isEmbeded` 실값 방출.
- `serializer/hwpx/package_check.rs`: 외부 참조 href 는 ZIP 실재 검사 제외.
- `parser/hwpx/content.rs`: 외부 참조 항목은 media-type 무관 수집.
- **계약 정제**: lib test `img_uses_manifest_id` 의 "id=5 → image1"(순번) 기대는
  숫자 불변식 위반의 고정이었음 → "image5" 로 갱신.

### 검증

- 73504: hwpx-roundtrip **PASS (diff 0, r2 0)** + render-diff **PASS 0.00**
  (사전: PKG_FAIL/IR_DIFF 7 + STRUCT 167px)
- `tests/issue_1891.rs` 2건 (렌더 자기정합 + Link/pic/콘텐츠 2-round 보존) PASS
- cargo test 전 스위트 (195 바이너리, 신규 픽스처는 hwpx_roundtrip_baseline 자동 포함) PASS
- big_hwpx 2,500 A/B (현 devel 동등 기준 vs fix exe): **완전 동일** (PASS 2483/
  STRUCT 9/OVER 8, 파일별 diff 0) — 통상 문서는 bd.id==순번이라 명명 불변,
  회귀 0 확인.

## 축 A — HWP5-in-.hwpx 10건 (후속 설계 사안)

- 같은 IR 인데 **로드 시멘틱 비대칭**: HWP5 로드는 빈 line_segs 를 reflow 하지
  않고(`document_core/commands/document.rs` `include_empty` 게이트, "페이지 수
  보존" 주석) 빈 문단이 h=0 으로 붕괴, 변환 HWPX 재로드는 합성 lineseg 로
  높이를 부여 → 쪽수 팽창 (80250: 16→19).
- **한글 2022 오라클** (pyhwpx, `output/poc/task1891/80250_hancom2022.pdf` 등):
  80250 한컴 **17쪽** (A 16 / B 19 — 양쪽 다 비정합). 페이지 앵커 대조:
  본문 빈 문단(Ⅰ.규제의 필요성 구간)은 **높이를 가진 쪽(B)이 정답**,
  표 영역(규제개요·조문대비표)의 쪽 경계는 **A 가 정답** (B 는 셀 내용
  팽창으로 +2쪽). 84522 의 <붙임> 표 위치도 B(2쪽 배치)가 한컴 정답.
- 결론: 높이 계산기 3자(라이브 ls=0 측정 / reflow 합성 / 한컴)가 모두 불일치.
  이상적 해법 = reflow 높이의 한컴 보정 후 게이트 소스 무관화 — 게이트 변경은
  LINE_SEG 부재 문서 전체의 렌더/쪽수에 파급되므로 **별도 타스크로 분리**
  (#1773 컴포저 조성 불변성 클래스).
- 부수 관찰: 84522 dump 의 CS bold=true→false 는 dump 가 raw attr 비트를 읽는
  표시 아티팩트 (렌더러는 `bold` 필드 사용, 실보존 확인).

## 산출물

- 수정 5파일 (serializer/hwpx: context·mod·content·package_check, parser/hwpx/content)
- 픽스처: samples/issue1891_external_bindata_link.hwpx (73504)
- 테스트: tests/issue_1891.rs + img_uses_manifest_id 계약 정제
- 오라클: output/poc/task1891/{80250,84522}_hancom2022.pdf (gitignore, 로컬)
- 문서: plans/task_m100_1891.md, plans/task_m100_1891_impl.md, 본 보고서
