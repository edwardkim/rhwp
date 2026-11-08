---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7445_rebase_repair_stage25.md
last_verified: 2026-10-21
---

# M100/7445 리베이스 후 보정25 — 저장 원점과 중첩 첫 프레임

Issue: #7445, 관련 #1488.

현재 부모 HEAD `a36e45e80f2d370a24d6da108f84f084673e7965`, fixed base `2e3b4f880bd11d35398e6ea0100846611f13a72d`의 승인된 로컬 보정이다. 전체 #7445 해결이나 PR 준비 완료를 뜻하지 않는다. 지침·원격은 변경하지 않았다.

## 원 계약과 관측

#1488의 원 해결 PR#1489(c7dbe8a2c930333d228dc1deee9f78155c39bbf1, merge6fb1666d847691d7dc60c161edb925f6634ed790)는 빈 분할 꼬리와 HWPX의 낡은 forward vpos를 보정하고 p10의 CSAP 제목 아래 표를 보존했다. 원 본문·논의·diff와 #7902/#7941의 후속 관계 검사(84764fa21c96cd5f8a0e158658e4162a4fb3ce9e)는 ignored924에 보존했다. 현재 p10 표의 실제 내용은 공공데이터 실무매뉴얼이며 section1/para1/control0이다. 명칭만 보고 다른 표를 진단하지 않았다.

HeightCursor는 현재 p10의 저장56771HU를 이미 낡은 forward 원점으로 거절했지만, 저장 TAC 단축이 첫 zero-frame 인증만으로 그 값을 다시 수용했다. 표는853.32CSSpx로 내려갔고 독립 Print의 제목 아래 표가 Native9.05432/fresh WASM9.24732%였다. 기존 HWP/HWPX 제목→표 간격 검사는 유지하고 구현을 고쳤다.

별도 p7의 제26조 셀은 HWP에 저장 child reset/첫 상자10882HU가 있지만 HWPX는 전체 child12482HU의 연속 줄로 저장되어 있다. 원본 parent cell27451HU, padding282HU, host 마지막 저장 줄14080HU, child offset1124HU·바깥566HU와 첫 child의 저장 잉크10600HU+padding282HU를 대조하면 내용 끝26652HU, 원본 Center 이동3.446667CSSpx다. HWPX의 scalar367.12px는 source 물리 상자366.013333px와 달랐다.

## 생산→측정→배치 소비

1. `HeightCursor::vpos_adjust`의 기존 stale-forward 판정·문턱은 유지하고, 거절한 source 원점만 명시적으로 전달한다. 구역 vpos의 기존 커서 구성을 공통화하고 표 문단 경로가 두 번의 저장 TAC 준비 전에 같은 판정을 조회한다. 정상 table의 원점·page base를 이 관찰에서 새로 적용하지 않는다.
2. `StoredTacPage`가 현재 para/source 원점의 거절을 받아 단축 계획에서 이를 되살리지 않는다. 일반 TAC fit/paint가 공통 수용 원점에서 계속 진행한다. column 전환은 거절 기록도 초기화한다.
3. `stored_mixed_opening_frame`는 reset child의 기존 닫힌 프레임을 보존한다. 연속 child는 전체 저장 잉크+padding이 선언 child를 정확히 닫고, parent 첫 선언 상자에 선택 source prefix가 fit하며 다음 줄은 fit하지 않을 때만 같은 물리 높이·내용 끝을 반환한다. 문서ID/쪽 번호/임의 px 상수는 생산 조건에 추가하지 않는다.
4. `CellCutFrame::height`의 블록/row-offset 경로와 `row_cut_content_height`가 그 인증 결과를 직접 소비한다. 낡은 scalar와 max하면 같은 source 상자가367.12로 다시 커져 paint의 원본366.013333 인증을 잃는다. 일반 ordinary 컷도 같은 query로 source 예산 부족을 거절하며, 그 뒤 scalar capacity 비교·tolerance도 인증한 동일 물리 높이를 소비한다. 논리 유닛 전진은 고아/원문 소유 판정을 위해 보존한다. `table_partial`은 source 프레임의 정렬을 windowed Top 최적화보다 먼저 확인하고, 같은 물리 상자/선택 내용으로 Center를 그린다.

## 단계별 실제 증거

- 926은 table 문단이 일반 vpos snap을 호출하지 않아 거절 기록이 전달되지 않았다. 기존19검사18PASS/1FAIL이므로 미수용이다.930은 같은 cursor query를 table 경로에 연결해19PASS.
- 933은 두 형식 각18쪽·양쪽 backend 전72개 측정을 끝냈다. p10을 복구했지만 HWPX7은87.01856/86.86729%로 미달이었다. 이 상태에서 새 회귀를 정식 추가하지 않았다.
- 935는 연속 source query를 마련해도 후속 scalar max가 이를 무효화하여7쪽 출력이 그대로였다.941/942의 기존 opt-in trace에서 child plain=true·reflowed=false·정상 저장 줄을 확인했다.
- 943은 인증 물리 높이를 직접 소비한다.945의7/8/10쪽은 두 형식 모두90% 이상이며 HWPX7은99.60978%.944의 기존19+관계 진단1은19PASS/1FAIL→20PASS.
- 946 fresh WASM은 Mac locked web/pkg/no-opt 로컬 대체 빌드다.947 전쪽은 HWPX 최저92.05401/92.05433%(15쪽), HWP93.07060/92.96483%(16쪽), 각18/18·미달0·누락0이다.7쪽은99.60978/99.61492%,10쪽은99.90903/99.90809%다. 입력/PDF/문턱은 바꾸지 않았다.
- 955의930/932→943/946 전쪽 대응 비교는 HWPX7만 변경되고 HWP18쪽/나머지HWPX17쪽 PNG가 각 backend에서 byte 동일하다.0.1%p 하락0쪽이다. 고정 devel 전체 비교와 #8064 하한 감사는 별도 남는다.
- 948의 일반 nested wrapper42065 Native17쪽은 전후 PNG SHA가 모두 같다. 그 대조군의 기존 PDF 미달을 수용한 것은 아니다.
- 953의 두 형식 전18쪽 Table/Cell/글줄/텍스트 순서와 source pi/ci·row/col 변화는 기하 외0이다.7→8쪽 child 전체 문자는 원본과 공백 제외 동일하며 누락/중복0이다.

## 정식 회귀와 직접 출력

전체 시각 선행 조건 후 `tests/cases/issue_1488_stored_frame_owners.rs`에 실제 소유/흐름·포함 및7→8쪽 자식 전체 문자열 보존 관계를 추가했다. Native/fresh 양쪽98% 이상인10쪽의 실제 Native PNG 윗선을 독립 Print stroke S의 y723.409pt, MediaBox top841pt→156.788CSSpx±2로 보호한다. 원본1488의16검사·Stage24 캡션3검사는 그대로 유지했다.

949 초안의137.129333px는 PDF clipping/background path를 윗선으로 잘못 읽은 기대값이었다. Native FAIL을 제품 회귀로 세지 않고 초안을 미수용 보존했다.950에서 두 독립 Print의 실제 stroke S와 Native default/public-font PNG가 같은 hash임을 확인하여 글꼴 탓으로 돌리지 않았다. 정정한951은 같은 테스트 소스로 frozen908 0PASS/2FAIL→943 2PASS(1.13초)다. 최종 formatted source957도 frozen908 0PASS/2FAIL(1.23초)→960 2PASS(1.29초)다. 초기951과 중복 합산하지 않는다.

실물 HWPX의 연속 child에 ordinary/row-offset 두 수용 경로의 정확한 원본 예산·9HU 부족 예산을 추가 검증한다. 기존 HWP reset child의 동일 경계 검사는 제거하거나 완화하지 않는다. 968에서 새 source 단위 함수를 늘리는 대신 기존 reset 계약 함수에 모든 추가 assertion을 보존했다. cfg(test) 부분 재배치 전후 runtime prefix가 byte 동일함을 확인했으며 제품 코드960/961과 같은 출력이다.

[전쪽 TSV·원본/PDF/build SHA·Native/fresh WASM review와 overlay](../pr/assets/issue7445/completed-20261009/stage25-20261021/README.md)를 보존한다. 변경7/10쪽과 이어받는8쪽을 직접 판독했다. 기존15/16쪽 미세 간격·그림 외곽·가는 괘선/자형 차이는 남고, 높은 raw를 엄격RGB 동일성으로 쓰지 않는다.967에서 실제 Studio7736 초기 JS/WASM 응답과root pkg/Studio public/frozen961 hash를 대조했고 초기화true/오류0이다. actual36/문서CanvasKit 수용은 아니다.

## 남은 전체 범위

새 head 전체 nextest·lint/build/Skia·전체 문서 corpus/base/head 대응·actual36, #8064 PrEP1/법령2·13·36쪽 하한, 과학 시험지 본문 기준선, sample11 머리말 그림TAC 검사와 생성 JSON42개 감사/정리는 별도 진행한다. 이전 전체 회귀에서 부분 통과 수를 빼 새 전체 결과라고 쓰지 않는다. PR/push·외부 게시에는 사용자 새 승인이 필요하며 전체 목표는 ACTIVE다.

## 추가 실패의 보정과 최종 확인

956은 새2/기존16/캡션3 PASS였으나 root57건 중56PASS/1FAIL이었다. 정확한 원본 예산에서 ordinary 컷이16 대신15까지 수용했으므로 원 기대16과9HU 부족 예산을 유지했다. 기존 source frame guard 뒤의 scalar capacity 비교를 같은 인증 물리 높이로 연결한959에서 새·기존2PASS를 확인했다.

Native960은 native-skia release-test/jobs8 재빌드 exit0, frozen CLI SHA `6cf280200468c670b8e06aa821c14d97a56c1562d3e5c1fc705860bdba7c6f05`다. fresh961은 locked web/pkg/no-opt273.492초 exit0, root 재컴파일 및 root pkg/Studio public 일치다. JS SHA `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`, WASM SHA `0bc3fc93aa2436da75e73a5ee2ff3a5edb9c6006104004a2a412b1620605e341`이다.

962는 최신 제품 소스로 두 형식 Native/fresh WASM 각18쪽을 다시 산출했고 최저·미달은947과 같다.969에서 최신72쪽 PNG 모두943/946과 byte 동일함을 확인했다.930/932 대비0.1%p 하락0이며 fixed devel 전체 보존 검증과 혼동하지 않는다.966의 일반 wrapper17쪽도943/960 PNG 동일이다. 제출용 after TSV와 대표 PNG는962의 실제 결과를 복사했다.

최종968은 새 integration2PASS(0.839초), 기존 RowBreak16PASS(0.467초), 캡션 대조3PASS(0.328초), 커서·기존 첫 프레임 계약56PASS(0.665초), 합계77PASS/0FAIL이다. source-side 단위 검사 수는 늘리지 않았고 연속 HWPX assertion은 기존 reset 계약 함수에 보존했다. manifest/check·unit-tier/check(base2e3)·fmt/diff 모두 exit0이다.963 대기 driver는 실행 계획 정정을 위해 시작 전 종료한 기록이며 검사 통과 근거가 아니다. 마지막 cfg(test) 정리의 runtime prefix 동일성은968의 hash로 입증하며 동일 WASM을 중복 빌드하지 않았다.

이 단계 결과는 표 두 입력의 집중 검증이다. 다음 단계의 머리말 그림·품질 하한 등 남은 필수 범위를 해결한 뒤 최신 전체 회귀와 전체 영향 시각/소비 검증을 완료한다.

## 생성 결과 원장 정리 후속

2026-10-21의972/978 소비 감사에서 남은 JSON42개 중40개는 생성 결과,2개는 정규 테스트가 읽는 FontEnvironment 입력으로 구분했다. 생성40개는 SHA를 유지해 ignored output978/raw로 옮겼고14개 자산 폴더의 `verification.md`에 당시 source/build·수치·판독 기록과 원문 SHA를 보존했다. PNG·TSV·원본·PDF와 두 정규 글꼴 환경 입력은 변경하지 않았다. 이 정리는 이전 결과를 최신 source 통과로 승격하지 않는다.
