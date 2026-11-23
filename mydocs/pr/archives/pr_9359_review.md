# PR #9359 보호문서 검증의 과거 증적 정리

## 최종 검토

`issue5140-version-boundary`와 `issue7754-nine-fidelity`는 #9280과 #9350의 당시 출력 및 검토 증적이었다. 현재 Visual Sweep의 편람·공통 검사와 정본 보호 검증은 정본 TSV, 측정 조건 manifest 및 공식 글꼴 환경을 사용한다. 과거 증적 경로 두 곳의 581개 파일을 현재 트리에서 제거했고, 검토 문서와 글꼴 재현 기록은 SHA 고정 증적으로 연결했다. [PR #9359](https://github.com/edwardkim/rhwp/pull/9359) · [이전 증적 커밋](https://github.com/edwardkim/rhwp/commit/17b17c934b527aa523f037ca12d8713394e43c82)

## 검증

- 삭제한 트리에서 편람 Native·WASM 기준 각 383쪽, 공통 보호 원본 4종, 정본 TSV 4,046행, 공식 실행 글꼴 환경 JSON 13개의 로딩과 해시를 확인했다. 글꼴 공급 manifest의 차이는 재현 기록 해시 46곳이었다.
- SHA에 고정한 Native·WASM 비교 JSON 두 파일을 GitHub에서 내려받아 기존 기록의 SHA-256과 대조했다. Markdown 링크와 `git diff --check`가 통과했다.
- 편람과 기여자 공통 보호 검사 단위 테스트는 각각 21건과 9건이 통과했다. 검증 범위는 기존 기준 자료를 읽는 절차이며 렌더링을 다시 산출하지 않았다.

## GitHub CI

삭제 후보 `f7b1260d39f9afce5528b52e546f74bfef183f2a`의 CI preflight는 `all-review-only-no-code-impact`를 선택했다. 전체 체크 중 성공 11건, 건너뜀 22건이었고 Build & Test와 CI Impact Policy가 성공했다. 이 문서를 포함한 최종 head의 검사 결과를 확인한 뒤 병합한다. [후보 CI](https://github.com/edwardkim/rhwp/actions/runs/45144715111)
