# 발화 행렬

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

발화 → 정지 규칙. 명령 칸이 '없음'이면 새 명령을 만들지 말고
문서/재측정으로 닫는다.

| ID | 발화 | 정지 |
| --- | --- | --- |
| I001 | 지식 지도부터 읽어 | R01 |
| I002 | llms.txt 다음 문서 | R01 |
| I003 | 어디에 무엇이 있는지 표 | R01 |
| I004 | 이 필드 이름 뜻이 뭐야 | R03 |
| I005 | schemaVersion 이 뭐야 | R03 |
| I006 | untrustedFields 경로 | R08 |
| I007 | capabilities 다시 찍자 | R02 |
| I008 | MCP 도구 선언 재측정 | R02 |
| I009 | 세션 포함 tools/list | R02 |
| I010 | 지도 last_verified 오래됨 | R04 |
| I011 | 바이너리가 v0.8.4 인데 지도는 v0.8.3 | R05 |
| I012 | 지도와 cli_commands 가 숫자가 다름 | R06 |
| I013 | recordField 이름을 내가 지어 | R07 |
| I014 | 서식 채워줘 | R08 |
| I015 | 표를 CSV | R08 |
| I016 | 배포 전 마스킹 | R08 |
| I017 | 폴더 일괄 추출 | R08 |
| I018 | 레이아웃 숫자 비교 | R08 |
| I019 | MCP 호스트에 붙여 | R08 |
| I020 | 대전 교본 장 순서 | R09 |
| I021 | 3층 계약으로 도구 추가 | R09 |
| I022 | 지식지도 명령 만들어줘 | R11 |
| I023 | gym pack 으로 지도 검증 | R12 |
| I024 | 지도를 처음부터 끝까지 읽어 | R10 |
| I025 | exit 2 가 났어 | R01 |
| I026 | identical false 는 오류인가 | R01 |
| I027 | 페이지 번호 0부터? | R01 |
| I028 | 어떤 샘플로 fields 시험 | R01 |
| I029 | provenance 계약 테스트 | R01 |
| I030 | 온보딩 첫 5분 | R08 |
| I031 | 작업 영수증 발급 | R08 |
| I032 | 안전 편집 dry-run | R08 |
| I033 | 출처 표지 읽기 | R08 |
| I034 | 문서 트리아지 | R08 |
| I035 | 기여하려면 | R08 |
| I036 | 시험지 PDF 를 HWPX 로 | R08 |
| I037 | 메일머지 N행 | R08 |
| I038 | 은닉 텍스트 스윕 | R08 |
| I039 | 세션 hwp_open | R08 |
| I040 | 프로필 행정서식 | R08 |
| I041 | 스키마 코드 생성 | R01 |
| I042 | IR 스키마 | R01 |
| I043 | filledCount 성공인데 빈칸 | R01 |
| I044 | 한글 파일명 깨짐 | R01 |
| I045 | export-png 없다 | R02 |
| I046 | changedPages 로 쪽만 렌더 | R01 |
| I047 | hwp_doc_save 만이 기록? | R01 |
| I048 | 배치 convert 는 MCP 있나 | R01 |
| I049 | null 과 모르겠다 구별 | R03 |
| I050 | recordFields 가 전부인가 | R03 |
| I051 | didYouMean 힌트 | R01 |
| I052 | 명령 가족 query | R09 |
| I053 | 세션 도구 몇 개 | R02 |
| I054 | samples 는 음성 코퍼스? | R01 |
| I055 | redact 가 잡는 표본 | R01 |
| I056 | cli_json_contract 가 고정하는 것 | R01 |
| I057 | 보안 축 계약 테스트 | R01 |
| I058 | 문서 축 권위표 | R01 |
| I059 | 지도 행을 더 자세히 풀어써 | R13 |
| I060 | 지도 숫자 손으로 고치자 | R14 |
| I061 | 첫 문서로 ROADMAP 을 읽자 | R01 |
| I062 | 필드 사전을 내가 암기한 이름으로 | R07 |
| I063 | replacedCount 0 은 실패? | R03 |
| I064 | notFound 는 isError 인가 | R03 |
| I065 | overflow 필드 | R03 |
| I066 | ambiguous 필드 | R03 |
| I067 | matchCount 필드 | R03 |
| I068 | findingCount 필드 | R03 |
| I069 | hiddenCharCount 필드 | R03 |
| I070 | verify.identical 필드 | R03 |
| I071 | docId 필드 | R03 |
| I072 | closed 필드 | R03 |
| I073 | profile.recipe 필드 | R01 |
| I074 | missingAxes 필드 | R01 |
| I075 | 에이전트 매니페스트 | R01 |
| I076 | capabilities --search 검색 | R02 |
| I077 | 지도와 대전이 필드 정의가 다름 | R06 |
| I078 | 표면 플레이북 수용 기준 | R09 |
| I079 | 레시피 07 을 지도에 추가 | R13 |
| I080 | 실패 증상 검색 | R01 |
| I081 | 선검사 스크립트 | R01 |
| I082 | JSON 파이프라인 배치 | R08 |
| I083 | 서식 함정 심화 | R08 |
| I084 | 경계 계약 | R01 |
| I085 | 위협 모델 | R08 |
| I086 | 지도 유지 규약 | R14 |
| I087 | 링크 검사 어떻게 | R01 |
| I088 | HWPUNIT 좌표 | R01 |
| I089 | 이름[N] 반복 필드 | R08 |
| I090 | 병합 칸 앵커 | R08 |
| I091 | pagesAfter 가 범위와 다름 | R01 |
| I092 | structuredContent 없는 도구 | R01 |
| I093 | hwp_batch 는 NDJSON | R01 |
| I094 | 닫힌 핸들 재사용 | R08 |
| I095 | 387쪽 세션 이득 | R01 |
| I096 | 비밀번호 stdin | R01 |
| I097 | 개발통합 프로필 | R08 |
| I098 | 없는 프로필 이름 | R01 |
| I099 | 바인딩이 철회됐다는데 | R01 |
| I100 | 첫 5분 레시피 지도 | R08 |
| I101 | inspect 3축 | R08 |
| I102 | sanitize 메타 제거 | R08 |
| I103 | insert-image 도장 | R08 |
| I104 | thumbnail 미리보기 | R01 |
| I105 | export-structure 목차 | R08 |
| I106 | word-count | R01 |
| I107 | bookmarks | R01 |
| I108 | charts 목록 | R01 |
| I109 | threat-scan | R08 |
| I110 | layout-anomaly | R01 |
| I111 | audit 재현율 | R08 |
| I112 | lineage 계보 | R08 |
| I113 | keygen 서명 | R01 |
| I114 | gate 정책 | R01 |
| I115 | bundle 반출 | R01 |
| I116 | disclose 선택 공개 | R01 |
| I117 | settle 정산 | R01 |
| I118 | conformance 수준 | R01 |
| I119 | scan 한 파일 | R01 |
| I120 | dump-pages 조판 | R01 |
