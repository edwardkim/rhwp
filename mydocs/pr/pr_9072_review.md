# PR #9072 메인테이너 검토

## 2026-11-15 최종 로컬 판정

**메인테이너 보완 후 수용 가능.** 기존 보호값·영문 스타일·그림 외곽의 차이를 예외 없이 보완했다.
제품 source는 `8ad1c5c08a`, 최신 base는 `2d83fa4c8a`이며 실제 비교용 `e7ee272cfa`와 제품·시각 harness가 같다.
전체 회귀 13,608건·Native 공식 4,073건·모든 lint와 필수 편람 gate를 통과했다.
Native/fresh WASM 보호 각 1,771쪽·3,542행의 공식값 하락은 0건이며 상승 19행을 새 하한에 반영했다.
원 사례·대조군 각 134쪽의 최저는 93.31%·92.35%다. 직접 확대 판독·최신 base 전쪽 비교도 완료했다.
[최종 검증·생산자·전쪽 TSV](assets/jinho-von-choi-20261114/final-source-8ad/README.md).
원격 push·새 PR·공개 review/comment·merge·issue close는 아직 수행하지 않았다.

작성자 JinHo-von-Choi, 원 head `d268b792eda1aa678ac5d67d285455f6760a2754`. reviewer `edwardkim`이며 원 commit을 보존했다. 이슈 #7381·#7711·#8083을 [검토 계획](jinho_von_choi_20261114_review_plan.md)의 maintainer 일반 누적 경로로 검토했다.

## 실제 계약과 실행

- **#7381 도움말:** replace-text usage를 공통 문자열로 만들고 occurrence의 0 기준·생략 시 전체 계약을 맞춘다. scoped help의 실제 legacy 옵션과 capability 요약을 구분했다. 실제 `rhwp edit replace-text --help`, JSON·치환 저장을 확인했다. 기존 #9052의 CLI 편집 계약은 유지한다. 선택적 제안인 전체 옵션 블록 통합을 구현했다고 주장하지 않는다.
- **#7711 커서 주소:** 본문 FieldInfo는 앞 구역의 문단 수를 누적한 전역 주소를 만들고 생성 API도 이를 구역·문단으로 해소한다. 중첩 리스트의 로컬 주소와 host 구역 로컬 주소는 따로 보존한다. field-01의 3구역·16문단에서 제목은 구역 2·로컬 0·전역 13이다. 실제 최적화 WASM Chrome에서 상태 0x12, 세 구역·중첩 셀 생성과 HWP/HWPX 각 2회 저장·재열기 16개 비교, 전역 16의 거부·무변경을 확인했다.
- **#8083 텍스트:** CLI export-text가 실제 render tree의 TextLine 밖 TextRun·수식·각주·양식 텍스트를 순서대로 모은다. 같은 소유·셀·방향·기준선의 fragment만 같은 줄로 묶고 첫 body run을 기준으로 줄 소속을 정한다. 실제 5개 TAC 원본의 원문·순서·단일 출현, 크기·굵기·첨자 대조를 확인했다.

[실제 CLI 실행·텍스트](assets/jinho-von-choi-20261114/final-source-8ad/cli-text-help.json) · [fresh WASM API 실행·프로그램 해시](assets/jinho-von-choi-20261114/final-source-8ad/typed-frame-wasm-functional-repeated-result.json).

WASM getPageText는 웹 컨트롤의 IR 본문 JSON 문자열 계약이고 CLI export-text는 셀을 포함한 render-tree 줄 추출 계약이다. 두 결과가 같아야 한다고 비교했던 초기 진단은 성립하지 않으며 회귀 근거로 사용하지 않는다. 실제 필드 주소·저장·텍스트 보존을 각 API 계약으로 검증했다.

## 최종 공통 검증과 다음 조건

[최종 원장](assets/jinho-von-choi-20261114/final-source-8ad/README.md)의 17개 필수 단계가 모두 통과했다.
정본 보호의 하락·누락은 0행이며 수용 예외를 적용하지 않았다. 최신 devel의 추가 문서 389쪽과
RowBreak 14쪽씩도 실제 source로 대조했다. 원 사례와 추가 문서의 2%p 초과 감소는 모두 0쪽이다.
감소 영역은 원본 Print·base·후보를 같은 배율로 직접 읽고 실제 export의 bbox·내용·순서를 확인했다.

실행 입력·독립 PDF 118개는 실제 SHA-256과 Git blob을 대조했고 미커밋 입력은 0개다.
초기 잘못된 필터 호출·보안 입력 전달·중간 source의 미달은 [조사 이력](assets/jinho-von-choi-20261114/protected-environment-review.md)에
보존했다. 과거 실패를 지우거나 과거 통과 수를 최종 검증에 합산하지 않는다.
현재 다음 조건은 준비한 upstream 수용 PR 경로의 원격 승인, 해당 최신 head CI·self-review·merge다.
원 기여 SHA와 fork branch를 보존하고 연결 이슈는 각 검증 범위에 맞게 후속 처리한다.
