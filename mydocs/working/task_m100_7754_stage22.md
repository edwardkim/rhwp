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

## 기준과 후보의 Native 쪽별 검증

아홉 이슈 대상 583쪽과 보조 사례 50쪽을 독립 한컴 Print에 대조했다.
43개 조건의 633쪽에서 PNG 해시와 TSV의 쪽별 값이 [직전 후보 결과](task_m100_7754_stage21.md)와
모두 일치했다. 이슈 대상 최저는 98.08%이며 389쪽이 100.00%였다. 보조 사례의
최저는 98.07%이며 42쪽이 100.00%였다. 시장구조조사 315쪽을 모두 확인했다.
제3절 제목이 있는 100쪽과 부록 B-2가 시작하는 247쪽은 100.00%, 부록 직전인
246쪽은 99.97%였다.

현재 기준의 633쪽도 직전 기준과 PNG 해시 및 쪽별 수치가 모두 일치했다.
같은 한컴 Print를 기준으로 현재 후보에서 73쪽의 일치율이 올랐고 하락은 없었다.
537쪽의 기준과 후보 PNG가 일치했다. 시장구조조사의 3쪽은 95.99%에서 99.40%로,
100쪽은 26.80%에서 100.00%로 개선됐다.

## 고정 좌표 회귀 대상 재검증

현재 후보의 Native와 fresh WASM에서 좌표 회귀를 적용한 다섯 쪽을 다시 확인했다.
암호 HWPX의 평문 대조군 2쪽은 각 99.99%였다. 규제분석 HWP 10·28·29쪽과
HWPX 26쪽은 양쪽 모두 100.00%였다. 독립 한컴 Print와 후보의 PNG 해시를
[전쪽 출력의 확대 판독 증거](../verification/issue7754-nine-fidelity/after-pr9322/coordinate-pages/validation.json)와
대조했으며 모두 일치했다.

## 보호 정본의 후보 Native 전쪽 검증

행정업무운영 편람 HWP와 HWPX를 각각 383쪽 전부 검증했다. 두 형식 모두 최저
99.97%였고, 쪽별 공식 하한 미달은 없었다. HWPX에서는 382쪽이 100.00%였다.
두 형식 전체의 PNG와 쪽별 값이 직전 후보와 일치했다.

보호 정본 23종의 후보 Native 2,023쪽을 모두 확인했다. 누락과 공식 하한 미달은
없었고, 2,023쪽의 PNG와 쪽별 값이 직전 후보와 일치했다. 전체 최저는 98.86%였다.
`sample10`은 763쪽 전체를 비교했으며 최저 99.57%, 100.00% 741쪽이었다.
같은 공급 조건에서 기준 Native를 전쪽 렌더링 중이다.

## #9335 원 수용 입력과 #7754 암호 문서

#9335 원본 17종의 19쪽을 Native와 fresh WASM에서 대조했다. 각 경로에서 기준과
후보의 PNG가 전쪽 일치했다. 독립 한컴 Print에 대한 전쪽 최저는 Native 99.59%,
fresh WASM 99.37%였다.

셀 주석 HWP 1쪽은 원 수용 기록에서 Native 99.65%, WASM 99.68%였다. 현재
서버에서는 기준과 후보가 Native 99.59%, fresh WASM 99.37%로 각각 같았다.
나머지 18쪽의 일치율은 경로별로 원 수용 기록과 일치했다.

Native의 기준과 후보 각각에서 #7754 암호 HWPX와 같은 환경의 평문 대조군을
다시 출력했다. 두 입력의 PNG는 24쪽 모두 일치했다. 독립 한컴 Print 대비 전쪽
최저는 기준 98.17%, 후보 99.48%였다. 별도 암호 검증과 전체 평문 실행의
10쪽 기호 차이는 [직전 검증](task_m100_7754_stage21.md)과 같은 해시로 재현됐고,
독립 Print에 대한 24쪽의 쪽별 실루엣 값도 일치했다.

## Rust 로컬 검사

현재 후보의 최소 글꼴 환경에서 Native Skia 기능을 켠 전체 라이브러리 검사
4,033개가 통과했다. 기존에 제외된 검사는 12개였다. 회귀 검사 자동 수집과
라이브러리 분류 검증, rustfmt, 집중 검사도 통과했다.

Rust 검증 46단계가 통과한 뒤 WASM Clippy가 렌더링과 겹치는 동안 메모리 부족으로
종료됐다. 컴파일 소스와 검증 환경, 명령, 통과 로그의 해시를 대조해 46단계의
결과를 유지했다. 완료한 렌더링 증거도 보존했다. 남은 WASM Clippy, 워크스페이스
빌드, 전체 Nextest는 렌더링을 잠시 멈춘 상태에서 다시 실행한다.

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
