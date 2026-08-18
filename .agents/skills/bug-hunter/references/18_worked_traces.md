# 18 — 재현 트레이스

에이전트가 "이미 이렇게 돌렸다"는 로그의 스키마다.
`fixtures/traces/T01.json` … 와 `fixtures/traces_index.json`.

각 트레이스는 다음을 가진다.

- `id` — T01…
- `journey` — J01…
- `title`
- `commands` — 복붙 가능한 기존 CLI / 도구
- `stop` — F01–F16
- `filed` — 이 트레이스 자체가 이슈를 올렸는가 (픽스처는 false)
- `note`

## 대표

**T01** J01 양식 채움 끝까지. 접수 거부.

**T02** J02 plan 0–34 fidelity_compare. 문자 랭킹.

**T03** J07 verify 통과 후 ZIP 이름 집합. F09.

**T04** J17 정답지 없음. render-diff + F04 한계 문장.

**T05** J06 언어이해 8쪽. 쪽 번호 0 기준(7) + provenance.

T06–T40 은 같은 계약의 변형이다. 발명 명령을 넣지 않는다.

## 쓰는 법

새 실측을 남길 때 이 스키마로 `working/` 로그를 적거나, playbook
4단 예시에 명령을 옮긴다. 픽스처 Txx 를 거짓 실측처럼 인용하지
않는다. 픽스처는 계약 시험용이다.

라이브 전사는 `fixtures/transcripts/` 의 세 파일이다.

- `kstartup_reread.txt`
- `verify_then_zip.txt`
- `self_only_limit.txt`
