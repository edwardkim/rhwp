# 10 — 기록값 재독

playbook: 기록값·종료 코드·JSON 계약은 기계적으로 판정한다.
쓴 것을 같은 CLI 로 되읽어 프로그램이 비교한다. "됐다는 보고"가
아니라 산출물 자체가 근거다.

## 표 양식 (누름틀 0)

```bash
rhwp export-tables --json 양식.hwp > before.json
# set-cell 로 칸을 채운다. 원본은 -o 로 분리
rhwp edit set-cell 양식.hwp --table 5 --row 0 --col 1 \
  --text 시연용가상기업 -o 작성본.hwp --json
rhwp export-tables --json 작성본.hwp > after.json
```

`after.json` 의 table 5 / row 0 / col 1 이 쓴 문자열과 바이트
단위로 같은지 UTF-8 파일로 비교한다. 콘솔 출력을 눈으로 보지 않는다.

불일치 → C04, `issueReady=true`. 정답지는 "방금 기록한 값"이다.

## 누름틀

```bash
rhwp fields --json 양식.hwp
rhwp edit fill-fields 양식.hwp --data @row.json -o 작성본.hwp --json
rhwp fields --json 작성본.hwp
```

데이터 파일의 키와 재독 값이 1:1 이어야 한다. 침묵 유실은 #3358
계열이다. 빈 칸을 성공으로 읽지 않는다.

## 스타일 요건은 재독만으로 부족

제출 요건이 "검정 글씨로 작성"이면 값이 맞아도 파란 안내문
스타일을 상속할 수 있다 (#3391). 재독 축은 문자열만 확정하고,
스타일은 정답지 렌더/픽셀 후보로 넘긴다. 한 이슈에 두 축을
섞어 "값이 틀렸다"고 쓰지 않는다.

## 체크박스

글머리표면 텍스트 편집 밖이다 (#3395). 재독 실패가 아니라
수단 격차다. 이슈에 "텍스트 축 밖"이라고 적는다.

## 관련

- 봉투: `fixtures/envelopes/reread_mismatch.json`
- 전사: `fixtures/transcripts/kstartup_reread.txt`
- 예제: [01_kstartup_form.md](../examples/01_kstartup_form.md)
