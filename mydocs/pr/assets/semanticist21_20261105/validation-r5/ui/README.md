# 현재 WASM의 실제 UI·타입 검증

제품 source `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 wasm-opt 최적화 package를 pkg/public에 동기화했다. [동기화 SHA](package-sync.json)와 [빌드 manifest](browser-build-manifest.json)를 기록하며, 실제 앱 초기화 때 받은 WASM 바이트는 [네트워크 관측](consumption/)에서 일치한다.

- [스타일 모달](style-browser.json): 실제 WASM에 잘못된 삽입 위치를 전달한 -1 반환 때 모달·스타일·undo를 보존한다. 정상 생성의 모달 닫기·이력 추가와 실제 undo 복원도 통과했다. 부정 입력 주입은 소비 경계 시험이며 자연 발생 문서 오류로 표현하지 않는다.
- [새 base PR #8630의 그룹 자식 편집](group-browser.json): 실제 클릭·문자 입력·undo/redo·Esc가 통과했다. [실행 로그](group-browser.txt)를 함께 보존한다.
- [TypeScript·production build](production-build.txt): exit 0. 생성된 public 타입 두 개를 현재 package와 맞췄고 [ABI 검사](public-types-abi.json)에서 각 파일의 631개 실제 WASM export 존재·인수 수를 확인했다. 반환 의미와 화면 동작의 검증을 이 검사로 대신하지 않는다.
- [직접 타입 소비 Node 검사](public-types-consumers.txt): 현재 10건 PASS. 이전 [전체 Node 로그](previous-node-suite.txt)의 2,723 PASS와 [명령](previous-node-command.json), [소스 대조](node-reuse-analysis.json)를 연결한다. UI·Node 검사 소스는 그대로이며 이번 동기화의 두 생성 타입 변화는 위 집중 검사·실제 ABI·현재 TypeScript build로 다시 검증했다.

원 PR #8526의 스타일 삽입 자체의 참조·직렬화 계약과, UI 실패 처리·실제 브라우저 소비를 구분한다. 해당 원본의 시각 일치는 개별 리뷰의 Native/fresh WASM TSV로 확인한다. 이 항목 통과만으로 통합 전체 Rust·시각 보호 문서의 완료를 선언하지 않는다.
