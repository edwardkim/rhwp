# 07 — 최종 산출물까지

playbook 3단: `rhwp` CLI 로 여정을 중간에 멈추지 말고 최종
산출물까지 실행한다. 명령·입력·산출물을 재현 가능하게 남긴다.

"최종"은 여정이 사용자에게 주는 물건이다.

| 여정 | 최종 산출물 |
| --- | --- |
| 양식 채움 | 작성본.hwp + (선택) 제출용.pdf |
| 한컴 대조 | fidelity `--out-dir` 의 report/text-report |
| 기안문 | 채운 서식 + 정답지 나란히 시트 |
| 왕복 | 변환본 + ZIP 이름 집합 표 |
| CLI 계약 | 종료 코드·봉투 실측 표 |

`info --json` 만 보고 끝내는 것은 F05.

## 재현 가능하게 남긴다

각 단계:

```
$ <명령>
# cwd: <저장소 루트 또는 외부 작업 디렉터리>
# 입력 SHA-256 (알면)
# 산출 경로 (worktree 밖을 권장)
# exit: N
# 봉투 키: …
```

한글 리터럴은 콘솔이 아니라 UTF-8 파일로 넘긴다
([15_utf8_console.md](15_utf8_console.md)).

## 기존 CLI 만

양식 채움 최소 사다리 (playbook 예시 1):

```bash
rhwp info --json 양식.hwp
rhwp fields --json 양식.hwp          # 0 이면 표 양식
rhwp export-tables --json 양식.hwp
rhwp edit set-cell 양식.hwp --table 5 --row 0 --col 1 \
  --text-file values/name.txt -o 작성본.hwp --json
rhwp export-tables --json 작성본.hwp
rhwp export-pdf 작성본.hwp -o 제출용.pdf
```

`--text-file` 이 없는 빌드면 값이 ASCII/가상 토큰이거나 UTF-8
파일을 표준입력으로 넘기는 기존 관례를 따른다. 새 플래그를
발명하지 않는다.

왕복 최소 사다리 (예시 4+7):

```bash
rhwp export-hwpx 원본.hwpx out.hwpx --verify --verify-pages
rhwp ir-diff 원본.hwpx out.hwpx --json
# 통과해도 멈취지 않음 — ZIP 이름 집합
```

## 원본 불변

`-o` 로 산출을 분리한다. 실패해도 원본을 덮지 않는다. 실제 접수는
하지 않는다 (F13).

## 관련

- 화이트리스트: [24_existing_cli.md](24_existing_cli.md)
- 예제: [01_kstartup_form.md](../examples/01_kstartup_form.md)
