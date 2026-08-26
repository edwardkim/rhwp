# 14 — 쪽 주소 (0 기준)

트리아지가 존재하는 이유 중 하나는 **근거 쪽번호**다.

## 규칙

| 표면 | 기준 |
| --- | --- |
| search.matches[].page | 0 |
| extract-data.items[].page | 0 |
| digest --pages a..b | 0, 양끝 포함 |
| export-text -p / export-png -p | 0 |
| 사람 답변 | page+1 |
| extract-pages --from/--to | 1 |

조판에 안 올라간 문단은 `page` 가 없을 수 있다. 없으면 쪽을 지어내지 않는다.

## 환산 반복 A01~

1. 봉투의 `page`=1 를 사람에게는 2쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 1 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A01).
2. 봉투의 `page`=2 를 사람에게는 3쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 2 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A02).
3. 봉투의 `page`=3 를 사람에게는 4쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 3 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A03).
4. 봉투의 `page`=4 를 사람에게는 5쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 4 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A04).
5. 봉투의 `page`=5 를 사람에게는 6쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 5 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A05).
6. 봉투의 `page`=6 를 사람에게는 7쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 6 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A06).
7. 봉투의 `page`=7 를 사람에게는 8쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 7 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A07).
8. 봉투의 `page`=8 를 사람에게는 9쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 8 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A08).
9. 봉투의 `page`=9 를 사람에게는 10쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 9 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A09).
10. 봉투의 `page`=10 를 사람에게는 11쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 10 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A10).
11. 봉투의 `page`=11 를 사람에게는 12쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 11 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A11).
12. 봉투의 `page`=12 를 사람에게는 13쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 12 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A12).
13. 봉투의 `page`=13 를 사람에게는 14쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 13 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A13).
14. 봉투의 `page`=14 를 사람에게는 15쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 14 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A14).
15. 봉투의 `page`=15 를 사람에게는 16쪽으로 말한다. `export-png -p` 와 `export-text -p` 와 `digest --pages` 에는 15 을 그대로 넣는다. `extract-pages --from/--to` 만 1 기준 (A15).
