---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase20-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase20-goal-checkpoint.md` · 원문 SHA-256: `a1a40f239246b9dc9c201ef66492fb3d61670fb07695b6bd2a532102055d13f9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 20차 체크포인트

목표 ACTIVE, 완료 아님. 직전 응답은 모순에 대한 설명으로 권위 상태를 바꾸지 않은 no-progress였고, 이번 실행은 현재 상태 재검증·독립 원본 확인·실제 집중 실행·전쪽 캡처·원장 판정·이슈 본문 갱신을 수행한 progress다. blocker나 기다리는 프로세스는 없다.

## 작업장과 승인

/Users/tsjang/rhwp, audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce. 시작/최종 Git clean, 제품·테스트 수정 및 개명 0건. 전수 조사 뒤 부적합 검사를 실제 보정·issue_8232 새 명명·helper/module/suite/생성/스크립트/E2E 갱신·필수 검증까지 해야 한다. 원 검사 방식의 무조건 보존은 요구하지 않는다. 유효한 결함 검증 목표만 독립 근거와 현행 지침에서 재구성한다. PR/push는 아직 승인되지 않았으며 subagent·다른 task 메시지·예약·메모리 쓰기는 하지 않는다. 공유 target/원본/PDF/다른 작업을 보존한다.

## 본문 정정 확인

live #8232를 읽으니 이미 3절이 '기존 검사 형태·단언·동작의 유지를 전제로 하지 않는다'로 정정돼 있었다. 같은 정정을 중복 게시하지 않았다. 이후 아래 실제20차 결과와 최신 원장 수치를 본문에 추가했다. remote/local 본문 바이트 일치를 다시 확인했다. issue는 OPEN, updated_at2026-10-25T22:25:06Z이다.

## 직접 분류와 원본 근거

이전 컨텍스트에서 전체를 읽은6개 파일의 마지막 누락 README/Stage2를 이번 실행에서 보완했다.

- issue_536_portable_glyph_geometry.rs 969줄:21검사.
- tests/support/issue_536_portable_glyph_geometry.rs 286줄:9helper. include caller는 위 파일 하나다.
- issue_7028_partial_table_diagonal.rs 281줄:9검사/7helper.
- issue_7061_stored_line_escape.rs 343줄:11검사/10helper.
- issue_6936_pdf_synthetic_bold.rs 245줄:9검사/8helper.
- layout_anomaly_glyph_band.rs 455줄:12검사/6helper.

총62검사/40helper=102함수, 이전 직접 판정2개와 중복돼 고유 직접 판정은100개 늘었다. integration-semantics-batch-35.json과 non-numeric-helper-semantics-26.json이다. f32 범위/decimal wire/자원 제한/작성 기하/현재 호출 결정성을 실물 페이지 px 핀으로 분류하지 않는다. 범위 밖 production 호출 폐쇄는 아직 pending이다.

inspect_phase20_raw_contracts.py는 rhwp 모델 없이 CFB/XML을 직접 읽었다. 제목 셀5개 모두 높이3928HU=52.373333CSSpx이므로 기존52.4±.1은 저장 치수와 양립한다. 7018 HWPX section0 p27의 두 줄은 textpos0/11, vpos39764/41924, height1200/15792, baseline1020/13423이다. (13423-1020)/75=165.373333이므로 진단기의165.37±.05는 원본 단위 차이 계약이다. 절대 y225.3/113.4±.1은 별도로 독립 Print·98% 자격이 필요한 핀이다. cell_box가 행열만으로 선택하는 표 소유는 더 확인해야 한다.

원 PR7036은 실제 merged/d408532ce9f333f7428bf91362829f2703f0c02a,7108은 merged/cd2d9e8a430e2664181326c684b7491aa451cd31이며 둘 다 현재 HEAD 조상임을 확인했다. 관련 보고서·fixture README·원 PR 본문과 현재 검사를 읽었다. 전체 API diff 출력은 과대해 잘렸으므로 전체 diff 완독으로 표시하지 않는다. 7028 쪽수 핀 보류 후속3a49836281도 Git 이력에서 확인했다. 현재 info/PDF 모두6쪽이므로 과거7쪽 문제를 현재 실패로 쓰지 않는다. provenance는 phase20-original-pr-provenance.json이다.

## 실제 전쪽 비교와 판독

run_phase20_layout.py exec44591/PID74670 terminal exit0,88.685692초. 드라이버0은 모든 시도 수집 완료이지 모두 시각 통과라는 뜻이 아니다. fcntl capture lock·capture총8·canonical PDFium/96CSSdpi/2px·compat2022를 유지했다. 기존 동일 소스 Native/fresh WASM 빌드 hash를 확인하고 PNG/TSV는 새로 생성했다. Mac web/pkg/no-opt 로컬 대체 빌드이며 Docker 최적화가 아니다.

- 정확한 HWP21761835 입력SHA d8e7e38b206f6e53b7d2d260115396f8a0dd94bd8b712bbc4428dc11bf66e5dc, 저장제품2010. 대응 PDF99d150b26805a29f3d3153528638f363e33f882d0e38ad11edb374baccdac127,6쪽. HWPX2176 조건과 혼동하지 않는다.
- Native6/6쪽:89.68453,89.04297,81.52731,47.26898,47.12885,40.01798. WASM6/6쪽:89.60801,88.94538,82.13254,48.10298,47.58334,40.45121. 양쪽 전6쪽90미달, capture/TSV exit1·re_review_required 유지. 누락·쪽수 불일치0.
- HWP 합성 굵게 입력 eefe5d89c29a8d7c8ba3a3148033fb3c5c2e960205ef1ff927ec36873089a3a7, PDF5b0dc916098847ed3c6e3eadae468f33a9810ee58de3e91c1e2bb2f3a7e2ae12.1/1쪽 Native99.94520/WASM99.91276, gate0/누락0.
- 정확한 합성 HWPX c751bf79...는 README가 '한컴 저장 전 수제 입력/LineSeg 없음'이라 명시한다. 이 HWP Print를 해당 HWPX의 독립 위치 자격으로 사용하지 않는다. HWPX는 파서·face-name 검사 입력이다.

review/standalone overlay8개를 직접 열었다: Native 대각선p1/p6, Native/WASM 굵게p1, 각2종. 대각선은 존재하고 제목행의 모서리와 대응하지만 본문 줄바꿈·표 경계·후속 내용은 다르다. p6은 PDF의 설계 행 앞 조각 vs rhwp의 디자인 행 시작으로 소속 차이가 보이며 표 끝·비고도 다르다. 원래 대각선 누락의 재발이라 단정하지 않고 시각 미달로 남긴다. 굵게는12행 내용·순서를 확인했지만 WASM의 획과 새굴림 글리프 차이가 보인다. font catalog에 New Gulim face가 없으며 한컴 당시 face와 현재 글꼴 환경을 동일하다고 주장하지 않는다. 실루엣99%만으로 확대 판독·글꼴 소비 완료를 승인하지 않는다.

phase20-layout-evidence/existing-pins-qualification.json에 이미지 SHA·판독·미달을 기록했다. 절대 px 자격 추가0건, 전체 시각 수용 false다.

## 기존 Native 실행

run_phase20_existing_tests.py exec71282 terminal0. metadata는 현재 source에서 suite008/017/021/027/028을 실제 수집했다(빌드29.80초). 실행 전후 실제 test binary 경로/SHA 일치.62/62PASS/0FAIL, Summary0.478초, 선택 target1210skipped, driver실행3.796168초. 새 검사FAIL→PASS·전체 회귀·시각 수용이 아니다. phase20-existing-tests.log와 producer를 확인한다. 현재 cargo/nextest/rustc/capture process는 없다.

## 원장과 검증 오류 정정

10450그룹/2106파일, 미분류4336/수치 의미 판정5325/독립 근거 pending451/semantic notes126/file context212. 직접5776개 모두 원장 연결·현재 source SHA 일치·누락0이다. 초기 검증 스크립트가 glob 순서로 이전26개 기록을 최신으로 잘못 골랐다. builder와 동일한 sorted 우선순위로 재검증하여 모두 일치했다. source/원장을 조작해 실패를 숨기지 않았다. phase20-semantic-ledger-validation.json에 정정했다.

paired catalog는102조건/각backend2569쪽/입력·PDF200파일.20차 두 조건을 실제 capture/auxiliary gate·90문턱·전쪽 count·hash 검증 후 추가했고 미달도 포함했다.

## 남은 작업

전수 조사 미완료4336그룹과451개 pending·전체 동적 호출 폐쇄를 계속한다.19차 이전 원장/체크포인트의 미해결도 유지한다.20차 exact 조건을 중복 실행하지 않는다. 다음에는 남은 source-side/공용 helper 후보를 직접 읽고 알려진 selector의 실제 owner/독립 근거 연결을 보완한다. 전체 조사·판정 이후 유효한 목표에서 부적합 검사 재작성·issue_8232 개명·호출/생성/등록 정리·출력 보정·재검증을 완료해야 한다. 아직 PR 없음, source수정/개명0건이다.
