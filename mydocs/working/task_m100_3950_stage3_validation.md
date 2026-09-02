# Task M100 #3950 Stage 3 — Studio 및 Windows Chrome 검증

## 자동 검증

Windows PowerShell에서 다음을 순차 실행했다.

```text
npx.cmd tsc --noEmit
결과: 통과

npm.cmd test
결과: 931 passed, 0 failed, 1 skipped
```

`npm.ps1`은 이 호스트의 PowerShell 실행 정책에 의해 차단됐으며, 동일 npm CLI를 실행하는
`npm.cmd`로 재실행해 판정했다.

## Windows Chrome 확인

로컬 Vite Studio(`http://127.0.0.1:5173/`)를 Windows Chrome에서 열고 새 문서에
`가나다라마바사`를 입력했다. 편집 입력에서 `Ctrl+KeyA`를 실행하자 문서의 해당 한글 전 범위가
selection overlay로 선택됐다. Studio 콘솔에는 제품 코드 오류가 없었고, 보인 오류 한 건은
Chrome 확장 content script의 message port 종료 로그였다.

브라우저 자동화는 운영체제의 한글 IME 레이아웃 전환·조합 생성까지 제어하지 못한다. 이 때문에
`e.key='ㅁ'` 및 `e.key='Process'`와 `e.code='KeyA'`의 두 실제 입력 계약은 Stage 1/2 Node 회귀
테스트에서 고정했고, Chrome에서는 한글 문서에 대한 실제 선택 명령 실행을 확인했다.

## 다음 단계

코드·테스트·작업 문서는 로컬 브랜치에만 있다. 원격 push와 PR 생성은 작업지시자 별도 승인 전에는
수행하지 않는다.
