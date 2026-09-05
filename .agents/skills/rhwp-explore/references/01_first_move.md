# 01 — 언제나 `rhwp explore <file> --json`

처음 보는 HWP/HWPX/HML 의 첫 수는 이 한 줄이다.

```bash
rhwp explore 문서.hwp --json
```

사람용 메뉴가 필요하면 `--json` 없이 같은 명령을 친다. 기계 소비는
JSON 이 계약이다.

## 왜 첫 수인가

- 메뉴가 문서마다 다르다. 표 보고서는 표가, 서식은 누름틀이, 외부
  메일은 보안이 위로 온다.
- `menu[0].command` 를 치환해 실행하면 다음 수가 결정된다.
- 본문을 읽기 전에 주입·은닉 신호가 있으면 메뉴가 그걸 올린다.

## 하지 말 것

```bash
# 본문을 먼저 퍼내지 않는다
rhwp export-text 문서.hwp
# 도구 일반에서 고르지 않는다
rhwp capabilities --json
# 없는 플래그를 붙이지 않는다
rhwp explore 문서.hwp --unknown-flag
```

## 첫 항목만

```bash
rhwp explore 문서.hwp --json | jq -r '.menu[0].command'
```

`<file>` 을 실제 경로로 바꾼 뒤 실행한다. 자리표시자를 그대로 치면
exit 1 이다.

## 비밀번호

전역 플래그다. 하위명령 플래그가 아니다.

```bash
rhwp --password-stdin explore 비밀.hwp --json
```

없으면 암호 문서는 exit 2 이고 봉투가 없다 (X02).

## 정지

사용자 질문이 "뭘 할 수 있어?" 이면 메뉴를 보여 주고 멈춘다 (X10).
다음 스킬로 넘어가는 것은 그 다음 요청이다.
