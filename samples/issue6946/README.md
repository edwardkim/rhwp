# 물류정책기본법 시행규칙 일부개정령안 — block 으로 앉은 자리차지 표의 형제 (#6946)

국토교통부 공개 입법예고 첨부 서식이다. 구역 3 의 문단 0 은 **빈 문단**(`text_len=0`)이고
비-TAC 자리차지(`wrap=위아래`, `vert=문단`, `vertOffset=0`) 표를 단다.

| 컨트롤 | 크기 | 높이 | 내용 |
| --- | --- | ---: | --- |
| `ci=3` | 24×13 | 903.3px | `(앞쪽)` 국제물류주선업등록기준신고서 |
| `ci=4` | 10×1 | 894.5px | `(뒤쪽)` 행정정보 공동이용 동의서 |

본문 높이는 918.4px 라 두 표는 한 쪽에 함께 들어갈 수 없다. 두 표는 같은 저장 줄
(`vpos=0`, `lh=69746`)을 공유한다.

## 한컴 기준

`pdf/issue6946/44529-logistics-policy-rule-amendment-hwp-2020.pdf` — 한컴 engine 2020 산출(원본
`lastSavedWith` 가 hancom-office-2010 이라 `visual_fixture_evidence.md` §3.5.1 의 2020
버킷). **13쪽.** 인쇄 `- 7 -` 은 `(앞쪽)` 서식만 담고, `- 8 -` 은 `(뒤쪽) 행정정보 공동이용
동의서` 로 시작한다. 산출 메타데이터(job id·엔진·해시)는 `MANIFEST.json` 에 있다.

## 수정 전 rhwp

12쪽. 7쪽에 두 표가 모두 앉아 `y=102.0..1005.3` 과 `y=125.9..1020.4` 로 879.4px 겹친다
(`dump-pages` `used=924.2`). 원인과 수정은 `tests/cases/issue_6946_block_seated_float_sibling_gets_its_own_page.rs`
머리말을 본다.
