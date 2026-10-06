# Task M100 #1315 — 4단계 완료 보고서

## 단계 목표

대표 샘플 8개 roundtrip 산출물의 시각 비교 자료(SVG) + rhwp-studio 로드 확인 + 한컴에디터 판정 요청 목록 작성.

## 대표 샘플 8건 선정

| 샘플 | 선정 사유 |
|------|----------|
| blank_hwpx | 최소 문서 |
| para-001 | 문단/서식 |
| basic-table-01 | 기본 표 |
| ta-pic-001-r | 표 + 셀 내 그림 |
| form-002 | 양식 개체 |
| footnote-01 | 각주 |
| math-001 | 수식 |
| 2025년 1분기 해외직접투자 보도자료f | 복합 실문서 (C등급) |

## SVG byte 비교 결과 (원본 vs roundtrip, `output/poc/task1315/svg/`)

| 샘플 | 원본 쪽수 | rt 쪽수 | byte-identical | 차이 양상 |
|------|----------|---------|----------------|----------|
| blank_hwpx | 1 | 1 | **1/1 일치** | — |
| para-001 | 3 | 3 | **3/3 일치** | — |
| basic-table-01 | 1 | 1 | **1/1 일치** | — |
| ta-pic-001-r | 1 | 1 | 0/1 | 셀 내 그림 소실 + 셀 높이/x좌표 큰 시프트 |
| form-002 | 10 | **17** | 0 | **페이지 수 폭증** |
| footnote-01 | 6 | 6 | 0/6 | 전 페이지 차이 |
| math-001 | 1 | 1 | 0/1 | 텍스트 x좌표 1~2px 미세 시프트 |
| 보도자료f (2025-1Q) | 9 | **13** | 0 | **페이지 수 폭증** |

## 차이 원인 진단 (조사만 수행, 코드 무수정)

### 확정 — 표 셀 subList 텍스트 전용 직렬화 (`src/serializer/hwpx/table.rs` `write_sub_list`)

셀 내부 문단을 `<hp:p><hp:run><hp:t>텍스트</hp:t></hp:run></hp:p>` + 합성 lineseg
(vertsize=1000, baseline=850 고정) 구조로만 출력한다. 그 결과:

- **셀 내 컨트롤(그림 등) 전부 소실** — ta-pic-001-r 원본 `hp:pic` 4개(전부 셀 내부) → rt 0개.
  BinData 엔트리(image1.bmp, image2.bmp)는 ZIP에 보존되나 참조 컨트롤이 없음.
- 셀 문단 lineseg가 원본 값 대신 합성값으로 고정 → 셀 내부 레이아웃 변형.

### 확정 — baseline 게이트가 이 소실을 통과시키는 이유

`diff_documents()`(roundtrip.rs)는 섹션 수·문단 수·DocInfo 리소스 수·BinData 수만 비교하는
**뼈대 비교**다. 문단 내부 컨트롤 수/내용은 비교하지 않으므로 셀 내 그림 소실에도 IrDiff 0.
이는 테스트 doc 주석에 명시한 "구조(뼈대) 보존 검증 ≠ 시각 충실" 한계의 실증 사례다.

### 관찰 — 본문 문단 lineseg 값 변형

본문 lineseg는 IR 기반 보존(#177)이 원칙이나, ta-pic-001-r 첫 문단에서
`lh=21974→19924, th=21974→1000, bl=18678→850, sw=51024→42520` 변형 확인.
form-002(10→17쪽)·보도자료(9→13쪽)의 페이지 수 폭증은 이 lineseg 변형 +
셀 직렬화 한계로 인한 레이아웃 재계산 결과로 귀속된다.
math-001의 1~2px 시프트, footnote-01 전 페이지 차이도 같은 계열.

> 이슈 #1315 주의사항("시각 차이 발생 시 serializer 전체 실패로 보지 않고 기능별
> known limitation으로 분리")에 따라 serializer 수정으로 확전하지 않고 기록만 한다.

## rhwp-studio 로드 확인 (호스트 Chrome CDP, `e2e/task1315-load.check.mjs`)

bytes 직접 주입(`window.__wasm.loadDocument`) 방식 — `/samples` fetch 제한과 무관.

| 샘플 | 원본 쪽수 | rt 쪽수 | rt 로드 |
|------|----------|---------|---------|
| blank_hwpx | 1 | 1 | OK |
| para-001 | 3 | 3 | OK |
| basic-table-01 | 1 | 1 | OK |
| ta-pic-001-r | 1 | 1 | OK |
| form-002 | 10 | 17 | OK |
| footnote-01 | 6 | 6 | OK |
| math-001 | 1 | 1 | OK |
| 보도자료f (2025-1Q) | 9 | 13 | OK |

**8/8 로드 성공** — 오류·무응답 없음. 페이지 수는 SVG 비교 결과와 일치.

## 한컴에디터 판정 요청 (작업지시자)

`output/poc/task1315/` 의 rt 파일 8건을 한컴에디터에서 열어 **열기 오류/무응답 여부** 판정 요청:

1. `blank_hwpx.rt.hwpx`
2. `para-001.rt.hwpx`
3. `basic-table-01.rt.hwpx`
4. `ta-pic-001-r.rt.hwpx`
5. `form-002.rt.hwpx`
6. `footnote-01.rt.hwpx`
7. `math-001.rt.hwpx`
8. `2025년 1분기 해외직접투자 보도자료f.rt.hwpx`

> 본 타스크의 최소 호환성 기준은 "한컴에디터에서 열림"이며, 시각 충실도(그림 소실,
> 페이지 수 변화)는 known limitation으로 별도 기록한다. 시각 판정 권위는 작업지시자에게 있다.

## 별도 이슈 후보 (승인 시 등록)

| 후보 | 내용 |
|------|------|
| ① 셀 subList 텍스트 전용 직렬화 | 셀 내 그림/컨트롤 소실 + 합성 lineseg (`table.rs` `write_sub_list`) |
| ② 본문 lineseg 보존 불완전 | lh/th/bl/sw 변형 → 페이지 수 변화 (form-002 10→17, 보도자료 9→13) |
| ③ exam_social borderFillIDRef 31 | 1단계 발견, xfail 등록됨 — parser→serializer ID 매핑 경계 |

## 구현 내역

- `rhwp-studio/e2e/task1315-load.check.mjs` 신규 — rt 파일 bytes 직접 주입 로드 검증
  (호스트 Chrome CDP, helpers.mjs 미의존 — pixelmatch 미설치 환경 대응)
- 산출물: `output/poc/task1315/svg/` 원본·rt SVG 8쌍 (git 미추적)
- 소스(`src/`) 무수정 — 조사·검증만 수행

## 판단 요청 (3단계 미결 포함)

1. C등급(ORACLE_UNFIT) 13건 목록 확정 (3단계 재기재)
2. `hwpx-01.hwpx` 샘플 정비 여부 — 현상 유지 제안 (3단계 재기재)
3. 별도 이슈 후보 ①·②·③ 등록 여부
4. rt 파일 8건 한컴에디터 판정 (열림/오류)

## 다음 단계

5단계 — `mydocs/manual/hwpx_roundtrip_baseline.md` 매뉴얼 작성, CLAUDE.md에
`hwpx-roundtrip` 명령 안내 추가, 최종 보고서, 오늘할일 갱신, push 전 CI급 검증.
