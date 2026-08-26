# 06 — 메타데이터 제거 (`edit sanitize`)

층: 1. 본문은 건드리지 않는다. `--dry-run`/`--verify` 플래그가 없다.
빠진 것이 아니라 재실행 `removedCount: 0` 과 `export-text` 전후 동일로 증명한다.

권위: [single_edit.md](../references/single_edit.md) §8.

## 1. 실행

```bash
rhwp edit sanitize 보고서.hwp -o 배포본.hwp --json | jq '.removed[] | "\(.field): \(.before)"'
```

`removed[]` 는 거짓 보고를 하지 않는다. 실제로 지운 필드만 온다.

## 2. 재실행 = 증거

```bash
rhwp edit sanitize 배포본.hwp -o /tmp/재확인.hwp --json | jq .removedCount
```

기대 0. 0 이 아니면 첫 실행이 덜 지운 것이다 — 이 스킬이 새 로직을 넣는 자리가 아니고
구현 회귀이므로 이슈로 남긴다.

## 3. 본문 불변

```bash
rhwp export-text 보고서.hwp
rhwp export-text 배포본.hwp
```

두 결과가 같아야 한다. 메타만 지운다는 계약의 재독이다.

## 4. `--keep-preview`

미리보기 **이미지**를 남긴다. 미리보기 텍스트는 언제나 대상.
HWP5 직렬화기는 빈 PrvText 를 본문 앞부분으로 다시 채우므로,
텍스트는 지금 본문과 다를 때만 지우고 보고한다.

## 5. 체크리스트

- [ ] `-o` 분리
- [ ] `removedCount` 재실행 0
- [ ] `export-text` 전후 동일
- [ ] `run` 에 sanitize 를 넣지 않았다
