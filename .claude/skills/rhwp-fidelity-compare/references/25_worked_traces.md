# 25 — 재현 트레이스

`fixtures/traces/T01.json` … `T30.json` 과 같은 이야기이다. 여기서는
사람이 따라 할 수 있게 명령과 산출을 풀어 쓴다. gym trajectory 가
아니다.

## T01 plan 0–34 시트

```bash
cargo build --profile release-test --target-dir target/pr-review
RHWP_BIN=target/pr-review/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --out-dir /tmp/rhwp-fidelity-plan
```

기대: run-state complete, stdout top 8, `cmp-p001.png`…`cmp-p035.png`.
다음: 상위 쪽 시트. 정지 F03.

## T02 오라클 없음

사용자: "이 서식 한컴이랑 같아?"
PDF: 없음.
행동: PDF 를 요청. 없으면 `rhwp-visual-regression`.
정지 F01. 하네스를 돌리지 않음.

## T03 text-only 215

`examples/16_direct_pair.md` 와 동일 명령. 기대: text-report 215행,
page-count ledger, 시트 0장. 정지 F02.

## T04 Chrome 없음

시트 모드, `find_chrome` 실패, exit 2, stderr 한국어.
`--text-only` 로 내리거나 정지. F10.

## T05 venv 없음

시스템 python, ImportError, exit 2. venv 생성. F09. break 플래그 없음.

## T06 Windows

`venv\Scripts\python.exe` + `$env:TEMP`. F03.

## T08 암호화

`is_encrypted` 참. 정지. 우회 없음. F13.

## T09 쪽수

ledger delta +2. owner 창만. 전역 패치 없음. F11.

## T10 두부

전면 □, 랭킹 폐기, FONT_PATH, 새 out-dir. F14.

## T14 #3385

glyph-risk PUA, text-report 치환, 본문은 살아 있음. 이슈 초안.
F04 + F05.

## T16 incomplete

run-state missing 3,5 exit 1. 전수 포장 금지. F12.

## T18–T20 거절

gym / visual-regression 수정 / hunter 재작성. F06 F07 F08.

## T21 stale binary

방금 고친 코드인데 옛 release-test. `target/pr-review` 로 rebuild.

## T23 cp949

한글 경로가 깨진 argv. ASCII 키로 재실행.

## T28 맞춰찍기 PDF

편집기 25, PDF 13. 오라클 강등. F17.

트레이스 JSON 은 기계 가드용이다. 이 장이 JSON 을 복붙하지 않고
명령만 적는 이유다.
