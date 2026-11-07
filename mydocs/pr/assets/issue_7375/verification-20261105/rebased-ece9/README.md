# 리베이스 후 #7375 검증 — 2026-11-05

새 base는 `ece9c3383bb2c73e0f055459ede727a9d3101ac6`다. validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/rebased-ece9/validation.json`)의 빌드 source·검사 source·기록 head를 구분한다. 이후 지침 변경 커밋은 제품 소스가 같으며, 앞 디렉터리의 c72 결과는 리베이스 전 이력이다.

- Native/fresh WASM 전 50쪽 최저 98.31507%/98.32714%, 미달·누락·쪽수 차이 0. [Native TSV](native.tsv)·[WASM TSV](wasm.tsv).
- 리베이스 전후 각 50쪽 PNG가 모두 SHA 단위로 같다. 페이지별 원값·해시 비교 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/rebased-ece9/pre-post-rebase.json`).
- Mac 프로세스 격리 전체 회귀 12,634 PASS·0 FAIL·45 SKIP. 모든 lint/build·Skia·PNG/PDF·manifest 통과. Studio 2,741 PASS·2 기존 SKIP·build 통과.
- 같은 격리에서 실제 27·28쪽 도형 선택·이동·삭제·두 Undo 단축키와 기존 연결선 드라이버가 통과했다. 한국어 UI/본문·번호와 원본 위치 복원을 직접 판독했다. 실제 소비 결과 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/rebased-ece9/browser-edit-validation.json`)·격리 범위 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/rebased-ece9/macos-font-isolation.json`).
- 편람 Native 383쪽 gate 통과, 기준 하락 0. [TSV](handbook-native.tsv)·gate (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/rebased-ece9/handbook-native-gate.json`).

대표 raw/review/overlay는 `native-p027-*`, `native-p028-*`, `wasm-p027-*`, `wasm-p028-*`다. 표·번호 도형·삽입 그림·본문 소속을 같은 영역에서 직접 대조했다. `minimal-font-studio-*`는 대체 글꼴 환경의 실제 편집 확인이며 원 글꼴 Print 피델리티 증거와 구분한다.

이 결과만으로 새 devel의 모든 기능 보존이나 PR 준비 완료를 선언하지 않는다. #8640·#8637은 새 base와 후보의 대응 경로 검증을 진행한다. 같은 브랜치에서 #7640 이후의 오래된 이슈를 차례로 보정하며, #7375만의 PR로 종료하지 않는다.
