# #8398 PDF 선택 대기 커서 수정 완료

- 이슈: [#8398](https://github.com/edwardkim/rhwp/issues/8398), 담당 edwardkim, OPEN.
- 승인: 2026-10-31 이슈 등록·계획 구현을 한 단위로 승인받았다.
- base: `8fbed1a9c89798f925279e42fed6cbeba26b9c71`.
- 브랜치: `task/8398-pdf-import-busy-cursor`.
- 계획 커밋: `366d638253`. 구현·회귀·매뉴얼 커밋: `3f57e95074`.
- 결과: 승인한 구현·로컬 검증 완료. 이후 사용자의 테스트 통과 판정과 PR Open·self-review 승인에 따라 [PR #8401](https://github.com/edwardkim/rhwp/pull/8401)을 생성했다. 코드 head CI·검토 결과와 남은 원격 절차는 [self-review 기록](../pr/archives/pr_8401_review.md)에 정리했다. 머지·이슈 종료는 수행하지 않았다.

## 수정 결과

`loadFile`의 busy 수명을 파일 식별·읽기 구간으로 제한했다. 공통 `loadBytes`에서 PDF 선택과
변환 대화창을 기다리는 구간을 전역 busy 밖으로 옮겼다. 일반 문서의 파싱·쪽 계산과
성공한 PDF의 뷰 초기화는 기존 busy 처리를 유지한다.

사용자 선택·실패 후 재시도 대기에는 일반 커서를 사용하고 상태 표시줄에 「PDF 가져오기」를
표시한다. 실제 읽기·변환 단계는 기존 대화창이 안내하며 취소 버튼을 계속 조작할 수 있다.
CSS 강제 덮어쓰기·busy 카운터 초기화·Worker 또는 Rust 엔진 변경은 없다.

[수행 계획](../plans/task_m100_8398.md)의 선행 #5740 → #5768, PDF 추가 #8360을 대조했다.
일반 문서의 빈 쪽 준비 회귀는 #5617(`61baa67835`)에서 도입되고 #8360(`f125c735c7`)에서
PDF 화면 보존 조건이 추가됐다. 이번 분리로 호출 문자열이 달라져 검사 대상 호출만 일반 문서
경로로 갱신했다. 빈 쪽 → 일반 문서 파싱 순서와 PDF의 기존 화면 보존 조건은 그대로 유지한다.

## 실제 검증

| 항목 | 결과 |
| --- | --- |
| 신규 브라우저 회귀, 수정 전 | `PDF 사용자 선택 중 전역 busy 해제`에서 true ≠ false로 실패 |
| 신규 브라우저 회귀, 수정 후 | 전체 시나리오 통과 |
| 사용자 선택 | Worker 시작 전 일반 커서, 미실행 파싱 25% 문구 없음, 프로필·고급 설정 조작 가능 |
| 바이트 입력·Escape 취소 | 문서 세대·본문·이름·저장 핸들·최근 문서·실제 복구본 바이트 유지 |
| 파일 입력·진행 중 취소 | 실제 Worker를 생성하고 요청 전송만 보류해 취소 시점을 고정, 취소 버튼 조작·Worker 종료·기존 상태 보존 확인 |
| 드롭·PDF 헤더 감지 | 확장자를 HWP로 바꾼 PDF도 정상 선택·취소 및 기존 상태 보존 |
| 한도 오류·재시도 | 입력 한도 1 byte로 거부 후 일반 커서로 설정 수정, 같은 PDF의 실제 변환·뷰 초기화 성공 |
| 일반 문서 대조군 | HWP 열기 중 busy, 손상 문서 실패 후 해제·기존 PDF 보존, 다음 HWP 열기·새 문서 생성 성공 |
| 기존 브라우저 `document-title-issue6566` | 실제 문서 열기·손상 파일 보존·HWPX 저장 등 13개 검사 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 2,692 PASS / 0 FAIL / 기존 skip 2, 총 2,694 |
| E2E manifest·문서 링크·diff whitespace | 통과 |

브라우저는 직전 최신 devel의 release + wasm-opt WASM을 그대로 사용했다. 실제 수신 WASM
SHA-256 `dcb50238ab65d703738030ff1498da5d6db46e9c74b50f1226f49825edc0527b` 일치를
확인했다. 이번 제품 변경은 Studio의 로딩 UI 수명뿐이며 Rust·조판 변경이나 한컴 피델리티 판정이 아니다.

신규 회귀는 `rhwp-studio/e2e/pdf-import-busy-issue8398.test.mjs`에 등록했다. CI 개별 호출을
추가하지 않았다. 기존 #8389(`c417310cab`)가 추가한 `supplementary-private-glyph-replay.test.mjs`의
E2E manifest 행 누락도 발견해 실제 파일·원 도입 커밋을 확인한 뒤 목록만 보완했다. 그 검사 구현은
수정하거나 이번 검증으로 통과했다고 주장하지 않는다.

## 재현 명령과 실행 기록

저장소 루트의 `output/pdf-import-busy-20261031/`에 `before.log`, `after.log`, `npm-test.log`,
`tsc.log`, `document-open-save.log`를 남겼다. 신규 E2E는 Studio 디렉터리에서 다음으로 실행한다.

```bash
node e2e/run-with-vite.mjs -- node e2e/pdf-import-busy-issue8398.test.mjs --mode=headless
```

기존 개발 서버의 7700 포트는 보존하고 검증용 Vite는 자동 할당된 별도 포트를 사용한 뒤 종료했다.
