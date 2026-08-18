# 예제 — 근거 조항 쪽만 렌더

이슈 #5324. playbook 카탈로그 RAG 인용. gym 아님.

## 정답지

공고/법령 원문의 조항 위치. 해당 쪽이 원문과 같은 조문 번호를
보여야 한다. 전문 dump 는 이 여정이 아니다.

## 명령

```bash
rhwp info --json 공고.hwp
rhwp search --json -- 조항키워드 공고.hwp
rhwp export-svg 공고.hwp -p N -o cite/
```

쪽 번호는 0 기준. 검색 히트 표시(1부터)와 혼동하지 않는다.

## 읽는 법

검색 총량 은폐는 #3353. `--limit` 봉투를 읽는다. 조항이 다른
쪽에 그려지면 픽셀/문자 owner 이동 후보. 정답지 없는 쪽 추측은 F02.
