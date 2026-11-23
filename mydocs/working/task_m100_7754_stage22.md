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
[원 PR과 통합 검증 기록](../pr/semanticist21_20261123_table_notes_shadows_review.md)을 대조한다.

두 브랜치 끝에서 추가된 독립 인쇄 쪽수 행은 원본 경로가 서로 달랐다.
`oracle_page_count_baseline.tsv`의 기존 행을 보존하고 #9335의 원본 17종과
현재 후보의 원본 6종을 합쳤다. 같은 경로가 중복되지 않는지 확인했다.
자동 적용된 `paint/builder.rs`와 `renderer/pagination.rs`의 그림자 변환,
글상자 미주 수집 경로는 양쪽 변경의 소비 위치를 대조한다.

Studio의 TypeScript 선언 두 파일에서 충돌이 발생했고 자동 적용된 JavaScript에도
같은 빌드의 생성물이 필요하다. 통합 소스 `3df7e740a8f9be501d1e24d6c9411befccdf44a3`에서
fresh WASM을 빌드해 JavaScript와 선언 파일 두 개를 함께 갱신한다.

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
