# 17 — 검색·추출 질의 카탈로그

긴 한글 행정문서에서 자주 나오는 질의. `search --limit` 또는 `extract-data --kind`로만 좁힌다.
해당 없으면 전문을 열지 않고 없다고 답한다.

| 질의 | 문서 유형 | 도구 | 권장 limit |
| --- | --- | --- | --- |
| `위임전결` | 행정 편람 | search | 20 |
| `전결규정` | 행정 편람 | search | 20 |
| `시행일` | 법령·고시 | search+date | 20 |
| `부칙` | 법령 | search+structure | 20 |
| `별표` | 법령 | search | 20 |
| `별지` | 서식 | search | 20 |
| `누름틀` | 서식 | explain.fields | — |
| `개인정보` | 배포 전 | security-sweep | — |
| `주민등록` | PII | security-sweep | — |
| `계좌` | PII/금액 | security+amount | 20 |
| `예산` | 업무계획 | extract-data amount | 50 |
| `백만원` | 예산서 | extract-data amount | 50 |
| `억원` | 예산서 | extract-data amount | 50 |
| `천원` | 예산서 | extract-data amount | 50 |
| `원정` | 공문 | extract-data amount | 20 |
| `금113` | 영수증 표기 | extract-data amount | 20 |
| `과업` | 용역 | search | 20 |
| `납기` | 계약 | search+date | 20 |
| `계약기간` | 계약 | search+date | 20 |
| `하자보수` | 계약 | search | 20 |
| `지체상금` | 계약 | search+amount | 20 |
| `특약` | 계약 | search | 20 |
| `목차` | 장문 | export-structure | — |
| `제1조` | 법령 | structure clause | — |
| `제1항` | 법령 | structure 아님 auto증거 | — |
| `붙임` | 공문 | search | 10 |
| `결재` | 공문 | search | 20 |
| `수신` | 공문 | search | 10 |
| `발신` | 공문 | search | 10 |
| `경유` | 공문 | search | 10 |
| `참조` | 공문 | search | 10 |
| `시행문` | 공문 | search | 10 |
| `직인` | 시각 | png 매치쪽 | 3 |
| `도장` | 시각 | png 매치쪽 | 3 |
| `서명` | 시각/검색 | search 후 png | 5 |
| `회의일시` | 회의록 | extract-data date | 20 |
| `참석자` | 회의록 | search | 20 |
| `안건` | 회의록 | search+structure | 20 |
| `의결` | 회의록 | search | 20 |
| `수당` | 행정 | extract-data amount | 30 |
| `여비` | 행정 | extract-data amount | 30 |
| `출장` | 행정 | search | 20 |
| `휴일` | 행정 | search | 20 |
| `근무시간` | 행정 | search | 20 |
| `연가` | 행정 | search | 20 |
| `병가` | 행정 | search | 20 |
| `교육` | 행정 | search | 20 |
| `평가` | 계획 | search | 20 |
| `성과` | 계획 | search | 20 |
| `KPI` | 계획 | search --ignore-case | 20 |
| `ISO` | 매뉴얼 | search --ignore-case | 20 |
| `HWPX` | 형식 | info format | — |
| `비밀번호` | 암호 | info/explain encrypted | — |
| `각주` | 논문/편람 | explain.footnoteCount | — |
| `미주` | 논문 | explain.endnoteCount | — |
| `표1` | 보고서 | explain.tables | — |
| `그림1` | 보고서 | search 후 png | 5 |
| `참고문헌` | 논문 | search 또는 --pages 뒤쪽 | 20 |
| `부록` | 장문 | export-structure + --pages | 20 |
| `색인` | 편람 | search | 20 |
| `정의` | 규정 | search+structure | 20 |
| `목적` | 규정 | structure heading | — |
| `적용범위` | 규정 | search | 20 |
| `용어` | 규정 | search | 20 |
| `벌칙` | 법령 | search | 20 |
| `과태료` | 법령 | search+amount | 20 |
| `서식` | 행정 | explain.fields | — |
| `신청인` | 서식 | form-fill | — |
| `생년월일` | 서식/PII | security+date | 20 |
| `전화번호` | 서식/PII | security-sweep | — |
| `주소` | 서식/PII | security-sweep | — |
| `이메일` | 서식/PII | security-sweep | — |
| `사업자등록` | 서식 | search | 10 |
| `법인등록` | 서식 | search | 10 |
| `담당자` | 공문 | search | 20 |
| `연락처` | 공문 | security-sweep | — |
| `기한` | 행정 | search+date | 20 |
| `제출` | 행정 | search | 20 |
| `접수` | 행정 | search | 20 |
| `반려` | 행정 | search | 20 |
| `보완` | 행정 | search | 20 |
| `승인` | 행정 | search | 20 |
| `반려사유` | 행정 | search | 20 |
| `근거법령` | 행정 | search+structure | 20 |
| `관련문서` | 행정 | search | 20 |
| `버전` | 매뉴얼 | info+search | 10 |
| `개정` | 매뉴얼 | search+date | 20 |
| `폐지` | 법령 | search | 20 |
| `경과조치` | 법령 | search | 20 |

## 동의어 1회 재시도

| 원 질의 | 1차 대체 |
| --- | --- |
| 위임전결 | 전결 |
| 시행일 | 시행 / 시행일자 |
| 예산 | 세출 / 세입 |
| 납기 | 납품기한 / 완료일 |
| 담당자 | 책임자 / 주무 |
| 기한 | 마감 / 제출기한 |
| KPI | 성과지표 / 실적 |

두 번째에도 0건이면 S08 — 없다고 답하고 멈춘다.

## 질의별 주의

- `위임전결` (행정 편람): search, limit=20. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `전결규정` (행정 편람): search, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `시행일` (법령·고시): search+date, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `부칙` (법령): search+structure, limit=20. 영문 약어는 --ignore-case.
- `별표` (법령): search, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `별지` (서식): search, limit=20. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `누름틀` (서식): explain.fields, limit=n/a. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `개인정보` (배포 전): security-sweep, limit=n/a. 폴더 질의는 batch search --query.
- `주민등록` (PII): security-sweep, limit=n/a. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `계좌` (PII/금액): security+amount, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `예산` (업무계획): extract-data amount, limit=50. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `백만원` (예산서): extract-data amount, limit=50. 영문 약어는 --ignore-case.
- `억원` (예산서): extract-data amount, limit=50. 질의가 -로 시작하면 -- 뒤에 둔다.
- `천원` (예산서): extract-data amount, limit=50. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `원정` (공문): extract-data amount, limit=20. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `금113` (영수증 표기): extract-data amount, limit=20. 폴더 질의는 batch search --query.
- `과업` (용역): search, limit=20. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `납기` (계약): search+date, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `계약기간` (계약): search+date, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `하자보수` (계약): search, limit=20. 영문 약어는 --ignore-case.
- `지체상금` (계약): search+amount, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `특약` (계약): search, limit=20. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `목차` (장문): export-structure, limit=n/a. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `제1조` (법령): structure clause, limit=n/a. 폴더 질의는 batch search --query.
- `제1항` (법령): structure 아님 auto증거, limit=n/a. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `붙임` (공문): search, limit=10. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `결재` (공문): search, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `수신` (공문): search, limit=10. 영문 약어는 --ignore-case.
- `발신` (공문): search, limit=10. 질의가 -로 시작하면 -- 뒤에 둔다.
- `경유` (공문): search, limit=10. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `참조` (공문): search, limit=10. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `시행문` (공문): search, limit=10. 폴더 질의는 batch search --query.
- `직인` (시각): png 매치쪽, limit=3. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `도장` (시각): png 매치쪽, limit=3. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `서명` (시각/검색): search 후 png, limit=5. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `회의일시` (회의록): extract-data date, limit=20. 영문 약어는 --ignore-case.
- `참석자` (회의록): search, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `안건` (회의록): search+structure, limit=20. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `의결` (회의록): search, limit=20. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `수당` (행정): extract-data amount, limit=30. 폴더 질의는 batch search --query.
- `여비` (행정): extract-data amount, limit=30. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `출장` (행정): search, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `휴일` (행정): search, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `근무시간` (행정): search, limit=20. 영문 약어는 --ignore-case.
- `연가` (행정): search, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `병가` (행정): search, limit=20. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `교육` (행정): search, limit=20. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `평가` (계획): search, limit=20. 폴더 질의는 batch search --query.
- `성과` (계획): search, limit=20. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `KPI` (계획): search --ignore-case, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `ISO` (매뉴얼): search --ignore-case, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `HWPX` (형식): info format, limit=n/a. 영문 약어는 --ignore-case.
- `비밀번호` (암호): info/explain encrypted, limit=n/a. 질의가 -로 시작하면 -- 뒤에 둔다.
- `각주` (논문/편람): explain.footnoteCount, limit=n/a. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `미주` (논문): explain.endnoteCount, limit=n/a. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `표1` (보고서): explain.tables, limit=n/a. 폴더 질의는 batch search --query.
- `그림1` (보고서): search 후 png, limit=5. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `참고문헌` (논문): search 또는 --pages 뒤쪽, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `부록` (장문): export-structure + --pages, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `색인` (편람): search, limit=20. 영문 약어는 --ignore-case.
- `정의` (규정): search+structure, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `목적` (규정): structure heading, limit=n/a. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `적용범위` (규정): search, limit=20. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `용어` (규정): search, limit=20. 폴더 질의는 batch search --query.
- `벌칙` (법령): search, limit=20. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `과태료` (법령): search+amount, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `서식` (행정): explain.fields, limit=n/a. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `신청인` (서식): form-fill, limit=n/a. 영문 약어는 --ignore-case.
- `생년월일` (서식/PII): security+date, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `전화번호` (서식/PII): security-sweep, limit=n/a. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `주소` (서식/PII): security-sweep, limit=n/a. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `이메일` (서식/PII): security-sweep, limit=n/a. 폴더 질의는 batch search --query.
- `사업자등록` (서식): search, limit=10. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `법인등록` (서식): search, limit=10. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `담당자` (공문): search, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `연락처` (공문): security-sweep, limit=n/a. 영문 약어는 --ignore-case.
- `기한` (행정): search+date, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `제출` (행정): search, limit=20. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `접수` (행정): search, limit=20. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `반려` (행정): search, limit=20. 폴더 질의는 batch search --query.
- `보완` (행정): search, limit=20. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
- `승인` (행정): search, limit=20. 날짜+어휘가 같이 있으면 extract-data date와 search를 한 번씩만.
- `반려사유` (행정): search, limit=20. PII 질의는 값을 프롬프트에 반복하지 않고 security-sweep으로 넘긴다.
- `근거법령` (행정): search+structure, limit=20. 영문 약어는 --ignore-case.
- `관련문서` (행정): search, limit=20. 질의가 -로 시작하면 -- 뒤에 둔다.
- `버전` (매뉴얼): info+search, limit=10. 조문 번호는 extract-data number가 잡지 않는다. search 또는 structure.
- `개정` (매뉴얼): search+date, limit=20. 시각 단서(직인·도장)는 텍스트 0건이 정상일 수 있다. 매치 쪽 png.
- `폐지` (법령): search, limit=20. 폴더 질의는 batch search --query.
- `경과조치` (법령): search, limit=20. 금액 질의는 search보다 extract-data가 싸다. 정규식이 필요 없다.
