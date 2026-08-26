# 예제 — 정부 실공고 양식 채움 (K-Startup)

이슈 #5324. playbook 예시 1. gym 아님. 실제 접수 없음 (F13).

## 정답지 (먼저)

- 제출 요건 문구: "검정 글씨로 작성" (공고 원문 인용)
- 재독 값: 가상 데이터 `(주)시연용가상기업`, 사업자번호 0
- 한컴 PDF 없음. 스타일 축은 요건 문구 + export-svg 후보

## 명령

```bash
rhwp info --json 양식.hwp
rhwp fields --json 양식.hwp        # 누름틀 0
rhwp export-tables --json 양식.hwp # 표 39 → set-cell
rhwp edit set-cell 양식.hwp --table 5 --row 0 --col 1 \
  --text 시연용가상기업 -o 작성본.hwp --json
rhwp export-tables --json 작성본.hwp
rhwp export-pdf 작성본.hwp -o 제출용.pdf
```

## 읽는 법

재독이 쓴 값과 같으면 C04 축은 닫힌다. 파란 안내문 스타일이면
#3391 계열 후보. 체크박스는 #3395. 접수 자동화는 하지 않는다.

관련: `references/07_run_to_final.md`, `fixtures/transcripts/kstartup_reread.txt`.
