# Task M100 #3950 Stage 1 — 한글 IME Ctrl+A RED 계약

## 대상

- Issue: [#3950](https://github.com/edwardkim/rhwp/issues/3950)
- 기준: `upstream/devel@93805ebb0548a48704a0046262044295aade4bcc`
- 브랜치: `codex/issue-3950-ime-ctrl-a`

## 재현

`rhwp-studio/tests/shortcut-map.test.ts`에 `key='ㅁ', code='KeyA', ctrlKey=true`와
`key='Process', code='KeyA', ctrlKey=true`를 추가했다. 기준 구현의 `Ctrl+A` 정의는 `key: 'a'`만
가지므로 두 경우 모두 `null`을 반환했다.

`rhwp-studio/tests/ime-shortcut-routing.test.ts`는 IME 조합 분기 안에서 매칭된 Ctrl/Meta 명령을
`dispatcher`로 보내고 반환하는 계약을 고정했다. 기준 구현은 Ctrl+M chord와 탐색키 보류만 처리한 뒤
반환하므로 이 계약도 실패했다.

```text
node --test tests/shortcut-map.test.ts tests/ime-shortcut-routing.test.ts
결과: 6 passed, 2 failed
```

## 판정

이슈의 원인은 선택 명령이 아니라 입력 이벤트 정규화와 IME 조기 반환이다. 한글 입력 상태의 `Ctrl+A`는
현재 구현에서 여전히 누락돼 있으므로 Stage 2 최소 보정이 필요하다.
