# 예제 — 법정 서식 생성 (기안문)

이슈 #5324. playbook 예시 3. gym 아님.

## 정답지 (먼저)

행정 업무 편람 별지 제1호(일반기안문)·제2호(간이기안문, 결재란 표).
편람 식별자와 별지 이미지를 확보하기 전에 서식을 "그럴듯하게"
만들지 않는다 (F02).

## 명령

```bash
rhwp export-svg 정답지렌더.hwp -o truth-svg/
# 표준 서식 제작은 기존 ingest/편집 CLI 만. 새 스키마 발명 금지
rhwp info --json 서식.hwp
rhwp fields --json 서식.hwp
rhwp edit fill-fields 서식.hwp --data @draft.json -o 기안.hwp --json
rhwp export-svg 기안.hwp -o out-svg/
```

## 읽는 법

표·항목체계·두문/결문이 ingest 스키마에 없으면 #3372 계열 격차.
빈 누름틀 안내문이 인쇄되면 #3375. 글자 겹침은 픽셀 후보다.

관련: `references/03_journey_selection.md`.
