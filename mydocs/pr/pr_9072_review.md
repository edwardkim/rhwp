# PR #9072 메인테이너 검토

**최종 판정: 머지 보류.** 이 PR의 기능 검증에서 미해결 결함은 발견하지 않았지만, 현재 누적 후보의 필수 보호 판정은 아직 완료되지 않았다.

작성자 JinHo-von-Choi, 원 head `d268b792eda1aa678ac5d67d285455f6760a2754`. reviewer `edwardkim`이며 원 commit을 보존했다. 이슈 #7381·#7711·#8083을 [검토 계획](jinho_von_choi_20261114_review_plan.md)의 maintainer 일반 누적 경로로 검토했다.

## 실제 계약과 실행

- **#7381 도움말:** replace-text usage를 공통 문자열로 만들고 occurrence의 0 기준·생략 시 전체 계약을 맞춘다. scoped help의 실제 legacy 옵션과 capability 요약을 구분했다. 실제 `rhwp edit replace-text --help`, JSON·치환 저장을 확인했다. 기존 #9052의 CLI 편집 계약은 유지한다. 선택적 제안인 전체 옵션 블록 통합을 구현했다고 주장하지 않는다.
- **#7711 커서 주소:** 본문 FieldInfo는 앞 구역의 문단 수를 누적한 전역 주소를 만들고 생성 API도 이를 구역·문단으로 해소한다. 중첩 리스트의 로컬 주소와 host 구역 로컬 주소는 따로 보존한다. field-01의 3구역·16문단에서 제목은 구역 2·로컬 0·전역 13이다. 실제 최적화 WASM Chrome에서 상태 0x12, 세 구역·중첩 셀 생성과 HWP/HWPX 각 2회 저장·재열기 16개 비교, 전역 16의 거부·무변경을 확인했다.
- **#8083 텍스트:** CLI export-text가 실제 render tree의 TextLine 밖 TextRun·수식·각주·양식 텍스트를 순서대로 모은다. 같은 소유·셀·방향·기준선의 fragment만 같은 줄로 묶고 첫 body run을 기준으로 줄 소속을 정한다. 실제 5개 TAC 원본의 원문·순서·단일 출현, 크기·굵기·첨자 대조를 확인했다.

[실제 CLI 실행·텍스트](assets/jinho-von-choi-20261114/cli-results.json) · [fresh WASM API 실행·프로그램 해시](assets/jinho-von-choi-20261114/wasm-api-results.json).

WASM getPageText는 웹 컨트롤의 IR 본문 JSON 문자열 계약이고 CLI export-text는 셀을 포함한 render-tree 줄 추출 계약이다. 두 결과가 같아야 한다고 비교했던 초기 진단은 성립하지 않으며 회귀 근거로 사용하지 않는다. 실제 필드 주소·저장·텍스트 보존을 각 API 계약으로 검증했다.

## 공통 검증과 남은 조건

제품 source `eeaecff59e834f28437c98b509089bbb36a7682d`와 검증 head `3b5e750d66351799fe81c0e1b67c5e78323763f5`의 제품 경로는 같다. 최종 전체 회귀 13,574개·실패 0, fmt·root/WASM32/workspace Clippy·workspace build·manifest 검사와 편람 Native 383쪽 gate를 통과했다. Native 공식 3종도 4,073개 모두 통과했다. [최종 실행 기록](assets/jinho-von-choi-20261114/local-validation.json).

Native/fresh WASM 각각 134쪽의 최저 raw는 93.30%·92.46%이고 독립 Print의 전체 쪽수가 일치한다. 새 devel 기능 보존 41쪽씩의 PNG 82쌍은 수정 전후 같다. 반면 정본 보호 3,542행 중 과거 공식값 미재현은 Native 51행·WASM 56행이다. Native는 최신 devel과 후보의 전 1,771행이 같으며 WASM도 exact base 대조를 진행한다. 특별 하락 금지 gate의 통과·예외를 아직 선언하지 않는다. [보호값 재현 조사](assets/jinho-von-choi-20261114/protected-environment-review.md).

초기 집중 60개에는 새 바탕쪽 모듈이 실제로 실행되지 않아 그 통과 주장을 정정했다. 이후 집중 126개와 최종 전체 회귀에서 새 검사 3개의 실제 PASS를 확인했다. `b60322c765`의 두 실패도 최종 source에서 보완·재검증했으며 정확한 기존 기대값은 유지했다. 원 head의 CI와 누적 보완 후보의 CI는 구분한다. 아직 공개 댓글·push·새 PR·승인 review·merge·issue close는 수행하지 않았다.


검증 입력 커밋 확인: **충족**. [113개 입력/PDF의 실제 Git blob·실행 SHA-256 대조](assets/jinho-von-choi-20261114/input-commit-verification.json), 확인 commit `0844c7d115`. 검증한 변환/재저장본도 동일 bytes로 보존했고 상용 글꼴 프로그램은 포함하지 않았다.
