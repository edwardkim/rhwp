# Task M100 #6107 — 활성 페이지·눈금자 2D 정합성 최종 보고서

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **브랜치**: `codex/issue-6107-active-page-ruler`
- **기준 commit**: `upstream/devel` `70ebacc4c9589e8c778907e179a6dab18cce8eb0`
- **제출 기준**: `upstream/devel` `6b5c4f871972380c0866e2a8d27ac2bc67d257e6` 통합
- **완료일**: 2026-08-26 KST
- **검증 서버**: `http://127.0.0.1:7700/` (Vite 8.2.2)

## 결론

가로·세로 이동과 한 쪽·두 쪽·맞쪽·여러 쪽 배치에서 상태 표시줄용 활성 페이지와 눈금자용 편집 focus
페이지를 명시적으로 분리했다. 상태 표시줄은 실제 2D 가시 페이지를 따라가지만, 눈금자는 순수 스크롤에
반응하지 않고 마지막으로 클릭하거나 캐럿·선택·개체를 둔 페이지를 유지한다. 가로 PageUp/PageDown과
휠 좌우 변환은 Y축을 바꾸지 않고 X축 페이지 경계를 따라 이동한다.

실제 브라우저에서 페이지를 선택하면 두 눈금자가 해당 페이지로 이동하고, 이후 순수 스크롤로 focus
페이지가 화면 밖에 나가도 눈금자 대상은 새 가시 페이지로 바뀌지 않는 것을 확인했다. 다시 보이는 페이지를
클릭하면 그 페이지의 눈금자가 표시된다. Studio 전체 테스트, TypeScript 검사, 프로덕션 빌드와 기존
PageUp/PageDown E2E도 통과했다.

## 단계별 구현

| 단계 | commit | 내용 |
| --- | --- | --- |
| 계획 | `5eb9fa491` | #6107 수행·파일 단위 구현 계획 |
| 1 | `0e1facfea` | 활성 페이지 resolver, 가로 PageUp/PageDown과 X/Y 캐럿 보정 |
| 2 | `a0b142ceb` | CanvasView snapshot, 캐럿·개체 페이지 전달, document-agent 2D 가시성 |
| 3 | `11aa581c1` | 가로·세로 눈금과 여백 핀을 활성 페이지 좌표계에 정렬 |
| 4 | `266976e64` | 전체 회귀·빌드·실제 브라우저 검증 |
| 후속 UX 정합 | 이 보고서 갱신 commit | 순수 스크롤과 눈금자 편집 focus 분리 |

## 최종 동작 계약

### 활성 페이지

- 보이는 캐럿·텍스트 선택·개체 선택 페이지는 `editing` 출처로 우선한다.
- 편집 페이지가 화면 밖이거나 없으면 뷰포트 기준의 실제 가시 페이지를 `viewport` 출처로 쓴다.
- 빈 맞쪽 슬롯과 범위 밖 페이지는 후보에서 제외한다.
- `current-page-changed` 상태 표시줄은 이 `ActivePageSnapshot`을 사용해 순수 스크롤도 반영한다.
- Ruler는 마지막 유효 편집 focus 페이지를 우선하고, 아직 focus가 없을 때만 활성 페이지를 초기 fallback으로
  사용한다.

### 페이지 이동과 2D 가시성

- 세로 배치는 기존 행 단위 PageUp/PageDown과 #2560 행 시작 계약을 유지한다.
- 가로 배치는 실제 페이지 X 경계와 뷰포트 폭을 사용하고 결과를 `deltaX`, `deltaY`로 반환한다.
- 캐럿 화면 위치 보정과 document-agent strict render 확인도 X/Y 가시 영역을 함께 사용한다.
- #6039의 휠 좌우 변환을 켠 가로 모드에서는 세로 우세 휠도 `scrollLeft`로 변환한다.

### 눈금자와 핀

- 가로 눈금자는 focus 페이지의 화면 X, 너비, 좌우 여백, 제본·맞쪽 정보를 사용한다.
- 세로 눈금자는 focus 페이지 한 쪽의 화면 Y, 높이와 위·아래 여백만 그린다.
- 여러 쪽 배치에서도 focus 페이지의 쪽 여백 핀을 표시한다.
- 순수 스크롤로 focus 페이지가 화면 밖에 나가면 눈금자도 그 페이지 좌표에 남아 현재 viewport에서는
  보이지 않으며, 새로 등장한 페이지로 자동 재지정하지 않는다.
- 초기 viewport fallback은 쪽 여백 핀을 유지하되 다른 페이지의 문단 들여쓰기 핀을 재사용하지 않는다.
- 핀 드래그는 시작 `pageIdx`를 보관해 미리보기와 최종 `setPageMargin` 대상이 바뀌지 않는다.

## 자동 검증

```text
$ cd rhwp-studio && npx tsc --noEmit
exit 0

$ cd rhwp-studio && npm test
tests 1157, pass 1156, fail 0, skipped 1

$ cd rhwp-studio && npm run build
229 modules transformed, build success

$ cd rhwp-studio && npm run e2e:page-key-scroll
PASS: 6쪽 문서, TC1~TC7 전체 통과
PASS: 모든 쪽 머리 착지, 캐럿 동기화, 외부 포커스, 머리말 문맥,
      Shift+PageDown 선택, 문서 처음·끝 clamp

$ node scripts/rust-test-suite-manifest.mjs --prepare
32 harnesses, 9 exceptions 생성·확인 완료

$ cargo fmt --all && cargo fmt --all -- --check
exit 0

$ git diff --check
exit 0
```

빌드의 500 kB 초과 chunk 안내는 기존 Vite 크기 경고이며 빌드는 성공했다. 브라우저 검증 뒤 수집한
console `error`/`warn`은 0건이었다. 단계별 commit SHA를 보존한 채 최신 `upstream/devel`을 merge한
제출 head에서도 TypeScript, Studio 전체 1,156건, 프로덕션 빌드, PageUp/PageDown E2E,
`cargo fmt --all -- --check`와 `git diff --check`를 다시 통과했다.

## 실제 브라우저 검증

검증 환경은 macOS의 Codex in-app browser, 1280×720 viewport와 현재 브랜치의 Vite 서버다. 빈 문서에서
`Ctrl+Enter` 쪽 나누기 3회를 실행해 4쪽 문서를 만들었다. 실제 fixture 기반 키 이동은 별도 headless
Chrome E2E에서 `samples/biz_plan.hwp` 6쪽을 사용했다.

| 배치·입력 | 관측 | 판정 |
| --- | --- | --- |
| 자동/세로, 1쪽 focus 후 순수 스크롤 | `scrollTop 0 → 1350`, 상태 `1 / 4 → 2 / 4`; 눈금자는 1쪽 focus를 유지해 화면 밖으로 이동 | 통과 |
| 자동/세로, 보이는 2쪽 클릭 | 세로 눈금자가 2쪽 좌표에 다시 표시 | 통과 |
| 두 쪽, 왼쪽 페이지 클릭 | 상태 `3 / 4`; 가로 눈금 범위가 왼쪽 3쪽 위에 위치 | 통과 |
| 두 쪽, 오른쪽 페이지 클릭 | 상태 `4 / 4`; 가로 눈금 범위가 오른쪽 4쪽 위로 이동 | 통과 |
| 맞쪽 | 선택된 오른쪽 2쪽과 가로 눈금 위치가 일치 | 통과 |
| 여러 쪽 2×2, 4쪽 선택 | 상태 `4 / 4`; 가로·세로 눈금이 우하단 4쪽 X/Y 위치로 함께 이동 | 통과 |
| 여러 쪽 2×2, 1쪽 선택 | 상태 `1 / 4`; 두 눈금자가 좌상단 1쪽 위치로 함께 이동 | 통과 |
| 가로 이동 100%, 세로 휠 | `scrollLeft 988 → 1638`(+650), `scrollTop 309 → 309`(+0) | 통과 |
| 가로 이동, 2쪽 focus 후 순수 휠 | `scrollLeft 580.5 → 1965`, `scrollTop 217.5` 유지, 상태 `2 / 4 → 4 / 4`; 눈금자는 2쪽 focus 유지 | 통과 |
| 가로 이동, 보이는 4쪽 클릭 | 가로 눈금자가 4쪽 좌표에 다시 표시 | 통과 |
| 가로 이동, PageUp | 페이지 경계까지 -357.5px 뒤 다음 단계 -804px, Y 변화 0; 상태 `4 / 4 → 2 / 4` | 통과 |
| 가로 이동, PageDown | `scrollLeft +804`, Y 변화 0; 상태 `2 / 4 → 4 / 4` | 통과 |

로컬 자동화의 숨은 파일 입력은 in-app browser의 native file chooser hook에서 캡처되지 않아
`exam_kor.hwp` 자체를 이 세션에서 시각 검증하지는 않았다. 이는 Studio 파일 열기 실패가 아니라 자동화
경계이며, 실제 fixture 문서의 키 이동은 위 headless Chrome E2E로 보완했다. 페이지별로 서로 다른
구역·용지·여백의 실제 핀 드래그는 같은 구역의 빈 4쪽 문서로 판정할 수 없으므로, `pageIdx` 고정과
맞쪽·제본·zoom 왕복 focused test를 자동 근거로 남기고 최종 사용자 확인 항목으로 유지한다.

## 범위 밖 후속 이슈

- [#6040](https://github.com/edwardkim/rhwp/issues/6040): 핀치 줌 중 토폴로지·Canvas 교체 안정화
- [#6041](https://github.com/edwardkim/rhwp/issues/6041): 배율별 render scale과 Canvas 픽셀 예산
- [#6042](https://github.com/edwardkim/rhwp/issues/6042): 행 가상화·LRU·프리페치 성능
- [#6108](https://github.com/edwardkim/rhwp/issues/6108): 쪽 배치별 맞춤 배율 정확성
- [#6109](https://github.com/edwardkim/rhwp/issues/6109): 확대/축소 설정 입력 검증

## 작업 상태

#6107의 로컬 구현, 계획된 4단계 검증, 작업지시자의 실제 동작 확인과 최신 `upstream/devel` 통합 head
재검증을 완료했다. 작업지시자가 다음 제출 게이트 진행을 승인해 원격 push와 PR 생성을 준비한다.
