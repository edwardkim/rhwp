# 11 — `overflow` 는 성공 안의 경고

층: 1. `set-cell` / `insert-image` 는 넘쳐도 채운다. exit 0.
신호를 무시하면 표·쪽 밖으로 나간 제출본이 나온다.

권위: [single_edit.md](../references/single_edit.md) §5.3, #3480.

## 1. 표본

픽스처 [../fixtures/envelopes/set_cell_overflow.json](../fixtures/envelopes/set_cell_overflow.json).

```json
"overflow": [{
  "target": "table0[1,1]",
  "text": "아주아주긴값",
  "cellWidthPx": 20.71,
  "textWidthPx": 48.0,
  "lines": 3
}]
```

실측 형태는 playbook §14. 칸 폭은 근사다. 정밀 조판이 아니다.

```bash
rhwp edit set-cell 양식.hwpx --table 0 --row 1 --col 1 --text "아주아주긴값" --dry-run --json
```

## 2. dry-run 에서도 온다

파일을 만들기 전에 알 수 있다. 선확인에서 잡고 값을 줄인다.

여러 줄이 정상인 칸(주소·사유)은 overflow 가 떠도 사용자가 승인하면 진행한다.
에이전트가 혼자 승인하지 않는다.

## 3. insert-image

```json
"overflow": [{
  "page": 0,
  "paperWidthHu": 59528,
  "paperHeightHu": 84188,
  "rightHu": 62000,
  "bottomHu": 76000,
  "overflowXHu": 2472,
  "overflowYHu": 0
}]
```

자르지 않는다. 좌표를 줄여 다시 dry-run.

## 4. 체크리스트

- [ ] dry-run 에서 overflow 를 읽었다
- [ ] 비어 있지 않으면 사용자에게 알렸다
- [ ] exit 0 을 완료로 바꾸지 않았다
