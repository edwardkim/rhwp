# Task M100 #1388 최종 보고서 — HWPX serializer secPr 페이지 여백 원본 보존

- 이슈: #1388 "HWPX serializer: secPr 페이지 여백(hp:margin) 템플릿 하드코딩 — 원본 여백 변형"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1388`
- 작성일: 2026-06-12

## 1. 결함과 해소

| 축 | 종전 | 해소 |
|----|------|------|
| margin 7필드 | 템플릿 고정값(left/right=8504 등) 방출 — 전수 51/74 섹션에서 원본 여백 변형, 본문 +56.7px 시프트·페이지 수 변동 | `replace_page_pr`에 margin anchor 치환 추가 — IR `PageDef` 7필드 방출 |
| gutterType | `LEFT_ONLY` 고정 — LEFT_RIGHT 원본 8섹션 변형 | IR `binding` 역매핑(파서 매핑의 역방향) 동적화 |
| 게이트 | `diff_documents`에 PageDef 비교 없음 — baseline 사각 | `SectionPageDef` 동승 (용지 크기/방향/제본 + 여백 7필드), 소비처 3곳 자동 동승 |
| height +2 유닛 (#1380 4단계 비고) | 결함 의심 | **결함 아님 판정** — 전수 45건 roundtrip 변형 0, 84186/84188/84189는 원본별 정상 편차 (A4 297mm=84188.97 HWPUNIT) |

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 전수 측정 (margin 변형 51/74 섹션·31/54 파일, gutterType 8섹션) + height +2 귀속 조사 + xfail 사전 판정(0 예상) | `776fe5c9` |
| 2 | `replace_page_pr` 확장 (margin 7필드 + gutterType) + 단위 테스트 4종 | `ce46d338` |
| 3 | `diff_page_def` → `SectionPageDef` 게이트 동승 + 테스트 3종 | `56926e68` |
| 4 | 전수 검증 + 페이지 수 대조 + SVG 귀속 + 매뉴얼·최종 보고서 | (본 커밋) |

수정 파일: `src/serializer/hwpx/section.rs`, `src/serializer/hwpx/roundtrip.rs` —
serializer/게이트 한정, 렌더러·레이아웃·파서 무변경.

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1388/`)

- `hwpx-roundtrip --batch samples/hwpx` (게이트 동승 빌드): PASS 48 / IR_DIFF 1(#1382) /
  SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1(제외 hwpx-01) — **#1380 시점과 완전 동일, 신규 실패 0**
- RT 전 섹션(45파일 63섹션) margin·gutterType 대조: **변형 0건** (종전 27/45 파일)
- `cargo test --test hwpx_roundtrip_baseline`: 4 passed — **신규 xfail 0** (1단계 사전 판정 적중)

### 3.2 페이지 수 대조 (원본 vs RT 렌더)

| 샘플 | 원본 | RT | 판정 |
|------|------|----|------|
| ta-pic-001-r | 1 | 1 | 일치 |
| business_overview | 1 | 1 | 일치 |
| expense_report | 1 | 1 | 일치 |
| form-002 | 10 | 10 | 일치 |
| 2024년 연간 해외직접투자 보도자료 _ ff | 9 | 9 | 일치 |

#1380 4단계의 sed 여백 패치 실험(10=10·9=9)을 **패치 없이 코드로 달성**.

### 3.3 SVG 비교 (ta-pic-001-r, `output/poc/task1388/svg/`)

- 첫 텍스트 좌표 원본=RT 동일 (115.44) — 이슈 기재 **+56.7px 시프트 해소**
  (4252 HWPUNIT = 56.7px)
- 잔존 차이: 원본에만 있는 좌표 18개 — 단일 행(y=422.85)으로 **#1387 캡션 18자 소실에
  전량 귀속**. RT에만 있는 좌표 0개 — 본 타스크 기인 신규 차이 없음.

### 3.4 CI급 검증

- `cargo test --tests` 전체 그린 — **2233 passed, 0 failed** (기존 2226 + 신규 7: serializer 4 + 게이트 3)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. 잔존 한계 (기지 이슈 귀속)

| 한계 | 이슈 |
|------|------|
| 표 캡션 subList 소실 (ta-pic-001-r 잔존 18자) | #1387 |
| hp:pic 크기 요소 IR 미반영 | #1389 |
| 표 pageBreak 일괄 TABLE 방출 | #1393 |
| MEMO subList / shapeComment 소실 | #1391 / #1392 |
| borderFillIDRef SERIALIZE_FAIL 4건 / autoNum 폭 1건 | #1384 / #1382 |

신규 발견 없음 — height +2 유닛은 결함 아님 판정으로 종결 (1단계 보고서 3절).

## 5. 한컴 판정 요청

`output/poc/task1388/`의 RT 3건을 한컴에디터에서 열어 판정 부탁드립니다:

1. **ta-pic-001-r.rt.hwpx** — 편집 용지 여백(왼쪽/오른쪽 15mm)이 원본과 동일한지
2. **[2027] 온새미로 1 본교재.rt.hwpx** — 편집 용지 제본(맞쪽) + 왼/오른쪽 비대칭 여백 보존
3. **form-002.rt.hwpx** — 10페이지 유지 + 본문 시작 위치

## 6. 산출물

- 계획서: `mydocs/plans/task_m100_1388{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1388_stage{1..3}.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (게이트 비교 항목 + #1388 해소 처리)
- 검증 산출물: `output/poc/task1388/` (전수 rt + margin_inventory.tsv + svg/)
