# Task M100 #4771 Stage 4 — table local resize 경계

- source `Table`의 transient local-resize marker와 morphology inference를 제거했다.
- HWP/HWPX가 독립 행·열 편집 의도를 표현하지 못하므로 `localResize:true`는 mutation 전에 거부한다.
- Studio의 Alt/Shift·균등화 진입점도 WASM 호출 전에 같은 포맷 제약을 사용자에게 표시한다.
- 일반 공유 grid resize와 완결된 저장 cell geometry의 기존 경로는 유지했다.
