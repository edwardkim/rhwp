# Task #7949·#7950 — 19쪽 전체 열기와 반복 해석 보정

- Issue: #7949, #7950
- 작성일: 2026-10-16 (Asia/Seoul)
- 제품 source: `1c426c5dc`, 검사 형식 정리 head: `43fddb437`
- 상태: 전체 열기·마스크 자원 계약 통과. 새 PDF의 시각 게이트는 미충족

동일 페이지의 텍스트·선·그림 소비자가 한 번 해석한 연산열을 재사용한다.
캐시는 요청 내 한 페이지로 제한하고 각 소비자의 owned 복사 비용은 계속 계상한다.
중첩 Form·외부 임의 스트림은 페이지 출처를 추정하지 않고 별도 해석한다.
Raw PNG에 존재하지 않는 decode/변환 예약과 실제로 만들지 않는 이미지 임시 버퍼의
예약을 제거했다. 원래 256MiB 누적·32MiB 개별 버퍼와 연산·sticky 오류 한도는 유지한다.

전쪽 변환을 막던 signed page offset·HWP 좌표 양자화와 선택적 표 추론 한도를 보정했다.
표 추론 한도 도달은 추론만 보류하고 원본 positional 내용은 유지한다.

검증 로그는 ignored `output/pr-review/issue7949/`에 보존한다.

| 검증 | 결과·근거 |
| --- | --- |
| 정식 자원 계약 | 112/112 통과, `resource-reuse-final.log`. 8개 연산의 텍스트·선·그림 재사용과 반복 cache hit 예산 소진 포함 |
| 실제 PNG alpha·Matte | 수정 전 9건 모두 `PDF_IMAGE_UNSUPPORTED`, 최종 Native 9/9 통과. `soft-reuse-final.log` |
| 전체 Native import | 원본 19쪽 → HWPX 19쪽, `type002/final-import.json` |
| Docker WASM | 기본 feature 빌드 성공, `docker-wasm-reuse-final.log` |
| 제품 WASM/Worker | 85 assertion·실제 Worker 5개 통과, `wasm-contract-reuse.log` |
| 실제 Studio | 새 WASM SHA와 capability·PDF 형식·19쪽·오류 없음 확인, `type002/studio-result.json` |
| 전쪽 Native/fresh WASM | 각각 19쪽 TSV 산출. 최저 17.63166% / 17.27370%, 2~18쪽 미달 |
| 공개 12쪽 대조군 | 재실행 전쪽 90% 이상, `reuse-native-control/`, `reuse-wasm-control/`. 이전 검증 producer와의 출력 보존이며 현재 base exact-head 검증 전체 완료는 아님 |

입력 SHA·생산 소스 파일 SHA·Native/WASM/JS SHA는 `type002/build-source-final.json`과
`type002/diagnosis.json`에 고정했다. `pkg/`·Studio public의 SHA 및 브라우저 실제 fetch가 같다.
중간 실패 producer와 로그는 최종 성공 증거와 구분해 보존했다.

원본·변환 HWPX·양쪽 TSV·전쪽 PNG 비교 화면:
`/mnt/e/tmp/issue7949-type002-review-20261016/index.html`.
1·19쪽은 약 99%이나, 직접 비교에서 2~18쪽의 도형 채움 누락·문자 paint 차이·18쪽 겹침을
확인했다. 첫 MOF 문서에 승인된 예외는 새 PDF에 적용하지 않는다. 전체 회귀·lint·최종 PR
준비 완료로 보고하지 않으며, remote push·PR 상태 변경은 수행하지 않았다.

**다음 단계:** 사용자가 남은 출력 차이도 이번에 보정하도록 승인했다.
PDF paint 상태·보이는 내용과 추출 텍스트의 차이·도형 경로 lowering을 원본 PDF와 대조하고,
현재 비교 자료를 수정 전 증거로 보존한다. 부분 개선을 시각 게이트 통과로 바꾸어 기록하지 않는다.
