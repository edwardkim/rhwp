# 02 — 문구 일괄 치환 (`edit replace-text`)

층: 1. 목표: 문서의 `2025년` 을 `2026년` 으로 바꾸고, 산출물에서
`2025년` 의 `matchCount` 가 0 인지 재독한다.

권위: [single_edit.md](../references/single_edit.md) §4.
코어는 `replace_all` (역순, 오프셋 안전). 새 치환 엔진이 없다.

## 0. 하지 않는 것

- `--find ""` (exit 2, 문서 전체).
- 0건인 줄 알고 1층에서 "성공"으로 보고하기. 산출 파일이 없다.
- 같은 파일을 여러 번 치환해 반쪽 이력을 남기기. 여러 치환은 `run`.

## 1. 발견

```bash
rhwp search 공문.hwp "2025년" --json | jq '{matchCount, pages: [.matches[].page]}'
```

`matchCount == 0` 이면 이 편을 끝낸다. 3층으로 같은 찾기를 넣으면
`'2025년' 일치 0건` 으로 선검증이 거부한다. 층이 다르면 판정이 다르다.

## 2. 선확인

```bash
rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" --dry-run --json
```

기대: `dryRun: true`, `replacedCount` == 발견의 `matchCount`, `output` 부재.

`--occurrence 3` 을 쓸 거면 dry-run 에서도 같은 플래그를 붙인다.
전건 dry-run 을 보고 occurrence 실행을 하면 선확인이 아니다.

## 3. 실행

```bash
rhwp edit replace-text 공문.hwp --find "2025년" --replace "2026년" \
  -o 개정본.hwp --verify --json
```

`replacedCount: 0` 이면 `output` 키가 없고 개정본.hwp 가 생기지 않는다.
무변경 산출물 금지.

체크박스 한 칸만 켤 때:

```bash
rhwp edit replace-text 신청서.hwp --find "□" --replace "☑" --occurrence 0 -o 체크.hwp --json
```

여러 칸 + 다른 편집이면 `run` 의 `set_checkbox`.

## 4. 재독

```bash
rhwp search 개정본.hwp "2025년" --json | jq .matchCount     # → 0
rhwp search 개정본.hwp "2026년" --json | jq .matchCount     # → 원래 건수
```

`--occurrence k` 였으면 첫 호출이 `원본 matchCount - 1`.

## 5. 눈검증

실측 (`hwp3-sample.hwp`, playbook §9-3):

- 전건 `의`→`의`: `changedPages` 0..14 (15쪽)
- `--occurrence 3` 으로 `의`→`★`: `changedPages: [0]` 만

```bash
rhwp export-svg 개정본.hwp -o out/svg -p 0 --json
```

배열에 있는 쪽만 렌더한다.

## 6. `--ignore-case`

CLI 기본은 구별. `--ignore-case` 는 1층 플래그.
3층으로 옮기면 `caseSensitive: false` 다. `ignoreCase` 키가 아니다.

## 7. 체크리스트

- [ ] `rhwp search` 로 건수를 먼저 봤다
- [ ] dry-run 의 `replacedCount` 가 그 건수와 같다
- [ ] 실행 후 `search` 재독
- [ ] `changedPages` 가 배열일 때만 눈검증
