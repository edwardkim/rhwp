---
kind: working
status: draft
canonical: mydocs/working/task_m100_7445_rebase_repair_stage10.md
last_verified: 2026-10-20
---

# 7445 리베이스 보정10: 완료 병합 꼬리의 실제 행 상자 정렬

작업장은 `fix/7445-completed-20261009`, parent는 `979dbf4ebd72fbd0f72bb1efd01f882cdd16d7ea`다.
base는 `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`로 유지하며 이 단계에서 반복 리베이스하지 않는다.
PR/원격 작업은 새 승인을 받기 전 수행하지 않는다. 아직 완료 보고가 아니다.

## 독립 근거와 소비 경로

원본1937/독립 한컴 Print50쪽은 보정9와 같은 입력·PDF다. 473의 기존 unit 진단에서38쪽
원본306의 셀214(row45,col2,span3,Center)는 실제 상자61.386667px에 남은 한 줄13.333333px를
Top으로 그린다. 원문 `정책실현`은37쪽, `재정조달`은38쪽이며 순서와 내용은 보존됐다.
블록 시작 컷은 내용 소속을 알려 주지만 이미 수용한 일반 행의 독립 물리 최소를 없애지 않는다.

원 공통 Query214b2477a4/거대 표2290·2287/보정9의 source row chain을 읽었다.6906HU 병합
원본 최소는2302HU 일반 행3개와 같고 남은46/47행은 각각30.693333px를 실제로 소유한다.
이 높이는 스캔과 paint의 공통 `row_block_cut_physical_height`에서 나왔지만 정렬 소유는
전달하지 않아 block-start가일괄Top으로덮었다. 단독vpos0이나현재내용의작음은새근거가아니다.

후보는 공통 반환을 높이+독립 원본 최소 소유의 `RowBlockPhysicalFrame`으로 만든다.
스캔은 동일 높이를 소비하며 paint는 그 결과가 발행한 행별 source height를 보존한다.
최종 행 높이가 이후 override로 달라졌으면 그 근거를 기각한다. 순수 저장 병합 꼬리가
남은 유닛을 모두 소비하고 보이는 모든 행을 그 최소로 실제 수용했을 때 원본 정렬을 적용한다.
부분 종료 컷·미소비 꼬리·dirty/reflow·원본 최소 불명·변경된 행 높이는 기존 경로를 유지한다.
앞37쪽 prefix의 독립 정렬 근거는 이 증거와 다르므로 별도로 판독한다.

생산→소비: `stored_complete_block_row_minimum`→공통 row-block frame→scan의height/
paint의row plan+source height→최종height대조→completed span alignment→effective_align→text_y.
실제수용한상자를새로추정하거나clamp하지않는다. backend는이같은render tree를소비한다.

## 현재 검증

ignored475의 원본 문단 소속·실제 한 줄 가운데 정렬 관계 진단은 수정 전11PASS1FAIL이다.
정식 회귀 수를 늘리지 않았다. source4파일 후보의476 Native build/477 선택37·38·39·42·43쪽,
기존508쪽512JSON 대조를 진행한다. 같은소스의Native/fresh WASM전쪽90%와직접판독,
정식관계/선정가능한핵심px회귀,전체nextest·범위게이트·다른문서전쪽·actual36은남아있다.
임시로그/중간원장은 ignored output의 `repair473`·`repair475`~`repair477`에보존한다.


## 실제 실패 시도와 공통 컷 보정

476·478·480은 Native 빌드에는 성공했지만 같은12진단은11PASS1FAIL이었다. 성공 후보로
수용하지 않았다. 476의0높이 소비 완료 행을 정렬 공간으로 요구한 오류,478의측정 분기
원본 최소 미전달을 각각 확인했다.480에서도 실제 block-start의 컷은 존재하지만 정렬은
`is_rowbreak_straddle` 경로의 컷만 읽어 적용되지 않았다.482는 기존 `cell_cut_window`의
블록 창을 정렬과 실제 그리기가 함께 소비하게 했다. 기존 일반 행/높이 구제의 컷 보완은
유지한다. 같은12진단은 수정 전11PASS1FAIL→수정 후12PASS다. 별도 진단 driver의 초기
state파일 생성 경합은 실행 오류로 보존하고 수정했으며 제품의RED로 계산하지 않는다.

원본1937의 이번 suffix가 아닌37쪽 prefix 등 기존 정렬 차이를 전체 해결했다고
보고하지 않는다. 제출 범위를 #7445 전체 개선으로 확대하지 않는다. 이번 변경이 만든
회귀와 필수 리베이스 기능 보존은 계속 판정한다.

## 전쪽·대조·회귀 보강

- 485 Native와486 locked web/pkg/no-opt fresh 빌드 후487 WASM에서 각50/50쪽을 새로
  산출했다. Native최저94.90661%,WASM94.96082%,미달/누락/쪽수차이0이다.192DPI/compat2022,
  현행 PDFium smooth_rgb_v1/2px raw90%를 유지했다. Docker 최적화 결과가 아니다.
- 같은 조건의 직전469/471 대비 **38쪽만** Native97.13861→98.55731%,
  WASM97.11528→98.52943%로 개선되고 나머지49쪽씩은 점수가 같다. Native와WASM38쪽
  review·overlay에서 완료 꼬리 ‘재정조달’이 실제 두 일반 행의 가운데에 놓이고,
  표 경계·후속 내용이 유지되는 것을 판독했다.37쪽 prefix 잔여를 글꼴 문제로 단정하지 않는다.
- 원래508쪽 대조의512개 논리/render-tree JSON은465와 모두 바이트 동일하다.
  이는 Visual Sweep 수용이나 전체 PR 통과가 아니라 소유/배치 변화 범위의 증거다.
- 기존1937 단일 검사 안에 독립50쪽·35쪽 표 시작·표/각주 비겹침·602셀 원문 내용 보존·
  31번 각주의42→43쪽 이어받기와40번 각주 소유·완료 병합 꼬리 정렬을 보강했다.
  자동 글머리표만 원문 문자열과 구분하며 원문 기호는 보존한다. 함수 수를 늘리지 않는다.
- 같은 핵심 경계의9쪽/13쪽은 Native99.88474/99.99969%,WASM99.88906/99.99970%다.
  같은 PDF의 괘선 좌표를96DPI CSS±2px로 보호하며 문서 전체50쪽 선행90%를 확인했다.
  13쪽은 같은 이어받기 경계 중 더 높은99%대를 우선 선정했고9쪽은 별도 첫 종료 경계다.
  대표 review/overlay의 표 시작·마지막 행·줄 소속과 쪽번호를 판독했다.
- ignored490에서 최초 줄 선택 helper가 실제 트리 깊이를 잘못 가정한 실패는 별도로
  보존했다. 공통 재귀 선택으로 고친 동일 보강 검사는 frozen467에서 실제FAIL(완료 꼬리
  정렬),482에서PASS다. 정식 자동 suite의prepare와 원PR 대조군nextest는492로 실행한다.
- 원본/PDF/TSV·생산 hash·대표 PNG는
  [완료 병합 꼬리 증적](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/completed-span-frame/README.md)에 보존한다.
  임시 로그·미채택 시도·진단·원본 glyph 공급은 ignored output에만 남겼다.

## 원 PR의 계약과 범위 재확인

사용자 지시에 따라 issue1937의 전체 논의→[원PR1976](https://github.com/edwardkim/rhwp/pull/1976)
본문/실제diff/검사→[통합1998](https://github.com/edwardkim/rhwp/pull/1998)의
`f1bf93763`/merge`2b67c1938d1d75431f70a3f727b77bb00311fc92`를 다시 읽었다.
초기의vpos reset/행높이3배 가설은 철회됐고, 실제 원인은 시작 쪽의 전각주 예약을 모든
continuation에 재사용한 것이다. 원PR의수정은 신선 쪽 예산을 사용하고 zone/tolerance를
유지하는1줄이다. 폭주231→52쪽을45..80범위로 보호했으며 독립50쪽의 전쪽 일치까지
입증한 검사가 아니었다. 이 폭주 방지 계약은 유지하면서 현재 원본·독립PDF로 검증한
실제50쪽과 내용/소유를 보강한다. 원PR이 나열한1073/1070/1417/1050/1658 정상 대조도
현재 정식 Native 게이트에서 실행한다. 공급망·거제시 등 별개 원인을 새 개선 범위로 넣지 않는다.

남은3930은 [원PR4058](https://github.com/edwardkim/rhwp/pull/4058)/merge
`efed25b43a972f79e948f8c28abf863eb18aa1d8`의 저장 전후 표·바탕쪽 보존 계약을 따른다.
시험지 두 실패는 [8064](https://github.com/edwardkim/rhwp/pull/8064)/merge
`9b01d5d1447220b3a7345703e79ad42cf3f502f4`의 원문 face·수식/문단/쪽 소유 계약과 대조한다.
1133이라는 fixture 번호의 중첩 표 원 이슈를 쪽번호 원인으로 혼동하지 않는다. 쪽번호 검사는
로컬`c34a6cb968`와 후속`e2ebf76b8a`,원본HFT 번호 보존은7902/7941의 실제반영을 추적했다.
원 PR의과거통과를 현재통과로쓰거나 새 회귀를 기대값 완화로 숨기지 않는다.

현재 기존 실패11함수를 frozen482로 재실행하면 모두FAIL이다(484). 고정devel4a07의
API호환10함수는8PASS2FAIL이다(489). 기존HWP3/1105 두 실패와,base정상인쪽번호5·
저장3930·시험지2를 구분했다. Native bold프로그램 검사는base에없는새API로 컴파일되지
않아 base통과/실패 판정을 하지 않았다. 이 비교는 현재 전체nextest나전쪽시각검증을
대신하지 않는다. 남은 필수 계약·전체 회귀·범위 게이트와 실제 소비 검증을 마친 뒤에만
PR 준비 완료로 판정하고,PR은 사용자 승인을 받아 진행한다.


### 정식 Native 게이트와 실제 초기화 완료

manifest `--prepare` 후 현재 derive와 실제suite004의기존1937 단일 함수를 실행했다.
원PR1976의1073/1070/1417/1050/1658 정상 대조와 함께 **18PASS/0FAIL/1246선택제외**,
실행0.425초,exit0이다. 빌드 시간은 별도492결과에 기록했다. 전체nextest결과가 아니다.
493의 Studio7736 실제 초기화 GET200/캐시미사용·WASM ready와 rootpkg/public/frozen hash를
확인했고 JS의Vite source-map rewrite도정확비교했다. 이는초기화확인으로actual문서paint36의
수용을대신하지않는다. 코드4파일은Native482와fresh486 빌드의원장과동일하며 이후수정은
기존검사1파일·증적뿐이다. 지침/remote변경은없고,이단계완료를로컬커밋으로보존한다.
