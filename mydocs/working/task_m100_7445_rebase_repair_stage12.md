---
kind: working
status: draft
canonical: mydocs/working/task_m100_7445_rebase_repair_stage12.md
last_verified: 2026-10-20
---

# 7445 리베이스 보정12: 원PR8064 시험지의 단 소유와 글꼴 체인 검사

parent455aac1ac07f58eac8b4a553a227cf09c0ca2bf1/base4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa.
사용자 지시에 따라 [원PR8064](https://github.com/edwardkim/rhwp/pull/8064)의 본문·코멘트·실제
반영9b01d5d1447220b3a7345703e79ad42cf3f502f4·원 검사/TSV를 읽었다. 원문 face·동일 기준선·
전쪽 수식 집합/소유·독립 Print의 쪽 소속이 원 계약이다. 기존 완료 범위·실제 회귀를 복구하며
#7445 전체로 확대하지 않는다. PR은 새 사용자 승인 전 만들지 않는다.

## 실제 원인과 공통 보정

과학 원본 all-42는고정base4쪽/후보5쪽이다.2쪽 왼쪽 마지막 TAC 표가 오른쪽 단으로 밀린 뒤
다음 문단의 명시 단나누기가 다시 쪽을 넘긴다. 앞 글줄의 실측 내용 하단이19HU(0.253333px)
늘었고, 같은 영점 축을 가진 독립 저장 표까지 그 차이만큼 재기준화해 마지막 표가넘쳤다.
첫 원본 글줄이 영점 소유를 인증하고 활성 base가0인 경우 단일 저장 개체는 그 물리 원점을
유지한다. 실제 흐름 하한·선행 내용·실측 외곽·캡션·각주·fit 조건은 기존 공통 Query가검증한다.
다른 축/재조판/합성 첫 줄에 같은 인증을 추정하지 않는다. 문서번호/쪽번호 예외는 없다.

소비: stored_tac_page의단첫원본줄/활성축 → stored_tac의same_frame → 공통origin/occupied_end
수용 → InlineBoxPlacement의y/advance_end/StoredColumnFrame → table layout 실제원점.
앞 텍스트의 작은 높이 증가와 개체의 독립 저장 축을 혼동하지 않고 수용·paint가 같은 계획을쓴다.
기존 all-42는수정전FAIL(5vs4)→수정후PASS이며 원문 수식 다중집합·누락/중복·순서도통과한다.

국어 기준선 실패는 전체 CSS체인 `HY신명조,Haansoft Batang`을 단일face와비교한 검사오류다.
원본 첫face를기존공통primary_font_name으로 읽는다. HY신명조본문/한양신명조구두점의독립소유,
font_position−10,같은실제그리기기준선과px좌표는유지한다. ignored508의같은진단에서원검사는
전후FAIL,체인판독만보완하면양쪽PASS다. 전쪽90%와14쪽Native99.83022/WASM99.84695%·
직접판독후기존함수만수정했다. 검사삭제/허용치확대/새함수추가는없다.

## 검증과 남은 보류

- Native507 jobs8빌드, fresh509 locked web/pkg/no-opt 실제컴파일·root/public동일성을확인했다.
  Mac로컬대체빌드이며Docker최적화가아니다.515의실제초기화GET200/no-cache·JSrewrite·WASM
  hash/ready를확인했다. 문서paint actual36수용은아니다.
- 원PR의정식조건은Print96DPI/raw2px다.192DPI선행진단Native44쪽은과학3/4·국어17미달,
  WASM은과학4쪽후국어캡처ENOSPC로중단됐다. 그부분을수용하지않고원PR과같은조건으로
  base/후보를새로전쪽산출했다.두DPI의PNG/TSV를혼합하지않는다.
- matched96에서각44쪽:과학4쪽Native최저93.11518/WASM93.04014%,국어HWP/HWPX각20쪽
  Native96.27225/WASM96.29373%,미달/누락/쪽수차이0.국어두형식의20PNG씩은backend별
  byte동일이다.14쪽본문/구두점·외곽·도형,과학2쪽표와다음단,3/4쪽잔여를직접판독했다.
- 고정baseNative44쪽의같은조건Decimal비교는과학3쪽95.06581→94.73771%,0.32810%p하락1건이다.
  **PR보류**로기록하고보정한다.최저90%나국어개선으로상쇄하지않는다.임시517에서한줄자연글자
  전진은base/후보동일하고후행공백정수반올림6px→실측6.4px때문에분배가달라짐을확인했다.
  독립PDF의B원점344.0/후속를370.72001은후보344.0/370.7에가까워해당정확한보정을되돌리지않는다.
  미일치증가타일과다른영역의원인을계속분리한다.글꼴문제로단정하지않는다.
- 정식Native516은원8064본문/수식/쪽소유와기존저장TAC·3930·1937대조 **83PASS/0FAIL**,
  실행208.531초/선택제외1917/exit0.전체nextest가아니다.최종fmt는조건한줄의줄바꿈만정리한뒤
  check/manifest를재확인했다.원본검사의논리와제품hash는같다.

[TSV·대표review/overlay·원장](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/exam-source-column/README.md).
ignored506~517에진단/검사/전체TSV/빌드로그를보존한다.종료된중복진단과옛CLI/lib를
SHA목록/lsof확인후정리했다(511:294135220byte,513:1025019464byte,514:6894757848byte).
정리된옛바이너리는재사용불가이며source/producer/PNG/TSV/log와필수before/base/current,
원본/PDF/글꼴/공유target/다른작업장은보존했다.전체회귀/필수범위게이트/기능보존/actual36은남는다.

최종fmt의조건줄바꿈으로발생한generated harness drift는prepare재생성후고정base manifestcheck로
복구했다.1643sources/7175attrs/48targets이며83개검사함수의논리와제품소스는같다.
