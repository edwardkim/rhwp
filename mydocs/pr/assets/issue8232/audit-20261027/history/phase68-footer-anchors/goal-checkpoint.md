---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase68-footer-anchors/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase68-footer-anchors/goal-checkpoint.md` · 원문 SHA-256: `d05bf651c10583bdd0340212e5eed1c70bc5d2709e57f8fd7ac75f7d8bcdbd5c`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 68차 체크포인트

목표 ACTIVE: 전수 조사·독립 판정 완료 → #8232 갱신 → 부적합 검사/필요 구현 보정 및 실제 보정 대상의 issue_8232 개명/호출 경로 갱신이다. 제품/테스트 수정0·개명0이다. PR은 나중에 하며 새 push/PR 승인은 없다. 새 agent/타 task 메시지/예약/메모리 쓰기를 하지 않는다.

작업장 /Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, head01ced3d42724285c8f9765a453393ab6af326617 그대로다. dirty는 기존 출처 미상 rhwp-studio/public/rhwp.js 97추가줄뿐이며 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존한다. source/samples/PDF/공유target/다른작업장을 보존한다. 최신devel 재동기화는 이미 완료했으므로 요청 없이 반복하지 않는다.

사용자의 반복 빌드 지적 이후 Cargo/prepare/새 빌드는 하지 않는다. 같은 head의 기존 완료 metadata와 현재 binary SHA가 일치하는 대상을 libtest --exact --test-threads8로 직접 재사용한다. 코드 수정 또는 필요한 미빌드 대상이 있을 때만 필요한 범위를 묶어 빌드한다.

현재 active-audit.json은 phase68-footer-anchors/audit-ledger.json을 가리킨다. 누적12296그룹/2139파일, 직접8701함수/Rust6026검사/781파일이다. 의미판정8201/file-scope8/근거pending500/미분류3301+의미메모97+문맥189=3587이다. 68차는 쪽번호 source13함수/7검사/6helper, 신규직접7/신규Rust2/새그룹6이다. 기존pending6개는 독립원점 확인 후에도 최신양쪽시각자격 미확인으로 유지했고 해소 수0이다. 이전67차의139함수/56실행과66차의131함수/78실행은 각 디렉터리의 근거를 따른다.

68차 실제 입력: tests/cases/issue_1133_page_number_bottom_margin.rs 전체213줄을 읽었다. 원본HWP/HWPX는 전3쪽 소유/숫자를 검사하되 절대1080.419270833±2는3쪽만 검사한다. HWPX 한 속성만 바꾼 footer15/bottom15 ZIP과 source margin을 비교했다. footer/bottom은2834→4252HU이며 bottom source 차이18.906666667CSSpx, 독립 PDF의 실제 이동18.859375CSSpx, 반올림 잔여0.047291667<기존0.15다. 상대 여백 관계와 절대3쪽 핀 자격을 구분한다.

독립 PDF 원점은 PDFium FPDFText_GetCharOrigin으로 직접 읽었다. 원본1133 HWP/HWPX3쪽 및 footer15 3쪽1080.419270833, bottom15 3쪽1061.559895833, style7336 6쪽1062.719970703, 각주1937 25쪽1061.7197265625, sample5-HWP5-2022 5/6쪽1091.519938151, release6575 6/8쪽1082.079996745가 실제 숫자 원점과 일치한다. PDF본문 하단 숫자도 포함된 후보 전체를 보존했고 자동번호와 구분한다. 정확한 입력/PDF와SHA/원점은 independent-footer-origins.json에 있다.

과거 pending 메모3건을 정정했다: assert_original은 전3쪽 절대 핀이 아니라3쪽만이다. zero_footer_margin_uses_the_empty_bottom_band_center는 실제5/6쪽이지1/32/64쪽이 아니다. zero_footer_margin_number_does_not_move_into_release_body는 실제6/8쪽이지2/3/8쪽이 아니다. 과거메모는 previous-selected-ledger.json과각previous_adjudication에 보존했다. source/test 기대값은 바꾸지 않았다.

같은 head의 기존regression_suite_028 binary를 source/hash검증 후 직접 재사용해7PASS/0FAIL/253필터제외/실행0.53초(전체0.873초)를 완료했다. Cargo/빌드0이다. producer.json/로그/validation.json에 연결했다. 대상sourceSHA는 실행전후 같다.

현재 PDF값의 독립 타당성과 옛 시각자격은 별개다. 기존 tracked footer-source-contract의12TSV를 다시 읽어 원본4종3쪽·style6쪽·각주50쪽의누락0/전쪽90이상/중요3·6·25쪽98이상을 확인했다. 원본3쪽양쪽100, style6쪽N99.88197/W99.88244, 각주25쪽N99.97779/W99.97744다. 이 자료는66859c...당시출력이며 최신head통과로 재사용하지 않는다. 검증문서의source/입력/PDF/hash/대표PNG연결을 따라야 한다. 현238de→01ced에는 여러renderer/composer/font/paint변경이 있어 byteunchanged소스만으로새시각결과를주장하지않는다. 최신Native/freshWASM전체쪽수/TSV와대표review/overlay판독이6pending에남는다. 이현재부족을보정완료나이관판정으로바꾸지않는다.

68차전미주height_cursor의경험완충pending12개도다시살폈으나독립새정답을확보하지못해원장을바꾸지않았다. 4/10/18/30/40/41px등생산상수와같은기대에관한기존phase40원PR/통합/보고서근거는유지한다. 소스카피를독립oracle로쓰거나pending을근거없이해소하지않는다. rootfile전체dump·SVG검색의과대한출력은관찰실수이며공식테스트실패가아니다. 향후rg는*.md/*.json/정확한범위·max-columns를지정한다.

#8232는68차누적수치로반영하고exactEOF readback/외부본문불변/이전상태역복구검증을완료했다. 본문UTF16길이65518이며여유18자뿐이다. 다음게시전현재본문조회및가용길이를계산하고정상최신상태만요약한다. 실제실패·외부본문·8266·완료조건을삭제하지않는다.

직전goal turn은독립PDF원점/실행/원장정정이라는progress다. 전체감사/보정은미완료다. 다음은남은미분류/문맥3587그룹의함수단위수치의미·호출경로·원PR근거를계속검토하거나,기존pending500의독립자료/현출력자격을확보한다. 새Rust95/삭제3/새JS callback1(schema2)은사용자제외유지한다. 다음단계를임의완료/PAUSED/blocked로바꾸지않는다.
