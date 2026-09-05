# 01 — stdin 한 줄 = 경로, stdout 순수 NDJSON, stderr 사람 요약

batch 의 입출력은 세 갈래다. 한 갈래라도 섞으면 게이트가 깨진다.

## stdin

한 줄에 파일 경로 하나. 글롭을 argv 로 펼치지 않는다 — 수백 건이면
Windows/Linux 모두 인자 길이 한계에 걸린다.

```bash
find 폴더/ \( -name '*.hwp' -o -name '*.hwpx' \) > 목록.txt
cat 목록.txt | rhwp batch info --json > meta.ndjson
# 또는
rhwp batch info --json < 목록.txt > meta.ndjson
```

PowerShell 은 [29_windows_powershell.md](29_windows_powershell.md).

규칙:

- 빈 줄은 경로가 아니다. 목록 생성기가 넣지 않게 한다 (B01).
- 주석 (`# ...`) 을 지원하는 문법은 없다. `#` 로 시작하는 줄은 파일명으로 시도되고 실패 봉투가 난다.
- 경로는 따옴표 없이 줄 전체. 공백·한글·괄호가 있어도 한 줄이면 된다.
- 인코딩은 UTF-8. PowerShell `Out-File` 기본(UTF-16 LE)으로 만들면 첫 경로가 깨진다.
- BOM 이 있으면 첫 줄 앞에 U+FEFF 가 붙어 os error 2 가 날 수 있다.
- `batch fill` 은 이 규칙을 쓰지 않는다 (`17_fill_not_stdin.md`).

## stdout

순수 NDJSON. 한 줄 = 문서 하나(fill 은 행 하나)의 JSON 객체.
진행 메시지, 색, 표, `batch: N건 중` 요약은 **한 글자도 없다**.

파이프 예:

```bash
rhwp batch export-text --json < 목록.txt | jq -c 'select(.error|not) | {source,pageCount}'
```

금지:

- `2>&1` 로 stderr 를 stdout 에 합치기 — jq 가 요약 줄에서 터진다.
- 터미널에서 복사한 혼합 출력을 결과 파일로 저장하기.
- 단건 명령처럼 "실패면 stdout 0바이트"를 기대하기. 배치는 실패도 한 줄이다.

성공 레코드는 단건 `--json` 과 같은 스키마다. 소비 코드를 단건/배치로 나누지 않는다.
fill 만 `row` 가 추가된다.

## stderr

사람용 요약과 진단. 예 (레시피 9 실측 취지):

```
batch: 5건 중 4 성공, 1 실패
```

사용법 오류(exit 2)는 레코드 없이 stderr 에만 이유가 난다.
`--password`, `--query` 누락, convert 이름 충돌이 여기 해당한다.

에이전트는 요약을 **집계의 힌트**로만 읽고, 행별 판정은 NDJSON 으로 한다.
"4 성공"만 보고 실패 경로를 버리면 N 게이트가 다음 단계에서 터진다.

## 왜 세 갈래인가

`set -e` / `$LASTEXITCODE` 는 집계다. 집계만 보면 어느 파일이 죽엇는지 모른다.
NDJSON 행이 그 답을 갖고, stderr 는 사람이 로그를 훑을 때 쓴다.
stdin 목록은 argv 한계와 쉘 글롭을 우회한다.

## 실측 원형

레시피 9 의 5줄 목록 → `batch export-text --json --threads 4`.
stdout 5줄 (성공 4 + 실패 1), 종료 코드 1, stderr 에 요약.
전사는 `examples/transcripts/T02.ndjson`.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `01_stdin_ndjson.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
