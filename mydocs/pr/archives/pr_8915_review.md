# PR #8915 검토 — 일반 웹페이지의 hydration DOM 비간섭

## 최종 판정

**승인 — 일반 페이지 비간섭과 명시적 통합·HWP 링크 기능의 공존을 실제 확장에서 확인했다.**
메인터너 제품 코드·테스트 보정은 없다. 작업지시자의 승인 뒤 원 head의 CI와 최신 base를
재확인하고 `95d1631bd81431b7c1639f0cc5f1e16fb9cfce76`로 원 PR을 merge했다. 후속 기록 반영 뒤 연결 이슈·감사 댓글을 처리한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8915](https://github.com/edwardkim/rhwp/pull/8915) / semanticist21 / devel |
| 관련 이슈 | [#8914](https://github.com/edwardkim/rhwp/issues/8914), 해결 범위 전체 |
| 원 code head | `61198fcf3f4d185c8f78eac5b3259d99303b8845` |
| 최신 검증 base | `e118de8abadd2dffb2798036b9ffd381b96ed7d5` |
| 누적 로컬 실행 source | `18623a34e8a6bea48d2908ee85c34ae786f4feab`, 원 #8913·#8915 보존 병합 |
| reviewer·접수 상태 | 미지정 Ready PR에 `edwardkim` 지정·API 확인. 2026-11-11 KST 재조회: head 불변, clean |
| 원격 처리 상태 | [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8915#pullrequestreview-5854945786)·원 PR merge 완료. 원 fork push 없음 |

## 원인과 수정 범위

최신 devel의 `rhwp-chrome/content-script.js`는 이전 기준
`5ea1227505978156082121f1601cb03044a0583f`와 blob
`aedb976f40464fb6b8dbe38419853346aaa651d9`가 같다. 일반 SSR 페이지에도 설치 표시용
`data-hwp-extension`·`data-hwp-extension-version`과 bridge script를 주입해 hydration 전 DOM을 바꾼다.

원 기여는 문서 head에 아래 선언이 있는 페이지에서만 설치 속성·ready 이벤트·`window.rhwpDev`
script를 제공한다. 일반 페이지의 HWP 링크 배지와 MutationObserver 기반 동적 링크 탐지는 유지한다.
주입 script의 load/error 뒤 제거도 유지한다.

```html
<meta name="hwp:integration" content="enabled">
```

Chrome 개발 가이드와 공개 웹 관리자 안내를 같은 opt-in 계약으로 고쳤다. permissions·
host_permissions·CSP·npm 의존성은 바뀌지 않는다. Firefox·Safari 또는 모든 웹페이지에서
설치 여부를 감지하는 기능의 확장으로 해석하지 않는다. 소유 통합 사용처의 meta 선언과 기존
extension smoke의 통합 검사를 대조했다.

## 검증 입력과 결과

[공통 검증 원장](../assets/semanticist21-20261111/README.md)에 명령·해시·실행 영수증·로그를 보존했다.

| 검증 | 실제 결과 |
| --- | --- |
| 원 head CI | [42810525003](https://github.com/edwardkim/rhwp/actions/runs/42810525003), Frontend package gate·집계 성공. 별도 required check 포함 36개는 success 또는 정상 skip·neutral |
| Chrome Node 검사 | unit·service worker·E2E runner 계약 100 PASS / 0 FAIL |
| JS 구문·실제 패키지 build | content script·smoke `node --check`, `node rhwp-chrome/build.mjs` 성공 |
| 실제 packaged Chrome 음성 대조 | 동일 패키지의 script만 최신 devel 버전으로 교체하면 일반 페이지 HTML 설치 속성 assertion에서 예상 실패 |
| 실제 packaged Chrome 후보 | 일반 페이지에 설치 속성·ready 이벤트·주입 script 없음. opt-in 페이지의 version·ready·rhwpDev, 동적 HWP 배지, viewer·options·print·service worker 검사 성공 |
| 최신 base 병합 | 개별·#8913 누적 merge clean, 문서 링크·공백 검사 통과 |

브라우저는 Chromium 152.0.7946.0(`linux-1660786`)이며 임시 headless profile을 사용했다.
fixture는 기존 `samples/hwp3-pagedef-1915.hwp`다. JS 원인을 분리하려고 확장 안의 WASM은
검증된 이전 producer `5ea1227505978156082121f1601cb03044a0583f`의 동일 파일로 고정했다.
base/candidate 확장의 WASM SHA-256은 모두
`d04aa82f7a395e6576d8d9af6f9ebbdb752ef742cd9c9c7ffc58e1a4918a5f02`다.
따라서 이 검사를 #8913의 새 WASM 검증으로 세지 않는다.

실제 확장 DOM 비간섭을 검증했으며 React 19 SSR 애플리케이션 전체를 별도로 실행한 것으로
주장하지 않는다. 수정 전 실패/수정 후 성공이 hydration 충돌의 원인인 비의도적 DOM 변경을
직접 보호한다. 개발자가 opt-in한 페이지는 통합 DOM을 제공한다는 의도된 계약이다.

문서 parser·layout·paint·저장·WASM 제품 소스는 이 PR에서 바뀌지 않는다. Rust·Native Skia·
fresh WASM 문서 렌더링·Print TSV·편람/정본 보호 게이트는 비해당이다. 원 head의 GitHub 검사를
재사용하며 새 제품 head나 검토용 통합 PR을 만들지 않는다.

## Merge 후 contributor PR comment 계획

원 head를 보존해 원 PR을 merge하고 merge SHA를 확인한다. review archive·오늘할일·증적을
mydocs 운영 기록으로 반영하고 devel 동기화 뒤 #8914 상태·완료 댓글/close와 원 PR 감사 댓글을
처리한다. 댓글에 실제 CI, Node 100건, packaged Chrome의 수정 전 실패/수정 후 성공과
명시적 meta 계약을 기록한다. UI 스크린샷을 문서 Print 비교 증거로 대신하지 않는다.
한국어 존댓말 `--body-file` 게시 후 API로 본문을 재조회한다. 이번 해결 범위에 남은 결함은 없다.

## 병합 확정 기록

- 원 head `61198fcf3f4d185c8f78eac5b3259d99303b8845`, merge `95d1631bd81431b7c1639f0cc5f1e16fb9cfce76`, 병합 시각 `2026-11-11T06:56:03Z`.
- 후속 문서는 maintainer 직접 반영 경로다. 코드·테스트·workflow를 포함하지 않는다.
- 원 기여 head와 fork branch를 보존했다. 문서 운영 기록·devel 동기화 뒤 연결 이슈와 감사 댓글을 처리한다.

- 연결 이슈 #8914는 자동 CLOSED 상태를 API로 확인했다. 운영 기록 반영 뒤 maintainer 완료 댓글을 게시한다.
