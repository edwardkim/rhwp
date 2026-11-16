# PR #9092 UI 검증 증거

- 원 head `c3b0b7191b91e7df57a80da81a43cf632059a609`, 최신 base `d954632aab5b80aadbea39cece4f5a783634856b`, 로컬 병합 `7fbed378bcf3fd2c20f85234610ec094867b2df7`.
- [검토 기록](../../archives/pr_9092_review.md) · [최종 검증과 SHA-256](validation.json).
- [base 800×360](base-800x360.png) · [후보 800×360](candidate-800x360.png) · [후보 360×640](candidate-360x640.png) · [후보 1280×900](candidate-1280x900.png).
- [base 실측](base-browser-review.json) · [후보 6개 화면·터치·실제 메뉴·다른 모달 대조](candidate-browser-review.json).

스크린샷은 같은 Chromium 152와 새 dev WASM을 사용했다. base 대조는 동일 worktree에서 제품 정보 UI 소스 3개만 최신 devel 내용으로 바꾼 뒤 실행하고 모두 복원했다. 문서 조판·한컴 PDF 비교가 아니며 Android Firefox 실기기 검증으로 확대하지 않는다. 임시 실행 로그는 루트 ignored `output/pr-review/pr9092/logs/`에 둔다.
