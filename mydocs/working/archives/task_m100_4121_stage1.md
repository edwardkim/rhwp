# Task M100-4121 Stage 1 완료보고서 — HF 선택 코어 계약

## 결과

머리말/꼬리말 텍스트 선택을 Studio에서 만들고 소비하기 전에 필요한 코어·WASM 계약을
구현했다. 이번 Stage는 선택 상태와 입력 라우팅을 연결하지 않으므로 #4121 전체가 해결된
상태는 아니다. Stage 2에서 마우스·Shift 선택, 반복 페이지 overlay, HF 모드 전환을
연결해야 실제 사용자 여정이 동작한다.

## 구현 범위

### 페이지 지역 기하와 target 식별

- `getSelectionRectsInHeaderFooter`를 추가했다. 선택의 논리적 target인
  `(sectionIdx, isHeader, applyTo)`가 요청 페이지의 active target과 같을 때만 페이지 지역
  사각형을 반환한다.
- 역방향 범위를 정렬하며 다문단 선택과 필드 표시 문자열의 모델 offset 매핑을 지원한다.
- active target의 잘못된 문단·문자 범위는 조용히 잘라 그리지 않고 오류로 거부한다.
- 기존 HF caret query에도 페이지 target 검증을 적용했다. Odd/Even 페이지에서 다른 정의의
  좌표를 잘못 재사용하지 않는다.
- `hitTestInHeaderFooter`가 resolved `sectionIndex`와 `applyTo`를 반환하게 해 Studio가 클릭
  페이지의 target을 추측하지 않게 했다.

### 범위 편집·복사·서식

- `replaceRangeInHeaderFooter`를 추가했다. 모든 경계를 mutation 전에 검증하고, 역방향
  범위를 정렬한 뒤 시작 문단 prefix와 끝 문단 suffix를 보존해 한 번에 치환한다.
- replacement의 CRLF/CR은 LF로 정규화하며 LF는 새 HF 문단 경계로 보존한다. 빈 문자열은
  범위 삭제 primitive로 사용할 수 있다.
- `copySelectionInHeaderFooter`는 기존 본문 클립보드 paragraph slice 규칙을 재사용하고
  다문단 평문을 LF로 연결해 내부 `ClipboardData`에 저장한다.
- `getCharPropertiesInHeaderFooter`와 `applyCharFormatInHeaderFooter`를 추가했다. 다문단의
  교차 범위에만 char-shape을 적용하고 선택 밖 문자는 유지한다.

### WASM과 Studio bridge

다음 5개 API를 Rust WASM binding과 `WasmBridge`에 같은 인자 순서로 노출했다.

| API | 역할 |
| --- | --- |
| `getSelectionRectsInHeaderFooter` | 한 페이지의 HF 선택 사각형 조회 |
| `replaceRangeInHeaderFooter` | 단일·다문단 범위 원자 치환 |
| `copySelectionInHeaderFooter` | HF 선택 조각과 평문 복사 |
| `getCharPropertiesInHeaderFooter` | HF 캐럿의 글자 속성 조회 |
| `applyCharFormatInHeaderFooter` | HF 선택 범위 부분 글자 서식 |

문서를 바꾸는 두 API는 mutation registry에 기록 대상 메서드로 분류했다. Stage 2 이후
Studio command/history 경로에서 직접 호출을 차단하는 기존 저작 시점 가드가 계속 적용된다.

## 회귀 테스트

새 integration test `issue_4121_header_footer_text_selection`은 다음 6개 계약을 검증한다.

1. Both 머리말의 다문단 선택 사각형과 target 불일치 빈 결과
2. Odd/Even 꼬리말의 쪽별 resolved target과 반대 target 차단
3. 파일 이름 필드의 긴 표시 문자열과 단일 모델 marker offset 매핑
4. 역방향 다문단 범위 치환, suffix 보존, invalid range 원자 실패, HWP 재파스
5. 다문단 복사의 LF와 내부 클립보드 평문
6. 다문단 부분 글자 서식, 선택 밖 보존, HWP 재파스

Studio 정적 계약 테스트는 다섯 API의 WASM/bridge 동시 노출과 HF hit-test target 타입을
검증한다. mutation routing guard는 두 새 mutator가 원장에 포함되지 않으면 실패한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| focused Rust integration | 6개 통과, 실패 0 |
| focused Studio bridge + mutation routing | 14개 통과, 실패 0 |
| Studio 전체 `npm test` | 1,227개 통과, 실패 0, 기존 skip 1 |
| `cargo check --locked --lib` | 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 경고 없이 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4,221 tests / 299 modules, 정책 통과 |
| `scripts/wasm-pack-locked.sh --target web --out-dir pkg` | 통과, 신규 5 API의 선언 생성 확인 |
| Studio `npm run build` | TypeScript·Vite build 통과 |
| `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` | 통과 |

integration source는 `tests/cases/`만 추가했다. focused 실행을 위해 현재 작업 디렉터리에서
derived suite를 생성했지만 `tests/generated/`와 manifest는 검증 산출물로 stage하지 않는다.

## Stage 2 경계

아직 다음 항목은 구현하지 않았다.

- `Cursor.hfAnchor`와 HF 선택 정렬·복원 불변식
- 마우스 드래그, Shift+클릭, Shift+방향키 선택 생성
- 같은 HF 정의를 쓰는 visible page의 반복 overlay와 scroll-in 재투영
- 다른 Odd/Even target 교차 선택 차단 및 본문 클릭 시 HF 모드 종료
- Esc 2단계 동작과 선택 소비자 연결

따라서 이 Stage만으로 #4121을 닫거나 사용자 동작 완료로 판정하지 않는다. Stage 1 변경을
체크포인트로 고정한 뒤 별도 승인을 받아 Stage 2를 시작한다.
