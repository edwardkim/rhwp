# PR #9335 병합 뒤 후보 동기화와 기능 보존 검증

[PR #9334](https://github.com/edwardkim/rhwp/pull/9334)의 기존 후보
`ddd62aec1fa3e58f20591dc3f3fb78b3d61c9915`에서
[Build & Test](https://github.com/edwardkim/rhwp/actions/runs/45086987257/job/137419358770)와
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45086987252)가 통과했다.
이 결과의 기준은 `93e861523623bcd84d57b808edb62af5af2aa82a`다.

## 동기화한 범위

[PR #9335](https://github.com/edwardkim/rhwp/pull/9335)가 병합된
`df263432be6135cdbf624dfc691d4c6367a47bfa`를 기준으로 삼았다.
표 편집과 각주·미주 경로, 평행 그림자 및 인라인 도형 변경이 들어왔다.
[원 PR과 보존된 통합 검증 기록](https://github.com/edwardkim/rhwp/blob/7d6628982c81e702d58d8d0e6f52059f68a0b9ae/mydocs/pr/archives/semanticist21_20261123_table_notes_shadows_review.md)을 대조한다.

두 브랜치 끝에서 추가된 독립 인쇄 쪽수 행은 원본 경로가 서로 달랐다.
`oracle_page_count_baseline.tsv`의 기존 행을 보존하고 #9335의 원본 17종과
현재 후보의 원본 6종을 합쳤다. 같은 경로가 중복되지 않는지 확인했다.
자동 적용된 `paint/builder.rs`와 `renderer/pagination.rs`의 그림자 변환,
글상자 미주 수집 경로는 양쪽 변경의 소비 위치를 대조한다.

Studio의 TypeScript 선언 두 파일에서 충돌이 발생했고 자동 적용된 JavaScript에도
같은 빌드의 생성물이 필요하다. 통합 소스 `3df7e740a8f9be501d1e24d6c9411befccdf44a3`에서
fresh WASM을 빌드하고 JavaScript와 선언 파일 두 개를 대조했다. 선언 파일 두 개의
함수 인자 표기가 바뀌었다. JavaScript는 #9335의 생성본과 바이트가 같았다.
공개 메서드 614개의 선언과 실행 시 노출을 확인했고, 양쪽의 기존 선언과 #9335가 추가한
메서드 다섯 개가 모두 보존됐다.

표 셀과 글상자에 각주·미주를 삽입해 번호와 내용을 읽었다. 저장 전과 HWP·HWPX
재열기에서 번호 2와 3, 내용, 쪽수 및 미주 소유 경로가 일치했다. 미주 안
캐럿은 같은 줄에서 글자 순서대로 전진했다.
[후보 WASM 해시·공개 API·주석 검증](../verification/issue7754-nine-fidelity/after-pr9335/wasm-candidate-api-and-notes.json)

## Studio 실행 검증

프로덕션 빌드가 통과했다. 격리한 Studio 작업본에 공용 셰이더와 렌더러 계약 문서,
Vite의 저장소 쪽 자료 6개와 공개 쪽 자료 5개를 현재 Git 트리의 바이트대로 준비했다.
같은 WASM 생산본으로 Node 바인딩도 생성했다. 두 Node 전용 검사를 포함한 Studio 테스트
2,834개와 렌더러 계약 검사가 모두 통과했다.

#9335에 기록된 브라우저 검사 5개를 다시 실행했다. 중첩 표 포인터와 개체 선택,
표 경계 드래그, 각주 단축키와 삽입 검사가 모두 통과했다. 각 검사에서 브라우저가
받은 WASM의 SHA-256이 후보 패키지와 일치했다.
[Studio 빌드·단위 테스트·브라우저 검증](../verification/issue7754-nine-fidelity/after-pr9335/studio-local-validation.json)

## fresh WASM 기준·후보 편집 대조

기준 WASM 빌드는 컴파일러가 쓰는 `/tmp`의 공간 부족으로 한 차례 종료됐다.
후보의 완성된 Native·WASM 산출물 해시를 대조해 유지했고, 기준은 별도 임시 파일시스템으로
빌드해 통과했다. 기준과 후보의 Native 실행 파일과 최적화한 fresh WASM 패키지, 총 네
산출물의 해시를 확인했다.

양쪽 fresh WASM이 공개 메서드 614개를 같은 선언과 함께 제공했다. 같은 표 셀·글상자
각주와 미주를 편집해 저장 전, HWP 재열기, HWPX 재열기의 쪽수·번호·텍스트와 미주
소유 경로를 비교했다. 용역내역서 HWP·HWPX의 원본, 행 삽입, 열 삽입을 각각 대조했고,
양쪽에서 12건씩 재열어 병합 셀 위치와 너비가 보존되는 것을 확인했다.
[기준·후보 WASM 편집 검증](../verification/issue7754-nine-fidelity/after-pr9335/wasm-paired-api-and-editing.json)

## 보호 정본 글꼴 조건 대조

공식 목록의 Native·WASM 46개 조건을 실행 명령과 대조했다. 글꼴 파일을 명시하는
37개 조건에서 파일의 SHA-256과 공급 순서가 일치했다. 문서가 글꼴을 선택하는 9개
조건의 공급 방식과 전체 조건에 포함된 환경 JSON 21개의 SHA-256도 확인했다.
실제 전쪽 렌더의 하락과 누락은 실행 결과에서 확인한다.
[공식 공급 목록](../verification/protected-font-supply/manifest.json)

## 후보 Native 쪽별 검증

아홉 이슈 대상 583쪽과 보조 사례 50쪽을 독립 한컴 Print에 대조했다.
43개 조건의 633쪽에서 PNG 해시와 TSV의 쪽별 값이 [직전 후보 결과](task_m100_7754_stage21.md)와
모두 일치했다. 이슈 대상 최저는 98.08%이며 389쪽이 100.00%였다. 보조 사례의
최저는 98.07%이며 42쪽이 100.00%였다. 시장구조조사 315쪽을 모두 확인했다.
제3절 제목이 있는 100쪽과 부록 B-2가 시작하는 247쪽은 100.00%, 부록 직전인
246쪽은 99.97%였다.

## 검증 계획

- 같은 조건의 Native·fresh WASM에서 아홉 이슈 원본 583쪽과 보조 검증 50쪽을
  독립 Print에 대조한다. [이전 후보의 전쪽 결과와 확대 판독](task_m100_7754_stage21.md)을
  새 결과와 함께 확인한다.
- #9335의 공개 입력 17종·19쪽을 양쪽 렌더러에서 대조한다. 표 편집, 셀과 글상자의
  주석 저장·재열기, 그림자 및 인라인 도형 검증을 원 수용 근거에 연결한다.
  [원 수용 결과](../pr/assets/semanticist21-20261123-table-notes-shadows/focus-pages.tsv)
- 같은 공급 조건으로 보호 정본에서 하락과 누락 여부를 확인한다.
  [공식 공급 조건](../verification/protected-font-supply/manifest.json)
- 현재 후보에서 집중 회귀, 전체 Rust와 Native Skia, Studio 빌드 및 실행 검증을 확인한다.
  두 소스가 각각 변경한 쪽수 및 새 WASM API의 소비자 경로를 함께 검증한다.
