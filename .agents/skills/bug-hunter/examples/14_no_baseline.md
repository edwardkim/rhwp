# 예제 — 정답지 없음 (자기 일관성만)

이슈 #5324. F04. gym 아님.

## 정답지

없음. 한컴 PDF·법정 서식·제출 요건을 확보하지 못했다.

## 명령

```bash
rhwp render-diff samples/form-01.hwp --via hwpx
```

## 기록 문장

```
독립 비교 기준을 확보하지 못했다.
수행: rhwp render-diff samples/form-01.hwp --via hwpx
결과: status PASS
한계: 자기 일관성만. 한컴 공식 출력·법정 서식·제출 요건과
대조하지 않았다. 충실도 결함으로 이슈화하지 않는다.
```

PASS 를 "한컴과 같다"로 읽지 않는다 (P09).
관련: `fixtures/transcripts/self_only_limit.txt`.
