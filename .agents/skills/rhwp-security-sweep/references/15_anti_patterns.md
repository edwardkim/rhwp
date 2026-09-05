# 금지 패턴

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.

## A1 새 서브커맨드

`rhwp inspect pii` / `rhwp security-gate` 를 만들지 않는다.

기존 redact dry-run 과 inspect 3축이면 충분하다.

## A2 탐지 로직 복제

스킬 테스트에 새 Luhn/mod11 구현을 넣어 '더 잘 잡기'를 하지 않는다.

계약은 기존 pii_scan 의 보수 표다.

## A3 gym 우회

이 이슈는 gym 금지. security pack SE* 과제를 여기서 확장하지 않는다.

## A4 다른 스킬 수정

safe-edit / provenance / doc-triage / onboarding 파일을 이 PR 에서 고치지 않는다.

인계만 적는다.

## A5 신호 준수

injection matched 를 run plan 의 action 으로 옮기지 않는다.

## A6 전문 덤프

수신 첫 응답으로 export-text 무제한을 하지 않는다.

## A7 raw 로그

CI 가 redact 봉투를 수집할 때 --no-raw 없는 경로를 기본으로 두지 않는다.

## A8 기본 산출 이름

redact 에 `_redacted.hwp` 기본값을 기대해 스크립트를 짜지 않는다. 없다.

## A9 워터마크 제거

inspect watermark 결과를 지우는 edit 를 발명하지 않는다. 보고만.

## A10 정화 저장

unicode 를 정규화해 원본에 다시 쓰지 않는다. 표시만.

## A11 범위 확장

계좌·여권을 잡도록 규칙을 넓히지 않는다. search + 사람.

## A12 DocumentCore 편집

이 이슈는 스킬·문서·순수 시험. 코어 편집 구현을 건드리지 않는다.

픽스처 `fixtures/cli_surface.json` 의 `doNotInvent` 와 같은 목록을 시험한다.
