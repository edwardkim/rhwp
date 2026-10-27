---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase36-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase36-goal-checkpoint.md` · 원문 SHA-256: `8ba77b23d5be1ee3ca1c352621831d49082843ccff96e5b54d8d5c54e5af96bf`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 36차

목표는 전수 조사·독립 판정 → #8232 반영 → 현행 지침에 맞지 않는 검사/필요한 공통 구현 보정 → 실제 보정 파일·함수를 issue_8232로 명명하고 helper/module/suite manifest/생성기/스크립트/E2E 호출 갱신 → 실제 검증이다. 잘못된 기존 판정은 보존하지 않고 독립 근거가 있는 검증 목적을 유지/재구성한다. 정상 검사를 이름만 바꾸려고 수정하지 않으며 이전 이름 wrapper·중복 등록을 만들지 않는다. PR은 나중, 현재 push/PR 승인 없음이다. 이슈 본문 수정과 필요한 MCP Print는 승인됐다. agent/task 메시지·예약·메모리 쓰기는 없다.

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD/base=238de6de41a208648f4ade6b494fe433a81c1fce. Git clean이며 제품/검사 변경·개명은0건이다. 이번 자료도 ignored output/issue8232-audit 안에만 생성했다. 원본/PDF/다른 작업장/공유 target/pr-review는 보존했다. Cargo는 한 실행만 사용했고 jobs8/tests threads8/capture 총8을 유지했다. 모든 이번 실행·MCP job·캡처는 terminal이다. 목표는 ACTIVE이며 완료/차단/중지가 아니다.

## 직접 조사와 실제 실행

다음 네 파일 전체를 읽었다.

- tests/cases/issue_5961_hwpx_lineseg_axis_projection.rs:193줄,9함수/8검사.
- tests/cases/issue_6624_chart_default_layout.rs:505줄,22함수/15검사.
- tests/cases/issue_6778_square_beside_lane_render_flow.rs:439줄,20함수/7검사.
- tests/cases/issue_6981_split_straddle_row_height.rs:349줄,17함수/7검사.

총68함수(37검사/31helper) 중 신규61·기존7재판정이다. 원장 integration-semantics-batch-49.json/non-numeric-helper-semantics-42.json과 source SHA를 따른다. 생산 chart renderer의35~75/390~463줄, HWP common parser의관련 부분과 과거 원 PR 검토 문서도 읽었으나 그 파일 전체의 함수 판정으로 세지 않았다.

현재 generated suite009/015/021/028과 nextest metadata로37개 등록을 확인해 실제 실행했다. 결과37PASS/0FAIL, Summary167.071초(2slow), 필터 밖1,032개 skipped다. wrapper168.503855초다. 마지막 the_pages_that_carry_it_have_no_escaped_line은 실제 CPU100%로166.767초 실행했으며 출력 공백/slow marker로 중단·재시작하지 않았다. source·원본·바이너리 전후 SHA가 같다. phase36-existing-tests-producer.json/log를 따른다. 전체 회귀/시각 수용/새 정식 회귀 FAIL→PASS가 아니다.

## 확인한 검출 누락 두 원인

1. 레인 root233은 source para13의 첫 줄이 아니라 표 오른쪽 창에 들어오는 어떤 TextRun의 최소y를 고르고 table_bottom−100px보다 위인지 본다. 현재 실제 첫 para13 TextLine1개(y762.5466666667)를 제거해도 기존 단언이 PASS했다. 정상/변형 각각1PASS다. root233과 nested lane_walk266은 같은 원인이다. 원문 소유 첫 줄·본문 보존과 저장 줄/개체 관계로 보정할 대상이다.
2. #6981 원본 대조군의 실제 escaped 결과는 현재0개인데 옛3개 상한이 남아 있다. 실제 수집 뒤 다른 새 오류3개(각5800문자)로 교체한 분석 사본도 기존 root189를 PASS했다. 정상/변형 각각1PASS다. 옛 오류 개수를 현재 오류 identity·크기 무회귀의 근거로 쓰지 않도록 보정할 대상이다.

phase36-predicate-counterexamples.json와 phase36-predicate-probes/에 source/rlib/binary/log hash·명령·관측값을 보존했다. 원본 단언을 바꾸지 않은 ignored 분석 사본이며 제품 원본의 새 결함이나 정식 CI 등록·새 FAIL→PASS로 주장하지 않는다. 두 사본 실행에서 source 원본 SHA는 같다.

원 #6784는 미병합 CLOSED이고 실제 통합은 #6847/a3a30d99d4aefb15ed0dbed96645eb1ca3595099다. #7143의 실제 merge는 a2cc0d236f0b348e00d6ee14af1ff86323a93c67다. chart 원 #6626도 merged_at이없고 실제 통합 #6645/2edbe62e5dc74db58c33df2c006ae587f86a1a71에서 수용됐다. 실제 merge tree의 같은 단언과 현재 조상을 확인했다. phase36-origin-chains.json와 각 PR metadata를 따른다. API의 merge_commit_sha가 있어도 merged_at 없는 원 PR을 병합으로 취급하지 않았다.

## 원시 입력과 단위

rhwp 모델 없이 CFB에서 Square host 표의 hpos0/폭17060HU, 실제 다음 레인 cs17626HU·vpos50106HU·line_height1400HU를 읽었다. 과거 주석17070HU와 실제 우단을 구분했다. aift HWPX의 host 직계 linesegarray는0/24이며, 최초 탐색에서 descendant 전체를 섞어 읽은 중첩 표6줄은 최종 근거에서 제외했다. 초기 그 선택 때문에 난 AssertionError는 제품/검사 실패가 아니다. phase36-independent-raw-contracts.json/log가 정본이다.

0/8/24/25/32/48은 논리 slot/source offset이고75HU 예산 변화는 입력 margin의1CSSpx 단위다. 상대 객체 포함의0.5/1.0 bound를 현재 실물 page 절대 원점 pin과 일괄 취급하지 않는다. 반면 내부IR slice의 고정4쪽은 독립 한컴 저장/PDF가 아니며 원 PR도 devel/PR 재현값4쪽이라고 구분했다. 그 기대의 필요성과 독립 근거는 pending으로 남겼다.

## 차트의 기존 PDF와 새 MCP Print

기존 다섯 PDF는 텍스트 추출이 없고 페이지 이미지다. Creator는Hwp2022이지만 파일명2020/Creator만으로 실제 Print 엔진과 방법을 확정하지 않았다. source info의 저장 제품은5개 모두Hancom2022/12.0.0.535다. 기존 원본/PDF를 보존하고 같은 입력을 MCP engine2020/timeout1800초로 Print했다.5job 모두succeeded, 실제11.0.0.9136, pdf_print_method0, pdf_output_mode hancom2020_pdf_driver_one_up을 확인했다. start/status/download·source/PDF SHA는 mcp-phase36/에 있고 인증정보는 출력하지 않았다. 새 PDF는 감사용 ignored output에 있으며 아직 기준 파일로 커밋하지 않았다.

같은 frozen Native/fresh WASM producer와 정본96CSSdpi/2px raw로 기존PDF5조건 및 새Print5조건을 각각 양쪽 전1쪽씩 실행했다. 이전 실행은 끝난 상태에서 다른 PDF 조건을 시작했으며 총 capture8을 넘지 않았다. source/fonts/backend/비교 문턱을 바꿔 미달을 숨기지 않았다. Mac WASM은 기존 locked wrapper web/pkg/no-opt 로컬 대체 빌드 재사용이며 새 Docker 최적화 통과가 아니다.

| 문서 | 기존 PDF Native/WASM raw | 새 Print Native/WASM raw |
| --- | --- | --- |
| 묶은세로막대형 |91.26030/89.44073|90.76307/87.88801|
| 꺽은선형 |85.83979/80.90940|85.74791/79.27503|
| 2차원원형 |97.56036/96.04306|97.34921/94.74776|
| 시가고가저가종가 |75.57078/69.83813|74.38419/67.96440|
| 직선이있는분산형 |87.99936/81.65286|87.37078/78.91848|

기존/새Print 각각10개backend/page 중7개가90미달이며, 다섯 문서 양쪽98자격은 모두 없다. exit1은 정상 산출된 raw gate 미달이지 build/환경실패가 아니다. 과거 검토 page diff1%대를 현재 raw98%로 재인용하지 않았다. 기존PDF Native5종 review/overlay10장과 새Print column/stock/pie 양쪽12장, 총22장을 직접 읽었다. 새Print line/scatter의TSV를 모든 PNG 직접 판독으로 확대하지 않았다. 축·계열·범례·라벨의 위치 차이도 보여 글꼴만 원인으로 단정하지 않는다.

별도 정책 위반: hancom_default_layout_pins_430x250 root385의 y35.47/60.85/208.67±0.05는 래스터 실측35.5/61/209를 생산 renderer에서1.9T/1.36T/3.1L로 근사한 값을 다시 정답으로 고정한다. 독립 subpixel 근거나 같은 실물 차트의98자격도 없으므로 실제 보정 대상으로 판정했다. 실행 반례두 원인과 이 정책 위반하나를 구분하며 관련 직접기록은4개다. 명시XML의pt/EMU·축 소유·원본 서식과 수평 선의 상대 동등성은 보존한다. 다른 경험 서식·비율과 IR 고정쪽수의 독립 근거는 별도 pending이다.

## 누계·이슈·잔여

직접7,175기록/759소스 파일, 원장11,381그룹/2,127파일이다. 미분류3,876+메모330=4,206개, 독립근거 pending511개는 직접기록 안에 포함된다. 통합 시각목록126조건/각backend2,601쪽은 실패·기존재사용도 포함한다. 같은5문서의구/신PDF조건을10개새문서나통과수로세지않는다. validate_phase36_ledger.py가 source/hash·현재 등록/37PASS·두반례·raw단위·5Printjob·구/신20backend/page·gate미달·22장판독·actualmerge ancestry를 검증했다.

#8232에 결과와 현재 집계를 반영하고 API 재조회에서 말미 개행까지byte 동일을 확인했다. updated_at=2026-10-26T02:04:08Z, bodySHA804c018ff5ca01bee38f60f7c336c451091d546f9b548f5ad2bd10a34220a40b. 이슈OPEN/완료checkbox미표시/PR없음이다. 실제 제품/검사 보정·개명은0건이다.

다음에는 남은 실제 문서 검사·중첩/공용 helper·source cfg·생성/동적 corpus·Studio E2E 경로를 계속 조사하고, 필요하면 현재 pending의 독립 입력/실행 근거를 보완한다. 차트 Print 실패는 기대값 삭제/완화로 감추지 않으며 실제 보정 단계에서 공통 출력과 정상 서식·소유 검증을 함께 해결해야 한다. 이번37개·네 분석1개씩·완료한 구/신 차트20backend/page·5Printjob은 자동 중복 시작하지 않는다. 전수 조사 완료 뒤 실제 보정·issue_8232 명명·모든 소비/등록 갱신과 필수 재검증까지 계속한다.
