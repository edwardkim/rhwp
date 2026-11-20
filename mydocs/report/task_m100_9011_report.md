# Task #9011 Studio 메모 관리 UI 1차 구현

- 이슈: https://github.com/edwardkim/rhwp/issues/9011
- 기반: upstream/devel `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`
- 선행: #8017/#8269의 메모 API와 #8885/#8906 후속 기능. API 완료와 Studio UI 완료는 다르다.

## 변경

입력 메뉴의 `insert:comment` stub을 메모 관리 창에 연결하고 항목 이름을 메모로 표시했다. 본문 또는 표 셀의 커서·한 문단 선택 범위에 메모를 넣고 문서 목록에서 내용을 수정하거나 삭제한다. 목록은 쪽과 작성자, 본문 미리보기로 구분하며 문서 문자열은 HTML로 해석하지 않는다.

모든 변경은 `executeOperation` snapshot을 거쳐 dirty·undo/redo를 기록한다. 메모 JSON 실패를 성공으로 기록하지 않는다. 빈 본문, 사라진/읽기 전용 메모, 문서 교체·대상 문단 또는 메모 외부 변경을 거부한다. 코어의 부모 삭제는 댓글까지 지우므로 이 UI에서는 댓글 있는 부모 삭제를 차단한다. 목록 전환은 미반영 입력을 유지하며 반영 성공 항목만 초안을 비운다. 창을 닫으면 미반영 입력은 버린다.

## 검증

- `git diff --check`: 통과.
- `npx tsc --noEmit`: 통과.
- `npm test`: 2,790 tests, 2,788 pass, 0 fail, 2 skip.
- `npm run build:no-hwpctrl`: 통과. 기존 Browserslist 데이터 갱신 안내는 남음.
- `CARGO_TARGET_DIR=<기존 공유 target/pr-review> scripts/wasm-pack-locked.sh --target web --out-dir pkg`: release 및 wasm-opt 완료, 종료 0.
- `CHROME_PATH=... RHWP_WITHOUT_HWPCTRL=1 node e2e/run-with-vite.mjs -- node e2e/memo-ui-issue9011.test.mjs --mode=headless`: 통과.
- 실제 UI: 선택 범위 삽입, 빈 본문 거부, 미반영 취소, 수정·삭제, dirty, 각 undo/redo, 여러 메모 구분, 목록 전환 초안 보존, 외부 편집/문서 교체 거부, 빈 표 셀 삽입을 검증했다.
- HWP/HWPX 각각 편집 → 저장 → 재열기 → 재저장 → 재열기에서 메모 내용 보존 확인.
- 브라우저 초기화 네트워크 WASM 응답 SHA256이 산출물 `fa575218a709127b351533cf9c7f9a66c9e1fdb04fab2365881dd4d18ecabe65`와 일치했다.
- `output/issue9011/memo-dialog.png`를 직접 확인하고 레이블 배치를 보정했다. 비개인 합성 문서만 사용했다.

초기 전체 테스트는 WASM 미생성과 변경 레지스트리 누락으로 실패했다. WASM 생성과 레지스트리·라우팅 감사 원장 보정 후 위 최종 결과로 재검증했다. 기대값을 약화해 실패를 숨기지 않았다.

## 한계와 남은 통합

#9256 보완에서는 메모 대상의 실제 표시 쪽을 기존 커서·선택 좌표로 조회한다. 쪽을 구할 수 있으면 목록에 함께 표시한다. 여러 문단 선택에서는 새 메모 추가만 비활성화하고 기존 메모를 열람·수정·삭제할 수 있다. 메모 창은 Tab과 Shift+Tab으로 조작할 수 있으며 한글·영어 표시를 제공한다. 보호 문서는 기존 편집 정책에 따라 명령이 비활성화된다.

후속 이슈는 다음과 같이 분리했다.

- [#9259 선택 없는 메모 삽입의 단어 범위](https://github.com/edwardkim/rhwp/issues/9259)
- [#9260 본문과 메모 목록 사이의 이동](https://github.com/edwardkim/rhwp/issues/9260)
- [#9261 여백 메모 상자와 안내선 표시](https://github.com/edwardkim/rhwp/issues/9261)
- [#9262 메모 댓글 편집과 부모 삭제 확인](https://github.com/edwardkim/rhwp/issues/9262)

Rust 조판·직렬화 구현 변경은 없다. 기존 API를 UI에서 이용하는 변경이다. 이 결과는 로컬 Studio 구현 검증이며 upstream PR 병합·배포나 알한글 릴리스 반영을 뜻하지 않는다. 배포판 메모 문제가 해결됐다는 사용자 안내는 실제 번들 반영 및 앱 수용 검증 후에 한다.
