# Task M100 #1388 — 1단계 완료 보고서 (전수 측정 + height +2 유닛 조사)

- 브랜치: `local/task1388`
- 작성일: 2026-06-12
- 코드 수정: 없음 (조사 전용)
- 산출물: `output/poc/task1388/margin_inventory.tsv` (74섹션 전수 인벤토리)

## 1. 원본 margin·gutterType 분포 (1.1)

`samples/hwpx` 54파일 74섹션 전수에서 `<hp:margin>`·`gutterType` 추출, 템플릿 고정값
(`header=4252 footer=4252 gutter=0 left=8504 right=8504 top=5668 bottom=4252`, `LEFT_ONLY`)과 대조.

| 항목 | 수치 |
|------|------|
| margin이 템플릿과 다른 섹션 | **51 / 74 (69%)** |
| margin이 템플릿과 다른 파일 | **31 / 54 (57%)** |
| gutterType이 템플릿(LEFT_ONLY)과 다른 섹션 | **8** (LEFT_RIGHT — 온새미로 5섹션, exam_social 2, exam_social-p1 1) |
| 고유 margin 패턴 수 | 16종+ (최빈: header=2834 footer=2834 left=7086/5669 계열) |

→ 결함 영향 범위가 전수의 절반 이상 — 단일 샘플(ta-pic-001-r) 증상이 아니라 광역 결함임을 정량 확인.

## 2. RT 변형 실측 (1.1 대조)

#1380 4단계 RT 산출물(`output/poc/task1380/*.rt.hwpx`) 45건과 원본 sec0 대조:

| 항목 | 수치 |
|------|------|
| margin 변형 | **27 / 45** (나머지 18건은 원본이 우연히 템플릿과 동일) |
| gutterType 변형 | 1건 (온새미로 LEFT_RIGHT→LEFT_ONLY; exam_social 계열은 #1384 SERIALIZE_FAIL로 RT 부재) |
| pagePr 여는 태그 중 width/height/landscape 변형 | **0 / 45** (#1166 동적화 정상 동작) |

실증 예 (온새미로 sec0):

```
원본: gutterType="LEFT_RIGHT" margin left=7086 right=14173 header/footer=4251 top/bottom=4251
RT  : gutterType="LEFT_ONLY"  margin left=8504 right=8504  header/footer=4252 top=5668 bottom=4252
```

## 3. height +2 유닛 조사 (1.2) — 결함 아님 판정

#1380 4단계 비고의 "pagePr height 84186→84188(+2 유닛)" 관찰을 추적한 결과:

- business_overview/expense_report/form-002: 원본 84186 = RT 84186 (변형 없음)
- ta-pic-001-r: 원본 84188 = RT 84188 (변형 없음)
- 전수 45건 pagePr 여는 태그 대조 불일치 **0건**

→ roundtrip 변형이 아니라 **원본별 정상 편차**로 귀속. A4 세로 297mm = 84188.97 HWPUNIT이며,
84186(한컴 기록 관행)/84188(정밀값)/84189가 샘플별로 혼재한다. 이전 관찰은 서로 다른 샘플
(또는 템플릿 상수 84186)과의 교차 비교에서 나온 것으로 판단. **이슈 분리 불요.**

## 4. 게이트 동승 영향 사전 판정 (1.3)

- 파서는 margin 7필드·gutterType을 IR로 정상 적재(0.2 확정)하므로, 2단계 serializer 수정 후
  parse→serialize→재parse 경로에서 `PageDef` 차이는 0이 기대치.
- SERIALIZE_FAIL(xfail #1384) 샘플은 게이트 진입 전 실패라 영향 없음.
- → **3단계 동승 시 신규 xfail 0 예상.** (4단계 전수에서 재확인)

## 5. 다음 단계

2단계 — `replace_page_pr` 확장 (margin 7필드 + gutterType 동적 치환) + 단위 테스트 4종.

승인 요청드립니다.
