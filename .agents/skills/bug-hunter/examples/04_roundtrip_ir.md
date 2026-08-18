# 예제 — 형식 변환 무손실 (IR 오라클)

이슈 #5324. playbook 예시 4. gym 아님. 이 오라클만으로 끝내지 않는다.

## 정답지

변환 후 IR·렌더가 원본과 같은가 (한컴이 깨지지 않는 수준).
IR 축의 정답은 `--verify` / `ir-diff --json` 이다. 구조 축은 J07.

## 명령

```bash
rhwp export-hwpx 원본.hwp 변환본.hwpx --verify --verify-pages
rhwp ir-diff 변환본.hwpx 원본.hwp --json
```

## 읽는 법

exit 3/4 = IR 손실 후보. 카테고리 type/ml 로 축을 특정.
통과해도 F09 — ZIP 이름 집합을 한 번 더. 발견 예: #3367 #3368 #3383.

관련: `references/02_judgment_traps.md`.
