# T07 / #4781 첫 필드 홍길동 복제 금지

첫 필드만 홍길동이어야 한다. 다른 칸에 같은 값을 복사하면 `clone_forbidden`.

| 식별자 | 서식 | 첫 필드 | firstOk | cloneCount | 판정 |
|---|---|---|---|---:|---|
| field-01-hong-first-only | field-01 | 회사명 | True | 0 | pass |
| field-01-hong-clone | field-01 | 회사명 | True | 5 | clone_forbidden |
| field-01-memo-hong-first-only | field-01-memo | 회사명 | True | 0 | pass |
| form-01-hong-first-only | form-01 | myMsg01 | True | 0 | pass |
| gian-1-hong-first-only | gian-1 | 행정기관명 | True | 0 | pass |
| gian-1-hong-clone | gian-1 | 행정기관명 | True | 22 | clone_forbidden |
| gian-2-hong-first-only | gian-2 | 생산등록번호 | True | 0 | pass |
| reg-80168-hong-first-only | reg-80168 | 서식명 | True | 0 | pass |
| hongbo-hong-first-only | hongbo | 기관명 | True | 0 | pass |
| hongbo-hong-clone | hongbo | 기관명 | True | 11 | clone_forbidden |
| trip-apply-hong-first-only | trip-apply | 소속 | True | 0 | pass |
| trip-apply-hong-clone | trip-apply | 소속 | True | 13 | clone_forbidden |
| leave-apply-hong-first-only | leave-apply | 소속 | True | 0 | pass |
| bokhak-hong-first-only | bokhak | 대학 | True | 0 | pass |
| minwon-hong-first-only | minwon | 민원제목 | True | 0 | pass |
| minwon-hong-clone | minwon | 민원제목 | True | 7 | clone_forbidden |
| minutes-hong-first-only | minutes | 회의명 | True | 0 | pass |
| labor-contract-hong-first-only | labor-contract | 사업장명 | True | 0 | pass |
| labor-contract-hong-clone | labor-contract | 사업장명 | True | 9 | clone_forbidden |
| hr-card-hong-first-only | hr-card | 사번 | True | 0 | pass |
| hr-card-hong-clone | hr-card | 사번 | True | 9 | clone_forbidden |
| eval-sheet-hong-first-only | eval-sheet | 평가회차 | True | 0 | pass |
| bid-hong-first-only | bid | 공고번호 | True | 0 | pass |
| bid-hong-clone | bid | 공고번호 | True | 9 | clone_forbidden |
| spend-hong-first-only | spend | 결의번호 | True | 0 | pass |
| foi-hong-first-only | foi | 청구기관 | True | 0 | pass |
| foi-hong-clone | foi | 청구기관 | True | 6 | clone_forbidden |
| overtime-hong-first-only | overtime | 소속 | True | 0 | pass |
| trip-settle-hong-first-only | trip-settle | 출장자 | True | 0 | pass |
| admin-appeal-hong-first-only | admin-appeal | 피청구인 | True | 0 | pass |
| admin-appeal-hong-clone | admin-appeal | 피청구인 | True | 6 | clone_forbidden |
| contract-hong-first-only | contract | 계약명 | True | 0 | pass |
| contract-hong-clone | contract | 계약명 | True | 7 | clone_forbidden |
| parental-leave-hong-first-only | parental-leave | 소속 | True | 0 | pass |
| payroll-account-hong-first-only | payroll-account | 성명 | True | 0 | pass |
| family-cert-hong-first-only | family-cert | 증명종류 | True | 0 | pass |
| family-cert-hong-clone | family-cert | 증명종류 | True | 7 | clone_forbidden |
