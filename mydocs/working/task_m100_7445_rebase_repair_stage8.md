# 원본 HWP3 sample16 전쪽 보정

Issue: #7445 작업 중 추가 확인한 원본 HWP3 범위. 새 이슈로 이관하지 않는다.

## 사용자 결정과 보존 범위

2026-10-17 사용자는 새 이슈 등록·픽셀 핀 이관 대신 현재 브랜치에서 원본 전쪽 개선을
선택했다. `samples/hwp3-sample16.hwp`, 독립 Print PDF, 기존 검사와 정상 구조·글꼴 검사를
보존한다. 기존 픽셀 기대값을 현재 미달 출력에 맞추거나 검사를 제거하지 않았다.

입력 SHA-256: `559fd94860cf836d5800436d56055c23a73e59eedef3da273a5ba365e6ed9e16`.
독립 기준은 한컴2020 11.0.0.9136 Print64쪽
`pdf/hwp3-sample16-current-print-20261017-2020.pdf`이며 SHA-256은
`8f0bfd27ae567481e244f39de6f7b596b3afa8b227dce895d4ebf9cebc416bd0`이다.
변환 HWP5의 한컴2024 수동 PDF와 다른 입력·기준이다.

## 수정 전 전쪽 확인

고정 devel `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`과 이전 후보
`6c8ee5adeca31bf15eace47d1e10a51eea10f4a2`에서 같은 글꼴·192DPI·compat2022·
실제 PNG·색상 경계 보정 전2px 실루엣을 비교했다. 양쪽64쪽은 일치하지만 후보는
Native44쪽, WASM44쪽이90% 미만이다. 후보 최저13.84189%/14.60170%이며
21–23쪽의 devel 대비 하락도 보존한다. 변환 HWP5 완료 결과로 원본을 수용하지 않는다.

## 세로 간격의 원인과 후보

동일 원본의 한컴2020 재저장 HWPX를 진단용으로 확보했다. 이것으로 입력이나 기준 PDF를
대체하지 않는다. 원본 속성에서 다음 세 항목을 계산하면3쪽 본문19개 문단 시작점이
독립 재저장본과 모두 일치한다. 아직 실제 수정본 렌더링 통과를 뜻하지 않는다.

1. 쪽 첫 문단의 앞 간격852HU를 `!starts_new_page` 가드가 버렸다. 일반 HWP3 본문의
   쪽 시작에도 원본 앞 간격을 보존하는 후보이며, 암호·중첩 문단의 별도 계약은 유지한다.
2. 텍스트가 없는 단일 TAC 시각 개체 호스트의 뒤 간격을0으로 지웠다. 대표 글자1300HU와
   문단160%가 소유하는780HU를 개체 높이와 별도로 보존한다. 고정 줄간격·암호 경로는
   이 후보의 백분율 규칙에 포함하지 않는다.
3. HWP3 줄간격은1/1800인치 단위에서 반올림한 뒤 공통 IR의1/7200인치로 변환해야 한다.
   13pt/155%는715가 아닌716HU,3pt/155%는165가 아닌164HU다. 잘못된 버림이 줄마다
   누적됐다. 기존 손상 입력 포화 계산은 i64에서 유지한다.

소비 경로는 `src/parser/hwp3/mod.rs`의 문단 속성·줄간격 정규화 → 공통 LineSeg의
점유 높이·세로 원점 → 공통 조판·PaintOp → Native/WASM이다. 렌더러에 HWP3 분기를
추가하거나 문서 경로·페이지 번호로 좌표를 맞추지 않는다.

## 상자 모양의 원인과 후보

그리기 파서는 원본 선 종류2–7을 일괄 실선으로 바꾸고 있었다. 같은 원본의 독립 Print와
재저장 HWPX는 해당 상자를 DASH로 출력한다. 가시 dash 종류를 보존하고 기존 무색 sentinel과
선 종류0·양수 폭의 별도 처리는 유지하는 후보를 작성했다.
V3 표69 options의 둥근 모서리 bit0·반원 모서리 bit2도 공통 사각형 곡률10/50으로
변환한다. 실제 후보 출력·직접 판독과 반례 검증은 아직 필요하다.

기존 `hwp3_sample16_business_box_border_solid`는 실선 강제를 기대한다. 기대값 변경은
원본 전쪽 Native/fresh WASM90% 이상과 직접 판독 뒤 판정하며, 지금 검사를 제거하지 않는다.

## 실행 상태와 증적

ignored 경로:
`output/pr-review/7445-completed-20261009/rebase-9415b920-20261017/`.

- `hwp3-original-print/`: 전쪽 base/head TSV·PNG, 사용자 결정, 원본 Print 출처.
- `hwp3-original-print/diagnosis/vertical-source-cause.json`:19문단의 원본 계산과 독립 값.
- 같은 진단 폴더의 `shape-source-cause.json`: 사각형 선 종류·모서리의 독립 대조.
- `hwp3-original-repair-c89/`: 후보 source hash·patch·Native 빌드 로그. 아직 수용 결과 없음.

이전 헤드의 전체 nextest10664PASS/45skip은 이전 코드의 결과다. 원본 전쪽 보정을 위해
48조건 캡처의 종료를 요청했고 완료한15조건과 부분 묶음은 보존했다. 실행 중인 이전
release build는 끝까지 두고, source guard가 다음 게이트 시작을 막도록 했다.
후보에 이전 전체 통과를 재인용하지 않으며 후보 집중 검증·전쪽 Native/fresh WASM·
영향 대조군·필수 게이트를 새 source/head로 수행한다. 현재 PR 준비 완료가 아니다.

## C89 측정과 C90 보정 이력

C89는 Native 빌드161초·Mac fresh WASM web/no-opt 빌드246초를 마친 뒤 각각64쪽을
측정했다. Native 최저36.85977%, WASM37.44407%이며 둘 다16쪽이90% 미만이다.
미달 쪽은1·3·5·6·18·19·21·22·23·24·30·33·39·46·50·57이다.
처음44쪽 미달에서 개선됐지만 완료·회귀 추가/기대값 변경의 선행 조건을 충족하지 않는다.
Native3쪽 review·overlay를 직접 열어 본문 시작점 개선과 남은 차이를 확인했다.

이어 다음 원인을 source 속성·독립 자료로 확인해 C90 후보를 작성했다.

- 수식2개의 스크립트는 남아 있지만 IR 글자 크기는0이고 글자 취급·여백·배치 정보가
  누락됐다. 압축 해제한 원본의 검증된 수식 헤더에서 기준 위치0·자리차지0·글자 크기
  300(×4=1200HU)·기준선465(1/1800인치)를 직접 읽었다. 이미 해석한 공통 개체 정보를
  수식에 전달하고, 스크립트 셀 글자모양으로 크기를 정하며 기준선 길이를 백분율로 변환한다.
- V3 그라데이션에는 전이 위치 필드가 없다.0으로 보존하면 공통 renderer에서는 실제
  0% 비대칭 전이로 해석된다. 독립 재저장본의 기본50%로 정규화하는 후보를 적용했다.
- TAC 표 호스트의 위치0 인라인 글자모양이 대표 모양을 덮어쓴 경우가 있다.33쪽 제목
  표는 실제1400HU·130%의420HU가 필요하지만 이전 대표1000HU의300HU를 사용했다.
  덮어쓴 원본 글자모양을 fallback 계산에도 사용한다. 중첩 셀의 최대 글꼴로 추정하지 않는다.
-33쪽 앞 빈 문단535는 저장 vpos0·점유1300HU지만 실제 조판에서는32쪽에 들어갔다.
  다음 제목 표536의 저장 vpos1300이33쪽에서0으로 재시작해 빈 줄 하나가 사라졌다.
  parser가 인증한 빈 문단 쪽 경계의 물리 소속을 유지하되, 저장 시 사용자 쪽나눔을
  합성하지 않도록 기존 `page_break_synthesized` 출처를 함께 남긴다.

C90은 Native 빌드와1·3·6·19·22·33·63쪽 집중 비교를 연결했다. 현재 이 후보의
전쪽 Native/fresh WASM·정상 대조군·기존 회귀 결과는 아직 없으며 기대값도 바꾸지 않았다.

## C91–C94 전쪽 개선과 기존 기대값 재검토

위 C90 문단은 당시의 상태다. 이후 단일 inline 개체의 종료 간격을 다음 원본 문단의
같은 물리 프레임에서 읽고, 다음 문단 앞 간격·명시적 쪽/단 경계·vpos 되감기를 구분했다.
빈 개체 호스트의 후행 간격을 무조건0 또는 백분율로 추정하지 않는다. 가시 본문과 장식이
섞인 줄에는 일반 백분율 간격을 쓰며, 수식 줄은 실제 수식 기준선·호스트 글자 간격을 쓴다.
장식 제목의 번호 구분 공백을 복원할 때 책갈피·번호 컨트롤을 버리지 않고 공통 갭 슬롯 모델로
UTF-16 위치와 글자모양 런을 다시 구성했다. 숫자 계층·소수점·이미 있는 공백은 바꾸지 않는다.

C94 source는 parent `6c8ee5adeca31bf15eace47d1e10a51eea10f4a2`와 parser 두 파일의
수정이며, 정확한 파일 해시는 [증적 README](../pr/assets/issue7445/completed-20261009/hwp3-original-20261017/README.md)에 있다.
동일 원본·독립 Print64쪽을 Native/fresh WASM 각각 다시 산출했다.
Native 최저91.52306%, WASM91.77199%, 미달·누락0쪽, 쪽수64/64다.
고정 `9415b920`의 같은 backend별64쪽과 Decimal 비교에서도0.01%p 이상 하락0쪽이다.
3쪽은99.49622/99.47834%,33쪽은99.25546/98.53604%로 직접 review·overlay를 확인했다.
수식이 있는6쪽은 두 식과 뒤 본문이 복원됐지만 분수·괄호·기준선의 미세 차이가 남아 있다.
6쪽과 원 기호 모양 차이가 남은63쪽을 완전 일치로 보고하거나98% px 핀 대상으로 쓰지 않는다.

이 전쪽 선행 확인 뒤 실선 강제·번호 구분 공백 누락·HWP3 수식 부동 소속 등 잘못된 기존
기대값을 교정했다. 기존 본문·스크립트 수량·읽기 순서·64쪽 왕복 검사는 유지한다.
3쪽의 세 제목 기준선과33쪽 본문 표 시작은 독립 PDF 좌표를96dpi로 환산한 ±2CSSpx와
문단·쪽 소속·순서·비겹침을 함께 보호한다.33쪽 빈 시작 줄·제목 표·본문 표의 공통 소속,
수식 원본 크기·기준선의 새 관계 검사를 집중 실행 중이며 보정 전후 실제 결과를 연결한다.

ignored `hwp3-original-repair-c94/paired-qualification.json`은 전쪽 두 TSV와 base 비교,
`regression-state.json` 및 `regression-*.log`는 집중 검사 실행을 기록한다.
nextest의 `--profile release-test`를 쓴 초기 실행은 잘못된 도구 옵션으로 테스트가 실행되지
않았으므로 `invalid-nextest-profile/`에 분리했다. 수정한 `--cargo-profile release-test` 실행만
실제 회귀 증거로 사용한다. 이전 전체10664PASS를 이 후보의 전체 통과로 재인용하지 않는다.
전체 검증·다른 문서 잔여 보정이 남아 있으므로 PR 준비 완료가 아니다.

## 사용자 요청에 따른04f8c2bd 리베이스

위 C94 수치는9415b920 기준의 이력이다.2026-10-17 사용자의 추가 리베이스 요청으로
upstream을 fetch해 `04f8c2bd1617f0dbec6f10543913391ff088c4a7`(#7969)로252개 커밋을
리베이스했다. 현재 rebased HEAD는 `53862d313417f6702d66a9e3e592000b0a92d907`이다.
Native 글꼴 catalog·shader·다중선·캐시와 기존 보정을 함께 유지하며 충돌을 해소했다.
실제 profile과 최종 paint 선폭은 새 공통 line replay에 전달한다. CanvasKit은 glyph bitmap의
명시적 filtering과 일반 그림의 Print profile을 구분하고, 실제 선택·전처리된 이미지에서
축소 입력을 만든다. upstream의 자원 키·색조·이진화·cache hint를 우회하지 않는다.

사용자 초안·보정·PDF64파일을 백업 ref와 stash
`a688ec39f3580a5e745735f1d9b2a2d1d86466ea`로 보존한 뒤 모두 원래 해시로 복원했다.
stash와 이전 backup ref는 유지한다. ignored `rebase-04f8c2bd-20261017/`의
`restore-verification.json`·`conflicts.json`·`history-repair-result.json`에 상세 근거가 있다.
결합 스크립트의 중간 오류로 임시 리베이스 커밋에 들어갔던 충돌 표식은 해당 커밋에서
수정했고, 최종 tree가 같으며252개 재작성 커밋에 표식이 없음을 확인했다.

리베이스 직전 새 원본 검사2건은 실제로2FAIL했다(빈 시작 줄이 앞 쪽에 남음·수식 글자 취급
누락). 실행 중 Cargo는 완료까지 두고, 보정본을 복원한 뒤 리베이스했다. 이는 이전 기반의
수정 전 증거이며 새 기반의 수정 후 통과를 의미하지 않는다.

새 기반 첫 Native 검사에서 cache 밖의 옛 지역 변수2개 참조가 컴파일 실패했다.
정규화·전처리된 immutable Skia Data를 decoded cache의 image와 함께 공유하여 hit/miss 모두
같은 입력을 소비하도록 수정했고, encoded 바이트는 기존 cache 비용 계산에 포함한다.
SVG는 원본 SVG 출처로 재필터링에서 제외한다. 수정 후 Native Skia `cargo check --lib`가
26.63초에 통과했다. Studio는2321PASS/0FAIL/2skip(315파일), TypeScript도 통과했다.
실패한 첫 컴파일은 `compile-attempt-1/`에 보존한다.

C95에서 Native 집중 검사 → HWP3 원본·왕복·추출·책갈피 검사 → Native 빌드 → fresh WASM
→ 전체 nextest를 Cargo 순차·jobs8·threads8로 연결했다. 캡처 watcher는 producer 준비 뒤
Native/fresh WASM 전64쪽을 총8 workers·공유 lock으로 순차 실행하며 실패 시 중복 시작하지 않는다.
아래 결과는 새 기반에서 다시 실행한 C95 증거이며, C94 이력과 구분한다.

## C95 재검증과 남은 보정

Native Skia 집중49건, 새 원본 소속2건, 기존 제목13건·그라데이션5건·왕복2건·수식 읽기2건·
책갈피2건·기존 파서2건으로 총77건이 통과했다. 첫 경로 소유 검사2건은 이전 parser에서
실제2FAIL한 증거와 연결된다. Native 빌드119.27초와 Mac fresh WASM web/no-opt228.93초도
완료했다. root pkg와 Studio public의 두 산출물은 frozen package와 같은 해시다.

같은 원본·독립 PDF·글꼴·192DPI·compat2022·Print로64쪽을 각각 새로 비교했다.
Native 최저91.52306%, fresh WASM91.77199%, 양쪽90% 미만0쪽이며 쪽수64/64가 일치한다.
새 base04f8 전쪽 비교는 별도 대기 상태이므로 C94의9415 대비 하락0 결과를 재인용하지 않는다.
`rebase-04f8c2bd-20261017/hwp3-original-repair-c95/`의 TSV·producer·PNG가 정본이다.
3·33쪽 WASM review/overlay,18·35쪽 Native review/overlay를 직접 열어 제목·본문·표 소속과
그림을 확인했다. 수식6쪽·원 기호63쪽·그림 및 일부 글꼴의 미세 차이는 해결로 보고하지 않는다.

전64쪽 Native/WASM 논리 렌더 노드도 일치한다. 생성된 내용의 `paraIdx=usize::MAX`만
64bit와32bit의 같은 소유자 없음으로 정규화했고, 좌표·글자·소유자 차이를 숨기지 않았다.
독립 PDF의 영숫자·한글 수량과 실제 TextRun을 쪽별로 대조하면61쪽이 그대로 일치한다.
6쪽의 부족64자는 실제 수식 SVG 두 개의 CJK와 HyhwpEQ 숫자 glyph를 풀면 정확히 일치한다.
18·35쪽은 PDF의 WMF 내부 문자가 Native 그림 노드로 그려지므로 TextRun 수량 검사의 대상이
아니며 별도 그림 판독을 남긴다. 이 수량 보조 검사는 순서·모양의 전체 검증을 대신하지 않는다.

전체 nextest는 컴파일9분41초 후548.039초에10,849건을 완료했다.10,845PASS·4FAIL·45skip이다.
네 실패는 sample11의 원본 쪽수151→152·ghost page·oracle 쪽수와 text-overlap4→8이다. 기대값이나 baseline을
바꾸지 않는다. historical frozen6c8=151, C94=152, C95=152로 비교해 재베이스 충돌이 아니라
C94 parser 보정에서 생긴 차이임을 확인했다. 첫 내용 소속 차이는50쪽이며 문단1819의
`PPP 사용자 ID`가 다음 새 쪽에 단독으로 밀렸다. 독립 기존 정본
`pdf/hwp3-sample11-hwp-2020.pdf`는151쪽이고 해당 문단은50쪽에 있다.

직접 원인은 mixed inline 도형의 줄 상자4412HU에160%의 후행 간격을 곱해2648HU를 만든
것이다. 같은 줄 실제 글자1000HU가 소유하는600HU와 구분해야 한다. 도형 높이는 줄 상자·
기준선에서 보존하고, 문단 비율의 뒤 간격은 해당 줄의 글자모양에서 계산하는 보정을 준비한다.
`sample11-*-boundaries.json`, 진단 HWPX, `sample11-independent-p50-lines.json`에 비교를
보존한다. 전체 검사를 중단하거나 네 실패를 전체 통과로 바꾸지 않으며 새 보정 후 재검증한다.

## C96 인라인 도형의 후행 글줄 간격 보정

큰 인라인 도형이 포함된 줄의 물리 높이는 줄 상자·기준선에 남기고, 후행 percent 간격은
그 줄의 실제 글자모양 크기에서 계산하도록 보정했다. UTF16 줄 범위에 겹치는 글자모양을
읽고 개체 자리표시자는 글꼴 높이의 증거에서 제외한다. 다른 줄의 큰 글꼴을 재사용하지 않는다.
명시적 고정 간격과 기존 암호 경로는 유지한다. 화면 좌표·문서명·페이지 번호에 의존하지 않는다.

C95에서 실패한 네 검사 모두 C96에서 실제 PASS했다. 원본151쪽과 HWPX 왕복151쪽,
ghost page·oracle 쪽수·기존 text-overlap 래칫을 기대값/TSV 변경 없이 유지했다.
sample16의 새 소속·기존 제목·그라데이션·왕복·수식/책갈피 읽기·기존 파서까지 집중33건이
모두 통과했다. Native 빌드138.80초도 완료했다. fresh WASM과 Native/fresh WASM 각각
sample16 원본64쪽·sample11 원본151쪽의 전쪽 비교를 이어간다. C96 전체 회귀 통과나
새 base 대비 하락0·PR 준비 완료는 아직 주장하지 않는다.

ignored 정본은 `rebase-04f8c2bd-20261017/hwp3-line-font-gap-c96/`이며 `validation-results.json`,
`focused-*.log`, `candidate-source.json`, `candidate.patch`, producer·전쪽 TSV·review/overlay를 연결한다.
원본 sample11과 기존 독립 PDF는 fixed04f8와 현재 작업본의 Git blob이 각각 동일함을
`sample11-reference-intake.json`에 기록했다. 이전 base 대조 대기는 C95 실패 때문에 안전하게
종료됐고, 새 base producer는 C96 집중 보정·fresh producer 완료 뒤 Cargo 순차로 실행한다.

C96 fresh WASM도249.37초에 완료했다.16쪽씩 동일 묶음으로 새로 산출한 결과는 다음과 같다.
Native 단일 전쪽 호출과 묶음 호출의6쪽 raw가0.00128%p 달랐으므로 새 base도 같은16쪽 묶음을
사용해 렌더 상태 차이를 통제한다. 어느 값을 올려 쓰거나 조합하지 않는다.

| 원본 | Native 최저 / 미달 | fresh WASM 최저 / 미달 | 쪽수 |
| --- | ---: | ---: | ---: |
| sample16 |91.52178% /0|91.77075% /0|64/64|
| sample11 |49.73493% /146|49.72061% /145|151/151|

sample11은 쪽수·회귀 복구와 별개로 시각 미달이며 완료로 수용하지 않는다. 새 base 비교도
완료 전이다. 원본·PDF를 바꾸거나151쪽 검사를 제거하지 않았다.

50쪽 그림의 구체적인 소비 누락도 확인했다. HWP3 drawing 파서는 자식의 `offset_x/y`를
보존하지만 공통 `render_tx/ty`는0이다. 공통 group paint는 합성된 `render_tx/ty`를 읽으므로
여러 nested group bbox가 같은 원점에 겹친다. 고정04f8의 같은 paint 경로도 동일하다.
진단용 HWPX에서 기존 상대 offset만 부모 좌표로 합성하면50쪽 raw는68.77344%→76.11827%로
개선됐다. Native 원본50쪽68.34021%와 진단 HWPX 대조군을 구분하며, 이 진단 입력을
원본·독립 PDF 대신 사용하거나 production 보정 완료로 보고하지 않는다.

같은 독립 재저장 HWPX와 원본의1803 문단은 줄 높이4412·textheight4412·baseline3750·
spacing600HU가 일치한다. 따라서 IR의 textheight를 임의로1000으로 바꾸면 안 된다.
`paragraph_layout.rs`의 큰 TAC Shape 줄을 font 높이·baseline으로 접는 본문 경로가
가시 글자와 개체가 함께 있는 줄에도 적용된다. 원본 글자 `pc1`의 독립 PDF 기준선490.2pt와
실제 paint를 대조해, 개체 전용 줄의 기존 계약과 가시 글자의 저장 기준선 소속을 구분할 필요가 있다.
현재는 원인 진단이며 해당 renderer 보정을 아직 적용하지 않았다.

base archive의 첫 컴파일은 누락한 `saved/blank2010.hwp`, 두 번째는 CLI의11개 embedded
문서 입력 때문에 실패했다. base 코드 오류가 아니라 소스 사본 준비 오류다. 모든 archived Rust의
literal `include_str!/include_bytes!` 의존 경로를 점검하고 같은04f8의 tracked blob만 추가했다.
원본 source를 수정하지 않았고 archive inventory·실패 로그를 보존한 세 번째 빌드를 진행한다.
정본은 `base-literal-include-dependency-audit.json`, `fixed-base-build/attempt-*/`이다.

의존 입력을 보충한 세 번째 base Native 빌드는16.55초에 성공했고 같은 base의 fresh WASM을
실행 중이다. base Rust의 schema V2 export가 tracked public glue에는 없어 wrapper가 public
산출물을 갱신할 수 있다. 성공한 빌드의 고정 source 검사에서는 선언된 두 public 빌드 산출물과
불변 입력을 구분하며, 나머지 모든 base 파일의 SHA와 실제 pkg/public 동일 해시·후보 bundle
보존을 확인한 뒤에만 producer를 확정한다. 실행 중 Cargo는 중단하거나 중복 시작하지 않는다.
`fixed-base-build/output-guard-state.json`과 `base-build-output-guard.log`에 이 단계를 기록하고,
완료 후 동일16쪽 묶음의 base/candidate 대조를 실행한다. 이 준비 상태를 base 시각 통과로 쓰지 않는다.


## C96 동일 base 전쪽 대조 완료

고정04f8 Native와 Mac fresh WASM은 실제 빌드 명령이 모두 성공했다. wrapper의 선언된
public JS/WASM 산출물과 불변 source를 분리해 원본 SHA를 확인했다. 처음 실패한
출력 guard는 성공한 명령을 실패 코드로 바꾸어 해석한 것이며, 원본 로그·불변 입력·생산물
SHA·후보 pkg/public 보존을 다시 검증한 뒤 producer를 확정했다. source 코드나 허용치를
수정한 base가 아니다. `fixed-base-build/build-output-guard-repair.json`이 정본이다.

같은16쪽 묶음·글꼴·독립 PDF·canonical raw2px로 모두 다시 산출했다.

| 원본·경로 | fixed04f8 최저 | C96 최저 |0.01%p 이상 하락 쪽 |
| --- | ---: | ---: | --- |
| sample16 Native |16.40830%|91.52178%|없음 |
| sample16 WASM |15.26304%|91.77075%|없음 |
| sample11 Native |20.01380%|49.73493%|55·56·59·117 |
| sample11 WASM |19.62747%|49.72061%|31·132·140 |

sample16은64쪽씩 누락 없이90% 이상이고 새 base 대비 하락도 없다. 수식·기호 차이의
확대 판독은 별도 미완료다. sample11은151쪽 일치·회귀 복구와 별개로 미달·하락이 남아
PR 완료 근거로 쓰지 않는다. `hwp3-fixed-base-batched-c96/paired-qualification.json`과
양쪽 원본 TSV에 Decimal 대조를 보존했다. 어느 하락 쪽도 평균 상승으로 상쇄하지 않는다.

## C97 가시 글자와 inline 도형의 저장 기준선

개체 전용 guide 줄의 font 높이 접힘은 문단의 shape 높이 바닥값과 짝이다. 가시 글자나
공백 run이 있는 줄까지 같은 접힘을 적용하면 그 run이 소유하는 저장 baseline을 잃는다.
공통 `paragraph_layout.rs`에서 접힘을 실제 `comp_line.runs.is_empty()`인 개체 전용
줄에만 적용하는 후보를 작성했다. 새 포맷 분기·문서명·페이지별 좌표 보정은 없다.

C97 Native 빌드는112초에 완료했고 원본 sample11의50쪽 raw는68.34021%에서70.55984%로
개선됐다.151쪽도 유지했다. sample16의6쪽91.52178%·33쪽99.25546%는 C96과 같고64쪽을
유지한다.50쪽 review·overlay를 직접 열어 `pc1`의 기준선 이동과 남은 그룹 도형 위치·외곽
차이를 확인했다.4쪽 focus만으로 전쪽·WASM·정상 대조군 수용을 주장하지 않는다.
첫 capture driver의 상위 경로 오류는 입력 처리 전에 실패했으며 `focus-invalid-path.log`에
보존하고 경로를 수정한 실제 실행만 증거로 쓴다. ignored `inline-shape-baseline-c97/`의
source·producer·focus TSV·review/overlay가 정본이다.

## C98 수식 연산자 의미와 글리프 투영의 간격 손실 진단

원본6쪽에서 독립 PDF의12pt 등호는 앞 간격1.68pt·자체 논리 칸12pt·뒤 간격2.04pt다.
이는 기존 legacy 연산자 규칙0.14/1/0.17 em과 일치한다. 새 HYhwpEQ glyph 투영 후에는
LayoutKind가 Symbol이 아닌 HancomGlyphs가 되어 `legacy_operator_gap`의 연산자 판정에서
빠지고 현대 연산자의0.15em 패딩이 적용된다. 새 계수를 문서에 맞추는 것이 아니라 AST의
연산자 소속과 기존 논리 칸·간격을 glyph 선택 후에도 유지해야 한다.
현재 합성 단위 검사로 glyph 투영 전후의 논리 전진 계약을 재현 중이다. 원본6쪽이98% 미만이므로
실물 좌표 핀은 추가하지 않는다. `%` face·분수 기준선·괄호 잔여도 별도 확인하며 이 원인 하나로
수식 완료를 선언하지 않는다. ignored `legacy-equation-operator-c98/cause.json`과 검사 로그를
기존 수식 source·독립 PDF 글자 좌표에 연결한다.


## C98 실제 결과와 C99 글리프·투명 래퍼 보정

C98 합성 검사는 수정 전 실제1FAIL(52.32 vs56.48)했고 수정 후 수식 모듈149건이 모두
PASS했다. Native 빌드134.60초 뒤 같은 원본6쪽 raw는91.52178%→93.66436%로 개선됐다.
33쪽99.25546%와64쪽 쪽수는 유지했다. canonical review/overlay를 확대해 첫 수식의
가로 전진 개선과 남은 `%`·분수 기준선·괄호 차이를 확인했다. 전쪽·WASM·최종 전체 회귀
수용 결과가 아니다. `legacy-equation-operator-c98/`의 source·before/after·producer·focus가 정본이다.

수식 `%`는 일반 이탤릭 Text가 아니라 HYhwpEQ의 수학 PUA E042가 소유한다. 설치한 공통
글꼴의 cmap·실제 glyph를 열어 확인했고, 독립 Print의 Type3 ToUnicode와1000/1000 HFT 논리
칸을 대조했다. TTF hmtx853/1024를 HFT 논리 전진으로 바꾸어 해석하지 않는다. 기존 legacy
분기의 문자 선택만 보정하며 명시적 현대 수식 버전은 유지한다. 숫자와 같은 수학 기준선을
소비하고 기존 Unicode 삼각형·중점의 Roman 기준선 처리는 보존한다.

C98에서 AST 연산자 소속으로 옮길 때 투명 Color 래퍼의 소속도 유지해야 한다. 색상 래퍼는
논리 전진을 바꾸지 않으므로 해당 경계와 `%` glyph 선택을 합성 검사로 보강했다. 이전04f8
기반에서 두 검사가 실제2FAIL했고, 이제 보정 후 검증을 새515bd72 기반에서 실행한다.
실물6쪽이98% 미만이므로 새 px 핀은 추가하지 않는다.

## 사용자 요청에 따른515bd72 리베이스와 Ubuntu 지침 반영

사용자 요청으로 upstream을 fetch해 새 #7970의 `515bd72dcde0eecdeece333666b675354b7fa91d`로
252개 커밋을 리베이스했다. 충돌 없이 완료됐고 순서·제목·range-diff의 패치252개가 모두
유지됐다. 리베이스 직후 HEAD는 `59d6095887843bae14f1e5311c0ca3db35b2cbd2`다.

사용자가 지정한 ubuntu-ted의 지침 커밋 `a21cb2ecb941b3a00ab0df4bbff3f9682b8a21db`를
해당 호스트의 clean 작업장에서 확인하고 exact commit을 -x 체리픽했다. 적용 HEAD는
`986d4cbd09e79c4510796826a3953dcf4d7b799c`이며8개 문서의 patch-id가 원본과 동일하다.
원격에 push하거나 PR을 생성한 상태는 아니다. 최초 SHA 직접 fetch는 unadvertised object로
거절돼 해당 source branch를 fetch한 뒤 지정 commit만 적용했다. 별도 지침을 임의로 추가하지 않았다.

작업·PDF·사용자 초안71파일은 backup ref·tar·stash
`c59fc3730e6275f4786177225eda26d86316b694`로 보존했다.70파일을 원래 SHA로 복원했고,
public JS 한 파일은 새 upstream의16줄과 기존 생성 glue 변경을 자동 병합했다. 기존 소유
patch-id는 같으며 사용자 초안 두 파일도 원래 SHA를 유지한다. backup/stash는 삭제하지 않는다.
정본은 ignored `rebase-request-20261017-c99/`의 rebase·range-diff·정책 intake·복원 확인이다.

새 지침은 충돌이 없는 자동 적용에도 반영 PR의 실제 기능 보존을 확인하도록 요구한다.
#7970의 PDF 요청 정책·자원 예산·JPEG·clip/paint/style·글꼴 PostScript identity와 실제
Native/WASM/Worker 사용 경계를 대조한다. 관련 구현·font-rule 파일은 새 base와 후보에서
현재 동일하지만, 이를 실행 검증으로 대신하지 않는다. 기존 재현·정상/거절 경계의 정식 검사,
공개 PDF의 실제 가져오기·저장/재열기·Native/fresh WASM 출력을 새 base/후보에서 연결한다.
원래 HWP 보정 전쪽과 실제 소비 경로도 유지한다. 이 기록 자체를 위한 전체 회귀 반복은 하지 않고,
최종 source가 확정된 뒤 필수 전체 검증을 한 번 수행한다.

새 C99의 manifest prepare/check(base515)·fmt는 통과했고 수식 모듈 검사는 실행 중이다.
현재 소스·base·parent가 `legacy-equation-glyph-c99/candidate-source.json`에 고정돼 있다.
상위 폴더명이 이전04f8 기반이지만 이 C99 새 후보의 실제 base는515bd72이며 과거 결과와
혼용하지 않는다. 새 base Native/fresh WASM·전체 회귀·#7970 기능 보존 검증 완료를 아직
주장하지 않고 PR 보류를 유지한다.


### C99 새 기반 실제 집중 결과와 실행 조정

515bd72 기반의 C99 수식 모듈은151PASS/0FAIL(3907skip), Native 빌드는167초에 완료했다.
원본 sample16의6쪽 raw는93.71521%,33쪽99.25546%이며64쪽을 유지한다. sample11의
50쪽70.55984%·51쪽89.04955%와151쪽도 유지했다.6쪽의 canonical review/overlay를
확대해 `%`의 수학 글리프 선택과 남은 분수·괄호·후속 본문의 미세 원점 차이를 확인했다.
4쪽 focus이며 새 전쪽·fresh WASM·전체 회귀 통과로 쓰지 않는다.

기준515의 source6830파일(784,209,753바이트)을 Git archive로 고정했고 입력 원장은
ignored `rebase-515bd72d-20261017/archive-source.json`이다. 이전 literal include 입력과
새 font-rule·Cargo config도 같은515 blob으로 포함하며 후보 source를 섞지 않는다.
기준 Native가 시작된 뒤 사용자 터미널(`/dev/ttys000`)에서 현재 작업장의 WASM 빌드가
별도로 시작돼, 사용자 작업은 그대로 두고 이번 소유 기준 builder·Cargo·rustc만 SIGSTOP으로
대기시켰다. 공유 Cargo를 순차로 유지하기 위한 프로세스 조정이며 goal pause가 아니다.
`external-user-wasm-coordination.json`과 `continue-after-user-wasm.py`가 실제 외부 프로세스
종료를 확인한 뒤 소유 작업을 이어가고 입력·bundle guard를 재검증한다. 외부 터미널의
직접 exit status를 읽지 못한 경우 그 빌드를 정식 fresh-build 통과로 보고하지 않는다.
새 기준 producer와 기능 보존 비교는 아직 완료하지 않았다.

### 515bd72 기준 producer와 PDF 실제 소비 경계

기준 Native 빌드는 exit0/413.593초, fresh WASM locked wrapper web/no-opt 로컬 대체 빌드는
exit0/307.940초에 완료했다. Docker 최적화 통과가 아니다. 소유 프로세스의 SIGSTOP은 모두
해제됐고 builder와 watcher도 정상 종료했다. Native 바이너리는
`80e8ca759055a9a85c792a4ab324f30789327bcbe2f841c3df9e0499b1ca83bc`, 기준 WASM은
`4bec2d69720bc526a72e9cd23e0ff6d0b9437f46e2344ff84c2e7f5bd0c005a5`다.
사용자 터미널이 root pkg/public을 갱신해 최초 root 보존 guard가 실패했으나, 기준 입력6830파일이
바뀌지 않고 Native 명령이 exit0인 것을 재확인하여 이미 완성된 Native를 보존했다.
Native를 중복 재빌드하지 않았고, 새 root bundle을 각각 보존하며 기준 WASM만 이어서 빌드했다.
이 운영상 guard 실패를 조용히 지우지 않고 원 상태·원장·보완 결과를 output에 남겼다.

후보의 사용자 터미널 생성 bundle은 root pkg/public의 JS와 WASM hash가 각각 일치한다.
관측한 후보 WASM은 `1fc24fe06ef16bc370ca48a8135bf6cb6ee93fc07e2dfd4e241e03478dcc569a`다.
터미널 빌드 exit status는 관측하지 못했으므로 이 사실을 정식 빌드 통과로 바꾸지 않는다.
그 bundle과 새 기준 bundle에서 저장소의 동일한 `pdf-import-wasm.test.mjs`를 실행했다.
양쪽 각각115 assertion·실제 Worker7개·실패0건이며12쪽 가져오기, 저장·재열기, 그림 소속,
실패·timeout·취소 후 복구를 확인했다. 검사 제목의 Docker는 기존 문자열이며 이번 실행은 Mac
로컬 bundle이다. 이는 직접 실행한 API/Worker 증거이며 시각·전체 회귀 통과와 구분한다.

새 기준과 C99 Native CLI에서 공개 PDF7건20쪽의 HWP/HWPX 가져오기·재열기·SVG 출력을
각각 실행했다. 쪽수, paint 순서 안의 FIRST/SECOND 소속·개수, 반복 그림33개, affine clip의
단일 그림, 스캔 그림의 모든 쪽 보존을 확인했고, 범위 초과·중복·역순 선택은 양쪽 모두 exit2와
출력 미생성이다. 두 저장 형식의 SVG는 각 producer 내부에서 동일하다. base/candidate 비교에서
paint-order의88개 text node는 fallback family 목록이 다르며 다른 XML node·속성·text는 동일했다.
이를 전체 SVG 동일 통과로 숨기지 않았다. 나머지6입력은 양쪽 SVG도 동일했다.

같은 원본 PDF 자체를 독립 Poppler 기준으로 삼아7건20쪽을 각각 Native Skia·WASM Print
Canvas에서96DPI/compat2022/2px raw로 대조했다(두 producer 합계80쪽). Native 후보 최저는
word-gap99.47183%, 나머지 Native와 모든 WASM은100%이며 쪽 누락·쪽수 불일치0건이다.
**넓은 글꼴 공급 경로의 Native word-gap은 기준100→후보99.47183으로0.52817%p 하락했다.**
원 결과의 `re_review_required`는 유지하고 준비 완료로 표시하지 않는다. 모든40쪽 후보에서
90% 이상이라는 수치만으로 이 하락과 직접 판독 요구를 면제하지 않는다.

기준 WASM에는 새 자동 글꼴 환경 선택 API가 없어 최초 캡처가 명시적으로 실패했다.
동일7PDF의 후보 public selection plan에서 선언 substitute가 모두 없고 environmentId가 null인
것을 확인했다. 두 빌드가 지원하는 public setter의 명시적 None 경로로 WASM만 재개했다.
이 입력들의 Native 기본 환경도 None이다. 프로그램 byte 공급·실제 FontFace 로드·glyph/paint
guard는 그대로이며, 선언 대체가 있는 일반 HWP의 결과로 확대하지 않는다. 실패 캡처는 별도로 보존했다.

word-gap은 다른 파일로 대체하지 않고 같은 입력·기준 PDF·frozen 바이너리에서 함초롬바탕
regular 한 파일만 별도 공급해 원인을 분리했다. 파일은 기존
`~/Library/Fonts/HANBatang.ttf` SHA`c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104`이며
시스템 글꼴을 수정하거나 저장소에 복사하지 않았다. 이 조건에서는 Native 기준·후보 각각100%,
PNG SHA도 동일했다. WASM 기준·후보도 각각100%다. canonical review/overlay를 직접 열어
세 줄의 소속·물리 단어 간격·연속 문자·음수 kerning 대조군을 판독했다. 입력 PDF의 CID/GID
39382/40558/41146/42322와970/1000 전진폭은 이 설치 프로그램과 같다. README의 원 생성
프로그램 SHA는 다르므로 같은 원 font binary라고 주장하지 않는다.
**직접 파일 공급의 유지 증거와 넓은 경로의 하락 증거를 함께 보존하며, 후자의 실제 선택 face는
아직 추적 중이다.** 비교 조건만 바꿔 원 하락을 구현 보정 완료로 처리하지 않는다.

스캔5쪽은 양쪽 모든 raw100%지만 PNG bytes가 다르다. 채널 최대 차이는 Native113~135,
Canvas154~171까지 관측돼 단순 압축 metadata 또는 미세 AA로 단정하지 않았다. 첫 쪽의
양 backend canonical review와 overlay를 열어 색 띠·검정 테두리·그림 소속을 판독했으며,
모든 쪽의 직접 경계 판독과 바뀐 bitmap 재샘플링 소비의 비교는 별도로 남아 있다.

이번 정본은 ignored `rebase-515bd72d-20261017/` 아래 `fixed-base-build/`,
`pdf-preservation-c99-observed-wasm/`, `pdf-preservation-base-wasm.log`,
`pdf-native-preservation-c99/`, `pdf-paired-visual-c99/`,
`pdf-explicit-hcr-font-probe-v3/`, `pdf-explicit-hcr-font-probe-wasm/`이다.
잘못된 단일 font 파일 인자와 Native CLI의 JSON null 환경 인자로 실패한 두 진단은
실행 준비 오류로 보존했으며 결과에 포함하지 않았다. Native는 기본 None, WASM은 public
setter의 명시적 None이라는 API 차이를 확인한 정상 실행만 위 수치에 사용했다.

이 결과는 #7970의 선택된7공개 입력 경계다. 그 PR의 모든365쪽, 원래 HWP 보정 전쪽,
C99 이후의 전체 회귀·fresh build·Studio 소비가 끝났다는 주장이 아니다. PR 보류를 유지하며
원본 HWP3 수식 프레임의 흐름 높이·기준선, 넓은 글꼴 경로의 실제 선택, 원래 검증 범위를 계속 보정한다.

### C100 저장 수식 프레임의 독립 근거

C99 이후 원본 HWP3를 내용 변경 없이 HWPX로 내보낸 대조군은 원본6쪽 Native PNG와
SHA까지 같다. 그 HWPX의 두 equation `sz@protect`만1로 바꾼 진단에서는 source width/height,
script, baseLine, lock과 다른 ZIP member를 그대로 보존했다. 현재 Native는 보호 여부에 따라
같은 수식을39.04px 또는37.44px 높이로 소비하고 뒤 문단을1.6px씩 다르게 배치한다.
진단 점수93.71521→95.53246은 입력 속성 변경의 탐색 결과이며 원본 구현 개선 수치가 아니다.
또한 protect는 EqLayout의 Latin projection에도 영향을 주므로 점수 상승 전부를 프레임 효과로
단정하지 않는다. 실제 수정에서는 원본 속성과 glyph projection을 바꾸지 않는다.

두 진단 입력을 각각 Hancom11.0.0.9136 / engine2020 / Print method0 /
`hancom2020_pdf_driver_one_up`으로 인쇄했다. control job은
`1f06c960-3f17-4bd5-b846-6fb06e2fce47`, protected job은
`f736827a-d125-4310-8470-a7f55137aeaf`다. 두 PDF는64쪽 모두 decoded page content와
텍스트·좌표가 완전히 같다. 첫 수식 본문 baseline621.60pt, 분모628.68pt 등도 같다.
기준은 `pdf/hwp3-equation-frame-{control,protected}-20261017-2020.pdf`와 ignored
`equation-frame-probe/independent-protect-pair-geometry.json`이다. 기존 원본 Print PDF를
교체하지 않았다. 이 계약은 크기 보호 UI flag만으로 저장 HFT 프레임을 무효화하지 않는다는
근거이며 모든 수식·편집 경로를 검증했다는 뜻이 아니다.

소비 사슬은 equation common width/height·baseline → `flow_metrics_hwp` → compositor의
inline control 크기·저장 줄 equation baseline 보완 → paragraph/shape/table 배치 →
EquationNode와 실제 glyph projection이다. 현재 `flow_metrics_hwp`는 HYhwpEQ 안의 한글을
대체한다는 이유만으로 저장 HFT 상자까지 AST 크기/기준선으로 바꾼다. legacy HFT의 논리
프레임과 대체 glyph paint를 분리하고, 현대 CJK intrinsic 경로는 정상 대조군으로 유지한다.
과거 CJK intrinsic 양성 대조군인 정책연구88쪽의 실제 원본 수식은 `Equation Version 60`임을
확인했다. 원본·한컴 물리 기대값 없이 그 경로를 함께 되돌리지 않는다.

편집 producer도 점검한다. `apply_equation_properties`는 실제 script/fontSize 변경 때만
크기를 갱신하지만 generic `EqLayout::new`로 계산한다. 저장 legacy 프레임을 보존하려면
편집 시에도 그 수식의 metric context에서 새 크기를 생산해야 한다. source 프레임 보존,
baseline 속성 편집, 현대 CJK 정상 대조군의 synthetic 계약3건을 먼저 수정 전 실행 중이며,
실제 편집 후 producer/paint 일관성은 별도 FAIL→PASS와 반례로 확인한다. 아직 구현·검증
완료 또는 새 전쪽 수용으로 표시하지 않는다.

### C100 구현과 집중 결과

유효한 legacy HFT 선언 프레임(width/height 양수)은 CJK paint 대체만으로 버리지 않도록
`flow_metrics_hwp`를 보정했다. 없는 프레임의 intrinsic 경로와 현대 CJK 경로는 유지한다.
문서 ID나 HWP3 전용 renderer 분기를 추가하지 않았다. 실제 script/fontSize 편집은 legacy
수식의 paint context에서 새 크기를 생산하고, 명시한 축의 크기와 크기를 바꾸지 않는 속성 편집
계약은 유지한다. 이로써 source 보존과 편집 후 새 크기 생성의 소유를 분리했다.

- 수정 전 계약3건은1PASS/2FAIL(0.021초)로 두 오류를 실제 재현했다.
- 보정 후 수식 모듈155PASS/0FAIL(0.271초), manifest prepare/check(base515)·fmt 통과.
- Native 빌드 exit0/124초, frozen binary SHA
  `64c846b36df44097820b34dc39241ec58e128a62d6e759c0f630f7522db09524`.
- **변경 없는 원본 sample16의6쪽** raw93.71521→95.56838%,64쪽 유지.
  canonical review와 overlay를 직접 열어 뒤 본문의 전진이 복구됐고 분자·괄호의 미세 차이는
  남는 것을 확인했다.95%라는 수치로 이 잔여를 완료 판정하지 않는다.
- sample16의33쪽99.25546%와215쪽 연구보고서의88쪽95.38166%는 before/after PNG SHA까지 같다.
  후자는 현대 CJK intrinsic 양성 대조군이며 전215쪽 완료 근거로 대신하지 않는다.
- 현재 manifest로 실제 suite를 도출해 수식 삽입·setter·raw 보존·원본 HWP3 소속·legacy 기호와
  #4138 원본115/분할197쪽을 포함한6묶음22건을 실행했다:22PASS/0FAIL/1408skip,3.931초.
  소스 재배정 경로를 hardcode하지 않았고0 tests 실행을 통과로 세지 않았다.

실제 public CLI에서 같은 시작 입력에 `가가가 over 나`로 편집했다. 이전 setter의 저장 폭은
4080HU(54.4CSSpx)인데 실제 재열기 paint 폭은62.7CSSpx였고, C100은4704HU(62.72CSSpx)를
저장하여 두 소비자를 일치시킨다. 이는 구현 계산값을 기대값으로 고정한 좌표 핀이 아니다.
변경된 각 저장본 자체를 독립 한컴 Print로 인쇄해 비교했다. 두 Print의 차이는6쪽에만 있고
다른63쪽은 content stream·텍스트/geometry가 동일하다. 각자의 기준과 Native를 비교한6쪽은
이전96.05192%, 보정96.76771%이며64쪽을 유지한다. 서로 다른 저장본의 PDF를 섞어 비교하지 않았다.
Print before job은 `11a4debc-3d0c-4888-90da-e1cae7888983`, after는
`83491b43-43be-4974-a64c-59e42e8983a1`이며 양쪽 Hancom11.0.0.9136/engine2020/method0/
`hancom2020_pdf_driver_one_up` 실행 증거를 확인했다.

정본은 ignored `rebase-515bd72d-20261017/legacy-equation-frame-c100/`의 source/producer,
`before.log`, `equation-module.log`, `focused-regressions.log`, `focus-results.json`,
`edited-print-pair-geometry.json`, `edited-visual-results.json`이다. 이 국소 검증 후에도 원본
전쪽 Native/fresh WASM, 다른 변경 범위와 최종 전체 회귀·lint·실제 Studio 소비는 남아 있다.
C100을 로컬 중간 커밋으로 정리하며 PR 생성·완료·전체 시각 수용은 계속 보류한다.


### C100 원본64쪽 Native·fresh WASM 전쪽 결과

같은 원본 `hwp3-sample16.hwp`(559fd948…)와 독립 한컴2020 Print64쪽(8f0bfd27…)을
고정 devel515bd72d와 C100 생산자로 각각 다시 비교했다. C100은 로컬 체크포인트4aca2f1a20과
`legacy-equation-frame-c100/source.json`에 고정한 기존 dirty HWP3·Native 소스까지 포함한다.
깨끗한 Git head만 빌드한 최종 PR 검증으로 표현하지 않는다.

| 경로 | fixed515 최저 | C100 최저 | C100 미달/쪽수 | 기준 대비0.01%p 이상 하락 |
| --- | ---: | ---: | --- | ---: |
| Native Skia Print |16.40830%|94.38750%|0 /64=64|0|
| fresh WASM Print Canvas |15.26304%|94.46126%|0 /64=64|0|

192DPI·compat2022·원값2px 기준을 유지했다. WASM은 locked wrapper의 web/no-opt Mac 로컬
대체 빌드이며 Docker 최적화 통과가 아니다. root pkg와 Studio/public JS/WASM 해시가 일치하고,
원본 HWP3에 실제 substitute 요청이 없음을 확인한 동일 None 환경을 base/후보 모두에 적용했다.
실제 Studio 초기화 응답과 CanvasKit 소비는 아직 이 결과에 포함되지 않는다.

정본은 ignored `rebase-515bd72d-20261017/legacy-equation-frame-c100/`의
`native-whole/paired-result.json`, `wasm-whole/paired-result.json`, 각각의 전64쪽 `silhouette.tsv`,
`wasm-producer.json`, `fresh-wasm-step.json`이다. Native/WASM 각6·33·40·63쪽의 canonical
review/overlay는 같은 전쪽 캡처에서 생성했다. 수식 확대 판독의 잔여 차이, sample11 미달,
최신 전체 회귀·실제 Studio 검증이 남아 있으므로 PR 준비 완료는 아니다.


### HFT 원본 공급 진단에서 빈 Type3 방지

sample11 본문의 요청 face는 원본 HFT `고딕`이다. 대상 문서/PDF의 글자를 가져오지 않고
빈 HWPX에서 별도 전체 현대 한글11172자 표본을 만들어, engine2020·한컴11.0.0.9136·
PrintMethod0·one-up으로 출력했다(job e62e43d1-c0c2-41c4-8ca7-0d5b6c36a4cd,
입력b511b069…·PDF e78a062f…). 해당 Print는 실제 ink를 이미지 자원에 두고 Type3에는
검색 문자·전진폭만 남긴다. 이런 스트림의 hash608개가 대상 PDF와 같아도 원본 윤곽 공급
증거가 아니다. 만들어진 CFF의 모든 한글 윤곽이 비어 있음을 확인했다.

이 진단 글꼴과 공급 후 캡처는 `font-source-diagnostic/INVALID-font-probe.txt`로 무효화했다.
시스템 설치·공개 배포를 하지 않았고, 기존 글꼴 환경·시각 수용 증거와 섞지 않는다.
원래 공급 환경의 C101 Native50쪽70.55984→77.95033% 개선은 도형 변환 보정에 대한
별도 증거이며, 글꼴 공급 진단의47.87584/55.66478%를 정상 후보 점수로 사용하지 않는다.

`prepare_hft_hangul_font.py`가 전진폭만 있는 Type3를 빈 원본 글꼴로 받아들이지 않도록
실제 윤곽 경계를 검사한다. 새 검사 수정 전11건 중1FAIL→수정 후11PASS를 확인했다.
첫 경계 계산에서 PrivateDict가 없는 자체 진단 오류도 같은 범위에서 고쳤으며 실패 로그를
보존했다. 실제 빈 표본 restore는 출력 파일을 만들지 않고 거부한다. 기존 유효한 독립
한양신명조11172자 표본은 계속 복원되고, 기존 생성본과 cmap·hmtx·11173개 CFF glyph program
(.notdef 포함)이 모두 동일하다. 이 도구 검증은 sample11의 전쪽 시각 완료를 뜻하지 않는다.
정본은 ignored `hwp3-drawing-frame-c101/font-source-diagnostic/`의 `guard-before.log`,
`guard-after-v2.log`, `guard-result.json`, `positive-glyph-identity.json`이다.


### C101 그룹 자식의 틀 좌표·평행사변형 복원

HWP3 표68은 상대 위치와 틀 기준 절대 위치를 구분하고 표70은 평행사변형 세 꼭짓점을 저장한다.
기존 파서는 offset 메타데이터와 정수 각도만 남겼다. 공통 group paint는 이미 합성된
render_tx/ty·2D affine을 소비하므로 자식들이 원점에 겹치고 전단·뒤집기·시작 꼭짓점이 사라졌다.
원본 헤더1390개를 일시 진단으로 읽고 추적 코드는 제거했다. 원본 문단1803의26개 구조 중
실제 도형20개의 누적 상대 위치는 독립 한컴 재저장본의 틀 기준 위치와 모두 일치한다.
이20개 세 꼭짓점도 순환 시작점을 구분한 뒤 독립 출력의 위치와 최대4.000488HU 차이로
대응한다. 이는 원본 단위·독립 변환의 원인 진단이며 신규 실제 배치 px 핀이 아니다.

파서에서 실제 자식 도형의 absolute_pos와 세 꼭짓점을 공통 전체 affine으로 정규화하고,
공통 직렬화 helper로 동일 행렬을 보존한다. 렌더러에 HWP3 분기를 추가하지 않았다.
최상위 단독 개체의 기존 흐름 앵커는 유지한다. 구조용 컨테이너는 독립 재저장과 같이
identity를 유지하며 자식이 자기 틀 좌표를 소유한다. 첫 후보가 이 구분 없이 빈 그룹에도
이동을 부여해 기존1892의 identity 계약1건이 실패했다(집중38건37PASS1FAIL).
검사를 변경하지 않고 구조용 그룹을 보정한 v2에서 파서9건+관련 왕복/깊이/flip/그라데이션/
소스 페이지/폰트 계약38건, 합계47건이 모두 통과했다. 첫 새 계약3건은 수정 전1PASS2FAIL이며,
행렬 쌍=그룹 깊이+1·단독 앵커·기존1892·151/64쪽 왕복도 유지한다.

v2 fmt·manifest check 및 Native 빌드156.31초가 통과했다. 실제 Native 원본50쪽 raw는
70.55984→77.95033%로 개선했지만 여전히90% 미만이다. 원본 sample11의1/56쪽86.00231/
86.87615%, sample16의6/33/40쪽95.56838/99.25546/94.38750%는 그대로다.
v2의6쪽 PNG가 v1 Native 보정 PNG와 바이트 동일함을 확인해 직접 판독한 도형 개선·잔여도
그대로 연결한다. source/painter 후보47PASS가 전151쪽 수용을 대신하지 않는다.

이 커밋의 drawing.rs에는 앞서 C94에서 검증한 dash·둥근 모서리 정규화도 현재 파일 상태로
포함된다. C100 원본64쪽 양쪽 결과가 이미 이 변경을 포함했고, 이번 gradient·border 집중
검사도 유지됐다. 다른 parser/paragraph/Skia dirty 변경과 사용자 초안은 이 커밋에서 보존한다.
정본은 ignored `hwp3-drawing-frame-c101/`의 `raw-headers.json`, `independent-group-translation.json`,
`independent-leaf-quads.json`, `before.log`, `v2/focused.log`, `v2/source.json`, `v2/producer.json`,
`v2/focus-results.json`, `v2/visual-v1-v2-comparison.json`이다. 새 fresh WASM·전쪽·전체 회귀는
아직 이 후보에서 완료하지 않았으며, font/꼬리말/도형 잔여를 계속 보정한다.


### C102 꼬리말 종류가 소유하는 바닥 정렬 복원

원본 HWP3의16번 컨트롤은 종류·적용 범위·본문을 저장한다. 파서는 Footer로 구분하면서도
공통 Header의 기본 TOP 정렬0을 그대로 남겼다. 원본 sample11의 독립 한컴2020 재저장본
(be089566…)은 꼬리말 BOTTOM을 선언한다. 파서에서 Footer 종류를 공통 list_attr BOTTOM으로
정규화하고, raw bytes·짝/홀/양쪽 적용·본문을 보존한다. 원본에 없는 선언 밴드 높이는
만들지 않는다. 실제 소비는 공통 layout_header_footer_paragraphs 및 바닥 밴드 배치이므로
HWP3 전용 paint 예외를 추가하지 않는다.

합성 저장 필드 계약은 수정 전2건1PASS1FAIL(TOP0≠BOTTOM2)→수정 후 통과이며, 원본 파서·
관련 왕복/그룹/수식/페이지/폰트 집중110건 모두 PASS(0.771초), fmt·manifest·Native 빌드
2분24초도 통과했다. 현재 전체 소스 ledger와 frozen Native는 ignored hwp3-footer-source-c102에
고정했다. 이 체크포인트는 C94 이후 아직 dirty인 파서 변경을 함께 소비한 결과이며,
해당 이전 변경을 이 커밋에 섞지 않고 C102 두 부분만 stage한다.

같은 원본·독립 PDF·글꼴의 Native 직접 비교에서 sample11은151쪽을 유지한다.
2쪽77.31158→80.46413%,50쪽77.95033→80.93424%,56쪽86.87615→89.06767%로 개선했다.
1쪽86.00231%는 PNG 동일이다. sample16은64쪽을 유지하며6/33/40쪽95.56838/99.25546/
94.38750%와 PNG가 모두 동일하다. 50쪽 canonical review와 overlay를 직접 열어 꼬리말
원점 개선을 확인했고, 키보드/허브 면 누락·본문 글꼴/간격 차이가 남은 것도 확인했다.
90% 미만 쪽을 수용하거나 새 실제 배치 회귀를 추가하지 않는다. 전쪽 fresh WASM·최종 전체
회귀 및 실제 Studio는 여전히 남아 있다.

진단용 HWPX에서 선언 밴드3600HU까지 복사한 후보는 바닥 정렬만 옮긴 후보보다 나빴다.
원본에 없는 수치를 넣지 않았으며 해당 진단 입력을 원본 시각 통과로 인용하지 않는다.
정본은 ignored hwp3-footer-source-c102/{before.log,focused.log,source.json,producer.json,
focus-state.json} 및 focus/after-sample11/{review,overlay}이다.


### 실제 Native 글꼴 선택과 독립 원본 공급 추가 진단

C102 frozen library의 공개 SkiaLayerRenderer.get_font_decision_trace를 CLI와 같은 font paths·
자동 FontEnvironment로 실행했다.6조건 모두 backendSummary.native는 complete이며,
일반 환경의 PDF word-gap13자는 요청 함초롬바탕→실제 Batang, 단일 HCR 공급에서는
동일13자가 HCR Batang으로 선택된다. 기존0.52817%p 하락을 fc-match나 SVG 이름만으로
판정하지 않고 실제 소비 경로에서 구분했다. OS의 대표 family 열거에 없는 이름을 바로
거부하는 분기가 설치 SFNT의 다른 언어 family를 조사하지 않고 일반 fallback으로 넘겼다.
기존 합성 family-contract 글꼴의 한국어 typographic family는 fontdb에 존재하지만 대표
이름만 있는 OS 목록에는 없다. 이 경계에서 실제1FAIL(0.071초)을 확인해 일반 원인을 보정 중이다.
새 동일 글꼴 별칭 규칙이나 다른 글꼴로의 강제 치환을 추가하지 않는다.

같은 관측에서 sample11 제목의 요청 하이텔울릉도제목체는 기본 Malgun Gothic,
독립 재저장에 기록된 명시 대체를 FontEnvironment로 지정하면 Haansoft Batang을 선택한다.
본문 고딕은 Malgun Gothic, sample16 본문 휴먼명조는 실제 휴먼명조다. 제목의 명시 환경만
높은 일치율을 보인 결과를 기본 제품이나 전151쪽 수용으로 사용하지 않는다.
HBATANG.TTF의 family는 한컴바탕/Haansoft Batang이며 HANBatang.ttf(HCR Batang)과 구분한다.
관측 driver의 HwpError 변환 누락도 수정했고, 재실행6조건 모두 완료했다.

독립 휴먼명조(HFT 요청) 전11172자 표본 Print는 성공했지만 실제 PDF는 Type0 TrueType이고
Type3는0건이다. guard를 완화하지 않고 restore를 거부했다. 독립 TTF 전11172자를 현재
Mac HMKMM.TTF와 같은 좌표·전진폭으로 비교하면 모두 일치한다(SHA d033df71…).
이는 기존 TTF 공급 확인이며 대상의 옛 HFT Type3 원본 공급 성공이 아니다. 시스템 글꼴이나
공개 글꼴 배포를 변경하지 않았다. 대상 PDF의 글리프는 비교에만 읽고 렌더 글꼴 생성에
사용하지 않는다. 정본은 ignored native-font-observation-c102의 observed-family-summary.json,
human-original-font-supply-c103의 Print 기록·format-diagnosis.json·
installed-truetype-program-comparison.json이다.


### C104 설치 글꼴 조회·TTC 소비와 공개 편집 계약 보정 중

OS의 대표 family 목록에 없는 실제 설치 SFNT의 언어별 family를 먼저 복구하도록
Native 조회를 보정한다. 없는 family를 OS에 임의 조회하지 않는 기존 경계와 실제
설치 파일만 복구하는 경계를 함께 유지한다. 새 이름 치환 규칙은 추가하지 않는다.
수정 전 한국어 typographic family 경계1FAIL을 보존했고, 수정 후 관련 글꼴·trace
검사에서 이 경계는 통과했다. 실제 word-gap 및 원본 HWP3의 시각 결과는 새 Native
생산자와 canonical 캡처가 끝난 뒤 따로 판정한다.

같은 공개 글꼴 검사에서 유효한 TTC의 두 번째·세 번째 face를 현재 Mac CoreText/Skia가
읽지 못하는 별도 실패도 확인했다. 독립 Skia API에서도 index0만 성공했다. TTC의
face별 SFNT 컨테이너를 재구성하되 테이블 내용은 head.checkSumAdjustment 외에
바꾸지 않는다. 원본 파일 크기·face 수 제한과 전체 교체의 원자적 실패 계약은 유지한다.
[OpenType 컨테이너 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/otff)의
TTC 파일 기준 테이블 offset과 checksum 규칙을 사용한다. 유효 TTC·손상된 후속 face
거부 및 같은 regular/italic face의 단일 TTF 대비 실제 Native 래스터 동등성은 통과했다.
다른 글꼴로 face0을 대신 선택해 통과시킨 결과가 아니다.

새 source-side 테스트 수를 늘리지 않는 현재 unit-tier 지침에 맞춰 그룹3·꼬리말2·
수식7 계약을 tests/cases의 공개 파서/DocumentCore 소비 경로로 이동했다. 설치 family
경계는 기존 missing-family 검사 안에서 정상 대조군으로 함께 실행한다. unit baseline은
올리지 않았으며 실제 check는4103 tests /296 modules /cfg support items28로 통과했다.
검사 이동은 실물 문서의 전쪽90% 선행 조건을 면제하지 않는다. 원본 sample11의 미달이
남아 있으므로 관련 실제 배치 완료·최종 제출 판정은 계속 보류한다.

공개 편집 검사는 수식 baseline이 바뀌어도 저장 LineSeg를 재구성하지 않던 오류를
드러냈다. 본문 또는 셀의 실제 소유 문단을 기존 공통 reflow로 다시 구성하고, 본문은
이전 문단 끝을 기준으로 후속 vpos도 재계산한다. 슬롯·여백·기준선 등 inline 흐름이
변할 때만 재구성하며 getter/no-op·색상 편집은 저장 프레임을 보존한다. 본문 편집의
실제 FAIL→PASS 및 기존 setter/raw/pass-through/삽입/삭제 대조군을 확인했다.
셀 검사에서 page y의 단순 증가를 기대한 자체 가정은 셀 원점과 분리해 수정했다.
또 baseline100%는 공통 paragraph 소비자가 descender를 예약하는 퇴화 경계이므로
90→100을 선형 이동으로 기대한 자체 검사도 잘못됐다. 실제 로그에서 저장 baseline과
배치 보정의 소유를 확인해90→95의 정상 구간으로 교정했다. 이 교정을 위한 제품
clamp 변경이나 기존 검사 완화는 하지 않았다. 최종 집중61건은61PASS(2.204초),
fmt·manifest·unit-tier와 Native 빌드126.365초도 통과했다. 관측 driver의 소스 파일을
복사하지 않은 자체 실행 오류는 복사 후 해당 단계부터 재개했고, 빌드·집중 검사를
중복 실행하지 않았다. 실제 Native 글꼴 관측6조건 모두 complete이며 word-gap의
일반 환경도 HCR Batang을 선택한다.

독립 V3 전11172자 휴먼명조/고딕 표본은 대상 문서의 글리프·본문을 복사하지 않고
별도 V3 컨테이너로 만들었다. 공개 export-text가 전11172자와 순서를 보존함을 먼저
확인했고, 독립 Print는 아직 실행 중이다. 대상 Type3와의 동일성·빈 윤곽 guard를
확인하기 전에는 공급 성공이나 원본 문서 개선으로 인용하지 않는다. 글꼴·글꼴 공급용
독립 PDF는 local-only로 유지한다.


C104 canonical Native 대조는 고정 base515bd72d 및 후보 각각7개 PDF의 전20쪽을
완료했다. 양쪽 모두 raw100%이며0.01%p 이상 하락0·90% 미만0이다. word-gap의
기존99.47183%는100%로 복구했고, 현재 review·overlay에서 세 줄의 단어 간격과
글자 소속을 직접 확인했다. raw100%는 엄격한 픽셀 바이트 일치를 뜻하지 않는다.
HWP3 영향7쪽은 C102와 같은 점수·쪽수다: sample11의1/2/50/56쪽86.00231/
80.46413/80.93424/89.06767%, sample16의6/33/40쪽95.56838/99.25546/94.38750%다.
따라서 설치 family 복구로 PDF 회귀는 해결했지만 HWP3의 미달을 해결했다고 쓰지 않는다.
정본은 ignored native-localized-system-family-c104/{focused.log,producer.json,
observation/state.json,focus-results.json,pdf-paired-result.json}이다. 렌더 전용 글꼴
대조C105는 완료했으나2/50쪽이 악화되어 채택하지 않았다. C106은 Johab 글꼴명을
EUC-KR로 적은 자체 입력 오류를 공개 HWPX export에서 확인해 INVALID로 분리했다.
Johab·종료 블록을 고친 C108의 짧은 표본도 독립 Print180초 시간 초과로 종료했고,
같은 실패 입력 재제출이나 전체 표본 출력은 하지 않았다. 새 fresh WASM·전쪽·
최종 전체 회귀·실제 Studio는 아직 완료하지 않았다.


### C110 인라인 개체 슬롯의 잉크 소유

원본 sample11의2쪽 pi23은 전진폭0의 U+FFFC 개체 슬롯이다. Native가 이를
실제 OBJ 글리프로 그려 상단 잔상을 만들고 있었다. 기존 문자열·문자 위치 인덱스는
유지하고 해당 클러스터의 잉크만 생략한다. 복합 문자 shaping도 이 슬롯을 넘지 않는다.
U+FFFD와 실제 개체는 그대로 보존한다. OFL 기반 독립 작은 글꼴로 결측/OS fallback
없는 실제 Native 재생을 확인했다. 최초 계약1FAIL 및 인덱스/복합 span 반례를 보존했다.
첫 after의 잘못된 대조 원점·합쳐진 복합 span 기대는 별도 실행으로 분리하고, 독립
각 run으로 대조를 정정했다. 유효 TTC의 nonzero face를 direct Skia로 열던 glyph_replay
경로도 C104의 같은 보존형 face 복구 함수를 사용한다. 최종 관련41건41PASS(1.850초).
새 Native 빌드124.072초, base/후보7PDF 각20쪽 raw100%·하락0이다.
상단 잔상은 실제 review에서 사라졌다. sample11 p2 80.46413→80.60623%,
p50 80.93424→81.29567%이며 여전히 미달이다. 잔상 제거를 페이지 완료로 쓰지 않는다.
정본: ignored hwp3-position-owner-c110의 after-placeholder.log·producer·focus-results·PNG.

### C111 원형 글머리의 독립 기호 프로그램 공급

사용자 지적 원본 sample16 p6/p33의 원문은 U+25E6이며 문자 원점은 독립 Print와
각각 약0.107/0.120CSSpx 차이다. 문제는 다른 작은 원 글리프로 대체한 잉크의 크기와
중심이다. 특정 문서/쪽의 위치 보정이나 원형 윤곽 근사를 추가하지 않는다.
대상 본문·글리프를 복사하지 않은 별도 빈 HWPX 표준 글머리 표본(·•∙○◦)을
한컴2020 Print했고, U+25E6 프로그램 SHA aeb96fbb8f28029864380c8e96bb2ec9632bdecf17869103a169e715da8ae1a7이
대상 PDF와 동일함을 읽기 전용 비교로 확인했다. 대상 PDF로 글꼴을 만들지 않았다.
독립 표본 프로그램을 이동·스케일·획 변경 없이 공통 공급 경로에 전달한 Native
진단 p33은99.25546→99.57555%, p6은95.56838→95.61977%, p40은 동일94.38750%다.
33쪽 overlay에서 원의 이중 윤곽이 해소된 것을 직접 판독했다. p6 수식 미세 위치는
별도 잔여다. 이것은 선택3쪽 공급 진단이며 전쪽/fresh WASM 수용이 아니다.

공통 준비 도구에 --with-bullets 선택을 추가했다. 기존 전체 현대 한글11172자 검증을
유지하면서 표준 기호5자를 함께 포함한11177자 독립 표본 전체 순서·완전한 glyph 집합을
확인한다. 기호가 하나라도 빠지거나 일반 대상 문서 PDF이면 거부한다. 기존 기본 동작은
한글 전용 그대로다. Python12검사PASS. 새 전체 독립 Print9a7d7f025f063ba6abe98f819511f748a5788e71ae9557c73c60626cc65dfde6,
로컬 원본 glyph 공급81ad60be2dd753f16fffec4972e8a02cd6035179a910358c934ce5b431b6a4dd.
폰트·폰트 공급용 PDF는 local-only이며 시스템 설치나 저장소 포함은 하지 않는다.
정본: ignored hft-outline-bullet-c111의 source/Print/restore·tool-tests·focus-result.

### C112 상하단 선·구두점·물리 용지 좌표계 보정 후보

사용자가 지적한 문단을 실제 최종 위치와 독립 PDF의 글자 origin으로 대조했다.
첫 글자는 거의 같고 '.' 뒤부터2.2~2.9CSSpx가 당겨지며 쉼표 뒤에 다시 누적된다.
별도 전체 ASCII(95자) 고딕 HFT 표본 Print의 자연 전진은 모든 글자500/1000em이다.
미확인 비례 기호0.3em fallback이 원인이다. 기존 declared-original-glyph 경로에
확인한 고딕 HFT regular ASCII만 전달한다. 실제 선택한 TrueType·미검증 굵기/기울임·
비ASCII는 승격하지 않는다. 실제 구두점 폭 before1FAIL(원본2812HU, 기대3750HU)을
보존했다. 첫 after는 잘못 지정한 TrueType 대조 환경으로16PASS1FAIL이었다.
존재하는 실제 HCR Dotum 선택과 trueType 선언으로 대조를 고친 관련17건은17PASS
(0.182초)다. 이 단계의 시각 재출력은 아직 완료하지 않았다.

p2의 Native→PDF 세로 오차를 전체 본문 줄에서 적합하면 dy=0.00103325*y+0.07724다.
밑으로 갈수록 커지는 비율은 논리 높이841.88pt→Print841pt와 일치한다. 하단만
이동시키지 않고 Print의 실제 용지 프레임 투영을 모든 paint 재생에 한 번 적용하는
후보를 작성했다. Screen·임의 하위 트리 bounds에는 적용하지 않고, canonical 루트의
원본 용지와 정수 point 출력 용지가 대응할 때만 사용한다. 바탕쪽 raster의 별도 선행
투영은 중복이므로 제거했다. 원본 조판·글자 위치·crop은 변경하지 않는다. Native,
SVG, Canvas2D, CanvasKit 및 기록형 Canvas가 같은 프레임 계약을 소비하도록 했다.
이 후보는 전쪽 재검증 전이며 기존 편람 Print 프레임 검사를 새 출력으로 자동 갱신하지
않았다. 기존 IR 프레임 검사의 적절성은 실제 전체 Native/fresh WASM과 함께 판정해야 한다.

상단 선은 원형/글꼴 전체의 문제가 아니라 U+F080F 표시 투영도 관련된다. 독립 PDF
원본 선 glyph의 전진0.5em·높이0.04199em과 비교해 굵은 U+2501(약0.0742em) 대신
가는 U+2500(약0.0371em)으로 공통 표시 매핑을 보정하는 후보를 작성했다.
상하단 어떤 쪽 번호나 문서 식별자도 사용하지 않는다. 남은 직접 판독과 전체 검증은 보류다.
정본: ignored hwp3-spacing-bars-c112의 coordinate 비교·vertical-origin-fit·독립 ASCII
Print·폭 기록·before/after 검사·새 빌드 로그. reference font는 비교에만 읽었다.

### C107/C113 사각 채우기 및 그림 잔여

독립 PDF는 사각 채우기를 회전한 사각 색 띠로 그린다.37개 clip의 중심/반변이
원본 상자 변까지의 최대 거리·원본 각도로 설명됨을 확인했다(출력 격자 오차 최대0.077pt).
Native와 CanvasKit은 같은 SkSL 거리 shader, SVG는 네 삼각형의 선형 램프,
Canvas2D는 제한된 현재 물리 해상도 타일에서 같은 거리/색 램프를 소비하는 후보다.
기존 원형 근사와 SVG 선형 근사는 사각 채우기의 의미를 보존하지 못한다.
Native 빌드2분01초·Studio tscPASS. 첫 TS 직접 gradient 호출의 배열 형식 오류는
Float32Array 배열로 고쳐 재검증했다. 고정 base/후보7PDF 각20쪽 raw100% 유지.
이 사각 보정만으로 p50의 원래 빨간 면은 해결되지 않았다. p50 PNG 색 픽셀은 바뀌었지만
raw81.29567%는 같고 큰 도형 차이가 여전히 보인다. 개선 완료/주원인 해결로 쓰지 않는다.
현재 parser의 그룹/자식 행렬 및 실제 leaf 경로와 독립 Print 기하를 계속 대조해야 한다.
그림자라고 추측한 경로는 독립 재저장847개 shadow가 모두 NONE임을 확인해 배제했다.
현재 후보에는 새 용지/구두점 보정도 추가됐으므로 C113을 최신 head 시각 결과로 재인용하지 않는다.

### C114/C115 공급 범위 재검토와 C116 공백 소유 보정

C114에서 전체 한글·기호 공급81ad60be를 함께 사용하자 기존 한글 모양도 바뀌고
sample11 p56이83.09308%로 낮아졌다. 이 실행은 채택하지 않았다. 준비 도구의
`--with-bullets --symbols-only`는11177자 독립 표본 전체 검증을 유지하면서 표준 기호5자만
내보낸다. 최종 로컬 기호 공급 SHA6a00e7259dcdb85652d54bcf0ea8235f27fc277c64bb897da7b1a56a904e4b67,
시스템 설치·글꼴 바이너리 커밋은 없다. 기존 Python12검사는 이 경로의 실패 조건도 포함한다.

C115는 원본 고딕의 HFT 선언을 parser에서 공통 IR로 전달하고 이5기호 공급으로 재출력했다.
Native p1/p2/p50/p56은85.97209/95.22271/85.09250/90.52942%, sample16 p6/p33/p40은
95.72381/99.63289/94.36641%다. 고정 base/후보7PDF 각20쪽은 raw100%·하락0이었다.
Native 선택 쪽의 결과이며 전쪽 수용은 아니다. fresh WASM 빌드는207.033초 exit0,
root/pkg와 Studio/public JS·WASM 해시 일치까지 확인했으나 해당 패키지의 시각 캡처는
하지 않았다. 이후 source가 바뀌므로 이를 새 수정의 fresh WASM 통과로 인용하지 않는다.

사용자가 지적한 C115 p2의 `5.2 BSD Accounting Utilities` 왼쪽 이동은 실제 회귀다.
원본 문자열의 앞 공백9개는 보존됐지만, useFontSpace가 표시용 돋움의0.333em을 선택해
공백당4CSSpx로 줄었다. 독립 전체 ASCII Print에서 확인한 원본 고딕 공백은0.5em이다.
C116은 기존에 해석된 영문 슬롯 원본 metric face로 공백을 먼저 조회하고, 미확인 원본·
명시 TrueType·내장 글꼴·미측정 style은 기존 표시 체인 조회로 되돌린다.
측정 결과→TextRun cluster advance→각 backend의 동일 위치 소비 경로를 사용하며
문서/쪽/문단 ID나 좌표 보정은 추가하지 않는다.
보정 전 실제 공백 검사는0PASS1FAIL(2503HU, 독립 기대3750HU)이었다.
첫 명령의 nextest `-j`/`--test-threads` 중복은 검사 실행 전 인자 오류로 분리했고
`--build-jobs 8 --test-threads 8`로 바로잡은 실행의 실제 실패를 기록했다.
공백 기대값은 새 미달 페이지의 절대 위치 핀이 아니며 기존 글꼴 원본 소유 후보 검사의
진단 확장이다. 현재 새 집중 검증·Native/fresh WASM 선택 쪽 확인을 진행 중이며,
정식 회귀 반영/완료 판정은 관련 전쪽 시각 선행 조건을 충족한 뒤에 한다.
정본: ignored original-space-owner-c116의 source·before-space-corrected.log·검증 로그.

C116 집중17검사는17PASS(0.195초), Native/fresh WASM 선택7쪽씩 재출력했다.
2쪽은 각각96.04658/96.05249%로 공백 prefix의 좌측 당김이 해소됐음을 직접 review·overlay로
확인했다.50쪽은85.09250/85.09749%이며 그림을 해결한 결과가 아니다. 원본 쪽수151/64는
유지했으나 선택 쪽 캡처이므로 전체 페이지 수용으로 기록하지 않는다.

### C117 그림과 그룹 도형의 공유 기준선

사용자가50쪽의 파란 PC 상단·연결선을 재지적했다. PDF와 같은 물리 배율로 확대하면
서버·라우터 전체가 같은 양만큼 움직인 것은 아니며, PC3개만 위에 놓여 있었다.
원본과 독립 한컴 재저장본의 같은 문단1803은 lineHeight4412HU·baseline3750HU,
PC 그룹 높이3588HU·서버4412HU를 보존한다. 기존 그림 전용 inline_picture_box_top_y는
높이가 다른 TAC 상자의85/15 공유 기준선을 적용하지만 Shape 발행은 각 상자 바닥을
기준선에 앉혔다. 같은 원본 줄 안에서 그림·그룹 도형의 의미가 달라진 것이 원인이다.
기존 함수를 inline_visual_box_top_y로 통합하고, 정규 line_tac_offsets에서 같은 줄 소유
그림/도형의 여백·캡션 포함 상자 높이를 함께 조회한다. Shape의 런 중간·빈 줄 발행도
그 결과를 inline_shape_position으로 등록하며 shape_layout은 이 등록 원점을 소비한다.
source frame→composed 줄→정규 줄 소속→공유 상자 위끝→등록→그룹 자식 원본 행렬의
연결이다. 줄 높이와 내용 순서를 바꾸거나 측정에서 쪽별 위치 예외를 추가하지 않았다.

공개 LayerTree 진단은 가장 바깥 control0..3만 소유자로 선택했다. 최초 진단이 같은
controlIndex를 가진 내부 구조 그룹까지 중복 수집한 것은 바로잡았으며 최종 before는
네 원본 개체만 비교한다. 보정 전 낮은 그룹 위끝 차이2.16CSSpx는 독립 줄 계약의9.33845와
불일치했고, 보정 후9.339로 관계가 성립했다. 이것은 완료 페이지 px 회귀가 아닌 진단이다.
관련 원본 글꼴 대조17건17PASS(0.195초). Native 선택50쪽87.58764%로 이전85.09250%보다
개선됐으며 직접 review/overlay에서 PC 상단의 큰 파란 이중 윤곽이 사라짐을 확인했다.
아직 연결선 길이·도형 면 차이는 남는다. fresh WASM 캡처 완료 판정은 별도 결과를 따른다.
정본: ignored common-inline-visual-baseline-c117의 source·before/after-shared-baseline·
공개 observer·focus TSV/PNG·빌드 로그.


### C118–C122 상하단 선·쪽 번호·분할 객체 뒤의 소수 좌표 재검토

C117의 fresh WASM 선택7쪽 비교는 완료됐고 p50은87.59395%였다. C118은
명조 HFT의 독립 ASCII/선문자 표본에서 ASCII95자 반각 및 확인한 `─│═` 전각만
전달했다. Native/fresh WASM p50은87.86786/87.87420%이며 전쪽 수용은 아니다.

사용자 지적 후 C112의 용지 전체 투영 가정을 철회했다. 동일 Print 트리를 재생하며
해당 변환만 제외한 진단에서 p2는96.04658→97.00852%, p50은87.86786→88.53801%였다.
정상 재생 진단 PNG는 C118 canonical PNG와 픽셀이 동일함을 먼저 확인했다.
이 진단의 루트 좌표 변경은 원인 분리용으로만 사용했으며 구현에 넣지 않았다.
실제 구현은 C112의 전체 용지 변환만 제거하고 기존 master raster 인쇄 프레임 계약을
복원했다. C104 글꼴 소유·C107 사각 채우기 등 다른 수정은 보존했다.

기준 PDF p2의 text CTM은0.119935/-0.119869, p50은0.120/-0.120이다.
같은 물리 용지 크기만으로 모든 쪽의 본문·기호를 축소한다는 가정은 성립하지 않는다.
원본 재인쇄 job30e48298-aa44-4972-b3b7-3563a39d082c는 timeout900으로 요청했지만
서비스 내부 `output did not become stable within 60 seconds`로 실패했다.
기존 PDF를 교체하거나 이 실패를 새 Print 증거로 쓰지 않는다.

쪽 번호 앞 `Inc.`의 마침표 전진도 별도로 확인했다. 독립 빈 HWPX의 산세리프 HFT
ASCII95자를 한컴2020 Print로 인쇄한 자연 폭은 전부500/1000em이다.
원본의 한컴 재저장본도 산세리프를 HFT로 기록한다. source parser의 프로그램 형식을
보존하고 기존 declared-original-glyph 측정 경로에 regular ASCII만 전달했다.
명시 TrueType·내장 프로그램·미확인 style은 기존 경로를 유지한다. 독립 표본의
마침표/숫자 전진 진단 before는252/506HU로 실패했고 after는각500HU로 통과했다.
쪽 번호 자체를 옮기지 않고 앞 글자의 측정 결과가 실제 위치까지 전달되도록 했다.

또한 p50 작은 marker-only router 도형의 저장 lineHeight1000HU에는 문단150%의
종료 간격500HU가 맞는데, 큰 TAC 개체의 고정600HU를 적용해 뒤 줄이100HU 밀렸다.
글자보다 줄 상자를 높이는 marker-only 도형에만 기존 큰 개체 종료 간격을 적용한다.
작은 inline 도형은 실제 글자 높이와 문단 비율을 쓰며 명시 fixed 간격은 보존한다.

이 묶음 C120은 기존 관련17검사 PASS, Native CLI 빌드1분57초, 선택7쪽 비교 완료다.
Native sample11 p1/p2/p50/p56 raw는86.00231/97.04017/89.99479/96.37768%이며
sample16 p6/p33/p40은95.61977/99.57555/94.38750%다. p50을 반올림해90% 통과로
쓰지 않는다. 쪽 번호는 개선됐으나 가로줄의 파란 잔여와 그림 아래 누락 면이 남는다.
가로줄 실제 ink 중심 진단은 p50 Native104.39682/1018.39682CSSpx,
PDF104.49281/1018.49281CSSpx다. 위치·두께·coverage를 별도로 기록하며 완전 일치를
주장하지 않는다. 이 수치를 새 절대 px 회귀로 고정하지 않았다.

C121의 글꼴/소수 원점 진단은 첫 실행에서 수정한 트리에 같은 renderer를 재사용해
정적 subtree cache 영향 가능성이 있었다. 그 결과로 원인을 판정하지 않고 각 변형에
새 renderer를 쓰는 fresh-replay로 정정했다. source/style 변형은 원인 분리용이며
글꼴 override나 fixed_glyph_advances flag를 구현에 넣지 않았다. 기준 PDF 글꼴은
읽기 전용 비교만 했으며 font source로 사용하지 않았다.

C122는 하단 연결선의 또 다른 실제 원인을 보정한다. 마지막 줄 대시 복원량 계산이
한 글자 자연 폭6.66667px를 run용 정수7px로 반올림해19자에서 약6.33px 누적시켰다.
기존 leader 복원 규칙과 측정·배치 소비 경로는 유지하고, 복원량에 소수 자연 전진을
전달한다. 독립 Print의 끝점과 source glyph 전진이 근거이며 해당 문서/쪽/좌표를
조건으로 쓰지 않는다. 현재 C122 Native/fresh WASM 재검증 중이며 이전 수치를 최신
후보의 최종 결과로 재인용하지 않는다.

정본은 ignored `rebase-515bd72d-20261017/{print-coordinate-diagnostic-c119,
common-source-coordinates-c120,legacy-rule-glyph-diagnostic-c121,
common-fractional-advance-c122}/`의 source ledger·생산 로그·독립 Print provenance·
실제 PNG·TSV다. source/Print 확인용 폰트와 진단 로그는 커밋하지 않는다.
전쪽 Native/fresh WASM·전체 회귀·실제 Studio 소비 및 최신 devel 무회귀는 미완료다.


C122 검증은 종료됐다. 기존 머리말/바탕쪽 프레임/원본 글꼴 소유22건은22PASS
(검사 실행0.703초, 빌드 포함143.416초)다. Native CLI는1분56초, Mac fresh WASM
locked web no-opt는3분19초에 완료했고 root pkg/Studio public의 JS/WASM hash가 같다.
이는 Docker 최적화나 실제 Studio HTTP 초기화 확인이 아니다.

| 원본 / 선택 쪽 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| sample11 p1 |86.00231%|86.00073%|
| sample11 p2 |97.04017%|97.04076%|
| sample11 p50 |90.04172%|90.05212%|
| sample11 p56 |96.37768%|96.38220%|
| sample16 p6 |95.61977%|95.72898%|
| sample16 p33 |99.57555%|98.83569%|
| sample16 p40 |94.38750%|94.46126%|

p50의 끝 세로선 원점은428.36→422.026666CSSpx, 독립 Print는421.918416CSSpx다.
원문·19개 대시 수량·기준선은 같고 대시 전진이7→6.666667px로 복구됨을 확인했다.
쪽 번호는 꼬리말의 표시 필드로 선택해 원점을 대조했고 Native−PDF 오차는
가로+0.053313/세로−0.026628CSSpx다. 이 진단을 새 px 회귀 검사로 등록하지 않았다.
가로줄의 미세한 중심·두께/가장자리 차이는 남아 있으며 p50의90% 통과로 면제하지 않는다.
C123에서 draw ratio만 줄이는 진단도 replay 위치까지 줄이므로 부적합해 폐기했다.
문서/쪽별 좌표 이동·폰트 강제 override·허용치 변경은 채택하지 않았다.
샘플16 p33의 Native/WASM 차이 및 최신 고정 devel 대비 전쪽 변화도 아직 별도 검증 대상이다.
현재 branch를 전쪽 통과 또는 PR 준비 완료로 보고하지 않는다.


### C124 독립 인쇄에서의 연결 문자·도형 소유 재검토

그림 아래 회색·갈색 면을 누락 도형이라고 단정하지 않고 독립 한컴 재저장 HWPX에서
연결 문단만 남긴 입력과 그림·연결 문단을 함께 남긴 입력을 별도로 만들었다. 원본과
기존 기준 PDF는 보존했다. 두 입력은 한컴2020 11.0.0.9136의 동일 Print method0으로
각각1쪽 출력했으며 연결 문자만 있는 입력에는 면이 없고 그림을 포함하면 재현됐다.

같은 그림 포함 입력에서 본문 XML과 BinData를 그대로 두고 charPr17의 글꼴 선언만
명시 TTF 함초롬돋움으로 바꾼 대조군에서는 면이 사라졌다. 이 대조는 원본의 글꼴을
바꾸어 시각 수용한 결과가 아니며, 면이 인쇄 과정의 글꼴 선택과 관련됨을 분리한 진단이다.
사용자가 제공한 한컴 원본 편집기 사진(하단 쪽번호49, 원본명 hwp3-sample11.hwp)에는
회색·갈색 면이 없고 연결 문자는 끊어진 선으로 보인다. 따라서 해당 면은 원본 도형의
기대 모습으로 채택하지 않는다. 같은 편집기에서의 Print PDF를 추가 대조하기 전에는 기존
기준 PDF를 폐기하거나
화면에 없는 면을 추측해 추가하거나 비교 영역을 제외하지 않는다.

C122 frozen Native/fresh WASM으로 연결 문자 대조군93.54505/93.53612%, 그림 포함
대조군76.00125/76.00799%였다. 이들은 독립 부분 입력의 진단이며 원본151쪽 수용이나
다음 리베이스 헤드의 결과가 아니다. 정본은 ignored reference-glyph-image-owner-c124의
입력·Print 출처·control-capture-results.json 및 review/overlay다.

### f9d2ab17 리베이스와 검증 구분

새 upstream/devel f9d2ab17ab438c6bed97d91c3c2a98a5be27e573에는 #7980 PDF 원본 마스크·
확대 sampling 및 #7981 기본 활성 wasm-api/library 선택 기능이 반영됐다. 기존57b394be
위의 사용자 초안과 후보109파일을 archive·hash·별도 stash로 보존하고259commit을 리베이스했다.
새 HEAD는 da77fa1da9d4fdf24435ca667da579a3d0e6b0ac다. 사용자 오늘할일과 pr7871 초안의
SHA-256은 복원 후 동일하며 기존 백업과 stash도 유지했다.

Native image_conv·WebCanvas·CanvasKit의3충돌은 새 원본 확대 interpolation과 이진 효과
우선순위를 보존하면서 기존 Print 양 축 축소 prepare_bitmap 경로를 유지하도록 해결했다.
후속 WIP를 복원했고 source4파일의 hash 차이는 새 base의 실제 변경이다. 자동 복원·컴파일을
기능 보존 통과로 간주하지 않는다. 새 source ledger에서 PDF 마스크·resource limits·SVG계약,
원본 글꼴·Print 프레임·머리말/꼬리말 검사를 실행 중이며 API 없는 Native/WASM library와
기본 Native build도 순차 검증한다. 이전 C122 점수·패키지는 새 헤드 증거로 재사용하지 않는다.
정본은 ignored rebase-f9d2ab17-20261017의 preservation·range-diff-explicit·restoration-audit·
rebase-final-result 및 validation-c125 로그다. 전쪽 시각·fresh WASM·전체 회귀·실제 소비 경로가
미완료이므로 현재 PR 준비 완료가 아니다.


### C125 새 base 집중 검증과 C126–C129 추가 증적

새 base/head의 Rust 집중178건과 Studio 이미지30건은 모두 통과했다. API 없는 Native/WASM32
library check도 각각50.58/44.31초에 통과했고 기본 Native release-test build는156.19초에 완료됐다.
C125 Native의 원본 선택7쪽은 PNG 모두 리베이스 전C122와 바이트 동일했다. 새 PDF 마스크의
독립5조건은 raw100/92.55307/100/100/100%이며 축소 coverage와 standalone stencil의 review도
직접 열었다. 전쪽·fresh WASM·실제 소비 경로·전체 회귀의 완료가 아니다. fresh WASM은 빌드 중이다.

독립 현대 한글11172자·표준 글머리 기호5자·Unicode Box Drawing128자의 표본을 MCP2020
Print로 추가했다. 텍스트 순서11305자는 완전히 같고 선문자103자는 실제 Type3 윤곽을 가졌다.
그러나 표준U+2500과 원본 HNC/PUA 선문자의 동일성은 미확인이므로 그대로 글꼴 공급에
채택하지 않는다. 추가 전체PUA F0800–F08FF 독립 표본은 PDF에서256자가U+F000으로
바뀌어 원본 순서 검증에 실패했다. 이 표본에서 원본 선 glyph를 공급하지 않으며 현재 위치나
윤곽을 점수에 맞춰 옮기지 않는다.

사용자 지시로 원본을 MCP2020 timeout1800초로 재Print한 job
`e2ed7dfd-4312-4021-8f7b-4517d9f63735`는 서비스 내부의
`Hancom conversion output did not become stable within 60 seconds`로 실패했다.
이전 PDF를 교체하지 않고 실패로 보존했다. 이어 사용자가 win10-ted의
`C:\temp\hwp3-sample11-ui-print.pdf`에 수동 Print 출력한 PDF를 제공했다.
실행 중 한컴11.0.0.9136, 원본 HWP SHA51b743b2…는 Mac 원본과 동일하고 PDF151쪽·28266804bytes·
SHAd787942b264a6593a0a5aafc50f44ecc85e7fca70b2617c2ec87c0e7315f7fda를 확인했다.
`pdf/hwp3-sample11-ui-print-2020.pdf`에 같은 bytes를 보존하고 이전 PDF도 유지했다.

이 수동 PDF에도 PC 아래 회색 면·라우터 아래 갈색 면이 있어 편집 화면과의 차이는 MCP에만
생기는 현상이 아니다. 원본에서 보이지 않는 도형을 추가하는 근거로 쓰지 않는다. 새 PDF 비교에서
Native 선택1/2/50/56쪽 raw86.32305/97.03969/90.08879/96.37722%다. 같은 최신 Native PNG를
재사용하고 기준 PDF만 바꾼 진단이며 전151쪽 최종 판정이 아니다. 정본은 ignored
rebase-f9d2ab17-20261017/{validation-c125,independent-line-program-c126,
independent-hancom-lines-c128,sample11-original-print-c127,sample11-user-print-c129}다.


### C131–C139 공개 대체 글꼴과 원본 기호 메트릭 보존

사용자가 제공한 sample11 수동 Print의 관측31글리프와 실제 공급 프로그램을 비교했다.
HBATANG은31글리프 윤곽·전진폭이 일치하지만 로컬 검토용으로만 보존하며 배포하지 않는다.
공식 Google Fonts Batang의 이름 분리본 `RHWP Public Batang`은31글리프 평균 IoU77.98255%,
advance 평균 절대차0.01823967em이며13개 한글은 윤곽·폭이 일치한다. 공식 Iropke
commit24e6a14cd95c903b6584bc947406773c2e0f164f와 동일 기준으로 비교하면
Iropke는37.98747%/0.07468120em이다. 이 수치는 글리프 표본 비교이며 페이지 점수가 아니다.
국립중앙도서관 공식 TTF와 사용자가 제시한 CDN Dosuguan WOFF는 같은 프로그램이 아니며
CDN본은 관측 괄호·마침표3자가 없어 동일 원본으로 취급하지 않았다.

공개 Batang의 ASCII95자와 현대 한글11172자 폭이 기존 Batang metric entry와 일치함을
확인하고 별도 family만 연결했다. 명시적 호출자 font environment의 C136 선택1/2/50/56쪽은
Native96.64017/97.03969/90.08879/96.37722%, fresh WASM96.65532/97.04028/90.09923/96.38174%다.
이 결과는 자동 원본 글꼴 대체 선택·전151쪽 수용 또는 실제 브라우저 문서 초기화 증거가 아니다.

WOFF2는 전체 Native TTF6,829,644bytes를2,832,012bytes로58.53% 압축했다.
FontTools4.46.0/Brotli1.1.0으로 전체22772글리프의 윤곽 명령·힌팅과 cmap20579항목,
가로·세로 전진폭, GSUB·name 등 테이블 보존을 확인했다. canonical `assets/fonts`와
라이선스 원장, validated registry의 Canvas2D 공급 및 CanvasKit 계획에 등록했다.
이 글꼴은 WASM 내부가 아니라 웹 앱과 함께 배포되는 외부 WOFF2다. Native는 TTF를 유지한다.
사용자가 제외한1648bytes `placeholder-contract.ttf`의 바이트·provenance는 변경하지 않았다.
기존 합성 파일의 공개 배포 고지만 원장에 연결했다.

C139 실제 Studio loader가 localhost7736의 canonical WOFF2를200으로 받아
SHA20d454f45cdf29cc4619e769f1f59f3b8920941b6b274a530e82d3b75e24dbfb를 확인했다.
한글·라틴·숫자/기호의4크기12조건에서 TTF와 Canvas 폭·픽셀이 모두 같다.
초기 진단은 /fonts/FONTS.md 아래에서 상대 URL을 해석하여 /fonts/fonts/를 요청한
스크립트 오류였다. INVALID로 보존하고 제품과 같은 root base로 바로잡은 재실행만 채택했다.
registry26·projection15·Studio runtime6·canonical font inventory3검사가 모두 통과했다.
기존 배포 dist 전체 검사·원본 글꼴 자동 대체·문서 전쪽 수용은 별도 미검증이다.

C125 전체 회귀의 sample10 본문 넘침101건은 원본 HNC F080F/A80F와 표시용 Unicode─를
동일 폭으로 측정한 회귀였다. 독립 Print의 원본 HNC는512/1024em, 별도 Unicode 표본은
전각이다. 일대일 표시 투영은 원문 metric identity를 보존하고, 필드값·복수 문자 확장은
실제 표시 문자열을 사용한다. 측정/조판의 확정 positions를 SVG·WebCanvas·Native Skia
replay에 전달하며, 별도 field/TAC 폭의 소유는 덮어쓰지 않는다.
새 공개 합성 계약4건은 fresh suite prepare 후 실제4PASS다. 진단 중 device grid를
무시한20CSSpx 기대값과 stale harness0건은 INVALID로 분리했다. 원본763쪽
Native layout-anomaly는 고정 f9d2ab17 base0 / 보정 전101 / C138보정 후0으로 복구했다.
쪽수763은 유지했고 기존 baseline·검사는 제거하지 않았다. 이것은 원본 전체 시각 수용이 아니다.
C138 Native 재빌드는124초에 완료했고 영향 선택4쪽은 C136과 동일했다. 이후 fresh WASM도 완료했으며, 아래 C141에서 새 base 재검증을 구분한다.
정본은 ignored rebase-f9d2ab17-20261017/{regression-expectation-audit-c134,
public-webfont-search-c135,public-font-layout-c136,iropke-comparison-c137,
source-symbol-metrics-c138,public-batang-webfont-c139}이며 실행 로그는 커밋하지 않는다.


### C141–C143 최신 devel 재검증과 배포 경로 보완

2026-10-18 KST 기준 upstream/devel8766fb47d8ec11547f0d2ae941c53cc3c9780ab2 위로
259개 로컬 commit을 리베이스했다. 현재 commit은837d6f7f31777473d512eff8568519b4e6577bfe이며,
후보 소스와 증적은 아직 working tree에 있다. 사용자 orders20261013 초안과 pr7871 초안은
보존 hash가 일치하고 stage하지 않았다. 첫 생성 JS 충돌 처리 스크립트의 오류는 public push 전에
전체 rebase를 abort하고 보존한 원래 HEAD에서 재실행하여 복구했다. 최종 충돌에서는 upstream
생성 JS를 임시 유지하고 이후 fresh wrapper로 JS/WASM 및 두 타입 선언을 함께 재생성했다.

C141 Rust focused9건은 새 upstream의 Unicode casefold4건, 원문 기호 metric identity4건,
sample10 overflow partition6 실제1건으로 모두 PASS다. 원본 sample10 layout-anomaly763쪽은
넘침0건이며 독립 PDF 전쪽 시각 수용을 뜻하지 않는다. Mac fresh WASM은 locked wrapper의
web/no-opt 로컬 대체 빌드다. Root pkg와 Studio public의 JS/WASM 및 두 d.ts hash가 모두 같고,
실제 Chromium155 초기화 응답 JS/WASM도 일치했다. 새 upstream의 exports8개와
HwpDocument 메서드489개가 빠짐없이 있다. 실제 브라우저 cross-section selection20개 및
undo-contracts99개 assertion이 통과했다. 이들은 실제 문서 출력36조건과 별도다.

C142는 original sample16의64쪽과 sample11의151쪽을 각각 Native/fresh WASM으로
전쪽 산출한다. sample16은 Native 최저94.38750%, fresh WASM94.46126%로 각64쪽 모두
90% 이상이다. sample11 Native는151쪽 중100쪽이88.89361%로 미달하여 제출 보류다.
WASM 전151쪽은 이 문단 작성 시 실행 중이며 부분 결과를 완료로 인용하지 않는다.
sample11은 명시적 caller font environment를 사용한 공개 Batang 대조군이며,
자동 원본 글꼴 대체나 독점 글꼴 미보유 환경의 일반 통과를 뜻하지 않는다.
100쪽 review/overlay를 열어 명령문 글리프 모양·굵기·폭 차이와 탭 뒤 시작점 차이를 확인했다.
원인을 글꼴 하나로 확정하거나 좌표 고정 보정을 적용하지 않았으며 실제 선택 경로를 추적한다.

C143 배포 검사 중 VS Code webview가 공유 CanvasKit renderer의 새 @wasm alias와
square_gradient.sksl?raw 공급 설정을 갖추지 않아 tsc 단계에서 실패했다. VS Code의
TS paths 및 webpack alias를 동일 pkg로 연결하고 raw SkSL을 asset/source로 공급하여
컴파일을 복구했다. Studio/Chrome/Firefox/VS Code 배포 빌드가 모두 exit0이고,
실제 산출물의 글꼴 바이트·전체 라이선스·VS Code 승인 subset 검사8건이 통과했다.
VS Code 런타임 출력 수용으로 확대해 주장하지 않는다. Chrome/Firefox 빌드의 의존성
해석을 위해 만든 이번 소유 root node_modules symlink는 빌드 뒤 제거했으며 Studio의
실제 node_modules와 기존 Vite7736 서버는 보존했다.

전체 회귀·나머지48문서 전쪽·실제 소비36조건·필수 게이트와 기존 기대값 감사는 아직 미완료다.
C125 전체 결과를 최신 후보 통과로 재인용하지 않는다. 정본은 ignored
rebase-8766fb47-20261018/{validation-c141,whole-originals-c142,font-distribution-build-c143}이며,
로그는 저장소 commit 대상이 아니다.


### C144–C147 굵은 HFT ASCII 전진·회귀 감사·기준 PDF 목록 교정

원본 sample11 전151쪽의 유일한 미달100쪽을 추적하여 일반 HFT 고딕의 굵은 ASCII가
미확인 대체 TTF 비례 폭으로 내려가는 것을 확인했다. 독립 ASCII95 × 세 HFT face ×10/20pt
Print의 자연 폭은 일반·굵게 모두0.5em이며 굵은 비공백 원점에만 약0.05em이 추가된다.
공통 측정은 이 기여를 별도 결과로 보존하고 SVG/WebCanvas/Native glyph fit은 실제 tracking을
윤곽 확대에 쓰지 않는다. 문자 언어 슬롯·음수 간격의 실제 측정 결과를 소비하며 문서/쪽 예외가 아니다.
검증하지 않은 기울임과 실제 TTF 선택에는 새 규칙을 적용하지 않는다.

C144 Native/fresh WASM 원본 sample16 전64쪽 최저94.38750/94.46126%, sample11 전151쪽
최저90.08879/90.09923%로 각 미달0쪽이다.100쪽은88.89361/88.81174→97.61437/97.55750%,
2/50쪽은 기존값을 유지한다. 원본 sample11은 명시 caller 공개 Batang 환경이며 자동 대체 완료는 아니다.
새 WASM 실제 초기화 응답은 JS479165bytes SHA7ba18737…, WASM20884389bytes
SHAf166563c…로 Root pkg/Studio public과 일치했고 upstream exports8/methods489를 보존했다.

같은 선행 조건 뒤 기존 issue826/5800의 F080F/A80F를 굵은 선으로 고정하던 두 기대값만
독립 Print의 가는 선으로 교정했다. IR 보존·일반 Unicode 굵은 선 통과·나머지 기호 검사는 유지한다.
source_symbol_metric_identity에 세 HFT face의 굵은 ASCII/일반체/명시 공개 TTF 대조를 추가했다.
완료33쪽은99.57555/98.83569%이며 첫 글머리표·뒤 본문을 원본 표와 내용으로 선택하고 독립
Print 원점을±2CSSpx로 검사한다. 정확한 최신 devel8766 Native library에 같은 정식 검사를
연결하면 baseline149.61333/기준172.64CSSpx에서 실제1FAIL, 후보에서는 실제1PASS였다.
별도 라이브러리 연결 검사와 정식 suite focused/full 실행은 구분한다.

C145 Studio 전체2331건 중2329PASS/0FAIL/2skip(52.427초), concurrency8을 확인했다.
Rust focused→release build/lib→전체 nextest→Native3종→lint/doc/manifest/tier 등의 순차 게이트는
실행 중이다. C146은 정확한 base8766을 독립 소스/target에서 Native/fresh WASM 빌드하고
48조건1745쪽/role/backend를 같은 font inventory와 raw2px90%로 전쪽 대조한다. 공유 target을
baseline 재빌드에 사용하지 않았다. 중복6GB tar만 SHA 확인 뒤 제거했고 추출 소스/산출물은 보존했다.

C146의 옛48조건 목록에 C81 사용자 수동 Print 이전 PDF가 남은 갱신 누락을 찾았다.
입력 SHA6a3cdf2c…가 동일한 sample16-hwp/2010-alias 두 조건은 C147에서 사용자 제공
lossless Print SHA9d0a504f…의 전64쪽으로 다시 계산한다. 현재 생산자 PNG는 source/producer/hash가
같음을 증명한 뒤 그대로 재사용하고 PDF만 정본 조건으로 래스터화한다. 새 Native 두 조건은
최저94.56610%/미달0쪽이다. 이전 PDF의18/35/41쪽 미달은 별도 진단으로 보존하며 결과를
덮어쓰거나 통과 쪽만 합성하지 않는다. 나머지46조건과 base/fresh WASM 비교는 아직 진행 중이다.

공개 Batang의 자동 기본 선택·실제 소비 경로·전체 gate가 아직 남아 있으므로 PR 준비 완료로
보고하지 않는다. ignored 정본은 rebase-8766fb47-20261018/{sample11-p100-c144,
final-gates-c145,paired-whole-c146,manual-reference-c147,fixed-base-8766}이다.
독립 입력·PDF·짧은 증거는 assets/regression-policy-20261015/hft-ascii-bold-20261018에 연결했고
로그·중간 JSON·글꼴 복구 진단은 commit하지 않는다.

## C148–C152. HWPX 독립 기준 대응과 최종 소비 범위 보완

정확한 sample16 HWPX SHA49e3e809…를 engine2024/1800초로 다시 Print했다.
job d84edad7-1319-4153-a90c-aa83995aaf58, 한컴13.0.0.3901, method0,
frame_print_to_pdf_ex_one_up 결과는 `pdf/hwp3-sample16-hwp5-hwpx-current-print-20261018-2024.pdf`
SHA107c7f410499dc395c7cd8fd114ffa329be1c597d9a05906a74f83362b43dc38다.
기존 HWP/HWPX와 새 HWPX의64쪽 본문/폰트/decoded image 스트림은 동일했고,
독립 한컴 HWP와 새 HWPX PDF를192DPI/Poppler shape로 실제 래스터화한128개 PNG도
대응64쪽 모두 해시가 같다. rhwp 출력은 이 대응 입증에 사용하지 않았다.

사용자 수동 Print는 HWP 입력에서 만들어졌으며 HWPX 직접 Print라고 주장하지 않는다.
전64쪽 독립 대응을 근거로 C150에서 같은 수동 Print와 정확한 HWPX 후보/base의 전체
비교를 별도 산출한다. 현재 후보 Native64쪽 최저94.56610%/미달0쪽이다.
기존 MCP PDF의 TTF/큰 JPEG 출력과 미달 결과도 보존하며, 서로 다른 출력 프로그램 조건을
기록한다. 통과한 쪽만 선택하거나 기준 PDF를 합성하지 않는다.2쪽 review/overlay의 제목,
본문 소속·표제/리더·페이지번호 순서를 직접 판독했고 경계·점선 위상의 미세 차이는 남는다.
base/WASM·나머지쪽 직접 판독이 아직 남아 있어 이 기록으로 전체 수용을 주장하지 않는다.

C149는 frozen C144 WASM에서48조건과 원본 sample11/16의 실제 글꼴 요청50건을 조회했다.
공개 fallback 대상 하이텔울릉도제목체 요청은 원본 sample11의7언어 슬롯에만 있고 나머지
49조건에는 없다. 현재 Chrome의 local 존재 probe는 없는 임의 이름/하이텔은 실패,
휴먼명조/Arial은 loaded다. 존재 확인을 TrueType/cmap/program 공급 증명으로 사용하지 않는다.
C145/C146 생산자 소스 guard가 끝난 뒤에만 canonical 조건부 fallback을 연결하고 검증한다.

이전 실제 소비 범위는 단순36건이 아니라 변경 경계를 확장한68관측이었다.
C152는 기존14쪽과 지원 CanvasKit 대조12관측을 보존하고, 원본 sample11의2/50/100쪽과
sample16의6/33쪽을 추가한 실제 제품 범위를 준비했다. Native38·Canvas2D38·기존 지원
CanvasKit12=88관측이 최종 요구 범위다. manifest 생성/입력 hash 대조만 완료했으며 새 후보
88관측 실행 완료로 인용하지 않는다. 기본 제품의 자동 글꼴 선택을 실제 확인하고 caller oracle
환경을 주입하지 않는다. ignored 정본은 hwpx-reference-c148, font-request-impact-c149.json,
hwpx-manual-correspondence-c150, public-default-plan-c151, actual-consumer-scope-c152다.

### C151. 기본 공개 글꼴 선택의 실제 FAIL과 미적용 보정안

실제 Chrome/Vite WasmBridge에서 원본 sample11을 열고 caller 환경·local-font 권한을
주입하지 않은 prepareDeclaredFontEnvironment(canvas2d)를 실행하면 prepared=false,
environmentId=null, 공개 프로그램 미공급으로 실제 계약1FAIL이다. root 경로의 최소 host
문서에서 다시 확인했으며 실제 초기화 응답 수집 오류는0이다. 최초 일반 Studio root 진단은
CDP 응답 body 수집 오류로 종료되어 별도 INVALID 로그로 보존했다. 페이지 기반경로를
수정한 계약 결과와 섞지 않는다.

현재 guard 대상 소스는 그대로 두고 ignored public-default-plan-c151/proposal에서 공통
조건부 요청/프로그램 snapshot/공급 보정안을 준비했다. Canonical registry의 sequence15,
layout-name178/전체845 rule로 reducer·generator validation을 통과했고 다른4projection의
내용은 바뀌지 않았다. 존재 확인과 프로그램 공급을 분리하며 원본/문서 지정 내장 프로그램의
source와 family도 같은 집합에서 보존하도록 생산·소비를 연결했다. 이 안은 저장소에 아직
적용하지 않았고 Rust compile/최종 통합을 통과한 것으로 주장하지 않는다.

보정안 supplier를 실제 Chrome FontFace에서 독립 실행한7조건(부재, 원본 존재,
문서 지정 존재, 원본 내장, 공개 이름 내장, 공급503실패, 문서 취소)은7PASS다. 실제 공개
WOFF2 SHA20d454f4…와 programId/TrueType flavor가 일치하며 원본 이름 CSS alias는 만들지
않는다. 이 실행은 수동 plan을 공급한 host 경계 검사이며 제품 자동 경로·전쪽 수용이 아니다.
현재 C144 frozen library에 연결한8개 프로토콜 검사 초안은5PASS/3FAIL: 공개 fallback 선택,
존재 전용 snapshot 수용, 문서 지정 내장 프로그램 source 제외가 FAIL이다. 실제 설치/폰트 공급을
증명하는 검사와 구분한다. 정식 suite 편입·새 source의 FAIL→PASS 증거는 아직 남아 있다.

### C153. 원본 두 문서의 정확한 최신 base 보완

C14648조건은 저장 변형 sample16을 포함하지만 원본 sample16 SHA559fd948…와 sample11
SHA51b743b2…는 포함하지 않는다. SHA로 중복 부재를 확인한 뒤 동일한 고정8766 Native/fresh
WASM과 같은 입력·PDF·글꼴/명시 public 환경으로 각215쪽의 baseline 산출을 시작했다.
C144 후보 전쪽과0.01%p 하락을 비교하며, 기본 제품 자동 선택과 explicit 환경 비교를 혼동하지
않는다. 원본 두 문서를 변형64쪽이나 과거 base 결과로 대신하지 않는다. ignored 정본은
original-fixed-base-c153이다. 공용 capture lock/8workers와 완료 SVG만 SHA manifest 후 정리하는
계약을 유지한다. 현재 C1453446파일/C1462249파일 source guard drift는0이며 중복 Cargo는
시작하지 않았다.

C147의 HWP/동일 alias와 C150 HWPX 수동 Print 대응은 후보 Native/fresh WASM 각64쪽
최저94.56610/95.11097%·미달0으로 완료했다. 정확한 base 같은 조건·직접 판독·전체 gate와
새 자동 경로 보정/전쪽 검증은 진행 중이다. 기존 MCP profile의 낮은 수치도 별도 보존한다.


## C154–C157: 실패 검사 재검토와 최신 기준 전환

- C145 전체 nextest는10,911실행 중10,908 PASS/3 FAIL/45skip이었다. C139의 기록된 공개 글꼴 규칙3개를 기존 봉인 의미 재구성에 포함하고 active844/metric71을 확인하는 C154 수정 후, 관련 정식 검사3개 모두 PASS했다. v1 봉인·기존 교체·기존 규칙 의미는 유지한다. 전체 nextest 재통과는 아직 아니다.
- C155에서 #7017 원본1쪽의 동일 독립 Print를 Native/fresh WASM으로 대조했다. 후보 raw96.51901/96.58529이지만 실제 절취선 점선60개가 누락되고 뒤 글자가 왼쪽으로 이동했다. base8766 WASM97.24031보다 하락하므로90% 이상이어도 완료가 아니다. 기존 검사를 완화하지 않았다.
- 원인: C144 원본 PUA 메트릭을 paint에 전달하는 경로가 TAC 흐름 판정용 삭제 투영을 호출했다. `expand_pua_display_text`의 기존 흐름 계약을 유지하면서 실제 paint 투영은 검증된 F081C 점선을 보존하는 공통 보정안을 C156 ignored output에 준비했다. 아직 운영 소스에 적용·컴파일·후속 시각 검증하지 않았다.
- C151 공개 대체 자동 선택안에는 실제 공급 snapshot의 조건부 canonical ruleId가 JSON 환경을 거쳐 공개 글꼴 trace에도 보존되도록 보완했다. 원본·문서 지정·내장·사용자 환경 우선권은 유지한다. 현재는 별도 proposal이며 실제 Native/브라우저 자동 선택 통과로 주장하지 않는다.
- upstream/devel4296fa045df2743a0eaa1b0cc71b2a3c1dd90bb6의 정확한 소스 snapshot을 준비하고 공유 target에서 순차 Native→locked web/no-opt WASM 기준 빌드를 시작했다. PR#8003은 초기 필드 소유 범위·SDK cursor/events/print를 포함한다. 실제 rebase는 아직 전이다.
- C146 후보 양쪽96조건이 끝나면 PNG/TSV/출처를 보존하고 미완료 구 base 비교를 최신4296 기준 검증으로 전환한다. 종료는 검증 통과를 뜻하지 않으며 재사용은 최종 source/input/font/runtime 영향 근거와 함께 별도 판정한다.
- C157은 C146의 PII 기본 환경4/20/21쪽 기존 PNG로 review/overlay만 만들었다. 새 캡처가 아니다. 기본 환경4쪽82.28043, 명시 검증 환경4쪽99.16244를 구분하며 기본 환경을 통과로 보고하지 않는다.

로컬 상세 로그·proposal·state: `output/pr-review/7445-completed-20261009/rebase-8766fb47-20261018/{registry-regression-repair-c154,pua7017-output-audit-c155,pua7017-common-paint-c156,public-default-plan-c151,upstream-4296-intake}/`와 새 `rebase-4296fa04-20261018/fixed-base-4296/`에 보존한다. 이 경로의 `.log`와 중간 JSON은 커밋하지 않는다.


## C158–C167: 최신4296 리베이스와 자동 공개 글꼴·검증 글꼴 범위 보정

- `4296fa045df2743a0eaa1b0cc71b2a3c1dd90bb6` 위에259개 commit을 실제 rebase했다. 새 HEAD는 `6e22e0a3c9c666b4c6937257015d9fdf803630dd`이며 dirty150파일을 stash `8be28b685dc0d6f55155cc347ebf57542f3f0674`와 별도 tar/hash 원장으로 보존·복원했다. 전후 변경 파일 바이트 차이0, 사용자 orders/PR7871 초안 hash도 동일하다. stash와 backup ref는 유지한다. 신규 초기 필드 helper의 내용과 호출은 새 upstream과 일치한다. 이것은 기능 검증 완료를 뜻하지 않는다.
- 위 C154–C157 절의 후보96조건 완료 대기 계획은 이후 확인된 점선 회귀 보정을 위해 취소했다. C146은93조건·3112완료쪽과 나머지 부분 PNG를 보존한 채, 다음 묶음129–144쪽을 시작하기 전 공용 capture lock에서 안전하게 종료했다. 부분 문서를 전쪽 완료로 계산하지 않는다. C147/C150의 구 base 대기도 종료하고 후보 대응 PNG와 TSV는 보존했다.
- 최신4296 기준 Native와 fresh WASM은 source archive/hash 고정으로 실제 빌드를 완료했다(333.142초/438.274초). Mac locked wrapper의 web/no-opt 대체 빌드이며 Docker 최적화 통과가 아니다. 공유 target은 보존한다.
- C158은 공통 paint의 F081C 삭제와 자동 공개 fallback·canonical rule trace 연결을 적용했다. Native 원본 점선1쪽은97.96961%로 개선됐고, 실제 “절취선” 줄의 글리프 누락을 복구했다. 독립 PDF와의 전체 배치·fresh WASM 재검증은 별도다. 최초 frozen rlib 단독 진단은 공유 deps가 바뀌어 링크하지 못했으므로 실제 FAIL 증거로 쓰지 않는다. 유효한 before는 C145 정식 검사 실패와 C155 실제 출력이다.
- 새 upstream 필드 문서의 C158 Native 전3쪽은99.99908/99.99969/100%였다. 원본 sample11은 명시 환경 없이 실제 자동 공개 선택 경로로151쪽수를 유지했고, 선택2/50/100쪽97.03969/90.08879/97.61437%였다. 이3쪽 결과를 자동 경로 전151쪽 수용으로 대신하지 않는다.
- C158 Rust registry3검사 중2PASS/1FAIL은 공개 metric 별칭의 서술형 ruleId를 hash ID로 가정한 trace 오류였다. C161은 실제 폭 선택에 쓴 생성 projection의 source/target/조건/order/ruleId를 검증하고 그 기록된 ID를 사용하도록 공통 경로를 보정했다. 기존 Ygodic230과 공개 Batang에 같은 규칙이 적용되며 canonical registry 이력·규칙 ID·기존 evidence 불확실성은 바꾸지 않았다.
- C161의 추가 전체71 metric trace 검사는 D2Coding 등4 identity 보존 규칙에도 별칭 변경 이벤트를 요구해2PASS/1FAIL이었다. C165는 같은 요청/해소 face와 실제 metric entry를 확인하고 identity에는 가짜 alias 전환이 없어야 한다고 검사한다. 나머지67개 실제 별칭은 정확한 canonical ruleId를 계속 요구한다. 제품 코드는 C161과 동일하며 기대값 오류와 제품 오류를 구분한다. C165 전체 집중 검사·fresh WASM은 현재 실행 중이다.
- Node font projection/supplier 계약28PASS, Studio 타입 검사exit0, editor package36PASS, hwpctrl26PASS를 확인했다. 실제 SDK Chrome·최종 전체 nextest·전체 lint/build/Skia·배포 및 actual consumer 검증을 대체하지 않는다.
- C163에서 PII 기본 환경의 미달이 검토용 글꼴 범위 오류임을 확인했다. HWP3에서 추출한6glyph/ASCII0·현대 한글0인 `HGSMJ-original-symbols.otf`가 일반 `한양신명조` family로 모든 문서에 공급되어 TTF 원본의 정상 대체 선택을 막았다. 같은 C158 frozen binary·입력·PDF에서 해당 검토용 HFT overlay 경로만 제외하자4/20/21쪽이99.16244/98.76278/99.62570%가 됐다. 원래82.28043/82.50942/89.42323%를 진단 증거로 보존한다. 글꼴 버전 차이라는 추정으로 처리하거나 예외 제출 gate를 사용하지 않는다.
- 모든52입력을 raw HWP5 FACE_NAME/HWPX font 선언 및 HWP3 parser 계약으로 확인했다. 원본 HWP3 또는 동일한 `한양신명조`의 HFT 선언이 있는32조건만 기호 overlay를 공급하고, 일반 TTF 선언에는 공급하지 않는다. 문서·원본·PDF·폰트 바이트·90%/2px 문턱은 바꾸지 않는다. 특정 문서의 렌더 분기를 추가한 것이 아니라 검토 자원의 공급 범위를 원본 프로그램 선언에 맞춘 것이다.
- C159 구 전역 글꼴 검증은55조건 완료 후 WASM PII49–64쪽 시작 전 안전하게 종료했다. C166은 같은52조건·backend당1964쪽을 올바른 font scope로 다시 검증한다. 동일 font scope인 HFT 조건의 기존 최신-base PNG만 frozen engine/input/PDF/environment/tool/font hash 대조 후 재사용하고 전쪽 TSV는 다시 계산한다. font scope가 바뀐 조건은 새로 캡처한다. 삭제한 SVG를 재사용 증거로 쓰지 않는다.

현재 정본은 ignored `output/pr-review/7445-completed-20261009/rebase-4296fa04-20261018/`의 `candidate-c165`, `whole-c166`, `focus-c167`, `font-scope-diagnostic-c163`이다. 이전 C158/C161/C159 결과는 각각의 source·font scope에 대한 이력이며 새 source 전체 통과로 재인용하지 않는다. 완료 전 remote push·PR·merge는 아직 수행하지 않았다.


## C169–C175 재접수·리베이스·검증 진행 (2026-10-18)

### 절취선 검사와 실제 출력

C165의 #7017 원본 전1쪽 Native97.96961% / fresh WASM98.00636%는 쪽수1/1이며,
두 backend의 review/overlay를 실제로 열어 두 표 사이 점선과 `절·취·선` 순서 복원을
확인했다. 글꼴 윤곽·일부 선의 미세 차이가 남아 Native98% 이상으로 보고하지 않는다.
기존 절대145.4±2px·용지793.3px 핀은 현행 조건에 맞지 않아 원본 여백·글자모양·
내용 소속 관계로 바꿨다. 원본 PUA60개와 실제 paint의 점선60개, 라벨 순서, 한 문단/한 run의
소속·누락/중복, 본문 폭 안의 점유를 보호한다. 가운데 정렬은 원본 음수 자간 한 번과
HU 반올림의 범위에서 좌우 여유를 비교한다. 현재 계산 좌표를 기대값으로 복사하지 않았다.

초기 작성의 u32→i32 변환 오류와 `bbox`의 끝 자간을 무시한 과도한 정확 동등 조건은
진단 실패로 보존했다. 원본 글자모양의-5% 자간과 실제 paint의 끝 전진을 대조해 검사 자체를
보정한 뒤 C169 정식1 PASS다. 이를 제품 결함의 수정 전 FAIL로 재사용하지 않는다.
제품의 수정 전 결함은 C145 전체 회귀 실패·C155의 점선60개 소실로 입증하며,
C156 공통 PUA paint 보정과 구분한다. C170 새 리베이스에서도 같은 정식 검사는1 PASS다.

### 6ed1682c 기준 동기화와 보존

원격 devel이4296fa04에서6ed1682c(#8004)로 바뀐 것을 live fetch로 확인했다.
신규 내용은 CRLF 입력/커서·Undo·저장 및 검색의 겹치지 않는 치환 범위다.
C166 두 드라이버는 capture lock을 얻고 실제 캡처/계산 자식이 없음을 확인한 묶음 경계에서
종료했다. base65조건(정상 명령54), head6조건302쪽과 부분 PNG·TSV·로그는 보존하며
새 base/head 전체 결과로 재인용하지 않는다. 리소스 추적기만 남은 상태를 실제 캡처로
오인한 첫 중지 helper는 종료 조치 없이 실패했고, 이를 구분한 재실행만 중지 증거다.

기존259개 commit을6ed1682c 위로 리베이스해 현재 HEAD는
`706f21e901201fbb5f8662505f18ff672eae9533`다. 보존161파일은 stash 복원 뒤 전부
이전 SHA와 일치했다. 사용자 orders20261013·PR7871 초안은 그대로이며 커밋에 섞지 않는다.
backup ref `refs/backup/7445-before-6ed-actual-20261018-042619`와
stash `30a8d8a131fbc0ee641930d4415698a5babaaa8c`는 유지한다.
충돌은 intake_and_review의 last_verified 날짜 한 곳이다. 최신2026-10-18 날짜,
upstream reviewer/API 선행 규칙과 기존 기능 보존 검증 규칙을 모두 보존했다.
새 지침을 만들거나 기존 규칙을 삭제하지 않았다.

### HFT/TTF가 섞인 원본의 공급 범위

같은 이름의 HFT/TTF 선언이 함께 있는3개 문서를 실제 공통 render-tree의
`source_font_is_hft`와 원본 기호 소유로 감사했다. sample16-2022의 해당 기호396개는
전부 TTF 소유이고 HFT 기호0개다. rowbreak18은 HFT 기호10개/TTF0개,
biotoxin205는 양쪽 해당 기호0개다. 따라서 선언 유무만으로 원본 HFT 다섯 기호용
부분 프로그램을 공급하지 않고, 실제 HFT 소유 glyph가 필요한 rowbreak만 유지한다.
나머지2조건은 그 review-only 부분 프로그램 경로를 제외하고 새 base/head 모두 재캡처한다.
글꼴 바이너리·버전·제품 조판 코드는 이 공급 감사에서 바꾸지 않았다. 이것만으로 실제
시각 개선·전쪽 수용을 주장하지 않으며 C171의 새 전쪽 결과를 기다린다.

### 완료한 실행과 남은 게이트

C170 새 HEAD에서 fmt·Native CLI/lib 빌드155.1초와8묶음 집중22 PASS를 확인했다.
여기에는 #7999의 검색3건, #8001의 CRLF5건과 강화된 #7288 분할 관계7건이 포함된다.
C172 Studio 전체는2341실행,2339 PASS/0 FAIL/2 skip,52.349초다.
공개 글꼴 선택 프로토콜8검사는 별도 정식 case로 등록했다. 실제 글꼴 공급이나 픽셀 golden을
모형으로 대신하지 않으며, source/embedded/document substitute 우선순위·저장 IR 보존을 검사한다.
이 새 case의 정식 실행과 전체 nextest는 아직 남아 있다.

C170 fresh WASM은 locked web/no-opt Mac 대체 빌드로 진행 중이다. 정확한6ed base도
독립 archive/source ledger와 동일한 base의 samples/PDF로 빌드·집중 검사한다.
공유 target Cargo는 C170→base→C173 전체 gate 순으로 연결했다. C171은52조건1964쪽을
Native/fresh WASM 각각 새로 캡처하며 이전 PNG를 새 head 출력으로 재사용하지 않는다.
base의 기존 쪽수 불일치는 숨기지 않고 대응 가능한 쪽과 미대응 쪽을 구분해 수집한다.
C174는 새 package의 실제 기본 웹폰트 선택과 HTTP JS/WASM 동일성을 재검증하고,
C175는 두 역할에서 CRLF 및 SDK 이벤트·프로필·중첩 caret 등8종을 실제 Chrome으로 대조한다.
모두 실제 완료 요약 전에는 통과로 표시하지 않는다. 변경한 출력의 실제 backend 범위와
최종 frontend 배포 build, 전체 검증·대표 직접 판독·증적/커밋·PR 및 후속 처리가 남아 있다.

### 디스크 정리

종료된 regression-policy-audit 아래 font_resources의 중복 추출 TTF 경로14460개,
논리100GiB를 정리했다. SHA별41개 byte-identical 프로그램은 별도 ignored 보존 경로에
그대로 남기고 모든 이전→보존 경로/hash mapping을 기록했다. 실제 여유 공간은
22,091,436,032→126,815,084,544바이트다. 원본 공급 폴더·사용자 글꼴·HWP/PDF·PNG·TSV·
로그·공유 target은 정리하지 않았다. 삭제된 추출 경로를 그대로 재사용 가능한 증거로 주장하지 않는다.

정본 ignored 경로는 `rebase-4296fa04-20261018/receipt-contract-c169/`,
`upstream-6ed-intake/`, `rebase-6ed1682c-20261018/`의 C170–C175 및
`font-resource-dedup-20261018/`다. 원시 로그·중간 JSON은 커밋 대상이 아니다.


### C176–C182 실제 완료와 실행 보정

C170 fresh WASM까지 완료했고 실제 WASM SHA는
`750665c377ec643fa4d977831561c8bb05d473a43a99a5ded6f843f9a8d13466`다.
C174 실제 기본 Bridge의 Public 선택·WOFF2 로딩·원본 저장 보존·초기 JS/WASM 응답 hash가
모두 통과했다. 정확한6ed base도 Native290.044초·fresh WASM360.754초와 #7999의3건,
#8001의5건을 완료했다. 두 역할의 제품 동작 대조는 별도 실제 Chrome 실행으로 계속한다.

C177은 새 package로 Studio/Chrome/Firefox/VS Code4종 배포 build와 자산·저작권/라이선스8검사
모두 통과했다. 첫 Chrome build의 vite 의존성 오류는 C143과 같은 작업장 root 의존성 경로 문제다.
원래 Studio 의존성을 가리키는 소유 임시 링크를 세 확장 build 동안 사용한 후 제거했다.
실패 로그는 보존하며 제품 source를 바꾸거나 사용자 상위 node_modules를 수정하지 않았다.

C173의 새8 protocol 컴파일은 공유 target의 이름에 hash가 없는 `deps/librhwp.rlib`가
base 빌드 산출물로 남아 후보에 있는 메서드·필드를 찾지 못했다. 제품 검사 실패나 정상
기대값 오류로 해석하지 않는다. C180은 후보 `src/lib.rs`의 byte hash를 보존한 채
mtime만 갱신해 후보 library를 재컴파일하고 원래 mtime도 복원했다. 전후 library hash가
다르고 source hash는 같으며, API8건을 같은 정식 경로에서 실제 실행해8 PASS다.
전체 cache·deps·다른 target 삭제나 테스트 완화는 하지 않았다. C173은 superseded이며
C181이 release build→release lib→전체 nextest→Skia/lint/doc/정책의 필수 묶음을 새로 진행한다.
Native feature가 없으면 실행되지 않는 변경9개 integration case도 C182에서 별도로 실행하도록
연결했다. 이들은 기본 전체 nextest·Skia lib만으로 실행됐다고 주장하지 않는다.

C175 base8종 실제 Chrome 검사와 각 실제 초기화 JS/WASM hash는 모두 통과했다.
그 뒤 소유 Vite 종료 code를0으로 가정한 harness 정리 단계가 실패했다. 제품 기능/검사 실패와
구분하며, 정상 종료는 stdin stop 요청과 명시적 종료 receipt로 보정했다. 새 lifetime smoke는
실제 exit0·소유 server 종료를 확인했다. 이미 완료한 base8종은 재실행하지 않고 미실행 head8종을
계속한다. C176 실제88관측은 이 종료 정리 실패를 보고 캡처 시작 전에 멈췄으므로,
첫 실행의 실패 상태를 보존하고 중복 캡처 없이 다시 대기시켰다. 실제 observer의 font trace도
현행 d.ts가 요구하는 JSON options 문자열로 수정했다. 이전 숫자 인자로 발생한 traceError를
선택 glyph의 증거로 쓰지 않으며 pixel capture·threshold·guard는 바꾸지 않았다.

C179는52조건 모두의 source owner를 읽었고, `source_font_is_hft=false`를 TTF로
일괄 해석하면 안 됨을 확인했다. HWP3 한양신명조는 타입0(미상)이며 원래 legacy 프로그램의
별도 근거가 있다. 원본 sample16과 issue265의 이 상태를 TTF라고 바꾸거나 원본 HFT 기호
공급을 제거하지 않는다. 반면 sample16-2022의 현재 WASM source trace를 전64쪽4096자 제한으로
읽어 관련396개 모두 실제 타입1(TTF), 생략0을 확인했다. 이는 mixed-family 공급 구분의 근거이며
paint byte/glyph 관측이나 PNG 수용의 대체가 아니다. 사용되지 않는 가용 resource의 존재만으로
출력 회귀를 추정하거나 추가21개 조건을 자동 제외하지 않았다. C171 scope는 보존한다.

관련 PUA 원문 복학원서의 C178 진단은 전1쪽 base→후보가 Native76.95570→78.08905%,
fresh WASM77.88951→78.12695%다. 기존 #938의 layer/snapshot 미달 범위와 구분하며,
새 하락은 없지만 여전히90% 미만이고 확대 판독도 남아 있다. 이 문서를 신규 완료 범위나
새 기대값으로 수용하지 않는다. 정상 구조·원문·PDF·기존 등록 범위를 보존한다.

C171의 후보 Native는 이 기록 시점50조건까지 명령/쪽수 정합·미달0이며 나머지 거대 표와
fresh WASM 전쪽, 정확한 base 대비0.01%p 하락 대조·직접 판독이 남아 있다.
C181/C182 전체 회귀·C175 head·C176 실제 출력도 아직 완료가 아니다. PR은 생성하지 않았다.


### C175 완료·C176 실제 관측 및 C183 프로필 원인 대조

정확한6ed base8종/후보8종 실제 SDK·Chrome 검사는 합계16건 모두 통과했다.
실제 초기화 응답의 JS/WASM hash를 각각 고정 package와 확인했고, head 소유 Vite도
정상 종료 receipt를 남겼다. 이전 종료 정리 오류는 제품 실패로 세지 않으며 실패 기록을 보존했다.

C176은 Native38·Studio Canvas2D42·지원 CanvasKit8의 총88관측과 독립 Print 대조를
완료했다. 모든 인쇄 프로필 관측 raw 최저는92.68081%이고, 화면 프로필의 생물독소143쪽
2건은 Native86.88424/Canvas2D89.02970%다. 88건의 명령/비교 완료가 각 문서 전쪽이나
모든88장의 직접 판독 완료를 뜻하지 않는다. 초기화 응답 hash·실제 backend 선택·fallback
없음은 browser report와 initial-response-provenance에 연결했다. 일반 문서의 CK 미준비
경로를 지원 CK8건에 포함하지 않으며 guard를 완화하지 않았다.

C183에서 해당 회색 선을 편집 전용 안내선이라고 추측한 초기 가설은 철회했다.
동일 frozen WASM의143쪽 read-only layer query는 screen/print root가 byte-equivalent한
내용이며 text261·선48·배경1, 총310 paint op다. 원본 회색(#bfbfbf)0.48px 선40개가 있고,
별도 안내선이 추가된 결과가 아니다. 실제 PNG의 독립 Print 경계 x700/y257·273·336·399에서
screen 최소 RGB는 Native236–240/Canvas2D234–239, print는각225/223이고 독립 PDF224에
근접한다. 공통 raster_stroke는 Print/HighQuality의 가는 직교 실선을 장치 픽셀 안에서
적분하며 screen 원좌표는 유지한다. Native line_replay와 WebCanvas render_line이 같은
helper·실제 transform을 소비한다. Print 종이 크기와 screen 논리 크기도 별도 계약이다.
실제 인쇄 raw99.97632/99.72861%와 review/overlay를 직접 열었고 내용·표·후속 본문 보존을
확인했다. 화면 원시 미달은 남겨두며 임의 마스킹·색상 보정 점수 승격·제품 수정은 하지 않았다.
전쪽 판정은 canonical Print C171로 별도 수행한다.

sample11의 실제 Canvas2D p2/50와 Native p50, sample16 Native p6/Canvas2D p33,
biotoxin Native p143 screen/print의 review·overlay7쌍을 직접 확인했다. p2의5.2 줄 소속·
들여쓰기, p50 연결 문자/세로 끝선·쪽번호, p6/33 원형 글머리/셀 내용은 보존된다.
sample11 수동 Print에서만 보이는 회색·갈색 면(C124/C129), 대체 자형·가는 선 coverage와
일부 수식 폭·분수 원점의 미세 잔여를 완전 일치로 보고하지 않는다. 실제 Studio screenshot은
제품의 페이지 여백 꺾쇠까지 포함하며 Native 인쇄에는 해당 UI 표시가 없다.
선택7쌍과 모든88관측의 직접 판독을 혼동하지 않는다. 정본은 ignored C176의
`direct-review-part1.json`과 C183의 `assessment.json`/`actual-line-pixels.json`이다.

C178 복학원서의 base/후보 Native review·overlay도 직접 판독했다. 양쪽 모두 제목·표·
후속 본문의 기존 원점 차이가 있어 신규 정상 대조군으로 수용하지 않는다. 수치 개선만으로
기존 #938 구조 미달을 해결했다고 하지 않으며 원본·PDF·기존 등록 범위를 유지한다.

### C171 Native 전쪽 완료와 C184 최신 base 대조 집계

현재 후보 Native52조건 전1,964쪽은 쪽수 일치·명령 실패0·raw90% 미만0이며 최저90.08879%다.
기존43조건/추가 qualification과 현재52조건을 혼동하지 않는다. fresh WASM 전쪽과
정확한6ed base 양쪽의 전쪽 비교는 계속 진행한다. C184는 동일 source/PDF·실제 font path와
TSV hash를 대조해 페이지별0.01%p 이상 하락을 따로 기록한다. base 쪽수가 다르면 같은
쪽 번호의 숫자를 내용 대응이 검증된 전쪽 결과로 취급하지 않는다. 최초 동시 완료9조건에
수치 하락0건이나 전체 대조 완료가 아니다. producer 실제 프로세스를 확인하며 결과가
추가될 때만 집계하고 capture나 Cargo를 중복 시작하지 않는다.

C181 release build564.899초·release lib521.255초는 exit0이다. 전체 nextest와 이후
Skia/lint/정책 및 C182 native feature integration9종은 아직 진행/대기 중이다.
이 단계에서 PR을 생성하거나 전체 검증 완료로 보고하지 않는다.


### C185 개인정보22쪽 하락 재검토와 C186 기존 PUA helper 검사 검토

C184에서 개인정보 명시환경 Native22쪽98.58528→98.56496%,0.02032%p 하락을 검출했다.
원본/PDF/실제 글꼴 경로/Print/192DPI가 같은 TSV다. base/head review·overlay를 직접 열었고
같은 원문·순서·소속을 확인했다. 원시 미해결 픽셀은3473→3462로11개 감소했지만 내용 union
분모가245490→241248로 줄었다. 평균 개선·98% 유지 또는 이 분해만으로 하락 기준을 면제하지
않으며 현재 `re_review_required`다. 별도 mask나 허용치 변경은 하지 않았다.

동일 source의698자 read-only font trace는 생략0이고 primary metric entry/base advance가
같다. 공백의 extraWordSpacing 일부가2.706667→2.817778CSSpx이며 실제 run 원점/폭25항목의
작은 차이가 있다. 이 진단의 paint 상태는 unsupported여서 Native face 관측으로 주장하지 않는다.
같은 독립 PDF의 글자 원점·실선 기하도 추출했다. 대부분의 미해결 픽셀은 기존 오른쪽 밑줄의
하단 AA 행이다. x1000에서 PDF실선1581행과 Native1582–1584행이 있고 base/head가 같다.
Native lower AA191인1584행은 PDF255다. 기존 공통 underline 기하와 실제 소비를 더 확인하며,
문서/쪽별 좌표 이동이나 upstream 수치로의 조정은 하지 않았다. 정본은 ignored C185의
`assessment.json`, `raw-contributions.json`, `underline-probe.json`, 원본 두 review/overlay다.

C181 전체 회귀는 종료 전 현재 #937의F081C helper1건이 실패했다. 원문 fixture를 읽는
서명란 검사와 달리 이 함수는 literal만 helper에 넘겨 실제 paint도 빈 문자열이어야 한다고
고정했다. 독립 #7017 Print의60개 점선과 현재 C171 전1쪽 Native97.96961/fresh WASM98.00636%
review·overlay를 직접 다시 판독했다. 이를 근거로 TAC 흐름의 빈 표시와 실제 paint의 점선
표시를 같은 함수 안에서 각각 검사하는 C186 후보를 ignored 경로에 준비했다. 복학원서의
기존 위치/그림 검사를 삭제하거나 미달 원본을 신규 정상 시각 근거로 수용하지 않는다.
실행 중 C181의 source guard를 보존하기 위해 그 실행이 명시적 결과로 끝난 뒤에만 해당
helper 검사1개를 보정하고 --prepare→집중→전체 nextest→남은 필수 gate/native-feature9종을
순차 실행한다. 변경 없는 release build/lib의 재사용은 생산 source 동일성으로 따로 기록한다.
이 단계는 새 제품 구현이나 새로운 original-before FAIL 검출 증거가 아니라 기존 기대값 검토다.

C187 제출 파일 예비 감사는 사용자 초안2개를 포함한 총162개 경로의135,311,168byte이며50MiB 이상 신규 PDF0,
추적/미추적 .log0이다. 사용자 두 초안은 최초 SHA와 같고 제외 대상으로 유지한다.
이 예비 감사는 아직 index 추가·commit·remote 게시 또는 최종 입력 commit 확인이 아니다.


C181 전체 nextest는 명시적 summary576.611초·10,930실행·10,929PASS·1FAIL·45skip,
exit100으로 끝났다. 실패는 위 #937 literal helper1건뿐이며 정상 나머지 검사는 유지했다.
C186의 테스트 보정 후 정식 wrapper가 현재 manifest의 suite003을 선택해6건 모두PASS다.
source case명과 총1,574개 case 소속은 유지했고, 파일 byte 기반 static weight 배정 때문에
자동 harness27개의 case 배정이 바뀌었다. 이전/새 harness는 canonical deriveManifest와
renderHarness로 모두 재구성해 이전 hash·현재 hash를 확인했으며 case 제거는0이다.
현재 실패 상태를 이용해 원문·fixture·구조 검사·baseline을 제거하지 않았다.

C186 첫 대기 후 시작은 nextest의 실패1건이 본문/최종 요약에 두 번 출력됨을 중복 실패로
세어 중단됐다. 다음 시도는 canonical 자동 배정 변화27개를 source guard가 감지해 Cargo
실행 전에 중단됐다. 두 초기 상태·로그를 보존하고 동일 실패 신원+최종 summary를 확인하며,
이전/현재 harness의 정본 생성·동일 case 집합을 입증한 뒤 실제 재검증을 시작했다.
이는 제품 실패 재현 또는 집중 통과로 세지 않는다. 보정 집중6건 통과 뒤 전체 nextest를
다시 실행 중이며 이후 Skia/lint/native feature9종도 같은 C186 순차 driver에 연결했다.
변경 없는 생산 source를 확인해 release build/lib만 재사용하고 실패 전체 회귀는 재사용하지 않는다.

C188은 rowbreak18 Native9쪽99.69650→99.67453%,0.02197%p의 두 번째 하락을 확인했다.
review·overlay를 직접 열어 셀/제16조/본문 줄 소속·순서 보존과 가는 표 선·일부 glyph AA 차이를
확인했다. 원시 미해결84→86, union27677→26423이며 높은 점수로 면제하지 않는다.
C185/C188 두 하락은 아직 보정·최종 재검증이 필요하다. 제품 source는 이 진단에서 바꾸지 않았다.


### C189–C194 독립 밑줄 프로브·소유 대조와 추가 하락 진단

C186의 새 전체 nextest는10,930실행·10,930PASS·0FAIL·45skip, 명시적 summary582.172초,
명령 총797.353초·exit0이다. Native Skia lib/workspace 동반 검사는3,861+15+165+2=4,043PASS·
0FAIL·12ignore이며 PNG/PDF 계약 gate도 통과했다. 남은 lint/정책/native feature gate와 전쪽
시각 대조는 계속 실행 중이며 전체 완료로 보고하지 않는다.

C189는 기존 #5730 원본6개 크기(9/12/15/18/24/36pt)와 font face만 HYSin으로 바꾼
독립 한컴2020 Print를 각각 실행했다. 두 PDF에서 밑줄 baseline 오프셋은0.159989~0.169943em으로
같다. 따라서 개인정보 문서의 차이만 보고 공통0.17em을0.12em이나 SFNT post 값으로 바꾸는
가설은 채택하지 않았다. 글꼴이 같은지를 파일명만으로 추정하지 않았고 별도 원본 보존·
입력 변경 ZIP entry·실제 Print job/build/method0·PDF SHA·벡터 기하를 ignored C189에 남겼다.
이 프로브는 본문 글자가 Latin인 진단이며 Hangul 동일 조건 프로브도 별도로 요청했다.

C192는 개인정보 원본 HWPX를 다시 한컴2020 Print해66쪽임을 확인했다. 원래 기준 PDF는
교체하지 않았다. 같은22쪽 실선의 x/y/width는 원래 PDF와 재인쇄에서 동일하다. 추출된 text
baseline은591.479980→590.834290pt로 달라 실제 밑줄과의 비율만0.116167→0.162289em으로
변한다. 임베드 H2mjsM subset 프로그램90,176bytes SHA484a84b806d0628ce9351fda6272666e44636a3fbb3b7b5e3575310cd311625a도
두 PDF에서 동일하다. 이 결과를 기준 PDF 교체나 하락 면제에 쓰지 않고, 글자 baseline/밑줄
소비와 실제 source line 계약을 구별하는 진단으로 보존한다.

C190은 현재 Native/fresh WASM 완료 조건의 전쪽 tree에서 좌표를 제외한 내용·순서·소유를
대조한다. 현재11조건624쪽 일치이며 전체52조건의 완료는 아니다. 64/32bit usize::MAX/MAX-1
머리말/꼬리말 합성 문단 ID만 layout.rs 실제 생산 근거로 정규화했고 실제 원문 PI는 바꾸지
않았다. 원래 정규화 전 차이도 보존한다. 이 진단은 독립 PDF 내용 보존이나 실제 paint의
대체가 아니다.

C191은 biotoxin205 Native10쪽99.98107→99.96997%,0.01110%p 하락을 직접 review·overlay로
확인했다. 같은 page layer root와1,023자 font decision trace의 base/head 차이는0이다.
원시 미해결12→17, union63,386→56,613이며 추가8/해소3픽셀은 오른쪽 한국어 표 글자 근방이다.
내용·셀 경계는 보존되나 실제 native paint 차이의 원인을 더 확인하며 고득점으로 면제하지 않는다.
C194는 새로 완료된 base 대조에서 civil2 Native1쪽98.61594→98.46085%,0.15509%p 하락을
검출해 동일 PNG review·overlay와 기하 진단을 준비했다. 현재 재검토4쪽이며 소스 보정 및
새 source 전쪽 재검증 전에는 PR을 생성하지 않는다. 이 절 추가 시 제품 source는 바꾸지 않았다.

C189 Hangul 대조군도 같은 한컴2020/build9136/Print method0로 성공했다. 기존 Latin6문자
묶음만 한국어로 바꾼 입력이며 독립 PDF6크기의 baseline 대비 오프셋은 Latin과 동일한
0.159989~0.169943em이다. Hangul/Latin만으로 밑줄 오프셋을 다르게 하는 가설도 채택하지
않는다. C192의 subset SFNT는 post/OS2가 빠져 있어 첫 메타데이터 진단은 KeyError로
중단됐으며 원본 스크립트를 보존하고 누락을 명시한 재실행 결과만 진단에 사용했다.

C194 civil2의 실제 review·overlay도 직접 판독했다. 원시 미해결386→380으로6개 줄었으나
union27,889→24,689로 줄어 raw 비율이 하락한다. 추가59/해소65픽셀은 등기 목적과 왼쪽
셀 문구에 집중된다. 표/내용 순서는 유지되지만 글자 간격과 경계 확대 판독을 계속하며
분모 변화로 면제하지 않는다. C193은 같은 공급 디렉터리를 실제 Skia observer에 준비해
네 쪽의 paint face와 직렬화에서 생략된 내부 style/advance를 관측하도록 준비했다.
C186 Cargo가 종료되기 전에는 별도 Rust 빌드를 시작하지 않고 실제 프로세스를 확인해
대기한다. 이 observer는 새 시각 통과나 테스트 정상 기대값을 만드는 절차가 아니다.

C196의 actual CanvasKit Print 직접 판독에서 줄 정보 없는 셀 대조군1쪽의 탭 원점에
작은 원6개가 보인다. 독립 Print에는 없고, 같은 core layer의 원문 TAB도5+1=6개다.
paragraph/control 표시 옵션은 false이며 core layer에 ellipse/emphasis 원형 primitive가 없다.
현재 CanvasKit positioned legacy replay가 TAB의 nonzero font gid를 drawCachedGlyphs로
그리는 소비 경로를 확인했다. Native의 whitespace ink 생략과 달리 source advance를
그대로 받으면서 제어 glyph를 칠하는 문제다.

실제 현재 renderer prototype에 nonzero TAB gid를 제공한 진단 계약2건(탭만/탭 뒤 글자)은
실행돼2FAIL이며 탭 gid9가 각각 원점0/40,10에서 실제 draw 호출에 들어갔다. 검사의 뒤 글자
원점40은 보존 대상으로 함께 확인한다. ignored C196의 제안 source를 Vite transform으로만
실행하면2PASS다. 아직 작업장 제품 파일에는 적용하지 않았으므로 현재 브랜치 PASS라고
보고하지 않는다. 양수 전진·원문 인덱스·그 뒤 글자·배경/장식 길이를 보존하고 비표시 잉크만
생략하는 일반 경로를 보정하며, 좌표나 쪽별 분기는 추가하지 않는다. C186/전쪽의 source
guard와 현재 실행을 보존한 뒤 실제 적용·정식 테스트·actual 재캡처가 필요하다.

같은 원본의 focused fresh WASM1쪽 선행 검증도 global lock 아래 요청했다. 첫 진단 driver는
필수 svg_rasterizer 인자를 빠뜨려 캡처 시작 전에 TypeError로 종료됐다. 실패 원문/상태/로그를
보존하고 canonical webfont 인자를 보완해 같은 고정 producer로 다시 실행했다. 이 오류를
제품 실패나 시각 실행 건수로 세지 않는다.

C186 최종 상태는 필수26gate 전부exit0이다. Native feature9종도 실제 실행됐다. 이는 현재
Rust/기존 frontend source에 대한 결과이며 이후 C196 TypeScript 보정의 실제 browser
재캡처를 대신하지 않는다. C193 첫 observer 컴파일은 다른 serde_core 빌드 인스턴스의
Deserialize를 직접 요구해 실패했다. 외부 crate 타입의 공개 FontEnvironment::from_json
계약을 사용하도록 진단만 고쳐4case 관측에 성공했다. 제품 API/trait은 바꾸지 않았다.

C193 실제 native snapshot은 biotoxin의 HCR Batang, 개인정보의 HYSinMyeongJo-Medium,
민원서식의 GulimChe/Gulim, rowbreak의 HYGothic-Medium을 system에서 선택하며 trace 생략0이다.
C195의 동일 HCR 프로그램·실제 두 한국어 run isolated default 재생은 현재 Native의
해당 영역과 RGB가 정확히 같았다. 기준선 snap을 끄면 첫 영역 원시 미해결9는 같고 두 번째
8→10으로 늘었다. None hinting도 전체17→19, Slight/Full은17로 동일했다. 이 실험을
전쪽 수용이나 임의 좌표 보정에 사용하지 않고 일괄 snap/hint 변경 가설은 채택하지 않았다.

C191의 독립 PDF subset에는 cmap가 없어 첫 getBestCmap 진단이 KeyError였다. 글꼴 부재로
해석하지 않고 PDF texttrace의 Unicode/GID 대응으로8개 한국어 glyph를 대조했다. PDF/local
HCR revision1.9099884·UPEM1000·advance0.97em과8개 분해 윤곽 해시가 전부 같다. 글꼴
파일명이나 점수만으로 대체 글꼴을 고르지 않았다. PDF text matrix는 약0.119869/0.119935pt
인쇄 장치 축척과 정수 Tf·TJ를 쓰고 두 문제 run의 첫 원점/전진에 차이가 있다. 현재는
원인 진단이며 이를 문서별 상수/지시문/커밋된 private metric으로 바꾸지 않았다.

C196 expanded 현재소비자 계약6건은5FAIL/정상대조1PASS이고 Vite 격리 보정안은6PASS다.
확정 positions·0전진 TAB·legacy nominal 경로와 다음 글자 원점·원문 보존을 검사했다.
새 source-position 테스트 초안은 일반 legacy 정상 대조를 추가해7건이며 실제 작업장 적용
뒤 실행한다. 비표시 공백 잉크의 일반 규칙만 보정하고 이미 별도 preflight가 보류하는
FFFC를 새로 허용하지 않는다. 원문 실제 px baseline을 고정하는 검사가 아니다.

현재 같은 원본1쪽 Native raw96.72201/fresh WASM95.35391%를 산출하고 두 review·overlay를
직접 판독했다. 탭 원형 잉크6개는 core 두 backend에는 없고 결재/본문/표/붙임의 소속은 유지된다.
두 backend98미만이므로 실물 절대px핀은 추가하지 않는다. C196 적용 driver는 실행 중인
C171 head전104조건 종료와26gate 완료를 실제 ps/상태로 확인한 뒤에만 정확한 원본 파일
SHA를 대조해 TS파일+정식7case를 적용하고 집중→전체 Studio→build→diff를 실행하도록
준비했다. 원래 두 사용자 초안과 Rust/Native/WASM producer/원본/PDF는 보존한다.

C176 실제 소비 경로의 추가5쌍 review·overlay도 직접 판독했고 ignored direct-review-part2.json에
파일별 SHA와 잔여를 기록했다. 지원 CanvasKit 두 대조군의 실제 replay만 포함하며 개인정보/
고용노동부의 미지원 경로를 지원 통과로 바꾸지 않았다. 추가 판독은 전체88건 직접 판독이라는
주장이 아니며 C196 탭 결함과4쪽 하락은 PR 보류 사유로 유지한다.


### C197–C201: 전쪽 완료 뒤 실제 소비자 보정과 구두점 소유 대조

C171 고정 후보는 Native/fresh WASM 각각52조건·1,964쪽을 모두 산출했다.
raw 최저90.08879/90.09923%,90% 미달0·실행 실패0이다. 이는 C170 Rust producer의 결과이며
이후 Rust 보정의 전쪽 통과로 사용하지 않는다. 같은 최신6ed 기준 base의 전쪽 비교는 계속
실행 중이고 현재 Native4쪽 하락은 보류한다.

C196은 head 전쪽 프로세스 종료·원본 source SHA·26gate 완료를 확인한 뒤 실제 제품 TS와
정식7case를 적용했다. 집중7PASS, 전체 Studio2,346PASS/0FAIL/2skip(24.119초), build와
`git diff --check` 모두exit0이다. source TAB의 positions/전진/원문/장식은 보존하고 비표시
whitespace 잉크만 생략한다. 실제 CanvasKit8·Canvas2D4 재캡처 C201은 같은 공유 lock 아래
실제 초기화 JS/WASM 응답과 backend/surface를 관측하며 진행한다. 새 TS 이후 actual 검증은
완료 JSON·PNG 직접 판독을 확인하기 전까지 미검증이다.

C197 독립 Print6크기(8/10/11/13/14/17pt) HCR/HY신명조의 전진은 단순한600DPI em 내림과
일치하지 않는다. 일반 인쇄 축척을 글꼴 전진에 일괄 적용하는 가설은 채택하지 않는다.
C198은 정확한 생물독소205쪽 원본을 engine2020/build9136/Print method0로 재인쇄했고,
문제10쪽의 한글·slash text origin/advance가 기존 독립 PDF와 같았다. 과거 PDF 오류로
면제하거나 기준을 교체하지 않는다. C192 개인정보 원본의 한컴 HWP 재저장 후 HWPX는
문제 문장의 텍스트·원본5개 LineSeg(textpos/vertpos/baseline1190/높이1400 등)를 그대로
보존한다. fresh Print의 glyph baseline 차이와 저장 LS를 혼동하지 않고 원인을 더 확인한다.

C200의 별도 한컴 Print 대조군3개는 HCR·HY신명조·바탕 각각에 Latin Times New Roman을
직접 선언했다. 여섯 크기 모두 한글 사이 `/ , ( ) - .`의 PDF font가 TimesNewRomanPSMT다.
실물 생물독소 두 셀의 원본 Latin face와 독립 PDF도 같은 소유이나 현재 legacy-only
ascii_punct_latin_slot은 직접 선언한 Times를 제외한다. 측정 char_lang_slot과 저장 줄
compose_paragraph_scoped의 paint 분할은 같은 flag를 소비한다. 치환 출처로 동작을
막는 가정을 제거하되 동일 face·CJK glyph 보유 Latin 대조군은 기존 소유를 보존하도록
검토하며, 문서 ID/수치 예외는 추가하지 않는다. 원본 두 셀의 본문 순서·문자 소속·slash의
측정/paint 소유를 검사하는 정식 관계 검사의 수정 전 실행을 시작했다.

C199 종료된 batch에서만 동일 font_resource를 SHA 검증 후 hardlink로 묶었다. 고유39개,
중복5,616경로·41,579,429,184논리 바이트를 정리했고 실제 free67,868,545,024→106,590,109,696
바이트다. 모든 경로/바이트 보존을 확인했으며 원본 글꼴 디렉터리·PDF·PNG·TSV·로그·공유
Cargo target과 실행 중인 batch를 삭제하거나 변경하지 않았다. 결과 정본은 ignored C199다.


C190은 frozen 후보 Native/fresh WASM52조건·1,964쪽의 내보낸 source 소속·노드 순서·본문을
전부 비교해 차이0을 확인했다. bbox는 비교 대상이 아니고, 원문 paragraph id는 유지하며
실제로 추적한 FootnoteArea/MasterPage의64/32bit usizeMAX sentinel만 같은 source offset으로
정규화했다. 이 보조 계약은 독립 PDF의 내용·그림 판독이나 시각 수용을 대신하지 않는다.

C201 actual12건은 전부 명시 backend·지원 software surface를 사용했고 초기화 응답 hash
검사와 independent Print 비교를 완료했다. Print6건 raw 최저99.50319%,미달0이다.
탭 원형6개의 문제 쪽 CanvasKit compat Print99.61291→99.85421%와 정상 low-slack
CanvasKit default Print99.89562%의 review·overlay를 직접 판독했다. 원형 잉크가 없어지고
결재/본문 표·붙임·원문 소속은 유지된다. 남은 얇은 선·글자 AA 차이는 기록한다. C196 TS
수정의 actual 증거이며 이어지는 C200 Rust 변경 결과로 재인용하지 않는다.


C200의 정식 관계 검사는 수정 전 원문 두 셀의 slash가 한글 run에 포함되는 실패1건을
실제로 검출했다. 직접 선언한 Latin 전용 face도 같은 ascii_punct_latin_slot을 측정/저장
paint가 소비하도록 legacy-only 가정을 제거했다. 기존 legacy 명시 환경만의 슬롯 보존
조건은 유지해 일반 CJK 환경 치환을 새로 영문 소유로 강제하지 않았다. 기본 환경1PASS,
최종 Native feature 생물독소9PASS, fmt와Native build 및 fresh locked web/no-opt build가
통과했다. 새Root pkg·Studio/public·frozen pkg 네 파일의 hash가 모두 같다.

C200 focused Native/fresh WASM 각6조건8쪽은 raw90% 미달0이다. 생물독소10쪽은 양쪽
100.00000%로 개선됐고 두 review/overlay를 직접 판독했다. 두 셀의 '/'와 뒤 한글 위치가
복원되고 표의 종/설명/주기·쪽번호 소속은 유지된다. 2px 관용100이며 실제 pixel 완전
일치는 아니다. 정상 대조군 PNG byte 비교와 선택 페이지 TSV를 보존했다. 이는205쪽
전쪽이나최종52조건 전체 재검증이 아니며 다른3쪽 Native 하락은 계속 보류다.

C194 추가 source 대조에서 원본 민원서식의 U+F09E는 일반 Unicode·가 아니었다.
독립 Print는 Wingdings-Regular CID3 /W312로 출력하지만 현재 측정은 표시 대체
굴림체·의1000/1000em을 사용해 뒤 글자와 가운데 정렬 원점을 밀었다. 별도 C202
한컴2020/build9136 Print는 한 문장에 U+F09E/U+F0A0와 실제 Unicode·를 넣었고,
일반·굵게·기울임 각각6크기에서 Wingdings 두 glyph /W312와 literal GulimChe /W1000을
확인했다. 원본·PDF를 바꾸거나 private 글꼴 binary/hmtx를 저장소에 추가하지 않았다.

기존 canonical PUA 투영의 원문 프로그램 전진만 별도로 선택하고 literal·와 확인되지
않은 기호의 기존 계약은 유지했다. 같은 char_width_decision이 줄/상자 폭과 최종 replay
positions를 생산한다. 원문/표 셀/표시 문자열은 유지한다. 새 관계/프로그램2검사는 수정 전
5PASS/2FAIL(원문기호1em), 수정 후7PASS(0.038초)이며 기존5검사도 그대로 통과했다.
실물 민원서식은 직전 전2쪽 Native/fresh WASM 최저98.46085/98.45182%로 신규 관계 검사
선행 조건을 충족했다. 현재 C202새source의 Native·fresh WASM build와 민원서식 전2쪽 및
기존 집중 대조6조건 재출력을 시작했고 완료·직접 판독 전에는 새source 시각 통과로 쓰지 않는다.

C194 SDK 진단 첫 시도는 선택 인자에 null이 아닌 문자열 "null"을 넘겨 환경 parse 오류로
종료했다. 원본 script/log를 보존하고 JS null로 수정해 두 frozen producer를 관측했다.
C202 source backup은 보존 stash30a8의 font_metrics_overlays가 C200 source hash와 같음을
확인하고 작업장을 바꾸지 않은 채 복사했다. 측정 파일도 이번 추가만 역산한 snapshot이
C200 hash와 정확히 같음을 확인했다. 첫 checksum 진단에서는 별도 rustfmt가 기존if의
줄바꿈 한 곳을 바꾼 차이가 있어 실패했고, 이를 code 손실이나 제품 실패로 세지 않았다.
두 사용자 초안 SHA와 backup refs/stashes는 유지한다.

## C207–C213: 완료 페이지 px 보호와 잔여 하락 원인 분리

- C202의 Native/fresh WASM 집중6조건9쪽씩, C204의4조건9쪽씩 재출력이 모두 종료됐다. 쪽수 불일치·선택 쪽 raw90% 미만은0이며, C212에서 같은 PNG로 선택 쪽 TSV를 다시 산출했다. 민원서식은 선택 쪽이 아닌 **문서 전2쪽**이고 양쪽 최저99.98717%, 1쪽은 양쪽100%다. Native/fresh WASM 두 쪽의 review/overlay를 모두 직접 열어 세 점 기호 뒤 원점·셀·표·내용 순서를 확인했다. 잔여 획/래스터 차이는 남으며2px 관용100을 픽셀 동일이라고 쓰지 않는다.
- C207은 원본 독립 Print의 세 '퇴' 원점을 pt에서96CSSpx로 환산해±2px로 보호한다. 기존 원문/셀/누락·중복·전진 관계 검사는 유지했다. 같은 새 정식 검사 원문을 C186의 보존 Native library와 C202 library에 각각 연결한 실제 실행은 **FAIL(309.12 vs300.79698CSSpx)→PASS**다. C186을 정확한 C200 빌드라고 쓰지 않으며, 그 사이 민원서식 출력 불변 근거도 구분한다. `--prepare` 후 정식 Native suite는 **8PASS/0FAIL(0.064초)**, run37b40505-6052-4e81-b91a-cb371a5aeeeb다. 증거는 ignored `civil-pixel-contract-c207/`이다. 제품 Rust 변경 없이 검사만 추가했으므로 C202 생산자 출력은 명시적인 test-only 근거 아래 보존한다.
- C202 frozen pkg·루트pkg·Studio/public 네 파일×세 위치12개 hash가 일치한다. 이는 파일 공급 확인이며 새 코드의 실제 브라우저 초기화 응답·actual36/88 최종 통과가 아니다.
- C171 고정6ed 대조군도104조건 전체 종료했다. 이전 C170 후보 비교에는 raw0.01%p 이상 하락85관측이 있으며 Native4/WASM81이다. 그중 생물독소10쪽·민원서식1쪽 두 backend는 이후 집중 보정으로 개선됐지만,85개를 최신 C202의 최종 잔여 개수로 쓰지 않는다. 특히 원본#4138 WASM60관측 등은 새 source 전쪽과 비교가 더 필요하다. 대조군 자체의 미달·쪽수 불일치도 원장에 보존하며 후보 전쪽 통과와 혼동하지 않는다.
- C209은 #4138 원본115쪽 중5/47/83쪽만 최신 C202로 재출력했다. Native/fresh WASM 선택3쪽 최저96.66313%, 문서 쪽수115/115다. 전115쪽 최신 후보 통과나 모든 하락 해결이 아니다. 첫 시도는 C207 검사 파일 hash 변화에 guard가 **캡처 전** 중지했다. 기존 실패 상태와 스크립트를 보존한 뒤 실제 drift가 해당 검사1개뿐·production0임을 확인했고 그 정확한 test hash를 별도 고정해 재실행했다. guard를 일반적으로 끄지 않았다.

### 정렬 폭과 글리프 paint를 구별한 독립 진단

C208은 기존 Print의 실제 글자 원점을 원문 순서로 대조했다. 개인정보22쪽·행분리10쪽은 새 공백 분배가 일부 원점을 더 가깝게 했고, 생물독소123쪽의 뒤 글자는 최대 약0.635CSSpx 오른쪽이다. 따라서 점수만 보고 공백을 정수 폭으로 되돌리는 보정은 채택하지 않았다. 원문·줄·PDF를 변경하지 않았다. C203의 실제 source alias/subst-only Print는 직접 HY신명조 대조와 같은 원점이었고, C205 Skia 기본/linearMetrics도 선택 전진이 같아 해당 두 가설은 채택하지 않았다.

C210은 자산관리56쪽 base/current 실제 WASM PNG의 차이3726픽셀이 굵은 제목9글자에 집중함을 확인했다. 이것은 새로운 차이 위치이지, raw 미일치 전체 위치가 아니다. Canonical 마스크와 같은 임계232·2px 팽창을 별도 읽기 진단으로 재현한 raw 잔여는 Native541/WASM545픽셀이며, 제목 밖의 붙임 번호와 네 개의 수평 화살표 등에도 남는다. 원본의 Unicode➜는 HFT 한양신명조 슬롯이고 Native actual font trace는 AppleMyungjo system fallback을 선택했다. 독립 PDF는 해당 원문 기호를 Type3 T6/HFT364로 그린다. 즉 표시 대체 glyph의 차이를 별도로 보정해야 하며 새 글꼴 공급/공통 구현 검증 전 해결로 쓰지 않는다. 해당 현재 review/overlay도 직접 열었다.

C211은 다른 문자와 글꼴을 섞은 독립 합성 Print 두 건(20pt3face×일반/굵게×6장평36개, 8/10/11/13/14/17pt×같은3face×일반/굵게×97/103%72개)을 실제 engine2020/build9136 Print method0/no-preprocess/1800초 한도로 생산했다. 두 PDF는2/3쪽이며 SHA b72db28e…/faa22fb9…다. 한컴의 실제 Tz는 명목 장평과 다르며 평균 폭의 장치 정수화를 포함한다. 첫 단순 평균 폭 모델은72개 중28개가 안 맞았고, 요청 평균 폭에는 반올림 전 nominal 크기를 쓰며 실현 평균 폭에는 반올림한 device 크기를 쓰는 모델이72개 모두를 설명했다. 이는 paint 진단이며 조판의1800DPI 전진 규칙을600DPI로 바꾸는 근거가 아니다. 원본 glyph 위치와 줄 구성은 유지해야 한다. 최초 추출 표에서 실제 Fitz size를 nominal sizePt로 덮어쓴 진단 버그는 원본 표를 보존한 뒤 `actualFitzSizePt`로 다시 추출해 정정했다. 원본 PDF와 Tf/Tz/CTM 증거는 바뀌지 않았다.

C213은 위 모델을 **운영 코드에 넣지 않고** 복제한 Native layer의 해당 TTF glyph paint 비율만 바꿔 반례를 확인했다. 원래 위치/상자를 보존했지만 진단 current PNG가 C204 CLI PNG와 byte-identical하지 않으므로 동일 생산자 전체 통과로 쓰지 않는다. 진단 짝의 raw98.68010→98.66768은 오히려 하락했다. 미일치541픽셀은 그대로이고 내용 합집합40988→40606만 줄었다. 따라서 이 비율 변경만으로 회귀 해결이라고 할 수 없어 제품에 적용하지 않았다. raw 문턱·검사·내용을 완화하지 않고 실제 남은 번호·화살표 glyph 차이를 추적한다. 폰트 프로그램 비교는 ignored output의 진단이며 private font binary·윤곽/메트릭 배열은 제출하지 않는다.

폰트 trace 첫 진단의 maxCharacters8192는 기존 API 상한4096에서 명시적으로 거절됐다. 원본 실패 로그/소스를 남기고4096으로 고쳐 실제 trace를 받았다. 제한을 우회하거나 제품 실패로 세지 않았다. 두 사용자 초안 SHA와 백업 ref/stash는 유지한다. 이 단계의 새 전체 nextest·최신 source 전52조건·실제 소비 경로 및 PR/CI/머지/후속처리는 아직 완료하지 않았다.

### C214: 동일한 독립 HFT 화살표의 선택 공급

화살표는 Unicode➜ 원문이지만 한양신명조 HFT 기호 슬롯이었다. 대상 PDF를 font source로
사용하지 않고, 기존 준비 도구에 `--with-arrows` 선택 옵션을 추가했다. 기본 전체 한글/
글머리 모드는 그대로이며, 전체11172자+기존 글머리5+표준 화살표5의 **11182자 전체 순서·
Type3 글리프 집합**을 먼저 확인해야만 부분 기호 face를 만들 수 있다. 기존 완전 표본·
행렬·빈 윤곽·변환 거절 guard를 유지했다. 요청 화살표 하나가 빠진 완전 한글도 거절하는
새 검사와 기본 모드 보존 검사를 포함해 도구 **13PASS/0skip**이다. 글꼴 binary는 ignored
output에만 두고 시스템 글꼴·기존 공급 디렉터리·원본/PDF를 덮어쓰지 않았다.

독립 Print job7c502c4a-9ab8-48d1-b5e5-7665882ce1c9는 engine2020/build9136,
Print method0/no-preprocess,7쪽/21초로 완료됐다. PDF SHA6dc39350…에서 전체11182자 확인 후
기호10개만 별도 CFF로 보존했다(폰트 SHA9cf41a22…). 기존5기호의 source glyph SHA는
전부 C111과 같고, 새➜의 source glyph SHA는 폰트 생산 **후** 읽기 대조한 대상 PDF의
HFT364와 같다. 대상 문서를 표본 생성/복원 입력으로 사용하지 않았다. 이동·스케일·획
보정 없이 원본 윤곽/전진을 그대로 공급한다.

양쪽 frozen producer에 같은 새 기호 공급 조건을 적용한56쪽 집중 결과는 다음과 같다.

| 경로 | 고정6ed | C202 Rust 후보 |
| --- | ---: | ---: |
| Native |62.42626%|99.22511%|
| fresh WASM |99.24188%|99.22867%|

base Native의 두부 글리프는 이전 공급에서도62.25806%로 이미 있었고 현재 새 폰트가 만든
새 퇴행이 아니다. 원장과 직접 review를 유지했다. 후보 Native/WASM은90% 이상으로
개선됐지만 WASM paired 하락0.01321%p는 여전히 보류다. 후보WASM review/overlay를 직접
열어 화살표 윤곽·본래 원점·본문/표/순서를 확인했다. 이 새 공급 조건의 전60쪽·전52조건
및 actual 소비 경로를 수행하기 전 일반 공급이나 기존 전쪽의 최종 통과로 확장하지 않는다.

첫 driver는 Native CLI에도 JSON null 환경 파일을 넘겨 API가 캡처 전 거절했다. 실패
산출물과 로그를 별도 보존하고 Native는 환경 인자를 생략, WASM은 명시적 null로 수정한
새 실행4관측만 사용했다. Native/WASM 환경 의미를 구분했으며 guard를 우회하지 않았다.
헤더 `<붙임19>`의 HCI Poppy HFT 원문은 실제 Palatino Linotype 대체 프로그램을 선택하는
것도 trace에서 확인했다. 번호/헤더 glyph 차이는 잔여로 남는다.

### C215–C220: 원본 기호 전진 보정과 HFT 굵기 근거 재확인

C215는 종료된 C171 양쪽 전104조건의 생성 font_resources만 SHA로 대조하여 동일
바이트를 hardlink로 공유했다. 13,650개 경로의 내용과 이름은 보존했고, 중복7,995개에서
논리 중복59,192,937,380B(약55.13GiB)를 줄였다. 여유 공간은42,902,069,248B에서
102,137,905,152B로 늘었다. 원본·PDF·PNG·TSV·로그·실행 중 산출물과 shared target은
삭제하지 않았다. 정본은 ignored `completed-font-dedup-c215/result.json`이다.

C216은 대상 문서의 문자 표본을 쓰지 않고 HCI Poppy HFT 전체 ASCII95자를6크기로
별도 인쇄했다. engine2020/build9136, Print method0/no-preprocess의 독립1쪽 PDF
SHA62a72c1a…에서 일반 `<`, `=`, `>`는 모두605/1000em이다. 표시 Palatino의
`<` 전진 약0.5em은 원본의 전진이 아니다. 공개 Pagella도 별도 윤곽 비교했지만 헤더
표본의 차이가 더 커 선택·설치·배포하지 않았다. 이 진단은 글꼴 라이선스 승인이나
전체 glyph 동일성 판정으로 사용하지 않는다.

C218은 기존 원본 부분 메트릭에 **일반 HCI Poppy의 확인된 세 기호만** 추가했다.
명시 TrueType 선택·미확인 스타일과 원본 저장 바이트는 유지한다. 정식 두 검사는
원본 프로그램 전진과 실제 exported layer의 뒤 한글 원점을 함께 검사한다. 최초
진단에서 저장 layout_positions가 항상 존재한다고 가정한 잘못된 검사는 무효 보존하고,
실제 공개 layer cluster의 원점을 검사하도록 고쳤다. 같은 교정된 검사로 수정 전
19건 중17PASS/2FAIL → 수정 후19PASS/0FAIL을 실제 실행했다. Native biotoxin
대조9건도 모두 통과했다. 기대값을 후보의 계산값으로 낮추거나 기존 검사를 지우지 않았다.

현 C218 소스의 fmt·Native build·집중9검사·fresh WASM locked wrapper는 모두 exit0다.
WASM은 Mac `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화
빌드가 아니다. 새 source ledger는 `poppy-regular-comparison-c218/source.json`이며
현재 Native binary SHAe5f5a75f…다. C219는 이전 보류 쪽과 대조군89쪽/backend를 같은
기존 글꼴 조건으로 재출력 중이다. 현재 자산관리56쪽 Native는98.68010→99.44075%로
개선됐고 review/overlay에서 헤더 기호 뒤 원점·본문·표·번호를 직접 대조했다. 선택 쪽
재검증이며 최신 전52조건/전체 회귀/실제 소비 경로 완료로 확대하지 않는다.

C220은 HFT 원본 한양신명조/명조/고딕의 일반·굵게와6크기를 별도 인쇄했다
(PDF SHA31bc3a84…, engine2020/build9136, Print method0/no-preprocess,2쪽).
처음 texttrace의 invisible 상태와 bitmap 이미지만 보고 굵은 원본 윤곽이 없을 가능성을
추정했지만, **CharProc를 직접 읽어 그 가정을 기각했다.** 한양신명조의 일반/굵게
`가`는 동일 path SHA085a7c8c…이고, 굵게만 `20 w B`를 더한다. FontMatrix.001에서
정확히0.02em fill+stroke다. `restore_outline`의 `Unsupported glyph operator: w`는
파서 지원 경계이며 빈 윤곽 증거가 아니다. 명조/고딕의 실제 빈 path와 구분한다.
따라서 HFT라는 이유로 공통 합성 굵기를 제외하는 수정은 하지 않았다. 기존 준비 도구의
빈 glyph/변환 guard도 완화하지 않았다. 정정 근거는 ignored
`giant-bold-program-c220/corrected-hft-bold-paint-proof.json`이다.

거대 표56쪽의790개 한글 원문은 HFT 한양신명조지만 실제 Native 선택은 시스템
HYSinMyeongJo-Medium TTF다. 원본과 표시 대체 프로그램의 차이를 분리하기 위해 기존
독립 전체11172자+10기호 표본으로 완전 HFT 공급의 제한 진단을 준비한다. 대상 PDF를
font source로 사용하지 않고 시스템 설치·binary 커밋도 하지 않는다. 이 새 글꼴 조건은
양쪽 frozen base/head를 동일하게 검증하기 전 기존 조건 통과로 바꿔 쓰지 않는다.

C213의 PNG container SHA 차이도 추가로 해소했다. 진단 control과 C204 CLI PNG를 같은
RGBA로 디코딩하면 크기와 **전체 픽셀이 동일**하다. 따라서 앞선 byte-identical 아님은
압축/컨테이너 바이트에 한정되며 다른 paint 결과였다는 뜻이 아니다. 이 교정 후에도
장평 counterfactual의 raw98.68010→98.66768 하락과 미채택 결론은 같다. 원본 PNG를
덮어쓰지 않았고 `program-ratio-counterfactual-c213/decoded-cli-control-pixel-identity.json`에
두 SHA 종류와 범위를 나눠 남겼다.

C218 frozen/root pkg/Studio public의 네 파일씩12해시와 기존 Studio7736의 실제
HTTP JS/WASM 응답2해시도 모두 일치한다. HTTP 응답 SHA 확인은 브라우저 초기화·
Canvas2D/CanvasKit의 실제36개 소비 사례 완료와 구별한다. C222는 같은 소스 원장을
고정한 뒤 release build/lib·전체 nextest·Skia3종·lint/manifest/unit tiers·Studio type과
변경 Native feature11조건을 총27단계로 순차 재검증한다. 빌드8·테스트8을 유지하며
중간 실행은 전체 통과로 쓰지 않는다. 정본은 ignored `current-gates-c222/`다.

### C221–C224: 글꼴만의 설명을 기각하고 물리 Print 좌표를 분리

C221의 완전 원본 공급은 C214 독립 표본11182자를 모두 검증해 준비했으며 기존11177개
source glyph SHA가 모두 같다(새 font SHA63aa077f…). 원본 HFT 선언이 확인된 거대 표에만
제한 진단으로 공급했다. **새 글꼴 조건**의 선택3쪽 결과는 다음과 같고 기존 조건을
덮어쓰지 않았다.

| 경로/생산자 |5쪽|56쪽|109쪽|
| --- | ---: | ---: | ---: |
|6ed Native|96.10257|96.68262|99.26509|
|6ed WASM|96.33960|96.77298|99.31834|
|C218 Native|96.31568|96.76061|99.30031|
|C218 fresh WASM|96.31568|96.76060|99.30031|

4조건12쪽이 종료됐지만 WASM 하락은 남아 있어 글꼴만으로 해결됐다고 채택하지 않았다.
후보WASM56쪽 review/overlay를 직접 열어 원문 순서·쪽 경계·줄 소속이 유지되고 잔여
위치 차이가 있음을 확인했다. font 생산 **후** 대상 PDF의 `가/용/허/단` CharProc를 읽어
독립 표본과 모두 같은 SHA·FontMatrix.001임을 확인했다. 이 읽기 대조는 font 생산
입력이 아니며 다른 검토자의 시스템에 원본 HFT가 있다는 주장도 아니다.

C223은 실제 source/배치를 복제한 Native counterfactual에서 굵은 HFT 윤곽만 직접
그리려 했지만, 문제56쪽에는 **bold run0건**이었다. zero replacement assertion을
해제하지 않았고 해당 시도는 실패/미적용으로 남겼다. 공개 최종 layer의48run을 따로
감사해 굵기 가설이 그 쪽의 원인이 아님을 확인했다. diagnostic control PNG는 C221
Native와 RGBA 픽셀 전체가 같았다. 첫 진단의 page 인자 usize/u32 컴파일 오류도
별도 로그 보존 후 u32로 교정했다. 제품 소스·정식 검사·gate는 변경하지 않았다.

C224는 다음의 **독립 Print 참조 프레임**을 새로 확인했다. 실제 PAGE_DEF/HWPX의
종이는 각각59528×84189HU(거대 표),59528×84186HU(밑줄 probe),59528×84188HU(개인정보)다.
세 입력의600DPI 정수 종이는 모두4961×7016이고 PDF MediaBox는595×841point다.
따라서 PDF의 실제 공통 CTM은 `595/4961`, `841/7016`이고 CSS 물리 축척은
각각0.999462…/0.998907…다. 독립 일반/굵기 표본, ASCII 표본, 실제 거대 표와
개인정보 기준 PDF **4건 모두** 이 CTM의6자리 출력과 일치한다.

현재 공통 밑줄 `.17em`을 이 물리 device 격자에 반올림해 투영하면 HCR/HY신명조
6크기씩 **12개 독립 대조군의 밑줄 y 오차가 모두0.0006CSSpx 미만**이다. 개인정보
22쪽도 논리 y791.666666…에서 물리 y790.814899…로 예측되어 독립790.814616…와
0.000284CSSpx 차이다. 기존 약0.852px 차이를 문서별 offset이나 `.17→.12em` 변경
없이 설명한다. 이 결과는 아직 **좌표 원인 진단**이며 전쪽/제품 수용 결과가 아니다.

현재 `paint/paper_viewport.rs`는 정수 point 출력 크기와 바탕쪽 raster만 투영한다.
본문·장식·다른 벡터는 논리 좌표를 유지한다. 이후 공통 Print 참조 프레임을 보정할 때는
화면/인쇄 분리, glyph baseline·글자 원점·그림의 이중 투영 방지·clip·모든 backend의
실제 소비를 함께 확인해야 한다. 폰트 전진을600DPI로 내리거나 실루엣/원본 PDF를
바꾸는 근거가 아니다. 실행 중인 C219/C222 source guard가 끝나기 전에는 제품 소스를
변경하지 않는다. 정본은 ignored `physical-print-frame-c224/results.json`과 `analysis.log`다.

한편 기존 글꼴 조건의 자산관리56쪽 **fresh WASM99.44617%**도 실제 재출력이 끝났다.
고정6ed WASM98.71100 대비+0.73517%p이며 Native99.44075와 함께 C218 기호 전진 보정의
집중 개선을 확인한다. Native base62.25806에는 원래 tofu가 있어 그 차이를 이번 보정
하나의 효과로 쓰지 않는다. 이 결과도 전60쪽·전체 검증 완료와는 구분한다.


## C225–C229: 전체 좌표 투영 반례와 캡션 선택 검사 교정

C219의 현재 C218 생산자 집중 재출력은22관측, backend별89쪽으로 종료됐다.
선택178쪽의 raw90% 미만·쪽수 불일치·누락은0이며 Native 최저90.08879%,
fresh WASM 최저90.09923%다. 그러나 고정6ed와 Decimal로 비교하면
0.01%p 이상 하락은 **81관측(Native2/fresh WASM79)** 이므로 PR 보류를 유지한다.
이는 최신52조건 전쪽 완료가 아니다. 정본은 ignored `held-page-refocus-c219/`다.

C225/C228은 제품 변경 없이 frozen C218 WASM의 실제 Canvas root CTM에 물리
Print 축척을 주입한 반사실 진단이다. identity control은 canonical CLI의 decoded
RGBA와 모두 같았고 개인정보22쪽은 두 hook에서도 동일했다. 첫 scale-hook은
96DPI에서 scale 호출이 없는 정상 경로를 만나 assertion으로 종료했다. 그 실패를
통과로 쓰지 않고 root clearRect·현재 CTM·종이 크기를 검증하는 별도 hook으로
C228의10관측을 마쳤다. 개인정보22쪽98.58178→99.33146, 거대 표56쪽
97.60038→99.91614, 자산31쪽97.75848→99.91190 등 개선이 있지만,
sample11의50쪽은90.09923→87.40156%,2쪽97.04028→95.87226%로 후퇴한다.
converted1쪽97.83818→97.74522, issue265의1쪽99.12815→99.04907도 후퇴한다.
**전체 root 물리 축척은 채택하지 않는다.** 기준·글꼴·문턱을 바꾸지 않았다.
C224의52종이 감사에서 MediaBox는 모두 일치하나 PDF operator의 CM은 혼합이고
일부 참조는 순수0.12이므로, 몇 문서의 개선을 보편 프레임의 증거로 쓰지 않는다.
개인정보22쪽과 sample11의50쪽은 control/physical 각각 review·overlay를 직접
열어 밑줄 개선과 본문·연결선·하단선 후퇴를 확인했다. 진단 PNG는 제출 source의
최종 수용 증거가 아니다. 정본은 ignored `physical-frame-canvas-diagnostic-c225/`,
`physical-frame-canvas-controls-c228/` 및 `physical-print-frame-c224/`다.

C226의 외부 제안 Native 합성 검사는 frozen C218에서4건2PASS2FAIL이다.
Screen/Letter identity는 PASS, A4 상단/하단의 물리 위치 가정은 FAIL이었다.
이는 적용 전 반례 진단이며 제품이나 정식 suite로 승격하지 않았다. 이후 C228의
정상 대조군이 전체 투영 가설을 기각했으므로 이2FAIL을 새 필수 계약으로 삼지 않는다.

C222의 release build529.081초·release lib424.956초는 PASS지만 전체 nextest는
**10,936실행,10,935PASS/1FAIL/45skip**, 실행456.336초이며 전체 gate 완료가 아니다.
유일 실패는 issue6593의 캡션이 글꼴 소유에 따라 '<', 한글 본문, '>'의3개 run으로
나뉘었는데 기존 검사가 한 run만 찾던 문제였다. C227은 같은 TextLine의 직접
TextRun을 원문 순서로 결합해 정확한 전체 캡션과 대조하고 각 run의 동일 y를
확인하도록 선택 방식만 교정했다. 기존 캡션·다음 표 y 기대값과1.5px 허용치는
그대로다. 실제 동일 정식 검사 수정 전1FAIL→수정 후1PASS이고 rustfmt 후 재실행도
1PASS다. --prepare 후 canonical wrapper도 **1PASS/0FAIL(0.037초)**,
nextest run8e6319b6-26fc-4ef4-8a94-776af7d07c35다. 분할 suite 번호 이동은
생성 manifest 변경이며 검사 제거가 아니다. 제품 Rust는 C218과 동일하다.

C229는 이 test-only 변경의 source equivalence를 기록하고 C222 release build/lib
두 gate만 재사용하며 전체 nextest부터 나머지 gate를 새로 순차 실행한다.
첫 시도는 nextest가 같은 FAIL 행을 본문과 요약에 반복한 것을 driver가 두 실패로
세어 실행 전 종료했다. 원 시도·로그·source를 보존하고 동일 행 중복만 제거한 뒤
재시작했다. 현재 full-nextest가 jobs8/threads8로 실제 실행 중이며 완료로 쓰지 않는다.
생성 suite도 --prepare 후 source guard에 포함한다. 정본은 ignored
`caption-selector-c227/`와 `corrected-gates-c229/`다.


## C230–C232: 페이지·출력 요소별 프레임 분리

C229의 수정 후 전체 nextest가 **10,936PASS/0FAIL/45skip(345.354초)** 로
종료됐다. driver의478.0초에는 준비된 suite 컴파일 시간도 포함된다. Native Skia
library와 후속 gate는 별도 결과이며 전체 gate 완료로 선행 보고하지 않는다.

C230은 최초 쪽 CM만으로 다른 페이지의 glyph 프레임을 단정하던 가정을 재검토했다.
동일52조건의 독립 PDF **전1,964쪽** operator를 다시 읽었고18조건에서 쪽별 CM
패턴이 달랐다. 개인정보66쪽 중63쪽은0.12,2쪽은0.119935/0.119869,1쪽은 혼합이다.
sample11도117쪽은0.12,33쪽은0.119935/0.119869,1쪽은 혼합이며 실제2쪽과50쪽이
다르다. 거대 표 원본115쪽은 모두0.119935/0.119869다. CM을 찾지 못한 쪽과 다른
transform은 원장에 남겼으며 이 수치만으로 PDF 출처가 부적합하다고 판정하거나
기준을 교체하지 않는다. C224의 source paper/MediaBox와 독립 밑줄 프로브 산술은
유효하지만, 그 산술을 모든 페이지 glyph의 보편 프레임으로 확대하지 않는다.

같은 frozen base/C218·동일 명시 환경에서 거대 표5/56/109쪽의 원문 glyph origin을
독립 PDF와 대조했다. 새 HFT 구두점 폭은 물리 frame을 분리한 x 원점 평균 절대 오차를
0.0935→0.0782,0.0956→0.0844,0.1230→0.0782CSSpx로 줄인다. 폭을 과거 값으로
되돌리는 대신 실제 paint 소비를 보정해야 한다. 원문·glyph·PDF는 바꾸지 않았다.
control origin 추출의 첫 시도는 동일 문장이 양 열에 반복될 때 다른 열을 선택했다.
같은 텍스트 일치율 안에서는 원문 glyph 첫 x와 가까운 열을 고르게 교정하고 재산출했다.
Python 실행 환경도 pymupdf가 있는 기존 uv 검증 환경으로 바로잡았다. 정본은 ignored
`giant-source-frame-audit-c230/`의 exact source/pkg SHA 검증·공개 layer·원점 JSON·
`all-page-print-matrices.json`이다. 좌표 대조가 raster 전쪽 수용을 대신하지 않는다.

C231은 제품 수정 없이 **공개 TextDecoration의 solid bottom underline 소유만**
확인한 Canvas stroke를 물리 frame으로 옮겼다. bbox/기준선/크기에서 식별하고 단일
source owner를 검증하며 root CTM·glyph·표/도형 선을 유지했다. 같은 PDF·글꼴·192DPI
조건에서 개인정보22쪽은98.58178→**99.73363%**다.59개 source-owned 밑줄의 실제
세로 위치를 독립 PDF와 짝지으면 평균 절대 오차0.653112→0.019633CSSpx,
최대1.105293→0.120119CSSpx다. PDF가 인접 run의 밑줄을 하나로 합치므로 처음의
양끝점 일대일 짝짓기는 잘못된 진단으로 보존했다. 수정한 대조는 각 run의 겹치는
참조 구간과 같은 y를 사용하고59개 모두 매칭되며, 이 결과로 x 양끝점 동일을 주장하지 않는다.
review/overlay를 직접 열어 본문·표 소속 보존과 밑줄 일치를 확인했다.

정상 대조의 sample11은2쪽97.04028→97.03647%,50쪽90.09923→90.09923%다.
2쪽 INDEX 밑줄은 직접 review/overlay로 확인했다. 대상 밑줄이 없는 sample11의
50쪽과 행분리9쪽은 decoded RGBA 전체가 control과 동일하며 행분리9쪽99.64987%다.
C231은6관측·8쪽의 **반사실 진단**으로 종료했고, Native·전쪽·actual36·정식 제품
수용은 아니다. 기존81관측의 PR 보류를 해제하지 않았다. 증거는 ignored
`underline-frame-owner-c231/`의 producer/각 모드 TSV/PNG/geometry/identity다.

C232는 같은 underline만 cloned Native layer의 bbox/확정 positions에서 변환하는
대조를 준비했다. C229 Cargo가 끝날 때까지 실제 PID와 상태를 확인하며 대기한 뒤
frozen C218 library로 외부 observer를 연결한다. source-owned 확정 positions가
없거나 canonical 현재 PNG와 pixel identity가 다르면 실패로 중단한다. 제품 소스에
아직 투영 보정을 넣지 않았으며 source/PDF/font/목표 count/실제 backend를 맞춘
Native 결과와 범위 대조가 확보된 뒤 공통 소비 구현을 결정한다.


## C234–C237: 실제 CanvasKit 밑줄 계약과 Print 안내선 소비 보정

C229는27/27 gate exit0으로 종료됐다. release build/lib2개만 앞선 동일 제품
byte-equivalence로 재사용하고 나머지25개는 재실행했다. 전체 nextest는10,936PASS,
0FAIL,45skip이며 Native Skia library는root3,861PASS/12ignore와다른library182PASS다.
PNG2PASS,PDF14PASS,doc8PASS/3ignore,Nativefeature11gate 및fmt/세 clippy/workspace
build/manifest/unit tiers/type도 통과했다. C227 캡션 선택 교정은 로컬 한국어 커밋
`0666befe5b8fbac7c5464ac654dbaf672054d016`으로만 정리했다. 변경 파일은 해당 검사
하나이며 사용자 초안 두 개는 보존했고 push하지 않았다.

C232의 Native 외부 observer 첫 컴파일은 frozen library와 외부 serde_json의 다른
serde_core 인스턴스 때문에 실패했다. public FontEnvironment::from_json 경로로
교정한 뒤 컴파일은 성공했지만, 개인정보22쪽의 일부 TextDecoration에는 저장
layout_positions가 없어서 의도한 guard로 종료됐다. 현재 Native control PNG는
C219 canonical Native와 decoded RGBA 전체가 같음을 확인했다. 후보 Native 결과는
없다. 원래 run은 canonical replay_positions_or_compute를 통해 공통 측정을 소비하며,
저장 positions 부재를 glyph 원점 부재로 간주하지 않는다. TextDecoration의 bbox/run을
직접 바꾸는 방식은 기존 TextRun과 같은 source/visual 계약도 깨므로 제품으로 승격하지
않는다. 필요한 paint 결과를 semantic source와 별도로 공유해 모든 소비자가 쓰는
보정이 여전히 필요하다. assertion을 지우거나 Native 후보 통과로 기록하지 않았다.

C233은 frozen C218 공개 layer에서52조건·1,964쪽의 solid bottom underline 소유를
모두 수집했다.21조건220쪽에2,482개 owner가 있었다. 절대 경로 입력을path.join에
붙인 초기 driver 오류는44조건 완료 후 발생했고, 원 시도 보존·정확한 입력/producer
hash 확인 뒤 절대/상대 경로를 구분해 남은8조건만 이어 갔다. 실제 underline 쪽의
논리 종이 크기도 같은 frozen producer로 별도 수집했다. 독립 PDF의 겹치는 수평
stroke와 대조한2,482개 모두 좌표 짝은 찾았지만, 단순 투영에서110개는 오히려
세로 절대 오차가0.01CSSpx 이상 커졌다. 평균 개선이나 이 좌표 대조로 전쪽 시각
수용을 대신하지 않는다. 기존12독립 프로브에서 확인한600DPI 장치 반올림 가설도
별도로 기록했으나 모든 잔차를 없애지 못했고 제품에 반영하지 않았다. 정본은
ignored `underline-scope-audit-c233/`이며 draw 명령/숫자·geometry 감사이지 PNG 전체
수용 증거가 아니다.

이 추적에서 실제 CanvasKit은 Native/SVG/Canvas2D와 달리 밑줄을 고정2px 아래에
그리는 별도 결함을 확인했다. C234는 그 소비를 독립 Print의0.17em 계약으로 맞췄다.
HCR/HY신명조 각각6크기의 독립 PDF y를600DPI 종이 CTM에서 논리 paint 좌표로
환산한 합성 descriptor 검사를 추가했다. 실제 문서의 임시 배치를 선택하는 px 핀이
아니다. 장치 반 점0.08CSSpx와 기록 정밀도0.001을 사용하며 source bbox/전진은
변경하지 않는다. **수정 전19건9PASS/10FAIL→수정 후19PASS**다. Studio 전체는
2,358PASS/0FAIL/2skip,글꼴 계약36PASS 및typecheck PASS다. Rust 소스·Cargo 입력·
C218 JS/WASM은 byte-identical이므로 그 출력/게이트 재사용 범위를 명시한다.

C235는 같은 원본1쪽6크기 프로브를 실제 Native2/Studio Canvas2D2/CanvasKit4 경로로
열어 screen/print를 각각 캡처했다. 실제 backend·software surface·초기 JS/WASM
응답 SHA와 원본/참조 SHA를 확인했고 oracle font 환경이나 preflight 우회가 없다.
Native와CanvasKit은각2px관용raw100%,Canvas2D는97.54390%였다. 이는1쪽 프로브의
8조건이며52조건 전쪽이나actual36 완료가 아니다. Native/CanvasKit/Canvas2D의
Print review와standalone overlay를 직접 열었다. 글자 AA·위치 미세 차이가 있으므로
100%를 픽셀 동일이라고 쓰지 않는다.

C235의 Canvas2D blue 잔차는 편집용 여백 꺾쇠였다. 실제 Print 소비에서도 PageRenderer가
그 꺾쇠를 합성했고 CanvasKit도별도의옅은 꺾쇠를 무조건 그렸다. C236은 공통
shouldDrawPageMarginGuides(profile)를 만들어 두 실제 소비자가 screen에서만 안내선을
그리도록 했다. 문서 paint·screen 가이드는 유지한다. public focused-patch 경로의
Screen/Print 계약에서 **수정 전Print1FAIL→수정 후9PASS**이고 실제 문서 paint 호출이
두 profile 모두 유지됨을 검사한다. Studio 전체 **2,360PASS/0FAIL/2skip**,font36PASS,
typecheck PASS다. 이는 product composition 보정이며 검증기의 마스크·문턱을 바꾼
것이 아니다. C237은 새 host TS를 실제 HTTP로 읽어 새 profile predicate와0.17em이
로드됐음을 확인하고 같은6 browser 경로를 새로 캡처했다. Native2는 제품 Rust와
입력/명령 동일 근거로 C235 PNG를 재사용했다. 결과는 ignored C237에 별도 보존한다.

진행 중 생긴 도구 오류도 분리했다. C235의 첫 Native 명령은1쪽 입력에 CLI page1을
넘겨 범위 오류였다. 0기반page0으로 바로잡고 정상 두 profile을 완료했다. 그 뒤 Vite가
0.17을`.17`로 직렬화한 것을 문자 검사에서 놓친 시도는 보존하고, 실제 JavaScript
응답 타입·동일 수치 표현 정규식·응답 SHA 검증으로 교정했다. 성공한 Native 두 PNG는
동일 제품 근거로 보존/재사용했으며 browser 캡처는 그 뒤 처음 실행했다. 비교 파일명에
canvas2d의숫자2를 쪽 번호로 추측한 초기 review는 무효로 분리하고, 실제PNG bytes를
보존한 채 raw/rhwp_001.png라는 명시 쪽 이름으로 전체 비교만 재산출했다. 재캡처나
raw 점수 변경은 없었다.

현재 보류는 유지한다. 제품 Rust의2px90%/devel 대비0.01%p 원래 문턱과 원본115·
분할197쪽 검사를 바꾸지 않았다. C219의81하락(Native2/freshWASM79)은 host TS의
밑줄/편집 안내선 보정으로 해소됐다고 쓰지 않는다. 공통 실제 paint 보정과 새 source의
필요한 전쪽/실제 소비 검증, latest upstream 확인,PR/CI/정상 merge/후속은 남아 있다.


C237 독립 비교와 직접 판독도 종료했다.6크기 원본1쪽의 Print raw는
Native(재사용)100%,새StudioCanvas2D100%,새CanvasKit compat/default각100%다.
Screen은 안내선을 유지하므로Canvas2D97.54390%를 그대로 기록하고 Print와
상쇄하지 않는다. Source UI guide 제거는 실제 Print product 경로에서만 적용됐고
Visual Sweep 마스크/허용치는 그대로다. 실제 Print의 Canvas2D와CanvasKit default
review/overlay를 모두 직접 열어 꺾쇠 소멸·6개 원문run·크기/밑줄 길이·순서를
확인했다. 각모드PNG/TSV·동일초기JSWASM응답·새hostmodule의SHA·직접판독receipt를
ignored `print-guide-actual-consumer-c237/`에 보존했다. 폰트 AA와 미세 위치 차이는
남으며100%는2px 관용 실루엣이다.52조건 전쪽/actual36 완료가 아니다.

현재 Rust 코드/검사 source는 C229와 동일하고,그 뒤 Studio host3개TS파일과
노드 검사만 달라졌다는 byte-equivalence receipt를 C236에 남겼다. 보호된 사용자
orders/PR7871초안 SHA도 불변이다. 제품 Rust의 공통 paint-frame 보정은 여전히
미적용이며 기존81관측의 devel 대비 하락은 유지한다. 다음에는 semantic source의
bbox/run을 바꾸거나 검증을 완화하지 않고 공통 paint 결과를 Native/Canvas2D/SVG/
CanvasKit가 함께 소비하도록 보정하고 필요한 영향/전쪽 검증을 계속한다.


### C238/C239 표·선만의 인쇄 프레임 가설 기각

C238은 정식 Native PNG와 달리 opaque white를 강제하여 오른쪽 종이 끝의 alpha가
달랐다. 흰 배경 합성 픽셀 차이는0개였지만 strict RGBA 동등성 guard를 완화하지
않았다. 원 시도는 보존하고 C239에서 정식 RasterRenderOptions의 transparent 기본값을
사용했다. 같은 frozen C218 lib/입력·PDF·글꼴·profile·DPI에서7쪽 control 모두 정식
C219 PNG와 decoded RGBA가 동일했고 총14관측을 완료했다.

단순 비회전 Line/Rectangle만 물리 종이에 투영했을 때 rowbreak9는99.67453→99.67448,
개인정보22는98.56496→98.54565, sample11p50은90.08879→90.08168, asset56은
99.44075→99.41516으로 낮아졌다. sample11p2·asset31·거대표56은 불변이다. 평균이나
다른 페이지로 상쇄하지 않고 이 후보를 기각했다. 제품 source에는 반영하지 않았다.
개인정보22 후보 review/overlay를 모두 열어 본문·표 소속은 유지되지만 밑줄의 파란
잔차는 그대로임을 직접 확인했다. 정본은 ignored vector-native-counterfactual-c239/
decision.json이며 이전 C238 실패는 사용하지 않는다.

C240은 별도 공통 밑줄 paint 결과 후보를 검증한다. semantic bbox/run/source를 바꾸지
않고 PagePaintContext가 물리 Print 프레임을 생산하고, TextDecoration만 그 결과를
Native/Canvas2D/SVG/공개JSON→CanvasKit 및 visual bounds가 함께 소비하도록 한다.
본문 glyph·source table·줄 소속·clip/표·Screen은 유지한다. 독립2020 Print의6크기
HCR/HY 밑줄 y를 사용하는 기존 probe를 합성 public LayerBuilder 계약으로 먼저
실행하며, helper 기대값을 재인용하지 않는다. 아직 제품 후보/전쪽 통과가 아니다.

C240 수정 전 public LayerBuilder 계약은 실제1FAIL이며 첫12px HCR 밑줄 y가
144.507 vs 독립144.3226725CSSpx로0.184px 낮음을 확인했다. 첫 후보 검사의
HYS 합성 입력에는 실제 producer의 `HYSinMyeongJo-Medium,Haansoft Batang` 대신
`HY신명조`를 적어, 기존0.1em 표시 기준선 보정이 추가로 적용되는 잘못된 입력을
만들었다. 그 실패는 attempt1-wrong-synthetic-font-name에 보존했다. 독립 기대값이나
0.081CSSpx 허용치를 변경하지 않고 실물 probe의 실제 family명으로 바로잡았다.
제품의 밑줄 프레임도 raw baseline 대신 기존 script_draw_metrics의 실제 표시
기준선을 소비하도록 연결했다. 같은 Korean alias의 기존 glyph 보정 자체를 제거한
것이 아니다. 중단된 build/focus 대기 driver도 별도 attempt에 보존했고 실제
통과 summary 이후에만 Native 빌드→새 focused 캡처를 순차 시작하도록 했다.
새 필드 TextDecorationPaintGeometry는 logical bbox/run/source와 별도로 paint
bbox/inline scale만 보관한다. Native·WASM Canvas2D·SVG·공개 JSON·visual bounds가
같은 decoration_replay를 소비한다. CanvasKit preflight는 원본 geometry와 새로운
paint geometry를 둘 다 검사하며 원래 work limit/폰트 readiness를 바꾸지 않았다.

### C240–C253: 밑줄 paint 프레임과 혼합 대시·공백 조판

C240은 독립 2020 Print 6크기·2family 계약 수정 전1FAIL→수정 후1PASS,
기본 lib3805PASS/12ignore, Native Skia lib3862PASS/12ignore 및 관련 라이브러리182PASS를
확인했다. Native CLI와 locked web/no-opt fresh WASM을 별도로 보존했다. 이는 Mac
로컬 대체 빌드이며 Docker 최적화 통과가 아니다. C245 fmt·Native/WASM Clippy·
workspace all-target Clippy·공개 SVG 계층25건·장식 trim3건은 모두 exit0이다.
현재 전체 nextest는 이 소스에서 아직 실행하지 않았다.

C243 영향 범위21조건220쪽씩, Native/fresh WASM 총440쪽의 TSV에90% 미달은 없다.
그러나 고정6ed devel 대비 Native0쪽/fresh WASM11쪽의0.01%p 이상 하락이 남았다.
이 영향 범위 밖의 기존 하락도 별도 보정해야 하므로 전체52조건 수용으로 쓰지 않는다.
C244 6크기 원본 probe의 Native·Studio Canvas2D·지원 CanvasKit Print는 raw100%지만
AA/세부 잉크 잔차가 남으며 픽셀 동일이라는 뜻은 아니다. 실제8회 및 초기 JS/WASM
12응답 hash와 root/pkg/public8개 파일 일치를 확인했다. 최종 actual36을 대신하지 않는다.
로그/TSV/producer/직접 review·overlay 기록은 ignored C240–C245 디렉터리에 보존했다.

C246 Y600DPI 반올림, C247 .04em pen, C248600DPI 양자화 pen은 독립 진단으로만
실행했다. 개인정보4의 하락을 해소하지 못했고 제품에는 채택하지 않았다. 같은 glyph의
원점을 추적한 C249에서 해당 줄은 같은 HYSinMyeongJo-Medium 실폰트를 쓰지만
오른쪽 `①`가539.56CSSpx로 Print544.96CSSpx보다5.4px 왼쪽임을 확인했다.
공통 양쪽 정렬 분기가 남은 폭 전부를 대시에 배분하여 대시16.4948px,
일반 공백9.3333px가 되었다. 독립 Print는 대시 약15.58px(자연 .833em)에 머물고
일반 공백 약12.16px에도 여유를 배분한다. 폰트 대체나 paint clamp의 문제가 아니다.

회귀 추가 전 C250에서 같은 원본/독립 Print66쪽 전체를 C240 Native/fresh WASM으로
다시 비교했다. 최저94.12098/93.77937%, 미달0쪽이다. 새 정식 회귀는 원본 두 셀의
내용 순서·누락·중복, 항목 기호 소유, 혼합 대시/공백 전진 관계를 보호하며 실제1FAIL을
얻었다. 알려진 잘못된 위치를 절대 px로 고정하지 않았다. C253은 런별 실제 글꼴·크기의
자연 대시 복원 예산을 공통 줄 계획으로 생산하고 방출이 같은 결과를 소비하도록 수정한다.
선행 공백 정규화 런·필드 제외 계약을 보존하며, 남은 폭만 일반 낱말 공백에 배분한다.
현재 수정 후 집중 회귀 컴파일 중이며 새 소스의 시각·전체 회귀 통과는 아직 아니다.


C253의 동일 정식 회귀는 수정 후1PASS(.172초)이며 Native Skia lib3862PASS/12ignore,
다른 라이브러리182PASS/0FAIL도 exit0이다. Native CLI 및 fresh locked WASM 빌드는
성공했다. C254 Native4쪽 raw100%로 개선했고 review와 overlay를 직접 열어 오른쪽
기호·대시 시작의 소속/위치 개선을 확인했다. 밑줄·표 경계의 잉크 잔차는 남아 있다.
초기 WASM 캡처에는 WOFF2의 brotli가 없는 기본 Python을 잘못 선택하여 폰트 catalog
단계에서 실패했다. Native6쪽 결과는 유효하며 보존했고, 실패한 WASM 준비 디렉터리와
로그는 attempt-default-python-no-brotli에 보존했다. brotli/fontTools/PIL이 확인된 기존
uv 환경으로 WASM만 재개했다. 생산 source·입력·허용치·PNG는 바꾸지 않았다.


C253 검증 완료: 선언/기본 글꼴 조건의 개인정보66쪽을 Native/fresh WASM으로 각각
전쪽 산출하여 총264쪽 미달0/devel하락0이다. 각 조건 최저Native94.12098/WASM93.77937%로
변경하지 않은 최저 페이지는 그대로다. 4쪽 Native100/WASM98.89537%이며 양쪽 review/overlay
직접 판독과 원본 내용 소속 검사를 완료했다. Native 정식 회귀1·공개 계층25·장식 trim3 및
fmt/3종 Clippy까지7gate exit0, 전체 nextest10,938PASS/0FAIL/45skip(405.452초)도 완료했다.
이는 C253 working source이며 다른 문서의 devel 하락/최종 actual36·전52조건을 대신하지 않는다.
대표 PNG·전쪽TSV·생산자와 before/after 회귀 근거는
mydocs/pr/assets/issue7445/completed-20261009/dash-word-spacing-20261018에 정리했다.

C257의600DPI pen/Y 진단도 거대 표 HWPX9/15의 하락을 해소하지 못해 제품에 채택하지
않았다. C259의 실제 Canvas trace와 독립 Print glyph 원점 대조에서15쪽 맨 아래 기준선이
1075.13330 vs1073.86646CSSpx로1.26685px 낮음을 확인했다. C260 글자만 물리600DPI
종이 프레임에 투영하면 거대 표9/15는99.71686→99.99771/98.58595→98.94577%,
rowbreak9는99.64987→99.71202%로 개선한다. 그러나 개인정보4/22는98.82906/99.34962%,
sample11p2는95.86988%로 낮아지므로 전체 적용을 기각했다. source·입력·기준·허용치를
변경하지 않았다. C261의 원본 font trace에서 거대 표·rowbreak는 명시HFT altType2의
선택 이름이 원본과 같고, 개인정보는TTF altType1이며 sample11 본문은HFT에서
돋움/Calibri로 치환된다. 좌표 계약의 실제 소비 경계를 이 구분과 독립 PDF로 계속 추적한다.
단순 문서별 조건으로 이 결과를 제품에 넣지 않는다.

C253는46e9ac990120118ae141edd58d69daf910e40b3f에 구현·관계 회귀·전쪽TSV·대표PNG를
로컬 커밋했다. paragraph_layout.rs의 C253 증분 patch만 index에 적용하여 이전 미커밋
70추가/16삭제와 다른 작업을 보존했다. 사용자 초안 두 SHA도 그대로 확인했다.
C262는 완료4쪽의 `①` 독립 원점(544.96,193.28)CSSpx에 각 축±2px 보호를 추가했다.
기본1PASS(.166초)/Native Skia1PASS(.168초)/fmt exit0이며 actual Canvas 원점은
(545.088867,193.240005)이다. 이전 C249 원점은(539.559998,193.240005)였다.
좌표 assertion은5afe5864754c09dea4089c9286e58b20c25e847b에 로컬 커밋했다.
전체10,938PASS는 좌표 assertion 추가 전이고 runtime source는 byte동일함을 확인했다.

C262 --prepare가28suite를 정본 정책대로 재배정하면서 case가25→23으로 이동했다.
C263의 첫 guard가 이를 허용하지 않아 캡처 전에 중단했다. 각 현재 harness를
renderHarness(deriveManifest()) 결과와 byte대조하고1555 source module의 유일성을
확인한 뒤, 변경이 좌표 test 한 파일+해당28개 정본 harness뿐임을 확인하고 재개했다.
이는 guard 해제나 기준 완화가 아니다. 현재266쪽(HWPX거대표115+rowbreak18, 양쪽)
qualification은 전쪽 완료·90%미달0, devel하락10쪽이다. Native최저97.75805/92.87548,
WASM최저97.75788/92.86964%다. producer는 runtime이 byte동일한 frozenC253이며
수정한 test/harness와 별개로 출처를 보존했다. source전용 회귀 추가의 전쪽 선행 근거이며
아직 글자 paint 프레임 보정이나 전52조건 수용이 아니다. 모든 이번 process는 terminal이다.


## C265–C271. HFT 출력 프레임의 교차 검증과 회귀 보류

C259의 독립 PDF 글리프 원점과 C261의 실제 source font program을 근거로, 원본 HFT가
치환되지 않은 경우에만 공통 `paintTransform`을 생산하는 후보 C265를 만들었다.
TextRun/CharOverlap과 같은 source의 GlyphRun/GlyphOutline에 동일 결과를 전달하고,
Native Skia·WASM Canvas·SVG·JSON·CanvasKit·visual bounds가 이를 소비한다. source bbox,
source ID·내용·줄 소속은 바꾸지 않는다. TrueType와 치환 HWP3는 같은 affine의 적용 대상이 아니다.
이는 특정 파일명·문서 ID·페이지 점수 분기가 아니지만, 아래 전쪽 반례 때문에 아직 수용하지 않는다.

정식 HFT 원점 관계 검사는 실제 수정 전 2PASS/1FAIL → 수정 후 3PASS였다.
Native Skia lib 3863PASS/12ignore와 다른 lib 182PASS, Studio frame/control 28PASS,
TypeScript 및 Native/fresh WASM 빌드가 완료됐다. Root pkg/Studio public의 JS/WASM/d.ts
8개 파일 hash도 일치했다. 독립 실제 제품 초기화와 최종 전체 검증은 이 결과에 포함하지 않는다.
C266 집중 13쪽×Native/fresh WASM=26관측은 raw2px 90% 미달·fixed6ed 대비0.01%p 이상
하락이 없었다. 정상 PII4/22와 치환 sample11 p2의 양쪽 backend 6관측은 C253과 RGBA 및
source render tree byte가 같았다. HWPX15의 Native99.23361/WASM98.95292%, HWP 원본63의
양쪽99.885/99.885%를 포함한 review/overlay에서 본문·밑줄·인접 내용과 작은 프린지를 직접 확인했다.

그러나 C267 전쪽에서 rowbreak7이 Native88.75384%로 미달했고 fixed6ed90.48856%보다
1.73472%p 하락했다. 같은 쪽 C253은92.87548%였다. 같은 문서9쪽의 통과만으로는 검출하지 못한
반례다. 기존 PDF와 보정 전후 review/overlay를 보존했다. 하부 제26조의 글리프 원점과 독립
PDF를 대조하면 이미 남아 있던 배치 차이 위에 후보의 이동이 겹친다. C270의 읽기 전용
fillText/strokeText 추적은 canonical fresh WASM과 RGBA가 같았으며 row7/9를 각각
88.74041/99.71178%로 재현했다. 이 반례를 제외하거나 점수 기준을 낮추지 않는다.

추가로 독립 PDF stream에서 GI의 device CTM은0.119935/-0.119869이고 rowbreak는
0.120/-0.120으로 다름을 확인했다. HFT 선언만으로 같은 Print 좌표 변환을 선택할 수 있다는
가정은 아직 입증되지 않았다. 파일·쪽 번호 예외나 폰트 이름 예외로 좁히지 않고 원점의 소비와
실제 Print 경로를 확인한다. 같은 원본 SHA10b6ab65…를 engine2024/timeout1800으로 새로
Print 요청했다(job6fe59868-c4f2-4994-83cf-6bbda8bbc4c1). 기존 PDF는 교체하지 않았고,
새 PDF의 출력 경로·좌표·동일 원본 대응을 확인하기 전 기준 적합성이나 시각 개선을 주장하지 않는다.

C268은 fmt·Native root/feature Clippy·WASM Clippy·workspace build·all-target Clippy·manifest
7게이트가 PASS였으나 unit-tier가 실제 FAIL했다. source unit2개와 새 top-level cfg(test)
helper가 최신 base의 정책에 맞지 않았다. C271에서 두 검사의 모든 기대값과 정상 대조를
`tests/cases/issue_7445_paint_frame_protocol.rs`의 public API integration 검사로 옮겼다.
공백 계산의 기존 test 호출 wrapper는 기존 test module 내부로 옮기고 production 계산은
변경하지 않았다. policy baseline을 올리거나 검사를 제거하지 않았다. 실제 재검증은 별도 로그로 남긴다.

확인된 미달 후보의 비용 큰 수집을 계속하지 않기 위해 C267은 소유 프로세스10개를 확인한 뒤
중단했다. 완료12조건851쪽과 진행 중 부분 PNG/TSV/로그는 보존했으며 whole52 완료가 아니다.
C269는 C268 실패를 감지해 실제 캡처 시작 전에 끝났고 새 actual88 완료 증거는 없다.
정본은 ignored rebase-6ed1682c-20261018의 hft-glyph-frame-c265, hft-glyph-focus-c266,
hft-glyph-whole-c267, hft-glyph-gates-c268, hft-glyph-actual-c269, row7-glyph-origin-c270,
unit-tier-layout-c271이다. 이 후보의 시각 미달과 필수 검사가 해결될 때까지 PR은 보류한다.


C271의 최종 범위 검사: unit-tier4103/296 modules 정책 PASS, public integration2PASS,
기존 alignment4PASS와 distribution3PASS로 총9건 PASS다. 두 검사의 본문과 wrapper 본문이
서식·public import 경로 외에는 동일함을 relocation-equivalence.json으로 대조했다.
초기 enum import 누락과 unit-module 중복 import 오류는 실제 compiler 로그로 보존했고
해소 뒤 재실행했다. 수동 재실행에서 CARGO_TARGET_DIR를 빠뜨려 시작한 별도 target 빌드는
정확한 소유 Cargo/Rust 프로세스만 중단하고 정본 공용 target으로 재실행했다. 그 시도에는
테스트 summary가 없으며 통과 증거로 쓰지 않는다. 기존 target 내용은 임의로 지우지 않았다.

7쪽 저장 프레임의 읽기 전용 trace에서는 (row3,col1) cut0..16의 현재 정렬이Top이고
content/centered0(windowed composition)이며 원본은Center임을 확인했다. 첫 몸글 y643.186667,
셀 y641.306667/h365.72/pad합3.76, 앞 행 높이406.44다. 이 값을 화면에 맞춘 상수로
고정하지 않는다. 원본 nested unit16의 stored frame break와 부모/자식 빈 띠·정렬 소유를
독립 Print와 대조할 다음 원인 경로로 기록한다. 아직 원인 확정이나 구현 복구 완료가 아니다.


## C272–C277. 독립 Print의 fill 이후 좌표 정밀도 전환

새 engine2024 Print(job6fe59868-c4f2-4994-83cf-6bbda8bbc4c1, timeout1800,
PrintMethod0, preprocess none)는 18쪽이다. 기존 PDF와 새 PDF를 같은 Poppler96DPI로
전쪽 비교하여 18/18 PNG가 동일함을 확인했다. 기존 기준을 바꿀 이유가 없고 rowbreak7의
미달은 제품 보정 대상으로 유지한다. C272 임시 opt-in 배치 trace는 native CLI 빌드 성공 후
원본 source SHA로 정확히 복원했다. 현재 셀의 visible physical height도365.72이므로 단순히
Center를 강제로 켤 근거가 없었다. 해당 진단 바이너리는 제품 검증 결과로 쓰지 않는다.

C273은 독립 HFT 6쪽 원본의 spacing24개만0→-6으로 바꿔 Print했다. 실제 A→B 전진도
달라졌지만 CTM은 모든 쪽에서 0.119935/-0.119869로 유지됐다. C274는 같은 문서의 첫 실제
ASCII run 하나만 명시 Arial TrueType으로 바꿨다. PDF에서 ArialMT Type0의 실제 사용을
확인했으나 CTM은 그대로였다. 자간 또는 TrueType 혼용을 좌표 프레임 선택 조건으로
채택하지 않는다. 원본 text와 다른 속성은 유지했고 두 실험을 제품 시각 수용으로 인용하지 않는다.

C275에서 중요한 독립 반례를 확인했다. 같은 정상 HFT 입력의 charPr1 shadeColor 한 속성만
none→#CCCCCC로 바꾸자, 첫 배경을 그리기 전 CTM은 6자리이고 그 뒤부터 6쪽 종료까지
0.120/-0.120의 3자리였다. 2,256개 글리프의 font/code/순서는 같았고 첫 원점은 동일하나
fill 뒤 원점은 바뀌었다. 이는 글꼴 종류만으로 정밀 device 프레임을 결정하던 C265의 가정에
반하는 인과 대조다. job0e3e2b46-4ff5-4f8f-8c2a-101dd0a8856b, engine2020,
PrintMethod0, Hancom11.0.0.9136, PDF6쪽/SHA057b9d8699d9f59a659f26b25012dc9c16aa28414745d15a2e1f5a5752457c32.
기존 rowbreak는 첫 쪽의 흰 polygon fill 이후, 개인정보는3쪽의 회색 polygon fill 이후,
sample11은34쪽 gradient의 polygon fill 이후 같은 숫자 정밀도 전환을 보인다.
단순 file/font/page 예외로 처리하지 않는다.

C276 후보는 source solid fill과 글자 음영의 paint 순서를 따라 정밀 프레임의 종료를 공유한다.
임의 페이지부터 요청해도 앞쪽 semantic page cache의 fill 이력을 읽고, 현재 쪽은 공통 replay
plane 순서를 소비하여 그 뒤 HFT glyph 대안에 정밀 affine를 재적용하지 않는다. 앞쪽 fill이
편집되면 뒤쪽 파생 layer/JSON cache도 무효화한다. source bbox·내용·줄 소속·쪽수·허용치는
바꾸지 않았다. 아직 focused 및 전쪽 수용 전인 미커밋 후보다. Native/fresh WASM 빌드와
C277 rowbreak7/9·거대표·개인정보·치환 HWP3의 집중 비교를 새 producer로 진행한다.
C275 생성 첫 시도의 ZipInfo 재사용 오류는 업로드 전에 검출·INVALID 보존했고, 복제한 ZipInfo와
전체 ZIP CRC/XML·단일 속성 diff 검증 후에만 Print를 요청했다.

정본 ignored output은 fill-coordinate-print-control-c275, fill-coordinate-frame-c276,
fill-coordinate-focus-c277이다. 실험·빌드 로그와 JSON은 저장소 커밋에 포함하지 않는다.


C276 Native/fresh WASM 빌드는 완료했고 root pkg/Studio public의8개 JS/WASM/d.ts hash를
대조했다. 이는 Mac locked web/no-opt 로컬 대체 빌드이며 Docker 최적화나 실제 제품
초기화 검증을 뜻하지 않는다. C277 집중9쪽×양쪽=18관측은 모두90% 이상이나 devel 하락3건이
남았다. C278은 그 보류를 기록한 상태로 진단용 전18쪽×양쪽을 완료했다. Native최저92.87548,
fresh WASM최저92.86964%, 미달0/쪽수18=18이다. devel 하락은 Native9쪽-0.02197%p,
WASM7/9/10/16쪽-0.06541/-0.04526/-0.05672/-0.02004%p로 총5건이다.
전체52조건·최종 nextest·actual88 완료가 아니며 PR 보류를 유지한다. 같은9쪽의 집중/whole
WASM은99.64987/99.64590%로 작은 차이도 있어, 하나의 유리한 실행만 수용 근거로 고르지 않는다.

C280은 canonical PNG와RGBA가 같은 control 뒤에만600DPI 원점 내림 진단을 적용했다.
WASM7/9는 control92.86964/99.64987, x만92.87216/99.68288,
y만91.98166/99.65310, xy91.97579/99.68228%였다. 독립9쪽의 동일 Unicode/가까운
원점212개는 y 내림과0.01CSSpx 안에서 맞지만, 7쪽의 더 큰 잔여 배치 차이는 악화한다.
점수에 맞춰x만 제품에 넣지 않으며 이 altered Canvas 결과는 제품 통과가 아니다.
C279는 exporter snapshot 뒤 내림 방식 수정이 겹쳐 hash guard가 altered capture 전에
실행을 차단했다. 그 시도는INVALID이고 새로 고정한 C280만 진단 수치로 사용한다.

C281 opt-in read-only trace는 빌드 후 source를 정확히 복원했다. rowbreak7 row3,col1의
cut0..16은 pinned/centers=false, Top이다. 안 높이361.96, 원점643.186666667,
배치 후 흐름 커서1002.32(내용359.133333333)이며 중첩 표 bbox는849.68..995.013333333이다.
예약 원장의 높이, 실제 자식의 끝점, 바깥여백을 같은 물리 프레임에서 대조해야 한다.
42065 대조군도 함께 trace했고 일괄 Center 적용이나 화면에 맞춘 상수는 넣지 않았다.

C282는 C278 전쪽90% 및9쪽 양쪽98% 이상을 확인하고 양쪽 review/overlay를 직접 판독한 뒤,
기존 public API HFT case에 후속 쪽의 fill 이후 CTM 관계와 독립9쪽 첫 glyph 원점
(182.08,113.60)CSSpx±2 보호를 추가한다. source bbox/내용/소유 관계는 함께 유지한다.
수정 전 C265의 source3파일을 안전하게 임시 보존·대조하고 실제 FAIL→후보PASS를 검증 중이다.
초기 nextest -j/--test-threads 중복 및 --profile/--cargo-profile 혼동은 테스트 실행 전에 거절됐고
매번 source 복원을 확인했다. 이 명령 오류는 before FAIL 증거로 쓰지 않는다.


C282 최종 실제 재검증은 before4건3PASS/1FAIL(0.335초, source CTM0.999462/0.998907로
새 관계 검사 실패)→후보4PASS(0.371초)이며 unit-tier도exit0이다. source3파일 복원 SHA를
확인했다. 비교 입력은 같은 rowbreak 원본이고 새 관계/px assertion 외 기존3검사도 유지했다.
C282 visual-eligibility.json의 처음 DPI192 표기는 잘못된 메모였으며 실제 scope52의
rowbreak 조건과 PNG794×1122는96DPI이므로96으로 정정했다. 좌표는 일관되게96CSSpx,
PDF point×4/3이고 수치·이미지·허용치는 바꾸지 않았다.

C281 원본 기하를 더 대조했다. 오른쪽 셀의 선언 높이27451HU, 위/아래 안 여백141+141HU,
호스트 마지막 저장 줄14080HU, child vertOffset1124HU, 바깥여백283+283HU,
child 첫 프레임 선언10882HU로 source 점유 끝은26652HU이다. 따라서 선언 프레임 안의
가운데 여유는517HU, 이동량은258.5HU=3.446666667CSSpx이다. 현재 셀 위641.306666667과
pad1.88 및 원본 baseline935HU를 합하면659.10CSSpx,600DPI 내림은659.04CSSpx로
독립 PDF의659.039998과 일치한다. 이는 아직 미구현 원인 가설이다. 컷의 선언 프레임 소유를
인증하고 같은 측정/배치 query로 전달해야 하며 42065 같은 페이지 고정 wrapper에 일괄
Center를 적용하지 않는다. frozen6ed도 첫 본문 y643.2로 같아 이 큰 세로 잔여 자체는
현재 후보가 새로 만든 것이라고 주장하지 않는다.

C283은 보류4쪽의 canonical fresh WASM과 읽기 전용 trace control을RGBA 동일하게
검증했다. 단, 부분4쪽 수집과 C278전쪽 수집 사이에서7/16쪽은PNG가 같고9/10쪽은 달랐다.
font catalog720개의 순서/내용은 로컬 resource 경로 외에 같고 canvas 옵션은 선택 쪽만
달랐다. 10쪽 raw는부분98.99434/전쪽99.17733%였다. 동일 worker에 배정된 선행 쪽과
font priming 조건을 확인하며 유리한 부분 결과만 채택하지 않는다.

기존 fixed6ed base driver는첫1쪽을 따로 확인한 뒤2..17/18을 수집했고 후보는1..16/17..18을
수집했다. 같은18쪽을 같은16쪽 묶음·capture8로 양쪽 다시 수집해 비교 조건을 맞춘다.
C284 첫 시도는 시작 간격만16으로 바꾸고 첫 base묶음 길이1을 남긴 드라이버 오류로
2..16쪽이 누락됐다. base TSV 계산이 실패했고 head의 paired coverage도 불완전하므로
C284의0하락 요약은INVALID이며 수용에 쓰지 않는다. C285는 실행 전에 전체 쪽을 정확히
한 번 덮는 묶음 계획을 검사하고, 각 role 명령 실패0·18쪽 대응·head/base paired18을
모두 확인한 뒤에만 비교를 해석한다. 입력/PDF/기존 결과와 기준은 보존했다. 이 재수집도
전체52조건·최종 회귀·actual88을 대신하지 않는다.


C285 동일 조건 전쪽 대조는양쪽 role의Native/WASM 각18쪽과 paired18을 모두 실제
확인했다. 후보 미달0이며 devel 하락5건은C278과동일하게 남았다. 따라서 이 보류를 묶음
조건 차이 탓으로 해소했다고 보지 않는다. 같은16쪽 묶음에서 후보 반복PNG는Native18/18,
WASM18/18이RGB 동일했다. 부분 수집 차이와 전쪽 고정 조건의 반복성을 구분한다.
현재 production source는C276 frozen ledger와 일치하고, C282 test만 추가된 상태다.


## C286–C290: 저장 첫 프레임의 물리 점유와 정렬 공통화

독립 Print와 원본 셀 선언27451HU, 안여백282HU, host 마지막 저장 줄14080HU, 자식 offset1124HU·바깥여백566HU·첫 프레임10882HU를 대조했다. 점유 끝26652HU와 선언 사이 Center 여백517HU는3.446667CSSpx 이동을 설명하며, 실제 첫 글리프659.10CSSpx의600DPI 내림659.04는 독립 Print659.039998과 일치한다. 문서 ID·쪽 번호를 production 조건으로 사용하지 않았다.

`stored_mixed_opening_frame`는 첫 저장 프레임·다음 미소비 유닛·정확한 child 저장 종료와 선언 예산 maximal prefix를 인증한다. `CellCutFrame::height` 및 `row_cut_content_height`는 같은 물리 높이를 예약하고 `table_partial`은 같은 점유 끝으로 Center/Bottom 정렬한다. 기존 일반 컷 경로까지 실제 예산 실패를 검증하는 C291 검사는 진행 중이며 helper 공유만으로 그 경로의 완료를 주장하지 않는다.

C286 Native/fresh WASM을 새로 빌드했다(locked wrapper `--target web --out-dir pkg --no-opt`, 로컬 대체). C287 집중20쪽 모두90% 이상, 7쪽 Native97.26195/fresh WASM97.32913이다. 양쪽 review+overlay를 직접 판독했고 첫 줄의 가운데 정렬과 다음 조각 경계가 유지되는 것을 확인했다. 미세 글자/괘선 차이가 남아98% px 핀은 추가하지 않는다. C290 전18쪽 각 backend가 완료되어 Native 최저95.86292%, WASM95.93221%, 쪽수18/18, 미달0이다. 고정6ed 동일16쪽 묶음 비교에서 Native9 −0.02197pp, WASM9 −0.04526pp·10 −0.05672pp·16 −0.02004pp가 남아 PR은 보류한다.

C288 정상 wrapper 대조42065는 수정 전후 각 backend17/17 RGB가 같다. 독립 PDF 대비 기존 최저51.40720/51.42529%와7개 미달은 시각 수용으로 인용하지 않는다. C289 실제 Canvas glyph 원점600DPI 내림 진단은 일부 쪽을 악화시키므로 production에 적용하지 않았다. canonical과 read-only control은4/4 RGBA 동일하여 관측기 영향이 없음을 확인했다. 전체52조건·최신 전체 회귀·actual88은 아직 미완료다.


## 사용자 승인: devel 하락 재검토 기준0.1%p

사용자가 최대0.05672%p의4건을 진행하도록 승인하고 공통 지침도0.1%로 변경하라고 지시했다. 로컬 문서 커밋`e84b0369f3`은 canonical 거버넌스·PR workflow·review template 등7곳을 정확히0.1%p 이상 하락부터 재검토하도록 통일한다. 문서7개 링크 검사와 diff 공백 검사 PASS. C290 Native/WASM18쪽 기존 TSV·원래0.01%p 판정은 그대로 보존하고 새 기준으로 재판정한0.1%p 하락은0건이다. 이는새 후보 전체 검증을 대신하지 않는다.

C291 공개 API 관계 검사는 C276에서5건4PASS1FAIL(0.454초, 앞/뒤 Center 여백0/6.6)로 실제 결함을 재현했고 C286에서5PASS(0.524초)를 확인했다. before driver의 후속 문자열 판정은 `top=0` 대신 실제 부동소수 -4.66e-15가 출력되어 실패했으나 source는 finally에서 정확히 복원됐고 원본 로그의 실제 test FAIL을 수치로 확인했다. 최초0건 suite 실행과 content 전용 API에 physical 예산을 넣은 초기 capacity 검사는 INVALID로 구분한다. 올바른 physical-band 호출의 부족 예산 검사는 아직 실패하며 소유 프레임/내용 전진/trim/예약의 실행 값을 추가 진단 중이다. 새 수용 결함이 해결됐다고 주장하지 않는다.

C293 font draw size600DPI 진단은 C286 frozen WASM과 같은 원본/글꼴로 수행한 별도 진단이다. canonical/control4쪽은 byte-identical이고 일부 값을 개선·악화시켜 production에는 적용하지 않았다. 이후0.1%p 기준에 따라 소폭 차이의 추가 점수 맞추기는 중단하고 실제 수용 경계와 전체 검증을 진행한다.


### C291 경계 보정과 C294/C295 후속 실행

실제 physical-band 호출의 부족 예산에서 `advance_row_cut_inner`는 선언된 첫 프레임을 확인하지 않아컷[3,16]을 수용했다. source query는물리366.013333/점유355.36, Unit 합363.84, 저장 종료 trim1.88, 실제 내용361.96을 반환한다. 별도 reserve만 더하는 시도는 ordinary 후보 수용을 보장하지 못해 폐기했다. 최종 구현은 ordinary 후보 컷에서 같은 `stored_mixed_opening_frame`의 물리 높이−같은 안여백이예산을 넘는지 먼저 확인하며, row-offset의 `CellCutFrame::height`, 최종 `row_cut_content_height`, 실제 partial 정렬도같은 선언을 사용한다.

C291 actual physical 호출: 수정 전1FAIL(0.070초)→수정 후1PASS(0.068초). 작은 예산의 이월·정확한 선언 예산의 수용·같은 컷의 실제 높이·다음 조각 전진을 검사했다. C294 관련 row_cut_tests 정상 대조39/39PASS(1.458초). source unit test attr 수는 늘리지 않고 기존 관련 계약에 경계를 보완했다. 현재 public 회귀 재실행·unit tier·fmt·새 Native/fresh WASM은 C294 순차 driver에서 진행한다. C295는 새 artifact 완료와 producer/source hash를 확인한 뒤52조건 전쪽을 검증하도록 대기한다.

C295의 페이지 묶음은 fixed devel과 같게 고정한다(rowbreak는C285의16쪽 묶음, 나머지51조건은C171 base의첫1쪽+16쪽 묶음). 이전 subset/full 요청 순서에 따른 Canvas font 준비 차이를 새 결과에서 선택적으로 사용하지 않는다. 현재 C295 측정 시작/전체 통과나 actual88 완료를 주장하지 않는다.


### C294 중단 사유 교정, C296/C297 진행

C294 public 추가 검사1건은 실제 회귀가 아니라 제2호·제3호 인용 문단의 반복 문구를 전역 검색한 검사 오류였다. 원본 child 문단1과3 모두 `하여 정보를 제공하거나 정보의 제공을`을 포함한다. 검사에서 마지막 줄의 cell_context+para_index로 제3호만 선택하고 양쪽 조각의 원문 결합·순서·누락·중복을 확인하도록 보정했다. 같은 공개 API case 최종5/5PASS(0.378초). 앞선 Center FAIL과 physical 부족 예산 FAIL은 해당 원인으로 유효하며, 이 전역 문구 assertion 실패는 구현 결함 증거로 사용하지 않는다. C294/C295는 build/capture 전에 종료했고 새 결과를 C296/C297에 산출한다.

C296은 현재 source unit39/public5 PASS의 동일 소스 hash를 확인해 재사용하고 unit tier·fmt PASS 후 Native Skia release-test 빌드를 시작했다. 이후 locked fresh WASM 로컬 대체 빌드를 순차 실행한다. C297 실제 driver는 C296의 live PID·build 상태를 확인하며 대기 중이고, build 완료 후 새 source와 artifact를 고정하여52조건 전쪽을 실행한다. 전체 nextest·나머지 필수 게이트·actual88과 최종 commit/PR/CI/merge/후속 처리는 아직 남아 있다.


### C297/C301/C302: 0.1%p 기준 재평가와 로드맵 하락 진단

사용자 지시로 devel 대비 하락 재검토 문턱을0.1%p로 바꾼 공통지침7개는e84b0369f3에 커밋했다. 기존 raw90/쪽수/내용/98px 조건은 유지한다. C296 Native/fresh WASM 빌드는 완료했고 C298 fmt/unit-tier/lint/workspace/release-build까지 통과 후 release-lib·전체nextest·나머지게이트를 순차 진행한다. 이 중간 상태를 전체통과로 쓰지 않는다.

C301은 새C297 PNG만으로 대표17쪽 review/overlay를 만들었다(새capture0). Native sample11 2/50, sample16 6/33, rowbreak7/8/9/18, asset14의 두 종류 PNG를 직접 열었고direct-read.json에각각이미지SHA·판독·잔여를 남겼다. 수식6쪽의 미세기준선과Print그림/일부괘선/글자의기존차이는완료로숨기지않는다. 새Native7/8쪽에서 첫프레임정렬·첫미소비줄·다음조각순서를 확인했으며public관계검사와별도로증거를연결한다. freshWASM판독은아직별도다.

C297 Native 로드맵302쪽 전쪽이 완료되자90%미달0이지만0.1%p이상하락14쪽(2/9/10/11/42/54/62/65/68/74/75/91/92/94)이나왔다. 최저91.00093%, 최대하락62쪽3.80607%p다. C302에서 정확한base/head입력·PDF·font-path·환경동일을확인하고저장PNG및directPDF글꼴목록으로원인진단한다. base와후보의62쪽본문표시글꼴/획이달라보이나독립Print의원문글꼴프로그램과구체원점확인전원인을단정하지않는다. 문턱을바꿔숨기거나점수만높아지는잘못된대체글꼴로되돌리지않는다. PR은이14쪽재검토가끝날때까지보류한다.

C300 actual92는C298대기중인소유PID76300/정확한명령과capture/server0을확인한뒤SIGTERM으로중단했다(waiting-stop-receipt.json). C297/C298는종료하지않고추가범위/회귀증거를계속수집한다. 보정후actual의동일92범위는새producer/head로실행하며기존대기를실제소비통과로쓰지않는다.


### C303: 24쪽 그림의 예약 장부와 실제 후속 커서 불일치

영역 진단에서 상단 수식99.97674%, 그림영역85.55812%, 그림아래본문83.52883%다. sourceGIF236×180과PrintJPEG같은크기비교81.59509%의압축차이는별도지만,본문의흐름차이도실재한다. Source p310의두그림은각각offset+본체+양바깥여백+caption gap850+높이1200HU를점유하며max19473HU다. p311 저장원점도이끝뒤에서시작한다.

기존sourceunit시험에임시읽기진단만넣어1PASS(0.59초)로장부를확인한뒤원본파일SHA를바이트일치로복원했다. default2022진단이며2024시각통과가아니다. p310 CellUnit은25.6+234.04=259.64px로정확한19473HU를예약한다. 그러나whole/partial paint는문단글줄1920HU를전진한뒤캡션없는generic그림상자17423HU를더한다. 따라서caption2050HU대신host1920HU가흐름에들어130HU(1.733333CSSpx)가빠진다. 원본값→CellUnit예약→실제para_y덮어쓰기까지추적했다.

보정은130HU상수를더하지않고이미예약한연속문단CellUnit의전진을실제whole/partialpaint가소비하게한다. 조각이호스트줄과그림band를전부소유할때만같은문단프레임이며,deferredband의이미소비한offset경로는보존한다. 기존issue_4138_child_frame_ownership에원본그림·캡션끝과다음빈줄/본문의비침범관계검사를임시추가하고--prepare후수정전실행을시작했다. 아직미달인원본의새회귀이므로독립전쪽N/W90%및판독을완료하기전커밋하지않는다.


C303수정전public5건4PASS/1FAIL(2.746초),수정후5PASS(2.793초). 첫after는Cell/Table타입미지정컴파일오류101이었고완전경로로보정했다. 이중간오류를테스트회귀/통과로쓰지않는다. 새cell_picture_host_flow_advance는reflow_picture_host_advance가인증한빈그림호스트의연속CellUnit을조각이전부소유할때만이미발행된height합을반환한다. 문단앞뒤간격도그장부에있으므로paint가다시추측하지않는다. whole/partialpaint의generic높이재가산분기를같은전진으로닫고,그림만이월된deferredband경로는유지했다.

C304정상48PASS(5.298초),row_cut39PASS(1.48초),unit-tier/fmtPASS. 새Native빌드119.45초에서두분할본각23/24/25쪽99.19578/95.82844/99.91700%,197/197을확인했다. 현재생산본24쪽review+overlay를직접열어캡션뒤본문원점/순서복원을확인했고GIF/JPEG압축·작은기호잔여는남긴다. freshWASM은locked web/no-opt로빌드중이며이집중결과를전197/52통과로재인용하지않는다. 새회귀는전쪽N/W90%등최종선행조건전커밋하지않았다.

C302독립전체한글CFF공급3쪽진단은N/W base/head4조건끝났지만하락을해소하지못했으며채택/설치하지않는다. baseWASM의새프로그램선택API부재첫실패는C171base와동일한명시null환경으로보정했고성공한Native결과를중복실행하지않았다. base자체는299/302이므로집중3쪽과전체수용/쪽수자격을혼동하지않는다.

로드맵14쪽의다음원인가설은인쇄device CTM정밀도상태가solidfill뒤영구단조상태가아니라raster출력뒤다시바뀌는지다(아직미확정). C305는C275독립6쪽표본의텍스트/HFT face/크기/장평/자간/음영2280글자관계를그대로보존하고2쪽에비균일bitmap하나만넣었다. 기존raw번호정의의namespacealias가Native재저장에서unbound가되는첫진단산출물은보존하고MCP에보내지않았다. headerprefix만canonical하게만든동일의미template로재생성하여XML/텍스트/글꼴속성동일을확인했다. 한컴2020Print job21678437-a3ae-40c9-a286-3edad148b4ad(timeout1800)을한번시작했고기존대상PDF를교체하지않는독립프로토콜진단으로관찰한다.


C304 freshWASM도완료했다(locked web/no-opt 로컬대체, Docker최적화아님). 두분할본각23/24/25쪽99.19578/95.42331/99.91777%,197/197이다. Native95.82844/WASM95.42331의24쪽review+standaloneoverlay를직접열었고후속본문앞당김복원과그림/작은glyph잔여를확인했다. Rootpkg/StudioPublic의JS/WASM/d.ts4개각각총8파일hash를검증했으나실제browser초기화응답과actual92는미검증이다. 전체197/52와전체gates완료를대신하지않는다.

C305독립Print는2020엔진/6쪽으로완료했으며PDF SHA0f0ce6986b212f10cf8f3a8d51023607f0d56e0c49cbaa593004058319368dc8이다. 추가bitmap은PDF2쪽에서실제로출력됐지만6쪽deviceMatrices는C275와모두동일하다. 따라서단순raster출력이solidfill이후정밀도를복원한다는가설은기각하며production에image-reset분기를추가하지않는다. 기존기준PDF는교체하지않았다. 로드맵14쪽은아직미해결이고glyph원점/실제프로그램/출력context전환을더대조한다. 현재Cargo/capture/MCP작업은모두terminal이며소유중단한C297/C298/C300의범위도여전히최종재검증대상이다.


## C306–C309: 셀 BorderFill과 원본 도형 채우기의 Print 상태 구별

- C304 새 후보의 그림 host 점유 결과와 whole/partial 소비 보정은 집중 Native/fresh WASM 23–25쪽에서 통과했다. 전체 완료를 뜻하지 않는다.
- 로드맵 원본의 첫 색 셀은13쪽이며, 이후 페이지의 HFT paint transform이 사라진다. 그러나 독립 Print의13/14/31/68쪽에는 실제 색 채우기가 있고 CTM은 계속 정밀값 `0.119935/-0.119869`다.
- 독립6쪽 원본 HFT ASCII 대조군에 회색 원본 사각형 하나만 추가한C307은 실제 회색 도형 출력 및 첫쪽 정밀/후속5쪽 반올림을 확인했다. 동일 원문에 회색 빈 셀 배경만 추가한C308은 실제 회색 셀 출력 후6쪽 모두 정밀값을 유지했다. 글자 음영C275 및 그림C305와 별도로 구별했다. 입력/PDF·원문 및 글자모양은 보존했다.
- `border_fill_background_nodes → RectangleNode(원본 도형 control 소유 없음) → PaintOp::Rectangle → paper_viewport` 경로가 원본 도형 polygon과 같다고 취급한 것이 원인 후보이다. 현존 원본 control 소유를 사용해 semantic 이전쪽 검사와 현재 paint 검사에 같은 구별을 적용한다. 색/문서명/쪽번호로 발동하지 않는다.
- 증적은 ignored `rebase-6ed1682c-20261018/{roadmap-paper-state-c306,polygon-device-print-control-c307,cell-fill-device-print-control-c308,border-fill-paper-frame-c309}/`에 있다. 새 실제 회귀 보완은 전쪽 Native/fresh WASM 검증까지 잠정 상태이며 커밋하지 않았다.


### C311: 같은 셀 배경의2020/2024 인쇄 계약

C309 첫 보정은 로드맵 관계를 복구했으나 기존 rowbreak2024 대조군이 실패했다(실제5건4PASS1FAIL). 이를 기대값 변경으로 숨기지 않고 같은C308 입력 SHA를2024 엔진으로 인쇄했다. jobdeba3d62-8a1a-4b83-9c61-5cbe9bdf2dae, timeout1800, PrintMethod0,6쪽, PDF SHA8d9306acea51561f009a40198c7be5386f10d5441ec2fd95ade06d1a0446df98이다.2020은6쪽 모두 정밀 CTM이지만2024는 첫쪽 foreground 셀 이전 본문이 정밀값, 이후5쪽은 반올림값이다. 따라서 source 소유만으로 구분하는 첫 후보는 불충분했다. 명시 Print compat를 이전 semantic scan과 현재 paint collector 모두 전달해 같은 버전 계약을 소비하도록 보완했다. 원문·쪽수·이전 rowbreak 기대값·허용치는 유지한다.

C309 최초 suite 실행은 생성 harness가 오래되어0건/exit4였으며 증거에서 제외했다. `--prepare` 후 실제 수정 전5건4PASS1FAIL(.858초)을 얻었다. 보정 후 결과는별도 `after-public-corrected.log`에서 확인하며 전체 검증 완료로 재인용하지 않는다.


## 지침 분리와 임시 산출물 위치 정리 (2026-10-18)

사용자 요청에 따라 지침13개 파일을 최신devel6ed1682c32 위 `docs/7445-review-policy-20261018`에 분리하고 [PR #8016](https://github.com/edwardkim/rhwp/pull/8016)을 생성했다. 지침 head는d3b6f0cee9341a5398711f8f8c8d908ac8467c3f이다. 원 브랜치의78bfe501327b9f113e65e191acfadca6db9bae6a은 지침13개를 원base내용으로 복원한 문서 전용 커밋이며, 코드·사용자 초안 등207개 미커밋 파일의 SHA 보존을 확인했다. 사용자 지시의0.1%p 기준은 계속 적용한다. 정책PR 반영 후 원 작업을 동기화할 때13개 지침을 새upstream/devel과 같게 유지해 정책 되돌림을 코드PR에 재도입하지 않는다.

완료된 임시 진단 PDF38개(29192225 bytes)는 ignored `rebase-6ed1682c-20261018/temporary-artifact-relocation-c315/diagnostic-pdfs/`로 이동했다. 파일별 이전/현재 경로와 동일 SHA는 그 상위 `result.json`에 있다. 삭제0, 활성52조건 기준PDF와 원본 글꼴 생산 근거·영구 시각 asset은 보존했다. 과거 명령/producer의 PDF 경로는 실제 실행 당시 위치이며 현재 진단 재사용에는 이 이동 장부의 경로를 사용한다.

C310 focused Native16쪽은90% 이상이었으나 freshWASM42쪽은89.86563%로 미달했다. 원본302쪽수는양쪽 일치하지만 전체수용·회귀추가·PR준비 완료가 아니며, 후보는미커밋 상태로 추가보정이 필요하다. C313 독립PUA책괄호 대조군은F0854/F0855가 각괄호와다른좁은겹낫표 윤곽을 그리는것을 직접 확인했다. glyph추적의U+F000표기는 .notdef라는 판정근거가 아니며 실제PDF모양과font프로그램을 기준으로 진단을 계속한다.


### 2026-10-18 지침 PR #8016 및 별도 후속 기록 #8024 처리

- #8016 head `d3b6f0cee9341a5398711f8f8c8d908ac8467c3f`의 CI 성공 후 정상 squash `a93b68586dc368e311a5220c52d016b1ad754995`로 병합했다. Adapter·Proptest는 root 지침 allowlist 차이로 실제 실행됐으며 fast-pass로 오기하지 않았다.
- 승인된 옵션 2의 별도 후속 기록 #8024 head `15eba652a7500715d09dd12c9e6318818c0f4bde`는 문서 fast-pass와 최종 aggregate 성공 후 `224a326063a5aeab5a1ca8dcaa92918f55843ffc`로 병합했다. 원 #8016 review·오늘할일을 devel에 반영했고 local devel도 같은 SHA로 fast-forward했다.
- 양 병합의 duration workflow는 success지만 `ready=false / no-verified-pr-duration-measurements`로 갱신 보류다. 검증 CI 재실행은 하지 않았다.
- 계획에 따른 원 PR 댓글: https://github.com/edwardkim/rhwp/pull/8016#issuecomment-6247614086 . exact merge SHA 고정 review 링크와 실제 CI·검증·정리 결과를 게시하고 API 본문을 재확인했다. #7445는 OPEN을 유지했다.
- 전용 작업장 `/Users/tsjang/rhwp-7445-policy-20261018` 및 local/upstream `docs/7445-review-policy-20261018`, `docs/pr-8016-followup-20261018`를 정리했다. 원격 최종 SHA lease를 사용했고 부재를 확인했다. 기본 작업장과 공유 target은 보존했다.
- 정책이 upstream에 반영됐으므로 보정 브랜치의 정책 전용 3커밋(50fe2ee3e3/e84b0369f3/78bfe50132)을 제외하고 261커밋을 최신 base `224a326063a5aeab5a1ca8dcaa92918f55843ffc`로 재배치했다. 새 HEAD `b80662087571f746579350d2ef00f3387b54b527`, 충돌0. 제품 committed tree 불변 및 미커밋169파일 SHA 동일·사용자 초안 보존·지침13개 upstream 일치를 확인했다. 보존 stash `7a7b7eb9bfd27d96da077fb4b05ee154af11f6a7`와 backup ref를 유지했다.
- `/Users/tsjang/rhwp/output`은 활성 #7445 원본·글꼴·동결 producer·PNG/TSV 및 baseline 작업장의 참조가 있어 전체 정리 **미완료·보존**이다. 이번 완료는 문서 PR 후속 처리 범위이며 C310 후보의 WASM roadmap p42 미달·기존 devel 하락·전체 회귀/실소비 검증 잔여를 해소한 것으로 주장하지 않는다.
- 실행 정본은 ignored `output/pr-review/7445-completed-20261009/rebase-6ed1682c-20261018/policy-split-20261018/`의 `followup-ci-final.json`, `devel-sync.json`, `owned-cleanup-result.json`, `source-pr-comment-final.json`, `code-branch-policy-rebase-result.json`이다. .log 및 중간 JSON은 커밋하지 않는다.


### C316–C319: 원본 기호와 첫 저장 표 프레임의 실제 소비 추적

C316은 독립 HFT 대조군의 F0854/F0855가 《》와 다른 흰 겹낫표 모양이고 PDF Haansoft Batang CID 전진이0.5em임을 확인했다. 리터럴 《》·『』는1em이다. local 원본 폰트 metadata와 PDF의 수치는 구분하며, 아직 production 매핑/기존 회귀 기대값을 변경하지 않았다.

C317에서 roadmap42의 표 앞 본문 위치는 base/후보 동일하지만 host892 첫 표 상단이 base616.2/후보612.5CSSpx로3.7px 차이였다. 원본 호스트 vpos35984HU, 개체 offset30HU, outer_top283HU 및 현재 Column 원점으로 기대 top616.2266667을 독립 선언에서 계산했다. 한컴 Print 괘선 y615.6466CSSpx와 정밀 종이 축척도 대조했다. 글꼴 획 차이만으로 설명할 수 없는 배치 회귀다.

C318 공개 관계 검사를 기존5개 중 첫 case에 보완했다. 수정 전5실행4PASS1FAIL(0.938초): actual612.4533333, source616.2266667. 원인은 source_single_cell_first_placement가 인증된 top에서 outer_top을 빼 새 계획을 만들었지만, 여백 소비 flag는 교체되기 전 host 계획에서 가져와 뒤 단계가 여백을 되돌리지 않은 것이다. source Query의 최종 테두리 top을 그대로 발행하고 그 계획이 여백을 소유한다고 표시해 예산·emit·paint에 같은 원점을 전달한다. 문서/쪽/283HU 전용 production 분기는 없다. 수정 후5PASS(0.924초). 새 실물 관계 검사도 전쪽 Native/fresh WASM 선행조건 전에는 커밋하지 않는다.

C319는 현재 parent b806620875와 전체 source hash를 고정하여 관련 정상·row-cut·unit-tier·fmt, 새 Native/fresh WASM 및 roadmap16쪽을 순차 검증한다. 빌드8·test8·capture총8·shared flock을 유지한다. 집중검증을 전체52조건·전체nextest·actual92 완료로 대신하지 않는다.


### C319–C326: 전쪽 검증과 실제 실행 범위 감사

- C319 집중 Native/fresh WASM16쪽의42쪽은99.97096/99.96016%이며 양쪽 review/standalone overlay를 직접 판독했다. 첫 표의 위 여백 소유 보정이 원점을 복구했고 내용/앞뒤 소속을 유지했다. 아직 이를 전체52조건 완료로 쓰지 않는다.
- C319 정상 대조군은46건 실제PASS이나, rustfmt가 source byte weight를 바꾼 뒤 `--prepare`를 갱신하지 않아 요청10그룹 중5그룹만 실행됐다. `normal-controls-coverage-audit.json`에 누락을 기록했다. C322 전체 실행의1557개 source module이 누락/추가/중복 없이 포함됨을 별도 감사했고, 실제 로그에서 요청10그룹60건 모두PASS를 확인해 이 후보의 집중 검증 누락을 보완했다. 다음 정식 실행 전에 generated suite를 다시 만든다.
- C322 전체 nextest:10945실행10944PASS1FAIL45skip(실행730.026초, 컴파일별도). 남은FAIL은 `render_p23_pdf_export_contract::document_core_pdf_export_preserves_page_range_errors`:요청99 대신 앞쪽 이력 조사 중PageOutOfRange(1)를 반환했다. 기존 기대99는 유지하며 API 입구에서 요청 범위를 먼저 확인할 구현 보정 대상이다. 새 후보 전체통과로 쓰지 않는다.
- C321 Native302/302쪽, 최저91.00093%,90미만0. fresh WASM은130–145묶음에서HCR Dotum의FontFace 적재가 실패하여 전쪽 미완료다. 같은HANDotum.ttf(SHA1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898)를 직전114–129묶음8worker에서 성공적으로 적재한 기록과 현재 fontTools 구조를 확인했다. C325는완료1–129 PNG만 hash와 batch완료 증명을 확인하여 재사용하고 실패 묶음의 일부 PNG는 배제했다. 같은글꼴/도구/8worker로130–145 재캡처가 성공했으며 뒤쪽을 계속 수집한다. fallback/guard 우회 없음; 최초 실패 원인을 파일 손상으로 단정하지 않는다.
- C323은 같은16쪽의 동일PDF/PNG를 pdftoppm과 같은Poppler의Cairo로 읽기전용 비교했다.62쪽 Native base/후보는Splash99.34535/95.40988%,Cairo95.65224/99.09707%다. Cairo의후보review/overlay를 직접 판독했다.2쪽은Cairo에서도하락하고 책괄호/뒤 본문 실제결함이 있다. C320동일독립glyph의Type3/동일vector 대조와 공식Splash26.04.0의캐시 정수원점 처리도 확인했으나, canonical/gate/지침은 변경하지 않았다. 구체안 `pdf-rasterizer-controls-c323/proposal.md`에 대한사용자 판단을 요청했고 독립 구현 보정은 계속한다.
- C324는기존#6057원본29494.hwp를engine2020/Print0/preprocessnone으로 독립출력(job2bd1ce6e-c057-4ff1-a2fb-a4bbec6e7314,2쪽)했다.원본PUA F0854/F0855의Haansoft Batang 전진은0.5em이며현재후보는1em으로측정한다.C313의흰겹낫표와일치한다. 기존《》매핑/전각기대는아직바꾸지않고같은문서전쪽Native/freshWASM 및대조군으로검증한다.
- C326첫교차스타일입력은namespace접두어를잘못찾아치환0건으로INVALID 처리했다. job11ae4f81-6da2-457b-8439-5d7c5bb699f4는사용하지않는다.별도v2입력은24textnode만교체하고모든element/attribute/font/header를보존했음을확인한뒤독립Print를요청했다.


### C327–C337: 독립 책괄호 기대와 배포 글꼴 소비 보정

- 사용자는 C323의 Cairo 전환 제안을 거절하고 기존 `pdftoppm` 기준을 유지하도록 지시했다. 제안은 적용하지 않았고 별도 정책 PR을 만들지 않는다. raw90%, devel 대비0.1%p 및 쪽수·내용·확대 판독 조건을 유지한다.
- C325는 같은 C319 frozen source/글꼴로 WASM 전302쪽을 완료했다. 완료1–129쪽만 hash/완료 receipt로 재사용하고 실패130–145쪽은 재캡처했다. 최저91.00093%,90미만0이나 devel 대비0.1%p 이상 하락12쪽이 남는다. 이는 후속 C327의 새 전302쪽 결과가 아니다.
- C327은 요청 페이지 자체를 먼저 검증해 PDF export의 `PageOutOfRange(99)` 계약을 복구했다. C322 전체10945건 중1FAIL의 원인이었던 선행 Print 페이지 순회가 더 이른 없는1쪽 오류로 요청99를 덮지 않는다. 기존 오류 기대값은 유지했다. 관련5건과 정상 대조60건, 합65PASS; fmt·Native Skia CLI·fresh WASM locked web/no-opt 빌드 PASS다. 전체 회귀 완료로 대신하지 않는다.
- C324 독립 원본#6057 Print의 Haansoft Batang A854/A855 윤곽과 원본 프로그램을 대조해 책괄호는 반각 흰 겹낫표임을 확인했다. 공통 PUA 측정0.5em, 표시『』, SVG textLength 소비를 일치시켰다. 원래 Unicode『』·《》의 원문 정체와 자연 폭은 유지한다. qualified PDF는 `pdf/issue6057-29494-2020.pdf`다. 사유 없이 원본 파일을 대체하지 않았다.
- C327 #6057 전2쪽 Native 최저99.81353%·fresh WASM99.82029%, 쪽수2/2·미달0이다. 양 backend 전2쪽의 review/overlay를 직접 열어 법명 책괄호와 뒤 제10, 다음 쪽 본문·절·마지막 문단의 소속·보존을 확인했다. 미세 AA와 굵은 제목 획 차이는 남는다. 로드맵 선택2쪽은92.59897→99.82206% 양쪽 개선,42쪽은99.97096/99.96016%,62쪽은95.40988/95.40775%로 아직 pdftoppm 기준 하락을 해결하지 못했다. 선택3쪽을 전302쪽 수용으로 쓰지 않는다.
- C332는 기존#6057 전각《》 기대와 하드코딩16px를 독립 흰 겹낫표·원본 글자 크기 관계로 교정하고, 양쪽99.95%인1쪽의 독립 PDF 원점(pt×96/72)을±2CSSpx로 보호한다. 책괄호 한 쌍·순서·뒤 한글의 비겹침을 함께 검사한다. 수정 전 PUA producer 두 파일만 안전 보존·임시 복원한 실제 실행에서2FAIL, 현재 소스는 byte hash대로 복원한 뒤 후속 검증 중이다. 이 임시 재현은 다른 WIP나 API 수정은 되돌리지 않았다.
- C331의 W1 source drift 검사는 정당한 소스 변경 경로를12개 상수 목록에 묶은 낡은 기대 때문에 실패했다. C332는 historical ledger를 갱신하지 않고 파일 SHA를 독립 계산하며 의도적 digest 변조1건·절대 경로 비노출·snapshot 불변을 검증한다. 단독12PASS, 후속 전체 글꼴 계약에도 포함했다.
- C328 Studio/editor/shared Node 검사는2371건 중2369PASS·2SKIP·0FAIL이다. C333 Studio build와 VSCode compile은 성공했지만 배포 파일 대조에서 VSCode의 새 공개 바탕 WOFF2 누락을 검출했다. C334는 webpack 포장 목록, CanvasKit 준비 가능 목록, Canvas2D `@font-face` 공급을 같은 공개 파일로 연결했다. 기존 배포 검사에 필수 공개 파일을 포함하자 수정 전 실제1FAIL, 보정·compile 뒤 글꼴 계약58건 중57PASS·1SKIP·0FAIL이다. Studio dist와 VSCode dist의 공개 WOFF2 SHA는 원본20d454f45cdf29cc4619e769f1f59f3b8920941b6b274a530e82d3b75e24dbfb와 같다. VSCode 포장 WASM도 C327과 같다. 실제 webview 초기화 소비는 아직 별도다.
- C335의 합성 HFT3face/2size/4style 대조는 전3쪽 Native/fresh WASM 모두 미달이다. 실제 그림은 비례 sans 모양과 추가 줄바꿈이며 독립 Print의 고정 HFT Latin과 다르다. 완전 generic HFT Latin 프로그램이 공급 목록에 없다는 점은 확인했지만 실제 선택 face의 trace는 없어, 대체 선택이라는 원인은 추론으로 남긴다. 해당 대조를 전체 통과나 새 회귀 추가 근거로 쓰지 않는다. C326의 독립 PUA0.5em 관측과 C327 실제#6057 전2쪽 수용 범위는 분리한다. 진단 입력/PNG/TSV는 ignored output에 보존한다.
- C336은 C327 frozen engine의52조건 Native/fresh WASM 전쪽 재수집, C337은 C332의 안전 복원/집중 검사 완료 뒤 현재 source fingerprint의 전체 nextest·lint·build·Skia·manifest/unit tier를 순차 실행하도록 준비했다. 이전 C322 실패나 C319/C325 완료를 새 후보 전체 통과로 재인용하지 않는다. 실제92경로와 남은 devel 하락 보정·전쪽 직접 판독도 남아 PR 준비 완료가 아니다.

로그·TSV·중간 JSON은 `output/pr-review/7445-completed-20261009/rebase-6ed1682c-20261018/`의 각 C327/C332/C334/C335/C336/C337 폴더에 보존한다. 지침·원격·원본·사용자 초안은 변경하지 않았다.

C332 후속 집중 검사는 실제11건 모두PASS(책괄호2·#8264·PDF export5), 소스 복원 SHA 일치·fmt PASS로 완료했다. 신규 px 보호의 전쪽 TSV·양 backend 전2쪽 review/overlay·독립 기대 근거는 `../pr/assets/issue7445/completed-20261009/regression-policy-20261015/pua-book-brackets-20261018/README.md`에 보존했다. C336 전쪽과 C337 전체 검증은 실제 실행 중이며 C338 actual92는 현 소스·관측 도구 provenance를 고정하고 게이트 완료 대기 중이다.

C337의 새 source fingerprint 전체 nextest는10945실행·10945PASS·0FAIL·45SKIP(실행631.162초)로 완료했다. C322의 요청99 페이지 오류도정상PASS이며, 뒤 lint/build/Skia 게이트와 C336 전쪽·C338 actual92·직접 판독/미세 차이는 아직 별도다. C339은 완료4문서의 Native8쪽 review+overlay를 직접 판독해 본문/행/최종문단 보존과 수식 간격·일부 테두리/첫 이어받기 줄의 잔여를 개별 기록했다. 특히 giant 원본76쪽의 수식 앞 미세 표시와 첨자 간격은 점수93.29536만으로 완전 수용하지 않고 원본의 인접 수식 스크립트/선언 크기를 조사 중이다. 문서 링크 검사 정본도종료0·상대 링크 정상, 추가exact문서/등록부검사도0오류를 확인했다.


### C341–C343: 실제 공개 글꼴 소비와 현재 하락 쪽 직접 판독

- C341은 실제 `HwpEditorProvider.getHtml()`이 만든 webview HTML/CSP와 포장된 viewer·WASM·WOFF2를 브라우저에서 초기화했다. ready 신호, 실제 Canvas 잉크1691픽셀, CDP의 `RHWP Public Batang` custom font17glyph 선택을 확인했다. 초기 요청의 viewer SHA6927f1fd…, WASM c4258bb6…, WOFF2 20d454f4…가 현재 포장 파일과 일치한다. 이는 webview HTML/배포 자원의 실제 소비 smoke이며 VSCode 앱 전체·문서별 backend 수용은 아니다. 첫 harness는 TS7 API 차이, 두 번째는20MB WASM 응답의 inspector buffer eviction으로 실패해 각각 보존했고, 세 번째는 초기화 전에 관측 buffer만 늘려 성공했다. production readiness·paint guard를 바꾸거나 사후 fetch로 초기 hash를 대신하지 않았다.
- C342 사용자 요청에 따라 실제 HBATANG.TTF(한컴바탕)·HANBatang.ttf(함초롬바탕)·공개 TTF를 읽기 전용 비교했다. 한컴바탕 SHA bef63718…와 공개7e3aee5e…는 다르다. 공통 Unicode20577자 중 분해 윤곽 명령11500자가 같고, 현대 한글11172자 중11171자 윤곽과 전부의 전진폭이 같다. `볼`은 실제 윤곽 경계도 다르며 영문·숫자·괄호까지 동일한 글꼴은 아니다. 공식 Google Fonts 원본0929031e…를 다시 받아 비교하면 공개본과 glyf/loca/hmtx/cmap/힌팅/GSUB 등15개 raw table은 모두 같고 name/head만 다르다. 윤곽 명령 동일성은 화면의 픽셀 일치율 또는 권리 확인으로 쓰지 않는다. Downloads의 Batang.ttf는 내부 김포평화바탕이므로 파일명만으로 공식 원본이라고 취급하지 않았다.
- C343은 현재 C336 Native 전302쪽에서 하락9/54/62/92쪽의 review와 standalone overlay를 만들고 모두 직접 열었다. 그림 목차·본문·캡션·차트의 내용/순서를 확인했으나, glyph/leader 가장자리·일부 차트 가로줄 차이와 devel 하락은 남는다. 62쪽95.40988%/하락3.93547%p,92쪽95.12860%/하락1.26737%p를 통과로 숨기지 않는다. 소스 글리프와 rasterizer 원인 대조를 이어가며 pdftoppm·90%·0.1%p 정책은 유지한다. 현 C336의 주 경로는 Native `export-png`(native-skia) 및 fresh WASM Print Canvas2D 직접 PNG이며, SVG raster 검증으로 오기하지 않는다. 전체 actual92 중 추가 profile·표시/선택 소비 범위는 별도다.

이 회차 증적은 ignored `font-identity-comparison-c342/answer-evidence.json`, `vscode-font-consumer-c341/result.json`, `current-roadmap-review-c343/producer.json`과 각 review/overlay다. 공유 target·사용자 초안·원본·지침·원격은 변경하지 않았다.


### C344–C351: 전쪽 출처·소속 감사와 raw 하락 위치 분리

- C344는 C301의 이전 직접 판독9쪽과 현재 C336의 해당 Native PNG를 각각 SHA로 대조해 완전히 같은 경우에만 판독 메모를 재사용했다. C336 자체는 전쪽을 다시 렌더했으며 이전 TSV/PNG 재사용 실행이 아니다. 6ed→224a326의15개 변경 경로는 지침/검토/오늘할일이며 runtime/assets/tools/input 트리는 동일하다. C171 baseline은 실제6ed 실행으로 남기고224 실행으로 고쳐 쓰지 않는다. 추가 현재5쪽(sample11의6/33/117, sample16의40, 개인정보4)의 review/overlay도 직접 열어 잔여를 기록했다.
- C345는 현행 HFT 공급 문서39/88행에 따라 글꼴 복원 표본 PDF6개 경로만 공유 `.git/info/exclude`에 추가했다. 이전 exclude를 byte 그대로 백업했고 PDF8개의 파일 바이트·원본·검증 입력은 보존했다. 6개 합계9,619,636bytes다. 이 적용은 기존 지침의 실행이며 권리 확인을 뜻하지 않는다. 사용자의 문구 출처 질문에는 최초84764fa2/#7941 도입과#8016의0.1 변경을 구분했다.
- C346 Native 완료 감사:52조건1964쪽, 모든 입력/TSV hash·쪽수·PNG의 누락/중복을 확인했다. 최저90.08879%,90미만0이나 로드맵12쪽의0.1%p 이상 하락이 남는다. fresh WASM은 진행 중이며 완료된 같은 조건의 본문/문단/개체 소속 순서만 순차 비교한다. context로 증명된 Native usize64/WASM usize32 합성 Header/Footer sentinel만 정규화하고 실제 본문 소속/순서는 보존한다. 양 backend의 일치는 독립 PDF에 대한 내용 정확성의 대체가 아니다.
- C347 현재 Native giant 분할1/24/90/197쪽, 영수증7017의1쪽, 새 field01의1쪽 review/overlay를 모두 열었다. 전체 내용·순서와 최종 종료 문단을 확인했으며 giant90쪽의140°C/180°C 문단은 독립 PDF와 줄 경계가 달라 해당 경계를 px로 고정하지 않는다. 실제 raw는 giant1=99.87848,24=95.82844,90=91.21712,197=99.63163%, 영수증98.12008%,field99.99908%다. 반올림100을 픽셀 동일이라고 쓰지 않는다.
- C348 독립 전체 한글 표본의11177glyph는 전부 단일 nonzero fill(f)임을 확인했다. 96개 균등 표본의12/16CSSpx·장평95%·원점0/0.5에서 Type3와 동일 literal path의 pdftoppm2px raw는4쌍 모두100%다. 단순 Type3/cache 차이만으로 현재 로드맵 하락의 원인이 확정됐다고 쓰지 않는다. 비교 PDF는 별도 진단이며 정본·지침·수용 문턱을 바꾸지 않았다.
- C349는 canonical WASM exporter의 ignored 관측 복사본으로62쪽 실제 Canvas.fillText('파')의 descriptor/변환을 기록했다. PNG SHA377dabd9…는 C327 정본과 완전히 같다. 같은 descriptor와 실제 FontFaceSet을 쓴 별도 DOM probe는 HYSinMyeongJo-Medium/H2mjsM custom1glyph를 선택했다. 실제 primary 한양신명조 프로그램은 기호 overlay이고 한글은 실제 공급 사다리를 사용한다. 첫 probe는 viewport 밖 DOM이 아직 배치되지 않아 빈 font 목록이었으며 별도 보존했다. 두 번째는 PNG 인코딩 뒤 probe만 보이게 하고 배치 대기해 관측을 완료했다. Native 실제 renderer trace 또는 DOM probe를 Canvas 내부 선택 API라고 주장하지 않는다.
- C350은 현재 raw 식을 그대로 사용해 미해결 픽셀 위치만 읽기 전용으로 집계했다.62쪽의2868개 중1981개는 양쪽 유사한 밝은 선의232 이진화 경계에 걸리고, 상위5개 행167/199/229/260/292에2163개가 모인다.54/92쪽도 차트 가로 격자선에 많은 차이가 있다. 본문 글꼴만 원인으로 단정하지 않고 source bitmap·원래 배치·축소 sampling 소비를 추적한다. 이 위치 분류로 gate에서 픽셀을 빼거나 허용치를 바꾸지 않는다. C351은 같은 C327/입력의 원본 그림 bytes와 Print 그림 메타데이터 대조를 준비했다.

실제 증적은 ignored `evidence-reuse-audit-c344/`, `submission-local-only-c345/`, `current-content-owner-audit-c346/`, `current-boundary-review-c347/`, `type3-density-diagnostic-c348/`, `actual-font-selection-c349/`, `raw-mismatch-localization-c350/`, `roadmap-source-image-audit-c351/`이다. 전체 nextest10945PASS와 이후 게이트/전쪽/실제 소비 경로의 완료 여부는 구분하며, 새 source나 head의 최종 전체 수용은 아직 아니다.


### C352–C359: 인쇄 비트맵 구간과 물리 원점의 공통 보정

- C337 수정 전 source의 필수 게이트33개가 모두 종료0으로 완료했다. 전체nextest10945PASS/0FAIL/45SKIP, release-lib root3804PASS/12ignore 및내부lib182PASS, Native Skia root3861PASS/12ignore 및내부lib182PASS, PNG2PASS/PDF14PASS, scoped Native feature/문서/manifest/글꼴 도구 검사까지 각 로그에 보존했다. 이는 C358 이후 소스 통과가 아니다.
- C336은 Native52조건1964쪽과 fresh WASM13조건을 완료한 뒤 C357의 배타 lock 안에서 다음 regulatory17 첫 묶음 대기 상태를 확인해 소유 driver만 SIGTERM했다. Native52/미달0/로드맵12하락과 완료WASM13의 결과·PNG·TSV는 보존했으며 fresh52 완료로 쓰지 않는다. child 캡처는 없고 resource_tracker만 남은 경계를 확인했다. C346은 이 의도적 종료를 감지해 `Whole process missing`으로 종료했으며 완료13쌍의 내용 소속 순서 일치는 부분 증거로 남긴다. C338 actual92는 아직 캡처0인 대기 상태에서 같은 수정 필요를 기록하고 종료했다. 새 소스에서 다시 수행해야 한다.
- C351의 실제 원본719×288 PNG와 Print에 내장된 같은 크기JPEG의 동일 해상도 raw는99.65932/99.71123%다. 이것만으로62쪽 페이지 하락3.9%p를 설명하지 않는다. source와PDF의 회색 가로선 픽셀을 대조하니 source행을1/2개씩 장치 구간에 나눈 평균이 실제PDF7줄과0–1gray 이내로 맞는다. 현재 Lanczos 재샘플링과 최근접 목적 경계가 다른 소속/밝기를 만든다.
- C352는 대상 문서와 무관한719×288 회색 격자·세 소수 원점의 한컴2020 Print0/1800초 대조다(job13e03742…,PDF79200b0d…). base99.18915/현후보100%,review/overlay를 직접 판독했다. C354는 같은 격자의 아래6source행에만256색 초과 strip을 넣어 Print가Flate→DCT로 바뀜을 입증했지만 base99.43436/현후보100%이므로 JPEG만 원인이라고 단정하지 않는다. C356은208–230의 균등한 옅은1픽셀 격자와 비등방 축소를 독립 생성했다(job52a3b26a…,PDF094cd94d…). base48.28756/현후보46.93115%로 반례를 재현했으며 미달 표본의 배치를 새 회귀로 추가하지 않았다.
- C357은 source 픽셀이 정확히 한 정수 구간에 속하는 평균과600dpi 물리 프레임으로 독립 계산 모델을 만들었다. 처음 최근접 장치 원점 가정은C35670.35768%로 미달, 평균의 반올림 변경도57.68090%로 채택하지 않았다. 독립Print의 세 그림 원점을 각각 대조해 장치 원점은ceiling, 크기는nearest임을 확인한 모델은 C352/C354/C356 모두 raw100%다. 이는 실제Native/WASM 구현 결과가 아니다. Poppler26.04의 공식 source도 읽기 전용으로 확인했으며 구현 코드를 저장소에 복사하지 않았다. 대상PDF에서 글꼴/그림을 복원해 product에 넣지 않는다.
- C358은 `RenderNode`의 원본 bbox→`PaintOp::Image`→최종 replay plane의 물리 원점/크기→공통 준비 bitmap→Native/WebCanvas/CanvasKit의1:1 draw를 연결한다. 최종 이미지는 선행 polygon/shade 상태를 glyph와 같은 순서로 소비하고 원본 layout bbox/crop/source bytes는 유지한다. Print의 정수 source구간 평균과 덮이는 장치 경계를 공유하고 HighQuality의Lanczos 및 회전 그림의 기존 프레임은 유지한다. cache key도Print/HQ를 구분한다. 수정한3파일의 이전WIP는 `print-bitmap-repair-c358/before/`에 hash와 함께 보존했다. 정규 기대값/지침 변경은 아직 없다. fmt·manifest-prepare 종료0이며 새 Native CLI 빌드가 실제 진행 중이다.
- C359는 새 Native producer 완료 뒤 독립3표본 전쪽과 로드맵9/54/62/92/153, 원본sample11의2/50, 원본sample16의6/33, 저장sample16의18/35, giant24/90,asset14/40,영수증1의 집중 캡처·정본review/overlay를 실행하도록 준비했다. 독립 계산 모델의100%를 실제 출력 수용으로 대신하지 않는다. 이후freshWASM/새전체52·필수게이트/실제92/정식 회귀 보완이 남는다.
- C353 종료Chrome8개 프로필249,401,000bytes는 PID/실제명령/Chrome상태 파일을 확인하고 hash원장 후 삭제했다. 증적 디렉터리와 활성 프로필은 제외했다. C355는 종료된8개 검증 폴더의 immutable `font_resources` 복사본2340개를 전부 hash 확인해39개 byte객체로 hardlink 공유했다. 중복2301개·추정17,041,178,624bytes(15.87GiB)를 확보했고 모든 원래 경로/바이트는 유지했다. 현재 capture·글꼴 정본·원본/PDF/PNG/TSV/log/공유target·다른 작업장은 변경하지 않았다.

C352 harness의 최초 괄호 문법 오류와 JSON bytes 직렬화 오류는 실행/다운로드 증적을 보존한 뒤 수정했다. 기존 같은 job/PDF를 재사용하고 변환을 중복 시작하지 않았다. 대표 이미지 생성 때 단일PDF의 정본 이름 `pdf-1.png`를 전쪽 집계 이름으로 잘못 가정한 오류도 실제 경로를 확인해 수정했다. 이 실패 실행을 통과 증거로 쓰지 않는다. 새 증적은 ignored C352–C359 경로에 있고 사용자 초안·HEAD·지침·원격은 그대로다.


## C360–C373: Print 그림 구간 확정과 독립 ASCII 공급 대조

- C358의 glyph shade/polygon 반올림 상태를 이미지에 전달한 가정은 잘못됐다. 독립 C305의 shade 뒤16×16 checker는 정밀 raster CTM이며, C358은 실제153쪽89.40041%로 실패했다. C360은 `builder.rs`를 C337 바이트로 복구하고 Print raster만 정밀600dpi 프레임을 소비하게 했다. 실제 추가 production 변경은 `bitmap_raster.rs`와 `paper_viewport.rs` 두 파일이다. source/layout bbox는 유지한다.
- C360 Native release-test/native-skia 빌드 및 locked `--target web --out-dir pkg --no-opt` fresh WASM은 종료0이다. Docker 최적화 통과가 아니다. root pkg/Studio public4파일 hash를 기록했으며 실제 브라우저 초기화 응답·actual92는 아직 남는다.
- C361 Native 집중10조건19쪽과 C365 fresh WASM9조건14쪽 모두90미만0이다. 독립 C352/C354/C356 세 입력은 양쪽100%다. 집중 결과를 전52조건 수용으로 쓰지 않는다.
- C366은 C360로 로드맵 Native302/302·fresh WASM302/302를 전쪽 새로 출력했다. 최저는96.69794/96.65770%,90미만0,153쪽은 양쪽98.66795%다. 이전 C327의12개 Native 하락 중9개는 해소됐으나9/10/11쪽의−0.13044/−0.14554/−0.47681%p는 남았다. WASM 하락은0이다. baseline6ed/C171은 C344 runtime tree 동일성으로224까지 재사용한 것이며224를 다시 출력했다고 쓰지 않는다. 기준299쪽의 뒤300–302는 짝 없음으로 명시한다.
- C366의 TSV1..302·원본/기준 PNG302개씩·중복/누락0을 확인했다. 정본 review/standalone overlay를 추가 생성해 Native153/62/152 및 WASM153/62/263을 직접 열었다.153쪽의 다섯 내부 격자·점·추세선·축·출처/제목·쪽번호 소속을 확인했다. 미세한 JPEG 명암/문자 AA 차이는 남는다. 실제 생성한 전302쪽 Native/WASM render tree의 본문·노드 type·소유·순서는 C190의 문서화된 architecture sentinel 정규화 후 불일치0이다. 이 비교는 PDF 내용 정확성 전체를 대신하지 않는다.
- C363은 기존 `reduced_scatter_plot_preserves_every_interior_grid_band`가 모든 선에RGB232 이하90%를 주장한 것을 독립 PDF로 감사했다. 원본303행의 PDF 자체가198/272열(72.8%)이므로 기대값이 틀렸다. C366 전쪽과153쪽 직접 판독 후 C369에서 독립 PDF의5/95분위수에 맞춘 각 선의 명암·연속성으로 정정했다. 네 선의 상한218/218/221/222는232보다 엄격하고, 옅은 선만238을 쓰며 중앙값 하한으로 검게 덮는 오답도 거부한다. 원본에서7개 행을 검출해 외곽2/내부5 소속을 유지하고, 독립 PDF의 그림 네 변을 ±2CSSpx로 함께 보호한다. 한컴2020 원본에 맞게 compat2022를 쓴다.
- C367 첫 검증은 제가 PDF `(x,y,width,height)`의 너비/높이를 right/bottom으로 잘못 읽은 테스트 작성 오류여서 before/after 모두 동일 경계 assertion으로 실패했다. `INVALID-expectation.txt`로 보존하며 결함 재현으로 쓰지 않는다. 정정한 C369는 같은 검사/입력으로 두 production 파일만 C337 이전 바이트로 대조했다. 수정 전2건1PASS1FAIL(원본87행0/277열), 수정 후2PASS(0.871초)이며 두 파일을 C360 SHA로 `finally` 복구했다. sourceRestored:true 및 before/after 실제 nextest summary를 확인했다. 새 stable `bitmap-print-20261018/`에 독립 분위수·전쪽 TSV·N/W153 review/overlay와 근거를 보존했다. 기존17일 증적은 이력으로 유지한다.
- C362는 Native9쪽 leading `[`의 PDF-only x140/Native-only x144를 분리했다. source는 HFT 한양신명조·charShape180·상속 Hangul 슬롯이며 공급한 부분 CFF에는5개 기호만 있고ASCII가 없었다. HYSin TTF의같은 이름/유사 메트릭을 원 HFT 윤곽으로 간주하지 않는다. 단순600dpi glyph origin 양자화 진단은0.003%p 정도 개선뿐이라 채택하지 않았다. Native alias 순서가 원인이라는 가설도 실제 프로그램 공급과 구분하며 아직 그 source를 수정하지 않았다.
- C364 full-header 기반 독립 ASCII 표본 job `dc628f76-c62a-4349-bb80-fd9a868e5043`는 한컴서버1800초 timeout으로 실제 terminal failed다. 중복 job을 만들지 않고 종료 후 C368을 별도 입력으로 만들었다. C368은 canonical blank specimen builder와 uniform `가+ASCII+가` 전체94자, 원본 문서 본문/전체 header/개체/윤곽을 복사하지 않는 최소 패키지다. job `29a9e3fd-5cd9-49fd-a341-946ee0bb7a62`는 engine2020/Hancom11.0.0.9136/directDLLhost/Print0/hancom2020_pdf_driver_one_up/preprocess none로10.886초 성공했다. PDF8bfa5a53…는1쪽이며 실제 사용 T1 Type3의 `[`447a4a7b…와 `]`ca0b041e…가 대상 PDF glyph program과 일치한다. 이전 독립 표본의 사용되지 않은 빈Type3 ToUnicode 항목을 공급 원본으로 쓰지 않는다.
- C370은 그 독립 전체 visible ASCII94개와 기존 독립5기호 윤곽/전진을 보존한99glyph 부분CFF를 ignored 별도 경로에 만들었다. SHA `687110a537402be23b33235e3cc95b693afd29ad9a0614c502f1eab51bc97428`이다. 대상 정답 PDF 윤곽을 추출하지 않았고 기존 글꼴/원본/PDF/정책을 바꾸지 않았다. 이 글꼴과 표본은 로컬이며 제품/저장소에 포함하지 않았다. 실제 glyph 공급은 검증 조건이며 일반 Studio fallback과 별도다.
- C370의 동일 글꼴 진단은 base/current × Native/WASM × 로드맵9/10/11 및 독립 표본의8조건이다. Native current9/10/11은99.95804/99.50114/99.96011%, 같은 글꼴 base99.52224/99.09400/99.81464%보다 높다. 독립 표본 current Native/WASM 모두100%다. current Native/WASM9쪽 review/overlay도 직접 열어 이전leading bracket의 가로 어긋남 해소와 원문 항목·리더·끝 쪽번호의 소속을 확인했다. 집중 진단으로 전쪽 수용을 주장하지 않는다.
- C368 준비 때 잘못 고른 Python 환경에 pymupdf가 없어 입력 생성이 실패했고, 파일 부재 요청은 서버 job 생성 전에 실패했다. 실제 PRIMARY venv로 입력을 만든 뒤에만 위 새 job을 시작했다. C370은 Python 환경의 Brotli 결측과 구 base WASM selector API 결측을 서로 다른 원인으로 보존했다. 완성된 Native2조건은 유지하고 실제 종료를 확인한 미완료 WASM 폴더만 별도 보존한 뒤 재개했다. PRIMARY venv에 Brotli1.2.0을 uv로 설치했고, base WASM은 C171/C366과 동일한 explicit null 환경으로 구API를 정직하게 비교했다. 실제 font inventory는 양쪽 모두 같은99glyph 파일을 로드한다. 실패 실행을 통과 증거로 쓰지 않는다.
- C371은 C360+정정 C369의 source fingerprint를 고정해 현재 전체 nextest·lint/build/Skia/doc/manifest·범위 계약33게이트를 다시 실행 중이다. C337의10945PASS는 이전 소스 이력이며 C371 통과로 재인용하지 않는다. Cargo는 순차, jobs8/test threads8이다.
- C372는 종료된 C366/C365/C361의 생성 `font_resources`1128개를 hash 확인해39개 객체로 hardlink 공유했다. 원래 모든 경로와 바이트를 유지하며1089중복·8,010,158,080bytes(약7.46GiB)를 확보했다. original/system font·PNG/TSV/log/PDF·현재 C370 원font·C371/shared target은 보존했다.
- C373은 새 immutable 부분 글꼴을 base/current 양쪽에 똑같이 공급해52조건1964reference쪽 × Native/freshWASM의 전쪽을 새로 산출한다. 구 글꼴 PNG/TSV를 새 조건 증거로 재사용하지 않는다. 한 driver가 case/backend/role을 순차 실행하고 capture 총8 lock을 유지한다. 큰문서16쪽 묶음의 완료SVG만 SHA manifest 뒤 삭제하며 생성 font copy만 hash/inode 공유해 ENOSPC를 피한다. baseline의 실제 쪽수 불일치·짝 없는 쪽과 현재90미만·devel0.1%p 이상 하락을 모두 남긴다. 시작을 완료로 보고하지 않는다.

최신 source/head 전체 수용·전52조건·actual92·최종커밋·PR 직전 upstream 확인과 필요한 리베이스/재검증이 남는다. 사용자가 승인한 PR/CI/merge/옵션2후속 절차를 유지하며 지침 수정·새 예약·다른 task/subagent는 만들지 않았다. 사용자 초안 둘은 보호한다.


### C375 독립 재감사와 실행 체크포인트

- 원본 serialized HWPX의 `BinData/image54.bmp`(1000×514, SHA5fe1aec3…, RGB SHA d7453896…)와 독립 PDF 실제 이미지 bbox를 직접 읽어5/95분위수를 다시 산출했다. Paint 준비 과정의 PNG 변환 SHA1dfcc08d…와 BMP 파일 SHA를 같은 것으로 쓰지 않는다. 구현 resize helper나 후보 PNG를 기대값 산출에 사용하지 않았다.
- 이 재감사에서 C369 테스트에 제가 추가한 출력 중성색 필터가 C363의 원래 all-channel 명암 집계에는 없다는 차이를 확인했다. 중성색만 남기면 기준 JPEG의 색 번짐이 일부 선을 가린다. 원본 회색 행의 식별은 유지하되 출력의 불필요한 색 필터를 제거해 독립 PDF와 같은 조건으로 정정해야 한다. C369의 실제 FAIL/PASS는 보존하지만 이 조건 차이까지 검증 완료했다고 주장하지 않는다. C375에서 원래 all-channel 방식으로 재산출한 기대값215/218,215/218,215/221,224/238,215/222는 정확히 재현됐다.
- 이미 실행 중인 전체 nextest를 중단하지 않았다. C371 순차 driver PID97310만 SIGSTOP하고 child nextest4327은 계속 실행한다. 종료 요약/exit를 보존한 뒤 Native-only 테스트를 정정하고 같은 전후 재현 및 최신 소스 게이트를 수행한다. 이 프로세스 체크포인트는 사용자 목표/작업을 PAUSED로 변경한 것이 아니다. `intentional-parent-checkpoint.json`에 근거를 남겼다. driver를 잊어버린 채 다음 Cargo를 시작하지 않는다.
- C373의 동일 새 글꼴 Native 로드맵은 base299쪽(97쪽부터 페이지 불일치 영향·미달 보존), current302쪽/최저96.69755%/90미만0/devel0.1%p 이상 하락0으로 완료됐다. fresh WASM과 나머지51조건은 계속 진행 중이다. baseline 자체 미달·299 vs302를 정상302쪽 baseline으로 바꾸어 쓰지 않는다.
- 최신 사용자 지시는 **PR은 승인받고 진행**이다. 기존 PR 자동 생성 승인을 적용하지 않고 로컬 보정·검증·구체적 제출물을 준비한 뒤 승인을 요청한다. 현재 열린 동일 head PR은 없고 #7871은CLOSED다. 원격 push/PR 생성은 아직 수행하지 않았다.


### C376–C378 최종 조건과 소비 관측

- C375의 all-channel 조건과 맞춘 최종 C376은 실제2건1PASS/1FAIL(87행0/277열)→2PASS(0.746초), production 두 파일 byte-exact 복구 완료다. C369는 이전 색 조건 버전으로 구분한다. 두 로그의 실제 summary와 `whole-resumed.json`을 직접 확인했다.
- C371 full-nextest는10,945PASS/0FAIL/45skip,671.873초·exit0을 보존했다. 테스트가 끝난 뒤에만 Native-only 검사 조건을 수정하고 정지했던 순차 driver를 재개했다. strict source guard가 의도한 checkpoint에서 다음 게이트 진행을 막았다. 이를 테스트 회귀 실패로 보고하지 않는다. 마지막 검사 소스 전체33게이트는 C377로 다시 실행 중이다. 이전 성공을 최종 소스 통과로 갈음하지 않는다.
- C376의 전후 재현 동안 C373 dispatcher만 안전하게 정지해 source guard와 충돌하지 않도록 했다. 실제 frozen CLI/pkg를 바꾸지 않았고, 두 production 파일의 현재 바이트 복구 후 `finally`에서 SIGCONT했다. 지금 C373은 실제 프로세스가 살아 전쪽 캡처를 계속한다. 목표를 PAUSED로 변경하지 않았다.
- C374는 current Native library를 read-only 링크한 renderer snapshot 관측이다.9쪽 괄호60개에서 이전5기호 공급은 `HYSinMyeongJo-Medium/system`, 독립99glyph 공급은 `한양신명조/custom`, certainty observed이며 glyph coverage를 실제 확인했다. 이전의 unsupported CLI font-trace/DOM 추측과 구분한다. 원본 HFT에 부족하던 ASCII 공급의 차이가 실제 소비 선택으로 입증됐으며 Native alias 정책을 임의로 수정하지 않았다. 모든 contour replay 분기나 actual92 완료를 이 관측으로 주장하지 않는다.
- C378은 C368과 동일한 최소 HFT 선언·크기·자간에서 uniform Hangul 둘레만 제거한 pure ASCII94자 대조다. job `ddcfbbea-f2a5-44b9-91d8-5dddef387bda`, engine2020/Hancom11.0.0.9136/Print0/기존driver one-up/preprocess none,10.803초 성공,1쪽 PDF SHA `c740fee327417eaa5e98d15f0bd87d082f0c49c8ee93fdeb4e342b383fca890d`다. 실제 쓰인 Type3의94개 glyph program이 C368과 전부 일치했다. 선언이 달랐던 과거 all-ASCII 표본만으로 문맥이 글리프 프로그램을 바꾼다고 추정한 가능성은 이 동일 선언 반례로 배제한다. `get_texttrace().type`는 rendering mode이며 PDF font subtype이 아니므로 C368의 원래 잘못 붙인 `fontType:0` 필드에는 별도 정정 note를 두고 실제 font dictionary Type3를 근거로 쓴다.
- PR 제출 감사에서 #6057의2쪽 원본/Print는 기존52조건 밖의 별도 C327 전쪽 검증임을 확인했다. C360/새 글꼴 조건에서 이2쪽도 별도 최신 Native/freshWASM 및 같은 글꼴 base 대조로 합쳐야 하며, 52조건 수치에 포함됐다고 쓰지 않는다. 기존 추가7조건521쪽은 stage5의 기존미달 qualification 범위로 따로 유지한다. 최신 base/변경 영향에 대한 자격·기존 issue 범위 확인 없이 현재 완료52조건으로 승격하거나 해소했다고 주장하지 않는다.


## C379–C383. 현재 소스 재검증과 중단 구간 복구

- 현재 parent `b80662087571f746579350d2ef00f3387b54b527` / base `224a326063a5aeab5a1ca8dcaa92918f55843ffc`를 확인했다. 원격 ls-remote도 같은 base이며 반복 리베이스하지 않았다. 사용자 최신 지시는 PR 생성 전 별도 승인이므로 원격 push/PR은 실행하지 않았다.
- C379 제출 후보는 보호 초안2개를 제외한192파일110,094,408bytes다. 지침 변경0·log 포함0이다. 검증에 직접 사용하는 PDF와 독립 대조군/임시 진단 PDF의 제출 필요성은 별도 분류 중이며 이 inventory를 그대로 stage하지 않았다.
- C373 최초 driver는 여유 공간20GiB 보호 조건에 걸려3조건만 완료하고205조건을 미실행 오류로 기록했다. `measurementsComplete:true` 필드가 전체 성공을 뜻하지 않으며 이 최초 state/results/run/producer/font 원장을 `same-font-whole-c373/disk-guard-interruption-c380/`에 그대로 보존했다.
- C380은 중단된 driver가 실제 종료됐음을 확인한 뒤 동일 producer/입력/PDF/글꼴 hash를 확인하고 완료3조건 및 현재 WASM17쪽의 완료 batch를 보존해18쪽부터 재개했다. 미완료 조건만 실행하고 낮은 디스크 여유에서는 대기하며, 전체208조건이 실제 완료된 때만 measurementsComplete를 쓴다. 삭제/문턱 완화나 과거 PNG 대체는 없다.
- C377의 현재 최종 C376 검사를 포함한 전체 nextest는 실제10,945PASS/0FAIL/45skip,611.282초,exit0이다. `full-gates-c377/full-nextest-summary.json`과 실제 log/result를 연결한다. 이후 Native/WASM lint·Skia 등 총33게이트는 순차 진행 중이며 전체 통과로 아직 보고하지 않는다.
- C381은 기존52조건에 포함되지 않았던 #6057 원본2쪽을 현재 C360 Native/freshWASM 및 같은 공급 글꼴의 고정 base와 새로 대조한다. 기존 C327 결과를 현재 재출력으로 쓰지 않는다.
- C382 실제92관측(Native40/Studio Canvas2D44/지원CK8)은 현재 C360 frozen producer와 C377 source ledger로 다시 고정했다. 이전 observer의 실제 bytes와 canonical diff를 재확인했고, 제품 초기 JS/WASM 응답 hash·실제 backend/readiness 검사 및 host default font 선택을 유지한다. 부분 HFT 검토 글꼴을 제품에 주입하지 않는다. 전체 게이트와 C373/C381 측정 완료를 대기하며 실제 캡처 완료가 아니다.
- C383은 이전 추가7문서521쪽의 자격 검증을 최신 base/현재 source와 공통 HyhwpEQ가 설치된 동일 Library/Fonts로 다시 산출한다. 기존90% 미달을 완료52조건으로 합치지 않으며 신규 하락·기존 미달·기준 쪽수 차이를 분리한다. 이 실행도 공통 capture lock/총8worker, 완료 SVG hash 보존 후 중간 SVG 정리와 immutable 생성 font 중복 공유를 사용한다. 원본/PDF/PNG/TSV/log/shared target은 보존한다.

- C381 완료: #6057 원본 전2쪽 현재 Native 최저99.81353% / freshWASM99.82029%, 쪽수2/2·90%미달0·devel0.1%p이상하락0이다. Native 기준 최저81.18482%, WASM기준97.20939%와 별도 비교했다. 양 backend1·2쪽의 review와 standalone overlay를 모두 직접 열어 법명 흰 겹낫표·뒤제10조, 1현황/2문제점과 다음쪽 이어짐·3개선방안/4기대효과/마지막본문의 순서·보존을 확인했다. 미세 AA·굵은 제목 획 차이는 남으며 actual92 또는 전체52 완료를 대신하지 않는다. `source6057-final-c381/direct-review.json`에 직접 판독 파일 hash를 기록했다.

- C384는 C373 현재 새 글꼴 조건의 완료 batch PNG에서 로드맵9쪽 Native/freshWASM review와 standalone overlay를 다시 구성해 모두 직접 열었다. 첫 책괄호의 기존 원점 차이가 없어졌고 그림목차1–30·점선·오른쪽 쪽번호·하단−6의 순서/보존을 확인했다. 미세 AA·점선 위상 차이는 남으며 이 선택쪽 판독은 전302쪽/전52조건 완료와 구분한다. 패널 생성 tool이 비동기 session98414를 반환한 직후의 최초 open은 파일 미생성 오류였으며, 같은 실행의 실제 exit0 뒤 정상 생성된4파일을 다시 열어 판독했다. 별도 중복 생성은 없다.

- C380 재개 중 로드맵WASM162–177에서 `HCR Dotum`의 FontFace ArrayBuffer 적재가1worker에서 실패했다. 원본 HANDotum SHA `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898` 및 카탈로그는 직전 성공146–161구간과 같고 다른 worker의 일부쪽 출력도 있으나 불완전 batch는 완료로 쓰지 않는다. 원인 확정이나 글꼴 교체/검사 우회는 하지 않았다.
- C385는 C373 최초161쪽의 원본 PNG를 hash 원장과 hardlink로 보존하고 실패162쪽부터 같은 canonical producer/font/8capture lock으로1회 재시도했다.162–177은 정상 완료 후178쪽 이후 진행 중이다. 전체302쪽 완료 전에는 새 WASM 전쪽 통과로 보고하지 않는다.
- C386은 주 실행과 단일 재시도가 실제 종료한 뒤에만 실패 조건1개를 검증된 새 전쪽 TSV로 연결한다. 원래 전체 실패 원장은 그대로 보존하며 다른 실패를 지우지 않는다. C382의 대기 dispatcher만 SIGTERM/실제exit143으로 종료 확인하고, 제품 캡처/명령/producer가 없다는 영수증을 남긴 뒤 이 복구 확정 단계를 선행 조건에 추가해 다시 대기시켰다. 실행 중인 빌드·캡처를 중복 시작하지 않았다.
- 현재 변경 문서10개의 내부 링크 검사는 실제exit0·이상없음이고 git diff --check도exit0이다. 보호 초안2개의 원래SHA와 빈stage를 재확인했다. 현재 source3479개 hash는 C377 지문 그대로이며, 임시 진단/복구 driver와 로그는 ignored output에만 있다. PR은 전체 필수 검증·전쪽 판독·실제 소비 경로·증적 정리 후 사용자 승인을 받아 진행한다.


## C387–C391. 현재 전쪽 내용 보존과 제품 산출물 감사

- C387은 C386 복구 확정 뒤 주52조건+별도#6057의53조건1966쪽에서 현재 Native/freshWASM render tree의 type/원문 text/실제 source owner/순서를 쪽별로 비교하도록 대기한다. bbox만 제외하고 C190에서 독립 확인한 synthetic usize sentinel만64/32bit에 맞게 정규화한다. 로드맵의 재사용161쪽 tree와 새162–302쪽을 명시적으로 조합하며 실패batch의 일부tree는 쓰지 않는다. 이 검사는 독립 PDF·실제 제품 검증을 대체하지 않는다.
- C388은 C328 Studio unit 소스376개·C333 frontend build 소스382개·C334 VSCode 보정4개의 byte hash가 현재와 같음을 확인했다. C328의2371실행/2369PASS/2skip와 C334 font계약57PASS/1skip의 동일 소스 근거다. C333은 Studio build 성공 뒤 VSCode font패키징 검사가 실패한 이력이므로 전체 pipeline PASS로 쓰지 않는다.
- 현재 C360 WASM을 소비한 Studio build8.461초·VSCode compile31.291초가 각각exit0이다. 양쪽 dist의 WASM과 RHWP Public Batang WOFF2는 현재 root pkg/공개 font asset과 byte hash가 같다. Studio rawJS도 같고, VSCode는 Webpack bundle이므로 원 JS와의 byte 동등성을 주장하지 않는다. 초기 verifier가 raw rhwp.js를 기대한 오류와 source-map sourcesContent 가정 오류는 그대로 보존했다. 현재 `nosources-source-map` 설정, 새 compile의 root pkg module 로그, 입력 JS 전후 hash·관련 bundle/map hash와 sourceMappingURL로 계보를 구분해 검증했다. 재빌드 반복 없이 verifier만 교정했다. 실제 VSCode webview 실행은 이 결과로 수용하지 않는다.
- C385 로드맵WASM은 실패한162–177을 같은 글꼴/정본/guard로 재시도한 뒤 전302쪽까지 완료했다. Native 전302 최저96.69755%, freshWASM96.69698%, 각각90%미달0이다. C390은 두 산출물의 쪽 번호1–302·누락/중복 번호0과 현재 쪽별 원문/type/sourceowner/순서가 전302쪽에서 일치함을 확인했다. 다른51조건은 아직 완료로 보고하지 않는다.
- C389는 기준299쪽과 후보302쪽의 같은 번호 Body subtree(type/text/sourceowner/order,bbox제외)를 보수적으로 대조했다. Native/WASM 각각81쪽만 완전히 같은 Body이며221쪽은 변경되었거나 baseline 쪽이 없다. 이221쪽의 단순 같은 번호 raw 차이를 내용 정합 무회귀 증거로 쓰지 않는다. baseline이302쪽이었다고 기록하거나 수치상0.1%p하락0만으로 전체 의미 보존을 승인하지 않는다. 후보의 정확한 독립 Print302쪽과 내용·경계·분할 보존 근거를 함께 판정한다.
- C390의 현재 최저152쪽·차순위263쪽 Native/freshWASM review와 standalone overlay를 모두 직접 열었다.152쪽의 막대 높이·순서·축·평균선·주석·쪽번호141,263쪽의 소제목/작은bullet/구획선/연구목적/마지막항목/쪽번호252의 소속과 순서는 유지한다.152쪽 grid/JPEG명암·획,263쪽 일부bullet행·강조제목의 미세 원점/간격·획 차이는 잔여다. 이를 완전 동일·글꼴만의 원인이라고 쓰지 않으며263쪽은98%미만이므로 pxpin을 새로 추가하지 않는다.
- C391은 기존92관측을 유지하면서 새 bitmap 수정의 실제153쪽을 추가10관측(Native2·주Canvas2·CKpairedCanvas2/CK4요청)으로 준비했다. C382 뒤에 같은 current producer·실제 product open·초기응답 hash·기본 host폰트·실제backend/readiness guard로 실행한다. 이는 요청 수이며 성공 수가 아니다. CK가 fallback하면 CK통과로 세거나 guard를 우회하지 않는다. 이전 kit명세의2source×3backend×2profile=12관측에는 Canvas4와 CK8이 포함되어 있었음을 확인했으며 현재92의Native40/Canvas44/CK8 분류를 유지한다.


## C392–C395. 필수 게이트 완료와 잔여 하락 원인 확인

- C377의33개 순차 게이트가 모두 실제exit0으로 완료했다. 전체nextest10945PASS/0FAIL/45skip·611.282초, release root lib3804PASS/12ignore·다른libs182PASS, NativeSkia root3861PASS/12ignore·다른libs182PASS, NativePNG2PASS/PDF14PASS, doctest8PASS/3ignore, HFT Python13PASS, manifest23PASS다. fmt·Native/WASM/feature/alltarget Clippy·workspace/release build·unit tiers·manifest와16개Native feature대상도 포함한다. `full-gates-c377/gate-summary.json`에 로그 요약과 current source 지문을 연결했다. 전체 시각 비교·실제 소비 경로 완료를 의미하지 않는다.
- C392는 종료가 입증된C336 생성 font_resources만 정리했다. 원래파일3237개를 동일SHA·mode·uid/gid의39개객체에 hardlink해 경로/바이트를 유지했고,3198개의 중복inode에서 추정23,684,349,952byte를 확보했다. 원본/설치폰트/PDF/PNG/TSV/log/shared target/현재실행은 보존했다. 과거C171의 이미 공유된100GB논리크기를 다시 확보했다고 쓰지 않는다. 계획·실제lsof/프로세스·파일별전후hash는 `stopped-font-copy-dedup-c392/`에 보존했다.
- C393은 base299쪽과 현재302쪽의 Body type/text/source owner/order를 bbox없이 의미로 대응시켰다. 양backend 각각219쪽이 유일하게 일치하며138쪽은 실제쪽번호가 다르다. 각 쌍의 원래base PNG를 그내용의 독립PDF쪽에 원크기·무마스크로 비교한219쪽에는0.1%p하락이 없다. 나머지83쪽은 Body구성이 달라 이 진단으로 수용하지 않으며 전302쪽의 canonical TSV·쪽수/내용/경계 검사도 대체하지 않는다. `source-matched-baseline-diagnostic-c393/results.json`에 명시했다.
- C394의 추가자격문서56345는 주완료52조건과 구분한다. Native8/9/17/18쪽의 기존base는 한글TOFU인데 현재는 읽히며, 양쪽 review/overlay 직접판독과171개소유TextLine의 원문/순서/원점0px차이를 확인했다. leaf분할/폭까지 같다고 주장하지 않는다. 기존 수직배치·17쪽추가원문prompt차이는 남는다. #7806은 같은원본SHA58013017·독립PDFSHAc66ea20c·전21쪽 미달을 추적하는OPEN이슈이며 `existing-issue-7806-live.json`으로 현재상태를 보존했다. 기존TOFU를 되살려 점수를 맞추지 않고 정상11/20anchor관계검사도 유지한다.
- 같은문서WASM3/13쪽의0.29878/0.12346%p하락은 아직 원인판정 중이다.51/50개TextLine의 원문/소유/순서/원점은 같고 두쪽에는Image가 없다. 실제공급휴먼명조TTF는 양쪽SHA동일·필요135한글coverage완전하므로 누락glyph/CP949추정은 근거가 없다. 현재Haansoft Batang별칭이 추가적재된 사실만으로 실제선택원인이라고 확정하지 않는다. C395는 같은frozen base/current·canonical폰트목록으로 실제fillText/fill호출과 별도DOM face probe를 관찰하고 원래C383 PNG와 byte동등성을 먼저 검사하도록 공통capture lock을 대기한다. source/font/허용기준은 변경하지 않았다.
- 사용자 최신지시에 따라 PR은 준비·최신devel확인·검증 결과를 보고한 뒤 명시승인을 받아 진행한다. 현재 원격push/PR생성/외부게시를 하지 않았다.

- C395 관찰은 양backend라는 표현 대신 frozen **base/current WASM** 각3·13쪽, 총4PNG가 기존C383와 byte동일함을 확인하고 완료했다. 실제Canvas fillText수도base/current3쪽566/566·13쪽861/861이고 원문순서는 같다. 별도DOM probe는 양쪽본문 모두휴먼명조를 선택해 Haansoft추가적재가본문face교체였다는 가설을 배제했다. current에서CSS bold가제거된것은 보였으나fill+stroke 합성 경로가있으므로 굵은글자상실이라고 단정하지 않는다. strokeText의 실제호출/독립PDF획굵기 대조는 남았다.
- 실제glyph원점은 TextLine원점과다르다. C395 actual-glyph-origins-comparison에서3쪽105/13쪽130fillText 원점이0.01devicepx넘게다르고 최대X20.880005devicepx(192dpi,10.440003CSSpx)다. 큰차이는30개연속dash의간격누적으로 확인했다. 독립PDF원래glyph위치를192dpi로환산하면 이run의대표advance약17.27067px이며base18.0533px→current17.3333px로근접했다. 점수를맞추려고기존dash폭을복원하지않으며 이독립간격대조가두쪽전체0.1%p하락을해명했다고도보고하지않는다. 초기PDF진단은nullable linewidth를round한TypeError로파일쓰기전에실패했고실제transform을반영하지않은중간x=0표시도진단오류였다. 수정된실효좌표·원본PDF대조만최종JSON에기록했다. 원본PDF·pdftoppm·raw2px문턱은그대로다.


## C396. 실제 합성 굵기와 독립 PDF 대조

- C396은 C395의 읽기 전용관찰에 strokeText만 추가했고, 원래C383의base/current3·13쪽4PNG와모두byte동일하다. current3쪽57/13쪽53strokeText 중휴먼명조는52/48글리프다. 독립한컴PDF의실제Tr2(fill+stroke)부분52/48문자와내용/순서가정확히같다. 실제Canvas획너비0.346666663885px는13pt×96/72×0.02=0.346666667px와일치한다. 원본PDFcontent의w2.16/Tf108=.02를직접확인했다. get_texttrace의비변환linewidth2.16을PDF최종pt와비교하지않았다. CSSbold가없다는이유만으로굵기상실이라고판정하지않으며종전Chromium자동합성으로되돌리지않는다.
- C394에서base/current각3·13쪽review와standaloneoverlay총8개를직접열었다.3쪽두열의기존줄바꿈/세로차이와13쪽오른열의기존추가wrap/행전진차이·하단목록차이는여전히보인다. current의lighter합성bold·첫30dash짧아짐은위의독립연산자/글리프원점근거와맞지만본문의다른작은glyph원점차이까지이자료로면제하지않는다. 원래raw0.1%p하락은원장에서유지하며기존#7806해결이나전21쪽시각수용으로쓰지않는다.
- C379의21개새PDF에현재source/tests/scripts/assets/Stage8/curated본문의파일명참조를추가감사했다. 이름참조없음을동적소비없음으로해석하지않고원래경로/바이트를모두유지했다. 최종채택자료·독립대조군·임시진단을구분한후필요한것만제출하며,6개local-only HFT복원표본을다시Git후보에넣지않는다.


## C397–C398. 수식 실제 소비 경로와 현재 제출용 로드맵 증적

- C397은 공통Linux수식글꼴을 현재제품에서도 소비하는지 각주242쪽문서의219쪽으로 추가4관측(Native화면/인쇄2·StudioCanvas화면/인쇄2)을 준비했다. 기존102요청을유지하고C391완료후 같은current producer/공통capture lock으로실행한다. 검토용부분HFT글꼴·oracle출력환경을제품에주입하지않으며 초기JS/WASM응답hash와실제기본글꼴선택을확인한다. S19의옛frozen글꼴검증을현재제품결과로재인용하지않는다. 추가범위의CanvasKit성공은주장하지않고이4건도아직대기다.
- C388의freshStudio실물배포파일37WOFF2(25,969,628byte)와21개license/notice가canonical원본과모두같고, VSCode는실제Webpack선택목록의13WOFF2와21개license/notice가모두같다. 첫감사에서VSCode도37개전부를포함한다고가정한verifier오류는별도invalidJSON으로보존했고실제선택목록과일치하도록감사만교정했다. 제품소스수정·재빌드·글꼴추가는없다. 이실물패키징검사를VSCode현재webview실행수용으로쓰지않는다.
- C398은이전curated C366자료8개를ignored output에원래hash로백업한뒤, C373Native/C385복구확정WASM의현재새공통글꼴전302쪽TSV와153쪽review/overlay를제출용안정경로에byte그대로교체했다. 기준299쪽Native/WASM TSV도별도이름으로추가했다. current최저96.69755/96.69698%,153쪽98.69180/98.68852%·미달0이다.153쪽양쪽review/overlay4개를직접열어외곽/내부5격자·점·추세선/축·출처/캡션·하단142의소속과순서를확인했다. 미세JPEG명암/색경계·글리프AA는잔여다. C376최종검사·C37733게이트와연결하되전체52조건/실제106요청완료로보고하지않는다.
- 로드맵README에서과거글꼴조건의9/10/11쪽하락은이력으로구분했다. 같은번호raw비교만으로기준299→현재302의무회귀를주장하지않고C393의정확한Body내용대응219쪽/미대응83쪽과C390현재302쪽소유검사를함께기록했다. 새첨부8파일의원본SHA·copy검증·판독과백업경로는 `curation-roadmap-final-c398/curation-result.json`에보존했다.


## C399–C403. 현재 sample11 증적과 추가 문서의 실제 배치 하락 추적

- 이번 진행 전 실제 HEAD `b80662087571f746579350d2ef00f3387b54b527`, 실행 중인 C373/C401 Cargo 및 후속 소비 경로 대기 프로세스를 확인했다. 원격 `ls-remote upstream refs/heads/devel`도 `224a326063a5aeab5a1ca8dcaa92918f55843ffc`로 같아 반복 리베이스하지 않았다. 사용자의 PR 사전 승인 지시를 유지한다.
- C399 sample11 원본 전151쪽은 current Native 최저90.06416%/freshWASM90.07462%, 양쪽90%미만0·같은 쪽번호0.1%p초과하락0이다. base는20.00112%/19.58063%,150/148쪽미달이다. 같은99glyph공급과C360/C377producer의정량결과이며 제품기본글꼴 실제소비와 혼동하지 않는다. 양backend현재2/50/117쪽review/standaloneoverlay12개 및base50쪽4개를직접열고 전쪽TSV4개와함께 [새안정경로](../pr/assets/issue7445/completed-20261009/regression-policy-20261015/sample11-current-20261019/README.md)에원본byte그대로보존했다. 원본sample16의64쪽이전자료와구분한다.
- 직접판독에서2쪽5.2행의왼쪽이탈은보이지않고50쪽PC/서버/회선및footer큰이동은base대비개선됐다. 하지만INDEX자형/자간·얇은상하선의색경계/미세위치·연결선과색면·117쪽route add/Basic routing ASCII도해의수평glyph원점/공백간격차이는남는다.90%통과만으로이차이를면제하거나전체시각완료라고쓰지않는다. 이대표쪽들은98%미만이므로절대px핀을추가하지않았다. 현재전체52조건/소유감사/actual106요청도완료가아니다.
- 추가7문서521쪽의C383는28조건(base/current×Native/WASM)전부실제완료,command실패0이다. current미달32쪽/backend조건·0.1%p초과하락25건을남긴다. 이범위를원완료52조건으로승격하지않는다. C402는25건모두의실제RenderTree에서완전한TextLine소유/본문/순서가같음을확인한뒤에만같은index원점을비교했다. 논리글줄원점동일22건은leafglyphpaint/자형의무회귀입증이아니며원점차이3건은가맹사업WASM26쪽약3.8CSSpx,각주Native/WASM201쪽9.6CSSpx다. gasNative26쪽은글줄원점동일하지만shapegeometry차이가있어추가판독대상이다. 자세한원장은ignored `extra-decline-owner-audit-c402/results.json`에있다.
- C400각주201쪽은양쪽실제소유글줄152개본문/순서가같고79개및하단5×7표가current에서9.6CSSpx위로이동한다. Native현재표y720.2/base729.8,독립PDF는약798이므로기존큰미달에추가한실제배치하락으로분류한다. 글꼴교체로면제하지않는다. frozentrace상앞빈문단4351끝700.22667은동일하고다음pi4352의첫item은FullParagraph가아닌Shape다. base는stored-vpos후709.82667로가고current는700.22667에남아뒤Square표가같이위로간다. caller가글줄을그리지않는객체에도computedparagraphsuccessor를소비시키는가설을검증중이다. 빈문단높이나9.6상수를더하는보정을하지않았다.
- C401은ignored source-variant/전용APFSCoWtarget만사용한기하진단이다. authoritative3479source와sharedCLIhash를guard하며sample/PDF는읽기전용입력이다. 초기workspace member/vendor/embedded include파일누락으로발생한Cargo101세차례는입력복제설정오류로별도보존했고실제코드결함FAIL로세지않는다. 입력의실제closure를보완한unmodifiedbuild는성공했으며201쪽RenderTree SHA `05dedbb9af2fc6b864b11d5fc7c0debabb479d94b15cb85ff86957d13ff441cb`가현재canonical결과와완전히같다. 그후resume driver의dedent가문자열내들여쓰기를바꾼assertion은변형파일쓰기전에실패했고별도attempt4로보존했다. 같은빌드를중복하지않고검증된baseline부터직접명시한variant단계를재개했다. 현재진단은computedbinding을실제leadingline소유item으로제한하는가설만검사하며실제NativeSkia/freshWASM수용이나production채택은아직아니다.
- C403은가맹사업26쪽의base/currentWASM원본PNG로canonicalreview/overlay를추가생성한다. base100%→current95.61441%하락과하단10글줄의3.7~3.8px이동을추적한다. Native동일쪽은92.54989%→95.60096%이므로Native점수개선으로WASM실제배치를면제하지않는다. 원본/PDF/문턱·검사제거/정답이미지이동은하지않는다.


- C401 판정은 수정 채택이 아니라 원인 가설 배제다. leading-line 소유자 guard만 바꾼 격리 CLI의201쪽 RenderTree SHA도 `05dedbb9af2fc6b864b11d5fc7c0debabb479d94b15cb85ff86957d13ff441cb`로 완전히 같았다. 이 variant는 제품에 적용하지 않는다. authoritative3479source와sharedCLI는모두보존됐다. C404의기존 `RHWP_VPOS_DEBUG` frozen추적은current가pi4351에서lazy base5760HU를확정하고base는다음pi4352에서5040HU를얻는차이를보였다. 이것을빈문단높이를바꾸거나9.6px를더할근거로쓰지않는다. 독립PDF의실제배치를먼저해결한다.
- C403의base/currentWASM26쪽review/overlay4개를직접판독했다. 표와cell본문은맞고뒤제목/본문10글줄만current에서3.7~3.8CSSpx내려간다. frozenNative2개도canonicaltree와byte동일하며 `RHWP_DIAG_FRAG` 상내부표원점136.0/내용높이553.7/caption0/logical delta0은같지만외부후속cursor는693.5→697.2다. 행높이를줄이지않고종료gap/바깥여백의공통소비를추적한다. C405는같은전용cache에기록만추가한진단build이며production원본을바꾸지않고최종JSON이canonical과같음을요구한다.

- C405 기록 전용 격리 build는5분55초에exit0으로끝났다. 각주201/가맹사업26쪽모두canonicalNativeRenderTree와byte동일함을확인했으므로이진단기록이실제출력을바꾸지않았다. 가맹사업의내부반환은689.693333,repeat outer margin은true,종료source gap은None,문단after는0이다. 그런데종료빈host에발행된 `Exclusion` 배치의occupied_bottom561.2를단원점132.266667에더해693.466667로덮은뒤바깥아래여백3.773333을다시더해697.24가된다. 같은계약의 `paragraph_end`는Exclusion/StoredSquareWrap을흐름소비자로쓰지않는다. 따라서표높이나독립PDF를수정하지않고기하계획이종료흐름을덮는소비분기를고친다.
- C406은위원인을검사하는격리후보이며authoritative원본에는아직적용하지않았다. 명시적caption계약은유지하고terminaloverride를NextLine흐름소유자로제한해가맹사업25/26/27/118쪽과각주201쪽배치JSON을대조하도록순차빌드중이다. 이는가설검사이며최종공통규칙채택전EmptyHost등다른합법적흐름소유자/정상대조군의보존도검토해야한다. 실제NativeSkia/freshWASM출력·관계회귀검사및최종새소스전체검증을대체하지않는다. 기존NextLine guard실험C401은무변화로기각했고현재소스에포함하지않았다.


## C411. 사용자 요청에 따른 선커밋·리베이스 체크포인트

- 사용자가 현재 변경을 먼저 커밋한 뒤 최신 devel로 리베이스하도록 지정했다. 리베이스 전 HEAD는 `b80662087571f746579350d2ef00f3387b54b527`, 기존 base는 `224a326063a5aeab5a1ca8dcaa92918f55843ffc`다. 실제 dirty 217파일의 바이트·SHA와 binary patch를 ignored `output/pr-review/7445-completed-20261009/commit-before-rebase-20261019-122334/`에 보존하고 백업 ref를 만들었다. 사용자 초안 두 파일과 미분류 진단 PDF는 이 커밋에 포함하지 않는다. local-only HFT 표본·공유 target·원본은 변경하지 않는다.
- 직전 C360/C377 source의 주 전쪽 측정은 C386 복구 후208조건, command 실패0, current104조건의90%미만0·devel 대비0.1%p초과하락1이다. C387 실제 소유 감사는53조건1966쪽, mismatch0으로 완료됐다. 이 결과는 새 리베이스 head의 통과가 아니며, 추가7문서 C383의32미달/25하락 및 실제 배치 회귀를 면제하지 않는다.
- 실제 제품 소비 C382는 캡처·비교 단계 완료/직접판독 대기다. C391은 CanvasKit 관측 명령 exit1로 실패했고 C397은 그 실패를 선행 조건으로 받아 중단됐다. 실제106요청 전체 완료로 보고하지 않는다. 리베이스 직전 실행 중 Cargo/rustc/캡처 프로세스는 없었다.
- C407은 ignored 격리 소스에서만 일반 흐름 소유 계약을 보정했다. Exclusion/StoredSquareWrap의 종료 cursor 덮어쓰기를 배제하고 EmptyHost·NextLine·명시 캡션 계약은 유지한다. 정정된 실제 Rust test executable에서 동일 formal 검사1건이 기존 CLI의 여백 중복 assertion으로 FAIL(1.01초), 후보 CLI에서 PASS(1.84초)했다. 현재 manifest로 실행한 관련 정상 대조군은7PASS 및27PASS다. 초기 잘못된 문단 소속 기대값/nextest CLI override, mixed JSON, stale suite의0tests는 증거에서 제외했다. 아직 authoritative 소스에는 적용하지 않았으며 실제 NativeSkia/fresh WASM 시각 수용도 남는다.
- 각주201쪽의9.6px 실제 배치 하락은 미해결이며 C401 가설은 출력 무변화로 기각했다. C409 baseline cursor 진단은 선행 실패 guard로 종료됐고 빌드는 시작하지 않았다. C410 worklife4쪽은 본문25줄 원점이 같고 current에 footer TextLine wrapper가 추가된 사실까지 확인했으나, raw0.16679%p 하락의 독립 판정은 남는다.
- 이 커밋은 승인된 진행 중 변경의 보존 단계다. 리베이스 후 새 devel 기능 보존·필수 전체 게이트·Native/fresh WASM 전쪽·실제 소비 및 대표 review/overlay 직접판독을 이어간다. PR은 준비 결과 보고 후 사용자 승인을 받아 진행한다.


## 2026-10-19 선행 커밋 후 최신 devel 리베이스

- 사용자 지시 `커밋 먼저 하고 리베이스`에 따라 WIP 203개 파일을
  `504e2e236c1585c2c11ba6d0cb1408544b8af201`에 먼저 보존했다.
- 이전 HEAD `b80662087571f746579350d2ef00f3387b54b527`, 이전 base
  `224a326063a5aeab5a1ca8dcaa92918f55843ffc`에서 새 base
  `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`로 262개 커밋을 리베이스했다.
  리베이스 직후 HEAD는 `ebb527e9e8d0d51f43feca879fc0604b3f5965fb`다.
  range-diff는 262개 모두 대응하며 169개 동일·93개 변경, 누락은 없다.
- 새 devel의 composer·style·수식·머리말/꼬리말 분리 구조를 유지하면서 기존 보정을
  해당 공통 소비 지점으로 옮겼다. 최초 컴파일에서 확인한 인자·필드·import 불일치,
  테스트 보조 함수의 API 불일치도 보정했다. 각주 경계의 줄 번호와 실제 각주 영역 높이,
  저장 단 프레임, 이미 계산한 문단과 원점 원장을 호출부까지 전달한다.
- 새 devel의 Noto Sans Math/LMM와 기존 Public Batang 라이선스·공급 항목을 함께
  보존했다. 최종 canonical WOFF2는 38개·26,196,084 bytes다. 글리프 공급명 검사와
  source bytes/cache·원자적 글꼴 face 공개·SFNT 정렬/헤더 복구 계약을 함께 유지한다.
- 사용자 초안 2개와 미분류 진단 PDF 12개는 별도 stash
  `f84286916b0e1316e9420e013544ec90791a9292`에 보존한 뒤 apply했다.
  14개 모두 리베이스 전 SHA-256과 일치한다. 백업 ref·stash는 유지하며 커밋에 섞지 않았다.
- 실제 확인: `cargo fmt --all -- --check`, 현재 base 대상 suite manifest,
  unit-tier 정책(4,101 tests), Native Skia 라이브러리와 전체 테스트 코드의 `cargo check`,
  WASM32 라이브러리 `cargo check`, Studio TypeScript 검사 모두 exit 0.
  Python 글꼴 catalogue 26건과 의존성을 갖춘 HFT 도구 15건도 통과했다.
- 이 단계의 `cargo check --tests`는 테스트 **컴파일** 확인이다. 새 소스의 전체 nextest,
  Native/fresh WASM 전쪽 시각 검증·신규 devel 기능 보존·실제 소비 경로 수용을 대신하지 않는다.
  웹폰트 배포 검사 3건은 기존 dist에 새 Noto Sans Math·통합 notices가 없어 실패했으며,
  최신 WASM/프런트엔드 배포 산출물 재생성 뒤 다시 확인해야 한다. 기존 코드의 통과율·PNG를
  이번 head 통과로 재인용하지 않는다. PR 생성·원격 게시·merge는 수행하지 않았다.
- 상세 커밋·fetch·충돌 원장·복원 해시·실패와 재실행 로그는 ignored
  `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/commit-before-rebase-20261019-122334/`
  에 보존한다. 지침 변경은 없다.


## C412. 리베이스 전후 실제 TSV와 신규 devel 기능 대조

- 사용자 지시에 따라 리베이스 직전 WIP `504e2e236c1585c2c11ba6d0cb1408544b8af201`,
  직후 후보 `1227fa979acd8750b50395d81d64c630ded4a6c5`, 새 기준
  `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`를 구분한다. 기존 TSV 통과를 재인용하지
  않고 같은 입력·독립 Print·글꼴·DPI·compat·계측기로 새 PNG와 전쪽 TSV를 산출한다.
- 새 기준에 들어온 PR은 코드 #8064와 기록 #8069다. PR 본문·실제 제품/회귀 diff를
  읽고 185개 공통 변경 경로를 확인했다. 수식·중립 공백·저장 줄/표/미주·그림 샘플링·
  Studio Print가 실제 대조 범위다. #8069는 기록만 변경하며 제품 경로가 없다.
- 직전 Native/WASM frozen C360 producer의 원장을 선커밋과 대조했다. `src` 1,467개를
  포함한 원장 3,451개가 Git에 대응하며 제품 소스·Cargo/lock/build가 같다. 차이는 차트
  테스트 한 개이며 미추적 28개는 generated suite다. 선커밋 WIP에는 crates 변경이 없다.
  이것은 기존 producer의 출처 대조이며 `504e`에서 새로 컴파일했다고 쓰지 않는다.
  과거 PNG/TSV는 가져오지 않았다. 원장은 ignored `pre-rebase-producer-equivalence.json`이다.
- 현재 정본은 PDFium으로 변경되어 양쪽에 동일하게 적용했다. 과거 pdftoppm 수치와
  섞지 않는다. 사용자 지정 pdftoppm 대조도 별도 reference/TSV로 분리한다. 새 계측기
  PDF 색·Type3·면 경계·PNG·raw gate 계약은 53PASS다.
- 첫 집중 회귀는 현재 manifest에서 16개 case를 찾아 실행했다. 실제 58건 중47PASS,
  11FAIL이다. WMF mask 5건, legacy 수식 frame/edit 3건, 빈 문단 caret1건,
  #4138 이어받기2건이며 실패 검사나 허용치는 바꾸지 않았다. 초기 nextest의 `-j`와
  `--test-threads` 중복 인자 오류는 실행 전 오류로 별도 보존하고 실제 요약에 합치지 않았다.
- 우선9문서 Native432쪽의 전후 TSV를 새로 확보했다. 전에는90%미만0쪽, 후에는34쪽이며
  0.1%p 이상 하락214쪽이다. 표33/40·미주CP02·field01은 유지됐고 나머지는 보정 대상이다.

| 문서 | 쪽수 | 직전 최저raw | 직후 최저raw | 직후90%미만 | 0.1%p이상 하락 |
| --- | ---: | ---: | ---: | ---: | ---: |
| sample16 HWP5 |64|94.76582|62.37332|16|36|
| sample11 원본·사용자 Print |151|90.67531|88.15021|5|150|
| #4138 현재 ASCII 저장본 |197|90.87627|28.58472|13|22|
| 일생활 |6|98.01425|97.09531|0|3|
| 위첨자 |3|98.74954|95.96704|0|3|
| 표33/표40/미주CP02/field01 |11|각 원문TSV 참조|같은 값|0|0|

- worklife1쪽과 sample16 6쪽의 전/후 실제 PNG를 직접 판독했다. 자간 압축·표/수식 원점
  차이가 보이며 점수만의 차이로 면제하지 않는다. 새 devel Native의 같은3문서73쪽도
  별도로 산출해 devel에서 이미 미달인 쪽과 후보의 추가 하락을 구분했다.
- #8064 기능 우선13조건47쪽 Native의 base/head 출력은 모두 생성됐으나 후보1쪽90%미만,
  14쪽0.1%p이상 하락이다. 실제 dyad·operator script·공백·영어·각주 후속 표의 하락이
  남아 있으며 원 PR의 통과 이력으로 이번 head를 통과 처리하지 않는다.
- 기준 archive의 workspace member `tools/rhwp-subsecond`와 CLI `include_str!` 문서가
  빠진 두 설정 오류는 exact Git archive로 보완하고 실패 로그를 남겼다. 제품 컴파일 결함으로
  세지 않는다. 초기 비교 panel helper의 출력 폴더 누락은 이미 만든 PNG/TSV 해시를 확인해
  panel 단계만 재실행한다. 해당 오류를 숨기거나 같은 캡처를 중복하지 않는다.
- fresh WASM의 최초 후보가 새 기준과 같은 JS/WASM hash를 낸 것을 확인해 무효로 분리했다.
  서로 다른 worktree의 source mtime가 공용 캐시보다 오래될 때의 재사용 문제를 현재 환경
  지침과 대조했다. 공유 target은 삭제하지 않으며 source 바이트를 보존한 timestamp 갱신과
  실제 root 재컴파일·API/해시 확인으로 복구한다. 무효 후보 pkg는 `pkg-invalid-shared-cache`와
  `wasm-producer-invalid-shared-cache.json`에 보존하며 후보 증거로 사용하지 않는다.
- 현재 수치는 미해결 회귀의 진단이다. 후보 fresh WASM·원52조건 전쪽·전체 nextest/필수
  gate·실제 제품 소비·대표 review/overlay 판독을 완료한 상태가 아니다. 소스 보정과 해당
  원 기능/기존 보정 양쪽 재검증을 계속하고 PR은 보류한다. 지침·원격 게시 변경은 없다.
- 실행·원본 TSV·페이지별 전후 차이·PNG는 ignored
  `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/`에 있다.

## C413. 전후 TSV로 발견한 공통 호출 회귀 보정

- 최초 head WASM은 공유 target의 오래된 base 산출물을 재사용했으므로 무효로 분리했다.
  소스 바이트를 보존하고 mtime만 갱신한 뒤 실제 root `Compiling rhwp`를 확인했다.
  Native lib308.509초와 fresh WASM245.019초가 exit0이며 새 WASM hash는 base와 다르고
  root pkg/Studio public hash가 같다. 이 빌드는 Mac `--no-opt` 로컬 대체 경로다.
- 새 Python 환경에 Brotli가 없어 전후 WASM18조건이 캡처 전 실패했다. PNG 없는 실패
  디렉터리를 `prepost-wasm-missing-brotli-attempt`에 보존하고 같은 interpreter에
  brotli1.2.0을 설치해 새로 실행했다. 초기 오류를 시각 미달 또는 통과로 세지 않는다.
- #8064 base는 자동 글꼴 선택 API가 없어 현재 sweep이 중단했다. 후보13조건의 실제
  선택 결과가 모두 `None`임을 확인해, 구 API도 지원하는 명시적 JSON null 계약으로
  동일 None의 base를 별도 산출 중이다. 제품·canonical 도구의 guard는 변경하지 않았다.
  실패 상태의 최초 base WASM을 통과로 재인용하지 않는다.
- WMF의 흑백 pattern definition index가 새 상태 기계로 전달되지 않은 리베이스 누락을
  복구했다. legacy 수식의 유효한 저장 상자를 현대 단순 수식 정규화가 덮지 않도록
  원본 frame 소유를 유지했다. 16case/58실행은 수정 전47PASS11FAIL에서
  수정 후52PASS6FAIL(15.976초)로 바뀌었다. bitwise WMF4건과 수식 frame1건이 복구됐고
  pink-mask1·baseline/cell2·caret1·#4138이어받기2건은 아직 실패다.
- 일생활 p1의 수평 압축은 설치 글꼴 환경의 None 여부와 무관하게 재현됐다.
  `terminal_tracking_for_distribution`가 마지막 grapheme을 받는 공통 질의에
  런 전체 문자열을 넘겨 전체 signed 자간을 종료 한 글자의 몫으로 계산했다.
  마지막 grapheme만 넘기도록 복구한다. 측정·배치의 공통 TextStyle/advance 계약과
  신규 devel의20% 공백 축소 규칙은 유지한다.
- 독립 Print PDF의 p1 마지막 '장' 잉크 끝은716.9707 CSSpx다. 원문·줄 프레임과
  자연 전진을 확인한 관계 검사는 기존1227 producer에서 실제1FAIL(0.17초)이며
  실패는 이미 폭 안에 드는 원문에 word−1.588444/char−0.278222 압축이 생긴 것이다.
  전6쪽 Native/fresh WASM raw가 각각97.09531/97.04187 이상임을 먼저 확인해
  `issue_7445_negative_indent_ownership`에 같은 관계 검사를 추가했다. 98% 미달 쪽을
  절대 px로 고정하지 않으며 줄/원문 소유와 끝 내용 보존도 검사한다.
- 추가 보정 후보의 Native 빌드·시각 재검증은 진행 중이다. 부모1227와 dirty 후보를
  구분하며 `repair2-source.json`·`repair2-native-build.log`에 출처를 고정한다.
  전체52조건·전체 nextest·필수 gate·실제 소비 경로 검증 및 PR 준비는 완료되지 않았다.

### C413 추가 실행 결과

- repair2 Native CLI는430초 exit0이다. `--bin` 빌드가 갱신하지 않는 profile-root의
  convenience rlib를 진단에 한 번 잘못 고정한 실행은 무효로 분리했다. CLI가 실제 소비한
  `release-test/deps/librhwp.rlib`로 정정한 관계 검사는1PASS(0.06초)다.
  Native 실행 파일은 처음부터 최신이며 이 rlib 정정으로 PNG/TSV를 재분류하지 않았다.
- repair2 fresh WASM도 실제 root를 컴파일해 exit0이다.15분20초가 걸렸으며
  root/pkg·Studio/public의 일치와 이전1227 WASM 대비 hash변경을 확인했다.
- 일생활6쪽은 Native최저97.98364/WASM97.96748, 위첨자3쪽은 Native98.68217/
  WASM98.64035다. 원 pre-rebase producer에 대한 전쪽 하락은 모두0.1%p 미만이다.
  1쪽 자간 관계의 실제 FAIL→PASS와 전쪽 양 backend 원본 TSV를 각각 보존했다.
- sample16 진단6/18쪽은 Native67.79449/98.42489, WASM68.19394/98.50096다.
  이것은 두 쪽 진단이며64쪽 완료로 쓰지 않는다. 6쪽 review/overlay를 직접 판독해
  표와 후속 본문이8.8CSSpx 위로 이동한 실제 원점 차이를 확인했다.
- 동일 frozen CLI의 `dump-pages`·`RHWP_DIAG_TAC`·`RHWP_VPOS_DEBUG`·`RHWP_DIAG_ADV`
  대조에서 원본 paragraph142의 total88.2133 중9.5467px 마지막 줄간격이 fit용 높이와
  함께 전진에서 빠지고, 이후 lazy base가660HU로 바뀜을 확인했다. 표152는 같은 높이인데
  typeset의 진입526.4→517.6, 실제 paint603.9→595.1로 달라진다. 표시 문자열이나
  글꼴 교체로 면제하지 않는다. 현재 관련 흐름의 가드/진단 후보를 별도로 검증 중이며
  repair2의 frozen source와 다음 dirty 후보를 섞지 않는다.
- 수식 baseline 두 검사도 감사했다. 내부 layout.y는1.56→6.426667로 실제 움직이고
  바깥 frame.y는 같다. 셀 포함 검사 중 하나는94.46666666666667과
  94.46666666666668의 roundoff 때문에 실패한다. 검사 수정은 아직 하지 않았다.
  독립 Hancom oracle용 두 입력은 로컬에 준비했으나 서비스 queue full로 접수되지 않았다.

## C414–C415. #8064 기능 보존과 저장 축·공백 소유 보정

- 사용자께서 #8064 개선을 되돌리지 않도록 명시했다. 새 기준4a07의 #8064 병합
  `9b01d5d1447220b3a7345703e79ad42cf3f502f4` 본문·소비 경로를 다시 확인했다.
  `final_sweep_20261019/summary.tsv`의179조건2082쪽에 대응하는 원본·독립 PDF의
  모든 경로/해시가 현재 저장소와 일치한다. 이것은 입력 감사이며 새 후보 통과가 아니다.
  세부 기능/소비/미검증 연결은 ignored `rebase-4a07d3ae-20261019/8064-preservation-matrix.json`이다.
- pre/post 집중9문서의 Native432쪽과 WASM432쪽 TSV가 모두 완료됐다. 부모1227의
  전후864쌍에는 Native214쪽·WASM64쪽의0.1%p 이상 하락이 남았다. 새 보정 후보의
  통과로 이 결과를 재인용하지 않는다. 별도 pdftoppm 기준 전쪽 감사도 진행했다.
- provisional 다단 dirty 가드는 sample16의 단단 원인과 무관하여 채택하지 않았다.
  #8064의 다단 소비 규칙과 HWP5 재저장본의 프로필 구분을 유지했다. 실제 단단 pi142는
  새 원본 좌표축으로 되감겼는데도 spacing-before 트림을 허용했다. 첫 문단의 후행
  줄간격을 제거하면 다음 lazy 원점이 잘못 역산된다. `paragraph_origin_rewinds`의
  공통 원점 증거로 해당 첫 문단의 전체 흐름을 보존하고, 뒤의 유효 anchor 계약은 유지했다.
  `flow_hints → flow_advance_height/flow_trimmed_spacing_before → typeset pen → 기록한
  원점 → paint`가 같은 판정을 소비한다. 문서ID나8.8px 상수 보정은 넣지 않았다.
- repair4의 pi142 전진은78.6667→88.2133, 다음 lazy base660→0HU로 복원됐다.
  sample16 Native64/64쪽이 완료됐고6쪽은67.79449→98.16591%다. 전쪽 최저는
 35쪽75.58354%이며 미달1쪽,0.1%p 이상 하락은22/35/43/53/60쪽이다.
  따라서 전체 수용은 보류다. 35쪽 review/overlay를 직접 판독했고 그림 내부의 차이를
  확인했다. 그림의 source bytes·crop·bbox는 새 base/repair4가 같아 paint 소비를 더 추적한다.
- repair2의 #8064 집중13조건47쪽 Native를 다시 출력했다. 초기 후보의14쪽 하락은
 7쪽으로 줄었고, dyad1·operator1–4·영어2–3쪽이 남았다. operator3쪽88.99961%는
  여전히 미달이다. saved-note·space/font-size 대조군의 회복도 원본 TSV에 기록했으며
  아직 #8064 전체 기능 보존이나 새 fresh WASM 통과를 주장하지 않는다.
- dyad의 가시 주석 표식 뒤 공백은 본문 내부다. 측정용 LeadingSpacePlan은 표식을
  경계로 쓰는데 별도의 출력용 들여쓰기 한도는 표식을 빠뜨려 이 공백의 압축을 막았다.
  source/order·실제 공백 advance 관계를 검사하는 새 테스트는 repair4 실제 라이브러리에서
  1FAIL(0.02초)이다. 한 쪽 전체의 사전 Native99.36532/WASM99.61456%를 확인한 뒤
  관계 검사를 추가했으며 절대 px 기대값을 붙이지 않았다. 같은 원본 scalar 경계를
  측정과 방출에 전달하는 repair5를 빌드 중이다. 수식 메트릭·20% 공백 축소·실제
  inline 표식 소유와 #8064의 새 이미지/Print 경로는 제거하지 않았다.
- 일생활 관계 검사도 저장소의 실제 테스트 파일을 컴파일하여 부모1227에서1FAIL,
  repair4에서1PASS(0.09초)임을 확인했다. 이것은 실제 테스트 소스의 집중 실행이며
  generated suite/nextest 전체 실행과 구분한다. 새 테스트 수 변화 뒤 `--prepare`를
  다시 완료했다(1639 sources/7153 static attrs/48 integration targets).

### C415. 사용자 공급 HancomEQN의 실제 사용자 글꼴 등록·전쪽 소비 검증

- 사용자가 Downloads/HancomEQN.ttf 사용과 사용자 글꼴 등록을 승인했다.
  `/Users/tsjang/Library/Fonts/HancomEQN.ttf`에 설치하고 `fc-match`가 해당 실제
  HancomEQN Regular 파일을 선택함을 확인했다. SHA는
  `c249c422e4acbafe2757c0f4a5d714414f84305c728776d889224f6116186e35`,244816바이트,
  revision1.0/UPM1024다. 이전 원 검증의9683b9...와 파일 해시는 다르다.
  필요한175코드가 모두 있고 코드의 HMTX175개와 실제 전진폭은 전부 일치한다.
  이전 파일의 윤곽과 완전히 같다고 주장하지 않으며 글꼴 바이너리는 커밋하지 않는다.
- 설치 전의 글꼴 누락 출력과 새 글꼴 출력을 구분했다. 같은 원본/PDF를 새 글꼴로
  새 base와 repair5 후보에서 각각 Native/fresh WASM 전8쪽 다시 출력했다.
  Native base최저98.90120→후보98.90357%, WASM양쪽98.96325%다.
  네 조건32쪽이 모두90% 이상/8쪽 일치이고16개 base/후보 쌍의0.1%p 이상 하락은0이다.
  실제 WASM font catalog의 해당 face 공급도 확인했다. 양 backend3쪽 review와 overlay를
  직접 읽어 기존에 사라진 C·앞첨자6·제곱2·연산자와 결합 줄이 보이는 것을 확인했다.
  작은 윤곽 가장자리/기준선 잔차는 남는다. #8064 전체 기능 보존 완료와 구분한다.
- Native에 JSON null을 FontEnvironment struct로 준 최초 driver는 metadata 단계에서
  exit2였고 PNG가 없다. invalid-native-null-environment-attempt로 보존했다.
  Native는 정본의 자동 환경 인자 None, WASM은 구 base API와 같은 명시적 JSON null
  경로로 정정해 새로 실행했다. 엔진·계측 guard를 변경하지 않았다.
- 공백/들여쓰기 실제 suite는 `--prepare`의 새 route regression_suite_007에서
  nextest6실행6PASS/249skip,0.430초 exit0이다. 직접 rustc의6PASS와 구분해 기록한다.
  repair5 fresh WASM은 실제 root 컴파일 후219.621초 exit0이며 root pkg/Studio public
  JS/WASM hash가 같다. 전체 회귀·전체 시각·실제 Studio 소비는 아직 완료되지 않았다.
- 설치·입력·source/pkg 출처·원문 TSV·대표 PNG·소비 증거는 ignored
  `rebase-4a07d3ae-20261019/8064-font-audit/`와 `8064-hancomeqn-installed/`에 있다.

### C416. 검증된 공통 보정의 로컬 체크포인트

- repair5 fresh WASM 집중 문서의 실제 완료: 일생활6쪽 최저97.96748%, 위첨자3쪽
 98.64035%, sample16 저장본64쪽95.66182%, 미달0/쪽수 일치다. sample16의
  pre-rebase 대비0.1%p 이상 하락은22/53/60쪽으로 남으며 Native35쪽은 별도 미달이다.
  #8064 집중13조건47쪽 WASM의 하락은영어2쪽(-0.33187%p)·saved-note1쪽
  (-0.15188%p)이다. Native의 영어2/3쪽도 남는다. 새 글꼴로 다시 산출한 연산자8쪽은
  Native/WASM 모두 별도 base/후보 대조를 완료해 글꼴 미설치 결과와 구분했다.
- #8064의 WMF 공통600DPI surface·명시 프로그램 등록·실제 source fingerprint 계약은
  유지한다. Native35쪽과 WASM35쪽 차이는 원본 그림 bytes/crop/bbox가 같아서 Native의
  기본 font database와 WASM의 명시 GulimChe 공급을 대조하는 공개 API 진단을 준비했다.
  현재 제품에 래스터 경로 우회나 글꼴 치환을 넣지 않았다. 이 원인 추적은 다음 단계이며
  현재 체크포인트를 전체 검증 완료로 쓰지 않는다.


## C417 장치 PNG의 Native 재생 소유 복구

이 절은 `c4c5632a21d4a34e76b18fc88bccfaf939fff4ab` 뒤의 후보다. 원본 source/PDF는 C412–C416과 같으며, 실제 source hash·명령·빌드 출처는 ignored `rebase-4a07d3ae-20261019/repair7-source.json`·`repair7/native-producer.json`에 고정했다.

### 기각한 후보와 원인 재현

투명 PNG를 alpha 가중 면적 평균으로 공통 축소한 repair6은 실제 Native35쪽을
75.58354%에서74.40564%로 악화시켰다. 단위 수학 검사2PASS를 실제 시각 보정으로 수용하지 않았다.
이 후보의 제품 코드와 검사는 철회했으며 patch·PNG·전64쪽 TSV·로그·거절 근거는
`repair6-rejected-transparent-reduction.patch`와 `repair6-rejection.json`에 보존했다.

Native와 WASM에서 추출한 공통 메타파일 PNG는4560×2853이고 SHA-256
`3bf3a10d9ce2d2e666fa9a479412a02fb7b9ac3c8c06a4962bada8317c1bc1ba`로 같았다.
GulimChe 원본과 준비 face를 Native에 명시 공급해도 실제 PNG가 같아 글꼴 공급 가설도 기각했다.
같은 PNG·bbox·CTM을 독립 Skia surface의 nearest/linear/mipmap/cubic으로 재생했다.
MIP 결과는 기존 Native 도해 영역과2px 실루엣100%, 독립 PDF와70.12708%였다.
linear는 기존 WASM과2px 실루엣100%, PDF와96.50339%였다. 여기서100%는 해당 영역의
허용2px 실루엣이며 바이트·픽셀 완전 일치가 아니다. 원본 도해의 위치·글자 소유는 바꾸지 않았다.

### 실제 생산·소비 연결과 보정 범위

`paint/metafile_surface.rs`가 WMF·장치 크기·글꼴·clip·전달색을 확정한 PNG를
`ResolvedImageKind::OutputDevice`로 제공한다. Native `PaintOp::Image` 소비자는 그 출처를
가졌지만 지역 `draw_image` closure에서 버리고 일반 원본 bitmap의 Print MIP 정책을 전달했다.
`image_conv.rs`는 그 결과를 다시 면적 평균/MIP로 축소해 가는 장치 문자와 무늬를 회색으로 만들었다.

이제 PNG인 `OutputDevice`에서만 출처를 `ImageSampling`까지 전달한다. 이미 만들어진
장치 표본은 최종 사각형에 선형 재생하며 일반 원본 bitmap의 면적 평균·MIP에 다시 넣지 않는다.
일반 `FormatConverted`, 직접 bitmap, SVG, 명시적 nearest, crop/transform/clip·tone·binary
처리는 기존 경로를 유지한다. WMF600DPI producer, 글꼴 선택, 전달색·alpha·source fingerprint와
#8064의 다른 기능은 되돌리지 않았다. 문서명·페이지·고정 좌표 조건은 추가하지 않았다.

Native 전64쪽을 새로 캡처했다.35쪽은75.58354→96.99437%, 전체 최저95.25071%(22쪽),
90% 미만0쪽, 쪽수64/64다.18쪽은98.42489→98.35911%로0.06578%p 감소했고 리베이스 전
98.38916%와 차이는-0.03005%p다.35쪽 review·overlay를 직접 판독하여 도해의 글자·선·음영과
앞뒤 본문이 보존되고 남은 색·미세 위치 차이가 있음을 확인했다. 완전 일치로 쓰지 않는다.

현재 사용자 HancomEQN 등록 조건에서도 WASM 전64쪽을 **새로 캡처**했다. 최저95.66182%,
미달0쪽, 쪽수64/64이며 이전 repair5의64 PNG와 바이트가 전부 같았다. 제품 변경 두 파일은
`cfg(all(not(target_arch="wasm32"), feature="native-skia"))`로 WASM 입력에서 제외되어,
C416에서 source 결합을 확인한 fresh repair5 JS/WASM을 재사용했다. 새 WASM을 빌드했다고
기록하지 않는다. `repair7-wasm-producer-binding.json`이 excluded 경로·기존 no-opt build/hash와
새 PNG/TSV를 연결한다. 실제 Studio 초기화 검증은 이 캡처로 대신하지 않는다.

#8064 집중13조건47쪽의 Native도 새로 캡처했다. 현재 글꼴 조건의 repair5와47 PNG가 모두
바이트 동일해 이번 Native 변경으로 새 차이가 생기지 않았다. 연산자8쪽은 글꼴 등록 뒤의
base/candidate를 대조했으며 등록 전 missing-face 결과와 섞지 않았다. 기존 영어2·3쪽 하락은
남아 있고13조건을 전체179조건2082쪽의 기능 보존 완료로 쓰지 않는다.

### 정식 소비 경계 검사와 남은 범위

`tests/cases/native_output_device_surface.rs`는 font·실물 쪽 좌표에 의존하지 않는 합성
paint 프로토콜 검사다. 같은 고주파 흑백 PNG를 `FormatConverted`와 `OutputDevice`로
실제 `SkiaLayerRenderer`에 반복 전달한다. 장치의 흑색 표본이 원본 bitmap의 평균화로
사라지지 않고, 캐시 전환 뒤 원본 경로도 그대로 돌아오는지를 검사한다. 문서35쪽의 절대px를
고정하지 않는다. frozen repair5에서1FAIL(0.05s), repair7에서1PASS(0.03s)를 확인했다.
정식 manifest를1640sources/7154staticattrs로 준비한 뒤 현재 배정 `regression_suite_011`의
실제 nextest에서도1실행1PASS/255skip(0.073s), exit0을 확인했다.
초기 CLI에 `-j8`과`--test-threads8`을 동시에 준 시도는 nextest에서 중복 threads로 거절되어
검사가0건 실행됐으므로 invalid로 보존했다. 수정한 명령은 `CARGO_BUILD_JOBS=8`과
`--test-threads8`만 사용한다.

증적은 같은 ignored root의 `repair7-paired-sample16-qualification.json`,
`repair7-incoming-native-preservation.json`, `repair7-device-surface-nextest.log`와
`repair7/{native-focus,wasm-focus}/sample16-hwp/{silhouette.tsv,review,overlay}`에 있다.

이번 단계는 Native MIP 회귀의 국소 복구다. 리베이스 전과 비교하면22·43·53·60쪽의
0.1%p 이상 하락이 남는다. 고정4a07d3ae Native와의 별도64쪽 대조에서는60–64쪽이
0.1%p 이상 낮으며 두 기준을 혼합하지 않는다.22쪽 문단446은 이전 줄의`응용프` 뒤
`로그램의`가 다음 줄을 시작했으나 후보는`로`까지 앞 줄에 담는다. 독립 Print의 줄 소속과
이 대조를 다음 보정의 근거로 남겼다. 평균 상승으로 하락을 면제하지 않았다.

나머지 원52조건/1964쪽·새 devel179조건/2082쪽의 전체 동일 조건 검증, 알려진 기존
실패6건의 독립 재검토·보정, 전체 nextest·필수 게이트·actual36은 계속 필요하다.
현재 PR 준비 완료가 아니며 외부 push·PR·게시·merge는 하지 않았다.


## C418 Native Skia 광범위 검사와 두 단위 계약의 정정

C417을 커밋한 `6215ec12b1552a78eb9092dfa1530b7defd9dd43`에서 Native Skia 공식3종을
순차 실행했다. PNG2건과 PDF4건은 통과했다. 첫 lib 실행은3857PASS/2FAIL/12ignore였다.
C417의 장치 PNG 소비 검사는 정식 integration suite에서 이미1PASS였으며 이2건과 다르다.

- `split_label_assigns_positive_slack_to_interior_spaces`: 테스트가 반올림한 호환 폭
  `estimate_text_width`로 전체 폭·공백·가시 끝을 구성했지만 실제 함수 입력은
  `est_x - est_x_start`의 소수 누적 폭이다. 테스트 입력과 독립적인 가시 끝 불변식을
  `estimate_text_width_exact`의 같은 좌표계로 맞췄다. 내부 공백2개만60px와 종료 공백을
  분배하고, 가시 끝이 프레임 끝에 도달하며 원문 종료 공백·0 문자/대시 배분이 유지된다는
  assertion은 모두 유지한다. 허용 오차0.001을 늘리지 않았다.
- `owned_rowbreak_tac_height_selects_current_or_multirow_frames`: `TAG_IMPLEMENTATION_PROPERTY`
  후속 줄은 원본 저장 사다리를 인증하지 않지만 저장 원점이 간격을 복원하지 못하는
  **후속 재조판**이다. 저장 frame 인증과 실제 전량 간격을 같은 bool로 부정하던 검사를
  구분했다. `restarted_tac_line_advance`가 합성 후속 줄을 저장 닫힘으로 인정하지 않음을
  추가 검사하고, 실제 후행 간격은 전량 남아야 한다고 확인한다. dirty/되감김만 있는 원점/
  불완전한 개체 높이의 거절 검사는 유지했다. 이 전량 간격의 실제 소비는 C416의6쪽
  독립 Print 회복과 C417 양쪽 전64쪽 검증에 연결한다.

제품 구현은 바꾸지 않았다. 두 변경은 `cfg(test)` 모듈의 입력·assertion뿐이다.
동일 Native Skia lib 명령을 다시 실행해 root3859PASS/0FAIL/12ignore, workspace의
다른 lib15+165+2=182PASS/0FAIL, exit0을 확인했다(총160.338초, 테스트root22.17초).
C417 PNG/PDF 명령과 실행 내용은 제품 코드·해당 검사·입력에 변화가 없으므로 보존한다.
증적은 ignored `repair7-skia-{lib,png,pdf}.log`, `repair7-skia-results.json`,
`repair8-skia-lib.log`, `repair8-skia-state.json`이다. 전체 nextest와 잔여6건의 통과를
이 root lib 결과로 대신하지 않는다.


## C419 98% 완료 쪽의 중요한 표 경계 px 회귀

사용자가 요청한 중요 위치 보호를6쪽의 평가 비율 표에 추가했다. C417에서 같은 문서
전64쪽의 Native/fresh WASM 최저가 각각95.25071/95.66182%이고6쪽은98.16591/98.56406%다.
직접 Native/WASM review·overlay를 판독했으며, 표의 외곽·앞 caption·뒤 평점산식이 보존된다.
수식·글꼴의 남은 미세 차이와 이 문서의 다른 쪽 하락은 별도로 남긴다.
이미 보호 중인3쪽99%대 제목·표 검사는 유지했다. 이번에 복구한 저장 원점 재시작을
통과하는 실제 경계는6쪽이므로 이쪽을 선택했다.

`verified_sample16_2024_print_boundaries`에 새
`completed_page_six_keeps_the_evaluation_table_and_its_source_neighbors_in_the_print_frame`를 추가했다.
원본 표의 내용으로 section/paragraph/control을 찾고 전64쪽에서6쪽에 한 번만 나타남,
앞 caption→표→뒤 평점산식 순서·비겹침·본문 포함을 보호한다. 독립 PDF6쪽 표 외곽 path의
stroke 중심을 pt×96/72로 환산한 네 경계117.697337/603.340007/709.218669/733.277344CSSpx에만
±2CSSpx를 사용한다. 고정 화면 영역으로 검사 대상을 찾거나 가시 식의 전체 SVG를 핀으로 삼지 않는다.

보정 전 frozen repair3에서 실제 위쪽595.093333과 독립603.340007이8.246674CSSpx 달라
1FAIL(0.29s), repair7에서1PASS(0.29s)였다. 실제 source를 직접 연결한 진단을 보존했다.
manifest를 다시 준비해1640sources/7155staticattrs를 확인하고 현재 자동 배정
`regression_suite_020`을 동적으로 조회했다. 공식 nextest에서도1실행1PASS/248skip,
테스트0.307s·전체118.012s·exit0을 확인했다. 이전 target 번호를 하드코딩하지 않았다.

최종 원본 TSV64쪽×양쪽과6쪽 review·overlay4개는
[`page6-source-frame`](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/page6-source-frame/README.md)에
바이트 그대로 보존했다. README에 원본/PDF/생산자/파일별 SHA와 Native code·WASM 재사용 출처를 연결했다.
로그와 중간 JSON은 ignored `repair9-page6-*-result.log`, `repair9-page6-nextest.log`,
`repair9-page6-nextest-state.json`, `sample16-p6-independent-table-paths.json`에만 둔다.

잔여4모듈의 현행 Native 직접 연결 진단도 수행해29PASS/6FAIL을 확인했다. 두 수식 baseline,
원본 WMF 분홍색 mask 소유, 빈 문단 편집 폭, #4138 첫/마지막 이어받기의 기존6실패이며
새6쪽 검사는 통과했다. source-equivalent frozen Native의 focused 진단으로 기록하며 전체 nextest나
정식 해당 suite의 완료로 대신하지 않는다. 첫 시도의 다른 serde_json dependency identity 선택으로
생긴 compile 오류는 실제 검사 결과가 아니며 수정한 의존성으로만 실행했다.
현재 PR 준비 완료가 아니며 기존6실패·시각 하락·전체 범위 검증을 계속한다.


## C420 마스크 합성 소유와 빈 원본 문단의 편집 계약 재검토

현재 runtime은 `6215ec12b1552a78eb9092dfa1530b7defd9dd43`와 동일하며,
`6cb3d68b95`·`5a01dd7072` 이후 제품 코드 변경은 없다. 다음2건은 검사 의미를
재검토했고, 수식2건과 #4138 두 이어받기 실패는 그대로 유지한다.

- 분홍 패널 검사의 도입은 `34f1579cc6`, 독립 한컴2024 Print와 GDI+ 근거는
  `regression-policy-20261015/wmf-contained-mask-20261017/README.md`다.
  새 base의 `6c8bd31e2c`는 WMF 마스크 교집합을 원래 장치 좌표계의 SVG viewport에
  적용한다. 그 자식의 분홍 paint 자체에 mask 속성이 없더라도 부모 viewport의 역휘도
  mask가 실제 합성된다. 원본 BIN0003의4개 분홍 paint에서 각각 소유 mask12/14/20/22,
  역색 filter와 원래 brush를 확인했다. 직접 속성만 찾던 검사를 실제 조상 합성 소유로
  고쳤고, 같은 SVG의 luminance mask 및 inverse filter 검사는 유지했다.
- 빈 문단 검사는 `560fae1644`에서 원본 BCP 후행 문단의 편집 소유 누락을 막으려
  도입했다. `layout_empty_runs_line`은 hit-test frame에 가용 폭을 줄 수 있으며,
  `get_page_text_layout_native`의 w는 그 frame 폭이다. 빈 문자열의 문자 전진은
  `charX=[0]`다. 화면 frame을 0으로 제한하던 검사를 문자 전진0·원본 소유 유일성·
  양의 줄 점유 및 실제 `get_cursor_rect_native`의 쪽/원점/높이 일치로 교정했다.
  원본 문구로 소유자를 고르며 기존 앞뒤 순서·비겹침 검사는 유지한다.

선행 시각 근거는 C417의 같은 원본·독립 Print·현재 글꼴 전64쪽이다.
Native 최저95.25071/WASM95.66182%, 미달0, 양쪽64/64쪽이며 새 PNG capture를
했다고 쓰지 않는다. 보존한 전쪽 PNG에서3/18쪽 review+overlay를 생성해 양backend 모두
직접 열었다. 18쪽98.35911/98.50096%에서 분홍 패널·역무늬·앞뒤 본문을,
3쪽96.85161/97.19902%에서 마지막 원문/빈 후행 줄의 순서와 비겹침을 확인했다.
**3쪽은 이전 px 보호 도입 때와 달리 현재 양쪽98% 미만이므로 시각 회귀가 남는다.**
기존 중요한 위치 검사는 제거하거나 완화하지 않으며 그 출력 개선도 후속 필수 작업이다.
그림의 미세 무늬·본문 획/간격 차이도 점수 통과로 숨기지 않는다.

정본 ignored `rebase-4a07d3ae-20261019/repair10-audit/`에 실제 SVG·편집 좌표와
실행 로그를 보존했다. 최초 현행 직접진단35건29PASS/6FAIL → 검사 교정 뒤31PASS/4FAIL
(3.51초)이다. 최초 임시 rustc의 resvg extern 누락은 실행 전 설정 오류로 별도 보존하고
실제 테스트 실패로 세지 않는다. 올바른 의존성으로 재실행했다.
`--prepare` 후 현재 manifest가 정한 suite008/023에서 두 모듈 **22PASS/0FAIL/453SKIP**,
실행0.360초·전체14.692초·exit0이다. `formal-nextest.log`·`formal-state.json`을 따른다.
수정 전 두 잘못된 assertion의 FAIL과 정상 마스크/본문/6쪽 표 경계 검사의 PASS가 연결된다.
전체 nextest 및 제출 게이트 통과가 아니다.

수식 baseline67/80의 독립 한컴2024 Print도 각1쪽 완료했다(method0,
frame_print_to_pdf_ex_one_up, Hancom13.0.0.3901). 두 PDF content stream은 동일하다.
현재 Native/fresh WASM runtime의 전쪽 각32.63506/34.93976%이며, 사용자 요청을 보존한
별도 pdftoppm 대조도30.88071/33.74439%로 미달이다. 이 합성 입력의 실패 기대값은
변경하지 않았다. 원본·Print·PNG/TSV·출처를 `equation-baseline-oracle/`에 보존하고
현재 실제 수식 배치와 기준선 소유를 계속 조사한다.


## C421 별도 개체 줄의 높이를 앞 텍스트 줄에 중복 예약한 회귀 보정

C420의 남은 #4138 두 실패를 테스트 변경 없이 복구했다. 원본115쪽·분할 저장197쪽,
최초·중간·마지막 조각의 내용 소유·누락/중복·뒤 본문 검사와 기존 px/관계 기대값을 유지한다.

- 선행 기능 `7807a9f8d5`/`task_m100_7642_stage78.md`는 혼합 문단 끝의 수식을 마지막
  물리 줄에 포함하고, 명시적 개행 앞 줄에는 중복하지 않는 계약이다. 이 기능을 되돌리지 않는다.
- 새로 식별한 경계는 `reflow_line_segs_impl`의 `forced_inline_line`이다. 개체를 전용
  후속 줄로 보내면서도, 일반 종단 포함 조건이 앞 텍스트 줄에도 같은 높이를 반영했다.
  `control_after_text` 소유 판정을 높이 발행 전에 공유해 해당 전용 줄의 개체만 앞 줄의
  합집합에서 제외한다. 보통 줄 안 개체·종단 개체·개행 뒤 빈 줄의 소유 조건은 그대로다.
  입력 속성→flow control→줄 높이/저장 LineSeg→단조 사다리→CellUnit→컷/실제 paint가
  같은 소유 줄을 소비한다. 문서 ID·문자열·좌표별 예외나 컷 허용치 변경은 없다.
- 정확히 같은 원본/같은 split API로 생산한 이전 검증본과 현행 보정 전 저장본은
  본문과 모든 줄 시작이 같지만 그림 문단2286의 앞 줄 높이가1200→23972HU로 바뀌고,
  뒤735개 줄 원점이22772HU씩 증가했다. 같은23972HU 그림을 다음 전용 줄에도 예약해
  마지막 쪽이 `(r)`에서 시작했다. 한컴2024의 해당 정확한 저장본 Print는197쪽이고
  마지막 쪽은 `용률`에서 시작한다. 원본/이전/실패 저장본·PDF를 모두 보존했다.
- 수식 문단548도 앞 공백 줄에1540HU를 중복 반영했다. 실제 CellUnit은25.6→30.133333px로
  커졌고44쪽의 빈 종료 줄이45쪽으로 밀렸다. 일반 논리 간격을 줄이거나 빈 줄을 삭제하지
  않고 실제 개체가 속한 줄에만 높이를 예약했다. 새 저장본은 자체 한컴2024 Print(method0,
  frame_print_to_pdf_ex_one_up,13.0.0.3901,job f889ff5b-91a3-462b-be49-dd868f22abf6)
  197쪽과 대조했다. 실패 입력의 Print를 새 입력의 정확한 기준이라고 쓰지 않는다.

ignored `repair11-split-audit/`의 source-metadata-differences·unit-before-current-differences와
`repair12-terminal-owner/`의 source/producer·저장본·Print receipt·focused/formal 로그가 정본이다.
현행 동일 Native frozen12의 직접35검사31PASS/4FAIL → 보정 후33PASS/2FAIL(3.55초),
두 잔여는 수식 baseline 계약이며 숨기지 않는다. 현재 manifest suite004/025의
#1139 정상7건+#4138 5건은 **12PASS/0FAIL/482SKIP**, 실행4.089초·전체138.422초·exit0이다.

Native192DPI/compat2024 집중 출력은 새 저장본1/44/45/196/197쪽 각각
99.98446/98.09400/97.98054/99.99132/99.98419%다. 이전 보존 저장본의44/45/196/197도
같은 수치다. 양쪽197/197쪽이며44/197쪽 review+overlay를 직접 열어 종료 내용과
후속 설명·마지막 문장 보존을 확인했다. 44쪽 수식 획·미세 위치 차이는 남는다.
이9쪽 진단을 전체197쪽·fresh WASM 통과로 확대하지 않는다. 새 픽셀 핀은 아직 추가하지 않았다.

Native12와 정식 집중 검사는 rustfmt 전 source의 실행이다. 이후 rustfmt가 closure의
중괄호를 생략한 형태로 정리했으며 원본·patch·두 SHA를 format-binding에 보존했다.
포맷 반영 source를 별도 producer로 다시 Native/fresh WASM 빌드하고 전쪽을 산출한다.
이 지점은 단계별 로컬 커밋이며 전체 nextest·전52/반영179조건·실제 소비·PR 제출 완료가 아니다.


## C422 수식의 줄간격 참여 속성을 Frame 소비까지 전달

C421의 전쪽 Native921쪽 대조에서 원본115쪽·HWPX115쪽과 sample16의64쪽은90% 이상이지만,
분할197쪽 세 조건의130쪽은32.89598%였다. 이 전체 결과는 HEAD60f1의 증거이며 아래 후보의
통과로 재인용하지 않는다. 별도 수식 대조군 HWP/HWPX18쪽도16쪽88.19667/88.24277%가 남는다.

원인 경로는 `Equation.common.affect_line_spacing → FlowInlineControl → FrameRowMetrics →
공통 줄 높이·간격 → CellUnit/실제 줄 paint`다. 새 Frame admission을 유지한 상태에서 일반
scalar 재조판은 수식의 줄간격 참여 높이를 반영하지만 Frame 경로는 글자 strut으로 간격을
계산한 뒤 점유 높이만 늘렸다. 원문 문단1698의 수식은2232HU·참여=true·160% 간격이며,
저장 줄의 간격1340HU와 달리 Frame의 뒤 본문이8.266667CSSpx 먼저 시작했다. 문단1689도
같은 원인의 참여 leading을 잃었다. 문서명·페이지·좌표별 예외를 추가하지 않는다.

개체의 점유 높이와 참여 높이를 분리해 FlowInlineControl에 전달한다. 해당 물리 줄의 소유
개체 중 참여한 수식만 text strut과 함께 간격의 기준이 된다. 더 큰 비참여 개체가 이를 대신하지
않으며 표·그림의 기존 점유와 일반 개행·종단 소유도 유지한다. projected Frame 입력도 같은
필드를 전달한다. 첫 빌드의 해당 생성자 누락 E0063은 수정했고 실패 로그를 보존했다.

ignored `rebase-4a07d3ae-20261019/repair14-equation-spacing/`의 source/native-producer,
`native-build-attempt2.log`가 실제 성공 빌드(240.17초) 출처다. Native192DPI/compat2024:
보존197쪽 입력44/129/130/131/197은98.09400/95.48686/95.45920/99.49413/99.98419%,
새 생산197쪽 입력44/130/197도98.09400/95.45920/99.98419%이며 양쪽197/197쪽이다.
130쪽 review/overlay에서 뒤 본문 원점 복구를 확인했으나 수식 획·첨자·미세 간격 차이는 남는다.
정상 HWP18쪽의8/9/10쪽97.06418/97.03591/95.89070%도 집중 확인했다.

정식 #1139 정상7건+#4138 5건은12PASS/0FAIL, exit0(전체304.65초)이다. 직접35검사는
33PASS/2FAIL이고 두 잔여 baseline 검사는 변경하지 않았다. Native Skia lib는 실행 중이며,
fresh WASM·영향 전쪽·전체 nextest·실제 소비 경로 검증은 아직 완료하지 않았다.
원문 HWP18쪽16쪽은 같은 조건의 고정devel88.90512% 대 후보88.19667%이므로0.70845%p
하락도 보정 대상이다. 점수 통과·제출 완료로 표시하지 않는다. 새 회귀 검사는 선행 시각 조건을
충족한 뒤 추가한다.

별도 합성 CJK baseline67/80의 한컴 재저장 HWPX를 확인했다. 양쪽 모두baseline67,
폭2199HU/높이2808HU로 정규화된다. 따라서 같은 Print가 나왔다는 사실은 유효한80%가
동일하게 paint된다는 증거가 아니다. PDF의 가/나 glyph outline·hmtx는 설치HBATANG.TTF와
각각 완전히 같음을 확인했다. 입력의 유효성과 실제 공통 배치를 먼저 보정하며 미달 상태에서
테스트를 바꾸지 않는다. `equation-baseline-oracle/normalization-conclusion.json`과
`cjk-pdf-installed-font-comparison.json`에 근거를 보존한다.


## C423 #8064 Native 품질 하락의 자동 검출

사용자 요청에 따라 기존 `visual_sweep.py`의 Native 실행에만 승인된 #8064 품질 기준을
연결했다. PR 번호로 임의 자료를 찾는 기능이 아니라 입력·독립 Print PDF SHA로 등록된
179조건/2082쪽의 Native 원본 TSV를 비교한다. WASM·SVG 경로에는 이 추가 계약을 적용하지
않으며 이 검사 때문에 WASM 실행을 요구하지 않는다. 기존 각 backend의 다른 gate는 유지한다.

같은 Print/96DPI/compat/PDFium153.0.7999.0/raw2px 조건에서 각 쪽의 하락폭을 Decimal로
계산한다. 정확히0.1%p를 포함한 하락, 누락·중복·쪽수 차이·기준 hash 손상은 보류다.
등록된 Native 입력의 측정 조건이 다르면 점수를 직접 비교하지 않고 재검토로 기록한다.
평균·보정 점수·글꼴 예외로 이 품질 하락을 상쇄하지 않는다. 통과해도 기존 `not_evaluated`를
승인으로 올리지 않으며 글꼴 출처·내용 소유·직접 판독·동일 조건의 최신 devel 비교는 별도다.
이 도구 계약 검사는 원인별 Rust 렌더러 회귀 검사를 대신하지 않는다.

도구 전체186건은 정상 의존성이 있는 Python에서 PASS(12.557초)다. 최초 시스템 Python
실행에서는 PDFium 미설치와 새 연결의 누락 DPI 처리 오류가 있었다. DPI 누락을 안전하게
처리하도록 고친 뒤 PDFium을 갖춘 기존 검증 Python으로 전체를 다시 실행했다. 기대값을
낮추거나 검사를 제거하지 않았다. 신규 계약 검사는 Native 기준 hash·전쪽 수집·정확한0.1%p
경계·실루엣 원값·조건 불일치·자료 손상·WASM 비적용을 확인한다. 기존 CI 자동 수집을 따른다.

실제 fresh Native는 고정4a07와 runtime211fff의 HWP/HWPX18쪽씩, 총72쪽을 동일 조건으로
재출력했다. 후보 최저98.09635/98.13231%, 두 문서 모두18/18쪽이며 base 대비0.1%p 이상
하락0쪽, #8064 Native 승인 기준 대비0.1%p 이상 하락0쪽이다. 최근192DPI 진단의16쪽
88%대를 이96DPI 승인 기준과 직접 비교했던 판단은 철회한다.192DPI 출력·실제 미세 차이는
별도 진단으로 보존한다. 사용자 지정 pdftoppm 대조도 별도 TSV로 보존하며 PDFium 점수와
혼합하지 않는다. Native-only 범위 지정 후 예정 WASM은 시작 전에 취소했다.

실행·producer·Native 원본 TSV·두 rasterizer의 review/overlay·최종 비교는 ignored
`rebase-4a07d3ae-20261019/repair16-8064-96dpi/`에, 도구 테스트 로그는
`8064-regression-guard/native-only-visual-tool-tests.log`에 보존했다. 현재 Native-only 코드로
원본 capture manifest를 다시 평가한 `native-only-assessment.json`도 기록했다. 원래 capture
시점의 도구 hash는 덮어쓰지 않았다. 이는179조건 전체 재출력·전체 Rust 회귀·실제 제품
소비 검증 완료를 뜻하지 않으며 나머지 보정·검증은 계속한다.


## C424 Native 전쪽 재검증과 레거시 분수 프레임 보정 중간 기록

runtime211fff의 frozen Native14로8조건921쪽(새 캡처885쪽, 같은 producer·입력·PDF·
글꼴·96DPI의 Native18쪽 두 조건36쪽은 PNG hash 확인 후 재사용)을 집계했다.
PDFium 기준 분할197쪽 세 조건 최저94.64257%, 원본HWP/HWPX115쪽 각96.97691%,
sample16 HWP5저장64쪽97.96262%, 일반 수식18쪽 두 조건98.09635/98.13231%다.
전체 TSV에서90% 미만·누락0쪽이며 독립 Print 쪽수와 일치한다. 이는 아래 새 수식 후보의
전쪽 결과가 아니다. Native capture8/16쪽 묶음/완료SVG hash manifest 뒤 정리를 유지했다.
첫 묶음 지표 계산의 spawn 재진입 오류는 driver의 폴더 생성을 main guard 안으로 옮겨
보정했다. 이미 완료한 Native/PDF PNG는 입력·CLI·DPI/쪽수 provenance와 파일 hash를
확인하고 지표만 다시 계산했다. 실패 state·로그와 재개 근거를 보존했다.

사용자 지정 pdftoppm도 같은 Native PNG hash를 확인해921쪽 전부 별도 재산출했다.
분할197쪽 세 조건94.88648%, 원본HWP/HWPX115쪽96.90334%, sample16 64쪽98.18615%다.
수식18쪽 두 조건은6쪽89.47931/89.52344%가 남는다. PDFium 통과로 이 미달을 숨기지 않는다.
다른 PDF rasterizer끼리 점수를 같은 측정값으로 취급하지 않는다. 두 종류 TSV와 review/overlay,
실제 producer는 ignored `repair17-native-whole/`에 보존했다. 직접 연 분할90쪽에는140℃
앞뒤 줄 소속 차이가 있고130쪽은 뒤 문단 위치가 복구됐지만 수식·첨자 차이가 남는다.
두 잔여를 분리해 원인·고정base 대조를 계속한다. 전쪽 점수만으로 완료·승인하지 않는다.

새 CJK 독립 표본은9/12/18pt의 단독·혼합·중첩 분수21개와 괄호·식 범위 표본21개다.
0크기/0baseline 입력을 한컴2024에서 정상 저장한 뒤 그 정확한 HWP를 Print 출력했다.
한컴 저장의 HFT 단일 줄은1.2em/84%, 단순 분수는2.34em/67%이고 아래 분수의 추가 줄은
논리 점유로 합성된다. 이 값은 사설 글꼴 binary의 복사나 문서별 화면 좌표가 아니다.
실행 입력·정상 저장본·Print·원점은 `equation-baseline-oracle/cjk-frame-controls/` 및
`repair18-cjk-frame-trace/parenthesis-oracle/`에 보존했다. 새 정식 fixture/golden은 추가하지 않았다.

원인 경로는 `versionless HYhwpEQ → EqLayout의 피연산자 논리 프레임 → 분수 높이/기준선/
수학 축 → 공통 LayoutBox → SVG/Canvas/Native`다. 기존 TeX형의 글리프 높이를 HFT의
논리 줄로 소비해 분자·괘선이 내려갔다. 공통 분수 결과에서 독립 논리 줄과 중첩 높이를
소비하도록 보정했다. 단독 CJK 분수는 피연산자가 이미 등록 원점을 가지므로 식 전체의
등록 원점을 중복 가산하지 않는다. 같은 분수에 다른 항이 붙은 경우는 원래 바깥 줄 등록을
유지하며 문서명·쪽번호·좌표 조건을 추가하지 않는다. 현대 Equation Version60은 유지한다.

채택한 중간 후보19의 Native192DPI는 단독 가/나 분수32.63506→98.32090%,
sample16 원본6쪽99.41666%, HWP5저장본6쪽99.03633%다. 원본3/18쪽99.47882/97.44402%,
저장본3/18쪽96.85161/98.35911%는 유지됐다. 복합 진단 표본의 미달은 남아 있어
이 결과를 전체 수식 지원 완료로 확대하지 않는다. 기존 수식8검사 직접 실행은6PASS/2FAIL,
두 FAIL은 바깥bbox 이동/셀 포함 assertion이며 변경하지 않았다. fmt 전체 check는 PASS다.
Native 빌드와 source1840파일 hash 검증을 완료했고 frozen19와 현재 구현의 동일성을 확인했다.
전체 nextest·새 후보 전쪽·fresh WASM·실제 제품 소비는 아직 새 후보에서 완료하지 않았다.

후보20은 피연산자 inset 제거와 현대 괄호 경로 재사용으로 단독 분수를100%까지 높였지만,
원본6쪽96.68274%·저장본6쪽96.47934%로 후보19보다 후퇴했다. 이를 철회하고 후보19의
두 구현 파일만 복원했다. 후보20의 source.patch·producer·PNG/TSV·철회 사유는 보존한다.
높은 한 쪽 점수로 다른 문서의 하락을 면제하지 않았다. 새 회귀/기대값 변경과 PR 제출은
남은 보정·독립 시각 선행 조건·새 head 검증을 마친 뒤 판단한다.


## C425 참여 수식의 실제 후속 줄과 독립 제목 간격 회귀

현재 runtime `fcf6d08dda00765d10bd35235cd91e986c00f09b`와 동일한 frozen19의
Native8조건921쪽을 새로 산출했다. 모든 쪽 raw90% 이상이며 이전 Native14와 비교한
0.1%p 이상 하락은0쪽이다. 이 비교는 이전 브랜치 구현과의 대조이며 최신devel 전수
보존 검증을 대신하지 않는다. 같은 fresh WASM23의 저장197쪽 입력도197/197쪽,
최저94.66706%·미달0쪽까지 완료했다. 나머지 WASM 조건은 계속 실행 중이다.

새 `issue_7445_participating_equation_frame` 두 검사는 기존 자동 수집으로
`regression_suite_010`에서 실행된다. 입력은 committed
`regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp`
(SHA256 `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`)이며,
독립 기준은 `pdf/issue4138-ascii-split-20261016-2024.pdf`
(SHA256 `5a82290de1571c692c7246fe6b57ae136602ae52c2068f2568fffc6c91bfd571`)다.

첫 검사는 원본 문단1698 수식의 참여 속성·높이2232HU·Percent160%로부터 후속 빈 줄1699가
47.616CSSpx 이상 전진해야 함을 확인한다. actual47.626667이며 비참여 수식1701의 저장
leading26.4CSSpx도 보존한다. 실제 TextLine의 문단·줄 소속과 유일성을 함께 검사한다.
두 번째 검사는 같은 쪽의 두 제목1697/1700을 소속으로 찾아 독립 Print의 같은 글꼴 기준선
248.0489173333/346.8209733333CSSpx 차이98.772056을 ±2CSSpx로 보호한다.
원점 자체 대신 같은 글꼴 제목의 간격을 비교하므로 글꼴 baseline 공통항이 제거된다.
현재 실제 간격98.826667이다. 이 참여 수식 경계의 해당 제목 쌍은130쪽에 있으며 다른
높은 점수의 페이지로 동작을 대체하지 않았다.

회귀 추가 선행 조건은 같은197쪽 전부 Native/fresh WASM raw90% 이상이다. 해당130쪽은
96DPI·2px의 Native98.19711%/fresh WASM98.16280%이며 양쪽 review/overlay를 직접
판독했다. 제목·빈 후속 줄과 아래 본문의 순서·소속이 유지된다. 수식 자체의 괘선·첨자·횡간격
잔여는 별도 보정 대상으로 남으며 이 테스트가 수식 전체 모양의 완성을 주장하지 않는다.
픽셀 핀은 확인된 두 제목 간격에만 적용한다. 양쪽 TSV·review/overlay는 ignored
`repair21-native-whole/whole/native/giant4138-current-ascii/`와
`repair25-wasm-whole/whole/wasm/giant4138-current-ascii/`에 보존했다.

실제 frozen13(C422 전)에서는 두 건 모두 FAIL(줄 전진39.36, 제목 간격90.56),
frozen19에서는 두 건 모두 PASS다. `repair26-frame-leading-regression/before.log`,
`after.log`에 보존했다. 초기 진단의 line_index 타입 오류는 수정하고 실패 로그도 보존했다.
`--prepare` 후 정식 Native nextest는2PASS/0FAIL, 실행3.198초·컴파일6분21초·exit0이다.
formal-native-nextest.log에 결과가 있다. 생성 suite·manifest·로그는 커밋하지 않는다.
전체 nextest나179조건 보존·실제 제품 소비 검증 완료로 확대하지 않는다.


## C426 HFT 분수의 등록 여백과 실제 괘선 범위 분리

C425 뒤 runtimefcf의 fresh WASM8조건921쪽도 전부 완료했다. 저장197쪽 세 조건은
94.66706%, 원본HWP/HWPX115쪽96.98894%, sample16 HWP5저장64쪽97.99309%,
일반 수식18쪽 두 조건98.04527/98.08044%다. 각 독립 Print와 쪽수가 일치하고 raw90%
미달0쪽이다. `repair25-wasm-whole`의 전쪽 TSV와 producer에 보존했다. 이 결과는 아래
괘선 변경 전 구현의 결과이며 새 괘선 후보의 전쪽 통과로 재인용하지 않는다.

새 원인 경로는 `분수 피연산자 CJK 등록 inset → 공통 Fraction의 내용 범위 → 실제 괘선
시작·끝 → Native/SVG/Canvas2D/JSON→Studio CanvasKit`이다. 기존 괘선은 논리 상자의
폭에서 양쪽0.05em을 뺐으므로 한글 대체 face의 선행 등록 여백까지 괘선에 포함했다.
독립 한컴2024 Print의9/12/18pt 표본에서 분자 원점·폰트 em과 PDF 선분을 직접 대조했다.
괘선의 내용 바깥 여백은 인쇄 격자의 반올림을 포함해 약0.18em이다(왼쪽0.18666/
0.17996/0.17331, 오른쪽0.17341/0.19003/0.18671). 사설 글꼴 윤곽을 복사하지 않았다.
기준 PDF SHA와 선분·원점은 ignored `repair27-legacy-delimiter-diagnosis/
independent-fraction-rule-metrics.json`에 보존한다.

피연산자 등록 전진을 별도 반환하고 실제 내용 범위에0.18em을 더해 괘선의 시작·끝을
공통 Fraction 결과로 발행한다. 기존 글리프 원점·논리 폭·높이·줄 소속은 유지한다.
현대 수식은 기존 괘선 좌표를 그대로 발행한다. 각 backend의 별도 상수 추정을 없애며,
Studio의 이전 JSON 입력에는 기존 좌표 fallback을 유지한다. 문서/쪽 번호 분기를 추가하지 않았다.

현재 후보27의 Native192DPI 집중14쪽 대조에서 단독 가/나 분수98.32090→100.00000%,
복합 CJK 표본3쪽은86.29758/81.15219/75.58331%다. 괄호 표본3쪽은75.53038/
66.84647/55.22232%로 미달이 남는다. sample16 원본3/6/18쪽99.47882/99.40948/
97.44402%, HWP5저장본3/6/18쪽96.85161/99.02941/98.35911%다. 기존 후보19 대비
6쪽은각0.00718/0.00692%p 하락하며 사용자 기준0.1%p 이상 하락은14쪽 모두0건이다.
일반 현대 수식18쪽의16쪽192DPI88.19667%도 이전과 같다. 높은 단독 점수로 미달을
상쇄하지 않는다. 단독 분수와 저장본6쪽의 review/overlay를 직접 열어 괘선·본문 보존과
잔여 괄호 차이를 확인했다.100%는2px raw 실루엣이며 엄격 RGB 일치를 뜻하지 않는다.

Native 빌드exit0(5분37초), Studio 타입 검사exit0, 기존 렌더 backend65PASS,
고정base4a07의 manifest/unit-tier 검사exit0이다. producer·source patch·후보19 대조·
TSV·PNG·로그는 ignored `repair27-legacy-rule-candidate/`에 보존한다.
source1840파일 hash를 빌드 전후 확인했다. 새 renderer 회귀/기대값 변경은 아직 추가하지 않았다.
이 단계는 원인별 구현 보정의 로컬 커밋이며 새 fresh WASM·전쪽·전체 nextest·179조건 보존·
실제 제품 소비 검증은 계속한다. 기존 두 실패 검사는 그대로 보존한다.


## C427 완료 단독 분수의 독립 px 회귀와 초기화 응답

C426의 runtime `72ef9c21bec6ee20a5aed74a1c329220454f8de9`를 fresh WASM으로 다시
컴파일했다. locked wrapper web/pkg/no-opt의 Mac 로컬 대체 빌드exit0(189.697초)이며
Docker 최적화 결과가 아니다. root pkg와 Studio public의 JS/WASM hash가 일치한다.
새 집중14쪽은 Native와 같은 정확한 입력·Print로 산출했다. 단독 한글 분수100%,
원본 sample16의3/6/18쪽99.80378/99.81323/97.54876%, 저장본은97.19902/99.42673/
98.50096%다. 복합 CJK·괄호 표본의 미달은 남으며 전쪽·전체 수식 지원 완료로 확대하지 않는다.
실행은 `repair28-fresh-wasm`과 `repair29-wasm-focus`에 보존했다.

새 `completed_legacy_fraction_rule_uses_content_bounds_instead_of_registration_space`는
기존 합성 model helper의 동일한 문서·수식 생성 결과를 사용한다. 정확한 현재 생성·저장본
`from-current-test.hwp`의 SHA256은
`b932adebd711e649825145dd7f02ec8b4374bf279f40f94ed209e743ea01b50e`다.
예전 oracle 파일과 바이트가 다르므로 그 Print를 재사용하지 않고 정확한 현재 입력을
한컴2024에서 다시 Print했다. job`ed5f546b-ff0a-48da-9084-aba9311b137c`, engine2024,
Hancom13.0.0.3901, method0/frame_print_to_pdf_ex_one_up이며 PDF는
`pdf/issue7445-current-model-baseline67-2024.pdf`, SHA256
`b033ccd684d6fbaaec402a31ba632f5a6697c72d1ed3e670a1f07d7dc46ca0c4`다.

정확한 입력·새 Print의 전1쪽은 Native/fresh WASM 각각100.00000%,1/1쪽이다.
인쇄 격자는192DPI이고 회귀 기대 좌표는96DPI CSSpx로 환산했다. 실제 PDF 선분은
x79.9573333333..101.8653333333이며 괘선의 양끝을 ±2CSSpx로 검사한다.
현재 테스트의 in-memory 입력과 Print용 저장본 재파싱 입력이 같은 수식 소유·bbox와
최종 수식 SVG를 낸다는 실행도 확인했다. 원점 대신 논리 상자만 비교한 증거가 아니다.
단일 수식/괘선, 분자·분모의 소속·순서·누락·중복, 괘선의 두 피연산자 포함과 세로 순서도
함께 검사한다. 이 동작의100% 완료 쪽을 우선 선정했으며 사설 font binary는 필요하지 않다.

수정 전 frozen19에서는 실제 FAIL(괘선 시작76.386667), 수정 후 frozen27에서는 PASS다.
정식 nextest의 괘선1건은PASS/exit0(실행0.015초·컴파일2분01초)다. 함께 선택했던 기존
참여 수식2건은 `--prepare` 뒤 suite010→027로 배정이 바뀌어 첫 명령에서 실행되지 않았다.
현재 파생 배정을 다시 조회하고 누락2건만 suite027에서 보완해2PASS/exit0(1.329초)을
확인했다. 처음1건 결과를3건 통과로 보고하지 않는다. CLI의 잘못된 --print 인자 순서와
진단의 상대 include/직렬화 API 오류는 정정했고 실패 출력을 정식 성공 증거로 쓰지 않는다.
기존 두 baseline 실패 검사의 입력·기대값은 이 단계에서 변경하지 않았다.

ignored `repair30-current-model-qualification`에 generator/입력 SHA, 같은 최종 SVG 증명,
Print의 start/status/download·독립 좌표·양쪽 TSV·review/overlay·실제 before/after·정식
로그를 보존한다. 양쪽 review/overlay를 직접 판독했고 단독 분수의 글자·괘선 소속이 유지된다.
100%는2px raw 실루엣이며 엄격 RGB 일치의 주장이나 미달 복합 수식의 면제가 아니다.

Studio7736의 실제 새 head 초기화 응답도 캡처했다. Chrome155.0.8059.84에서 WASM 초기화를
확인했고 JS/WASM 모두HTTP200이며 frozen fresh 빌드와 일치한다. JS 원본 hash는
`23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`이고
실제 Vite 응답은`f2294e7a48873a5ffe66aa3409d96be9a7d191fce5973dbf6b514faa09cc929b`다.
JS는 인라인 source map의 정확한 asset URL 변환을 검증했으며 raw byte 같음으로 보고하지 않는다.
WASM은`b26d2d2293e8e5bffb9b1faa04d98fea0a341f0269d9bd97af55935f7530ab57`로
실제 응답과 byte hash가 같다. `repair31-browser-initial-response/result.json`에 보존했다.
최초 helper 기본값이 host CDP를 선택해 ENETUNREACH로 종료된 실행은 INVALID로 분리했다.
브라우저/탭 생성 전 실패했으며 실제 프로세스 확인 뒤 local headless를 명시해 정상 재검증했다.
이 초기화 증거는 실제 문서 paint36건·글꼴 선택·CanvasKit 준비/지원 검증을 대신하지 않는다.


## C428 기존 두 실패 검사의 정확한 편집 상태 전쪽 대조

기대값을 바꾸기 전에 기존 inline 검사67/80과 셀 검사90/95의 실제 공개 속성 편집 순서를
그대로 실행했다. 각 상태를 현재 runtime72ef(frozen27)에서 저장하고 저장 제품이2024임을
확인한 뒤 그 정확한 HWP를 한컴2024 Print로 출력했다. generator·입력/PDF SHA·job·실행
제품/Print 경로·Native/fresh WASM producer·전체 TSV·review/overlay는 ignored
`repair32-existing-test-input-audit`에 보존한다. 모두 독립 Print와1/1쪽이다.

| 기존 검사 상태 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| inline 기준선67 | 100.00000% | 100.00000% |
| inline 기준선80 | 47.73869% | 47.73869% |
| 셀 기준선90 | 40.62027% | 40.62027% |
| 셀 기준선95 | 37.57485% | 37.57485% |

이전 진단은 바깥 예약bbox와 안쪽 표시 기준선을 혼동한 assertion 및 부동소수점 포함 검사의
문제를 찾았고 수정 prototype은 통과했다. 그러나 이번 독립 Print 대조에서는67 이외의 상태가
90% 선행 조건을 충족하지 못한다. prototype 통과만으로 기존 두 검사를 정정하거나 출력이
정상이라고 판정하지 않는다. 저장된 수식 기준선·후속 줄/셀 소유와 한컴 재계산의 실제 경로를
계속 확인하며, 기대값·baseline·허용 오차를 바꾸거나 검사를 제거하지 않았다.
이8건은 진단 완료이며 전체 회귀·시각 수용·PR 준비 완료가 아니다.


## C429 HFT 기준선 편집의 내부 프레임 보정과 기존 검사 재검토

C428의 정확한 편집 상태를 더 대조했다. inline67/80의 독립 한컴2024 Print는 내용 스트림
SHA가 같고, cell90/95도 서로 같다. `repair33-baseline-origin-diagnosis/independent-origins.json`은
같은 분자·분모·괘선 좌표를 확인한다. 정확한 cell90 입력을 한컴에서 HWPX로 다시 저장하면
수식은 baseLine67·height2808HU로 정규화된다. 원본 저장 baseline을 전부67로 덮어쓰는
규칙으로 확대하지 않는다. 유효한 HWP3의66과 다른 소유 프레임은 계속 보존한다.

소비 경로는 `flow_metrics_hwp → composer의 줄 기준선 → paragraph_layout의
host baseline - equation baseline → 바깥 EquationNode bbox → layout_for_paint의 내부
layout.y → SVG/Native/Canvas`다. 저장 백분율은 바깥 슬롯의 글줄 정렬을 결정하는데,
HFT 논리 글줄·분수 프로그램의 표시 원점까지 같은 값으로 다시 옮겨 실제 글리프가 이동했다.
공통 HFT 글줄/분수 프레임을 root와 피연산자에서 함께 생산하고 그 프레임이 있는 HFT paint는
프로그램의 내부 원점을 유지한다. 지원한 프레임을 얻지 못한 종류의 기존 경로와 현대 수식은
유지한다. 문서ID/쪽 번호 분기, 저장 cache 삭제, baseline 강제 변경은 추가하지 않았다.

처음의 후보34는 내부 이동만 제거해 기존 실패 입력4쪽은100%가 됐지만, HFT의 순수 글줄
논리 프레임을 만들지 않아 단독 한글 대조군이 후퇴했다. 이 후보를 최종으로 채택하지 않고
`repair34-legacy-paint-origin-candidate/review-decision.json`에 보존했다. 후보36은 독립
9/12/18pt 생성본에서 이미 확인한 글줄1.2em/84%, 분수2.34em/67% 계약을 공통 root에도
사용한다. 단독 CJK 분수의 registration inset 소유도 유지하며 현대 matrix frame으로
다시 포장하지 않는다. 이는 표시 glyph의 ink box와 글줄의 논리 크기를 구분하는 보정이다.

기존 실패 입력4개는 그대로 두고 새 생산자에서도 같은 공개 속성 편집 순서를 실행했다.
inline67/80은 이전 입력과 byte가 같고, cell90/95는 생산 높이가2760→2808HU로 바뀐다.
새 셀 입력2개는 정확한 파일을 각각 한컴2024 Print로 다시 출력했다. 후보34에서 만든
새 입력과 후보36에서 재생성한 입력의 SHA가 모두 같음도 확인했다. 기존 실패 입력을
새 입력의 성공으로 대체하지 않았다. `repair35-current-edit-state-qualification/`에 원본
대조·새 Print job/제품/출력 방식·Native36 생산자·전1쪽 TSV를 보존한다.

| 입력 상태 | Native36 raw | fresh WASM37 raw |
| --- | ---: | ---: |
| 기존 inline67/80 각각1쪽 | 각각100.00000% | 각각100.00000% |
| 기존 cell90/95 각각1쪽 | 각각100.00000% | 각각100.00000% |
| 새 생산 cell90/95 각각1쪽 | 각각100.00000% | 각각100.00000% |

전체6상태·양쪽12출력은 쪽수1/1이며192DPI·2px raw 조건을 유지했다. 상태별 Native/PDF
PNG hash는4그룹이며 각 그룹의 review와 overlay를 직접 확대 판독했다. 두 피연산자·괘선의
소속/위치/순서가 유지되며 얇은 윤곽의 색상·안티앨리어싱 차이는 남는다.100%는 엄격 RGB
동일성의 주장이 아니다. `repair39-baseline-contract-prototype/qualified-image-groups.json`과
`direct-read.json`에 원본 이미지·SHA와 판독을 연결했다.

기존 두 검사를 이제 정정했다. 바깥 예약 bbox가 반드시 이동해야 한다는 기대값과 셀 내부
표시 원점에 source baseline delta를 다시 더하던 기대값을 제거하고, 실제 공개 수식 SVG의
두 글리프 원점·내용 순서·단일 수식/셀 소유를 검사한다. source baseline을 줄이 소비한다는
관계는 유지한다. 셀 포함 검사는 합계의 부동소수점 반올림만8ULP로 허용하며 물리 여백을
늘리지 않는다. 새로운 고정 전체 SVG hash/golden을 만들지 않았다. 기존 완료 분수의
독립 Print ±2CSSpx 괘선 양끝 검사도 그대로 실행한다.

같은 정정 prototype9건은 frozen27에서7PASS/2FAIL, 후보36에서9PASS다. 수정 전 실제 실패는
glyph 원점의 중복 이동을 검출하며, 이전 바깥 상자 assertion만 고친 prototype의 통과와
구분한다. 정식 자동 수집 경로의 실행 결과는 아래 확정 로그를 따른다.

기존 집중14쪽의 이전 후보27/29 대비0.1%p 이상 하락은 양쪽0건이다. 원본 sample16의6쪽은
Native99.40948→99.38125%(-0.02823%p), WASM99.81323→99.79231%(-0.02092%p)다.
이 미세 차이도 기록한다. 복합 CJK/괄호 표본의 Native 최저75.58331/55.22232%,
WASM75.92796/52.29732% 미달은 남아 있다. 일반 현대 수식16쪽192DPI의 기존 미달
Native88.19667/WASM89.01922%도 유지된다.96DPI 전체 통과 결과와 섞지 않는다.
이 값들 때문에 이번 국소 보정을 전체 수식 지원/PR 준비 완료로 보고하지 않는다.

Native36 빌드exit0(1분45초), fresh WASM37 locked web/pkg/no-opt Mac 대체 빌드exit0
(199.002초)이며 Docker 최적화가 아니다. source ledger1840파일과 root pkg/Studio public의
JS/WASM hash를 확인했다. WASM은`0706f7a344f77b7eec2099ca4776068838498d2819395f4a635c89613f3f4a27`다.
실제 Studio7736 초기화의 새 JS/WASM HTTP200 응답도 fresh 빌드와 일치한다. JS는 Vite의
정확한 URL 변환/inline source map, WASM은 raw byte hash로 확인했다. 초기 첫 계측은
추가 응답의 CDP body 취득 오류로 실패했으며 원인을 preflight로 확정하지 않는다. 원 실패를
보존하고 실제 응답을 다시 수집해 성공했다. 이는 실제 문서 paint36건·글꼴/CanvasKit 준비
검증을 대신하지 않는다. 증거는 `repair37-fresh-wasm/`과 `repair40-browser-initial-response/`다.

남은 복합 수식의 공통 원인도 분리했다. 같은12pt 독립 저장본의 단순 분수4종은 Native의
논리 폭보다 각각159HU 크고, 중첩 분수는583HU 크다. 중첩 분수의 CJK 원점은2.6~3.4CSSpx
왼쪽이다. `repair33-baseline-origin-diagnosis/cjk-logical-slot-width-differences.json`과
벡터 원점 대조에 기록했으며, 아직 이 폭 차이를 임의 상수로 보정하거나 정상 기대값으로
고정하지 않았다. 괄호 프로그램·복합 프레임의 실제 독립 원인을 계속 보정한다.

정식11건 결과: **11PASS/0FAIL**, test 실행1.356초·컴파일2분15초·전체139.522초, exit0.
기존 수식 계약9건과 참여 수식/제목 간격2건이며 전체 nextest 통과로 확대하지 않는다.
`--prepare` 뒤 실제 자동 배정은 legacy_contract→suite022, participating_frame→suite021이다.
초기 정식 명령은 nextest에서 -j가 test thread 별칭인데 --test-threads와 중복되어 파싱 단계에서
종료했다. 테스트는 실행되지 않았고 원 로그를 보존했다. 빌드 jobs는 환경 CARGO_BUILD_JOBS=8로
유지하고 중복 인자를 제거해 두 target을 순차 Cargo 실행 하나에서 다시 검증한다.
prototype 최초 누락 serde_json extern의 컴파일 오류도 별도 보존했으며 before/after의 실제
검사 결과와 섞지 않는다. fmt·diff 확인 및 manifest prepare는 성공했다.
최신 전체 nextest·Native/fresh WASM 전쪽·incoming179조건 보존·실제 소비 경로는 완료 전이다.
PR은 사용자 승인 후 진행한다.

대표 전후 review/overlay와6상태×양쪽 TSV 원본은 `mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/legacy-baseline-edit/`에 보존했다.


## C430 C429 code head의 전체 회귀와 직전 코드 대조

code head `9b1f0e198787ce4d9b70889b8803160d5fa4cdd0`에서 현재 정본 전체 명령
`cargo nextest run --locked --cargo-profile release-test --target-dir /Users/tsjang/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast`를
실행했다. 빌드 jobs8은 환경변수로 고정했고 Cargo는 순차 실행했다. 최종 결과는
**11,140실행 / 11,120PASS / 20FAIL / 45skip**, test727.549초·전체1160.487초·nextest exit100이다.
로그 끝의 실패 재목록은 같은20건이며40건으로 중복 집계하지 않는다. 실행 프로세스와 최종
summary/exit를 확인했다. 원 로그·명령·전체 source/test hash는 ignored `repair41-full-nextest/`다.
고정base4a07의 manifest 검사1641sources/7158attrs/48targets, unit-tier4101tests/296modules도exit0이다.
전체 실패가 남으므로 C429의11PASS를 전체 통과로 보고하거나 PR을 생성하지 않는다.

이번3개 runtime 파일의 기여를 분리하려고 테스트·fixture·default feature와 target/filter 조건을
같게 유지한22건을 대조했다. 현재 전체 실패20건에 C429의 기준선 양성 대조2건을 포함했다.
현재 코드에서는 **2PASS/20FAIL**(38.952초), 직전 `3d023ba4d7a1abec664c1697e8d46a8b6fb89602`의
runtime3파일로만 되돌린 실행에서는 **0PASS/22FAIL**(38.016초)이다. 두 명령과 테스트 source는
같고, source variant별 별도 로그·rlib/hash·backup ref를 남겼다. 결과적으로 이22건 범위에서
C429로 새로 발생한 실패0건, 복구2건, 양쪽에서 재현한 기존 실패20건이다.
이는 직전 브랜치 코드 대조이며 **fixed upstream/devel의 정상/미달 입증이나 parent 전체 회귀가 아니다**.
기존 실패라는 이유로 검사를 제거하거나 별도 이슈로 넘기지 않는다.

대조 뒤3개 runtime 파일을9b1f의 byte로 복원하고1840 runtime source hash를 다시 확인했다.
현재 branch/HEAD는 유지했으며, 사용자 작업·원본/PDF·다른 작업장은 바꾸지 않았다.
backup ref는`refs/backup/7445-before-parent-default-triage-20261019`다. root pkg/Studio public의
새 JS/WASM hash도 그대로다. **공용 target의 마지막 default compile은 대조용 parent runtime**이므로
그 CLI/rlib를 현재 head의 산출물로 재인용하지 않는다. 현재 code head의 Native는 frozen36,
fresh WASM은 frozen37이며 다음 Cargo/Native 게이트는 실제 source로 재빌드한다.
대조·복원 증거는 `repair45-parent-default-comparison/{comparison,restore,producer}.json`이다.

실패20건의 원문 inventory는 `repair44-full-failure-triage/final-failures.json`에 보존했다.

| 실패 계약 | 관측 / 다음 검증 |
| --- | --- |
| issue7445 native_bold_tracking | issue265 그림 뒤 본문686.7467 vs 독립708.1600CSSpx; 원본/저장 경로·Print 대조 |
| issue3930 hwpx_hwp_save_layout | 저장 HWP294쪽 tree가 원본 HWPX와 다름;84개 y 필드차·대표3.3px, 양쪽 해당 쪽 Equation0개 |
| verified_exam source_content exam_all42 | 독립4쪽 vs 실제5쪽; 최초 경계·내용 소유 대조 |
| verified_hancom paragraph_frames original_font | 서로 다른 source face의 본문/구두점 소유 |
| issue7017 PUA dotted_line | 실제 점선0 vs source60개 |
| issue7445 hft_glyph_paint_frame | roadmap 실제304 vs 독립302쪽 |
| issue5846 mixed_nested_partial_row | 원본 cell(9,2) 소실 |
| issue7445 hwp3_source_frame | 원본 inline 수식의 TAC/flow_with_text 소유 |
| issue7642 sample16_2022 paragraph | 2022저장본 BCP 꼬리1줄 vs 실제2줄; #8064 보존 범위로 우선 대조 |
| issue6520 svg_layer_contract | multi-codepoint subset의 font/opentype data URI 기대값; 실제 공급 포맷/소비 확인 |
| issue7418 ascii_punct latin_slot | source face와 fallback chain의 필드 의미 대조 |
| issue4962 font_metric_coverage | positive exact-hit source 분류0 vs1 |
| roadmap agency_table | Germany 이어받기와302쪽 source 소유 |
| roadmap fixed_term_table | Finland 이어받기와302쪽 source 소유 |
| roadmap closing_note_fragment | 마지막 note fragment source lookup 소실 |
| issue7642 header_shape | PAPER 참조 사각형의 header 원점 비율0 vs0.0311268; #8064의 독립 Print 대조 |
| issue6078 inline_table_seg_lookup | 표 앞 빈 줄의 후행 띠360 vs0HU |
| off_canvas partition15 | 기존 paper 밖 요소 baseline 증가; 원래 TSV와 개별 문서 원인 대조 |
| issue1892 group roundtrip | source 누적 원점(0,0) vs(1352,0)HU |
| issue937 f081c flow/paint dot | 실제 점선 glyph의 소실 |

기대값·baseline 변경이나 assertion 완화는 하지 않았다. 각 기존 원본의 독립 출력과 source
계약을 확인하고, 현재 코드의 실제 회귀는 구현에서 보정한다. 특히 #8064의 BCP 꼬리·header
도형, 현재 완료 대상의 쪽수/내용 소유를 유지하는 검증을 우선한다. C429와 별도로 복합 CJK/
괄호 미달과 논리 전진폭도 계속 보정한다. `repair42-logical-advance-proposal`은 적용 전 후보일
뿐이며 현재 source에는 적용하지 않았다. 전쪽 Native/fresh WASM·incoming179조건 보존·실제
제품 소비 검증은 여전히 완료 전이다. PR은 준비 완료 보고 뒤 사용자 승인을 받아 진행한다.


## C431 #8064 헤더 사각형의 부모 좌표계 복원과 독립 px 보호

기존 `issue7642_paper_relative_rectangle_keeps_header_origin_and_precedes_body` 실패를
기대값 변경 전에 원 PR8064→통합/후속 코드→독립 Print 순서로 추적했다. 원 merge
`9b01d5d1447220b3a7345703e79ad42cf3f502f4`는 fixed base4a07의 조상이다. PR8064의
179조건 TSV에는 이번 header fixture/2022 sample16이 없으므로 그 전쪽 수용으로 대신하지 않았다.
정확한 source/PDF는 `tests/fixtures/issue_7642/header-origin/`의 한컴2020 저장본/Print이며,
SHA는 대표 asset README에 고정했다.96DPI/compat2022의 같은 전1쪽에서 fixed4a07의
Native/WASM은100%, 수정 전9b1f의 양쪽은8.70583%다. 실제 회귀이므로 구현을 보정했다.

원본 Paper/Paper 사각형 offset1500/2250HU가 branch0de2에서 도입한 실제 body/paper
영역을 소비하면서 물리 용지(0,0)에 더해져20/30CSSpx가 됐다. 최종 Rectangle paint가
그 원점을 그대로 소비하고 Header bbox도 자식을 포함하도록 확장되어 기존 상대 원점 비율이0이
됐다. 위반 규칙·생산→소비 사슬은 구현 전에 `repair46-header-fixed-base-comparison/violated-frame-rule.md`에
기록했다. 헤더/푸터의 도형·수식 호출에 부모 area를 공통 참조 프레임으로 전달하고 불필요한
body/paper 파라미터를 제거했다. 측정/위치 계산이 같은 area를 소비한다. 실제 본문의
물리 용지/본문 참조와 별도 머리말 그림 경로는 바꾸지 않았다. 문서/포맷 예외·clamp도 없다.
원본 sample11의 header는 TAC Picture이고 해당 경로에 도형이 없음도 source probe로 확인했다.

Native47 CLI 빌드exit0(108.870초), fresh WASM49 locked web/pkg/no-opt Mac 대체 빌드
exit0(221.527초)다. Docker 최적화가 아니다. runtime1840파일 ledger와 patch SHA
`979f87a9fe1e3f37bb2bc7bfc057e7c5d75269d9b025ecdd9aa426533228c50b`를 고정했다.
Native CLI SHA `da2838e0628f13132702c3d888ec669e5f567bedab44b9b15e48702bfdc5d665`,
WASM SHA `56887cbaec709bb56ac9781e01fe4fea07b668b49da1be264a5bf9cef34ed9fb`다.
Root pkg/Studio public hash와 실제 새 Studio7736 초기화 응답도 일치했다(Chrome155.0.8059.84,
HTTP200, JS의 정확한 Vite변환/source map 및 WASM raw byte). 최초 browser 명령은
headless 인자 누락으로 기본 Windows CDP에 접속하다 ENETUNREACH로 실패했고, 원 실패를
보존한 뒤 실제 Mac Chrome/headless/VITE_URL 명시로 성공했다. 제품 실패나 문서 paint/CK
통과로 혼동하지 않는다. 초기화만 확인했으며 actual36/문서 소비 검증은 별도 미완료다.

헤더 전1쪽은 Native/fresh WASM **각100%로 복원**됐다. Native review와 WASM overlay를
직접 판독해 원점·본문 내용 소속을 확인했다. 이는2px raw 실루엣100%이며 엄격한 픽셀 일치는
아니다. 같은 source/Print 전쪽 양쪽98이상·확대 판독 조건을 충족한 뒤 기존 검사에 독립
PDF stroke 경계95.468/67.765/195.415/107.723 CSSpx(96DPI)의 **±2px** 보호를 추가했다.
Header bbox의 상대 관계만으로 양쪽이 같이 틀리는 경우를 막으며 기존 source 소유·비율·본문
내용/순서 검사는 유지한다. 같은 새 검사에서 frozen36은 `20 != 95.468`로1FAIL(exit101),
frozen47은1PASS(exit0)다. 정식 NativeSkia 자동 suite 경로의 header shape/머리말 그림/
머리말 행 높이3건은 **3PASS/0FAIL**, filter730건, test0.056초·compile2분18초·전체142.369초,
exit0이다. prepare 후 실제 target022/027/002를 다시 계산했다. fmt/diff도 통과했다.

| 대조 범위 | Native 수정 후 최저 | fresh WASM 수정 후 최저 | before/after |
| --- | ---: | ---: | --- |
| header 사각형 전1쪽 | 100% | 100% | 양쪽8.70583→100 |
| pic-in-head02 전6쪽 | 57.60580% | 57.60692% | 전쪽 동일;4쪽 미달 유지 |
| sample11 선택2/50쪽,192DPI | 89.13433% | 90.66313% | 동일;Native50쪽 미달 유지 |
| 본문 Paper A3/A4/Letter 각 전1쪽 | 100% | 100% | 동일 |

24쪽 전후 쌍에서0.1%p 이상 하락0쪽, header2쪽 쌍 개선·나머지22쪽 PNG SHA까지 동일하다.
이는 이번 헤더 보정 전후 대조이며 sample11 전151쪽 수용이나 대조군의 upstream 미달 증거가
아니다. current canonical PDFium raw TSV와 별도 pdftoppm audit TSV를 혼합하지 않았다.
`repair48-header-focus/`에 실행 로그·producer·전쪽/선택쪽 TSV·비교/PNG hash를 남겼다.
최종 대표 review/overlay6장, fixed base/header 및24조건 원본 TSV, 비교 TSV는
`mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/header-paper-frame/`에 보존했다.
최종 PR 검토에 쓸 전후 Native 및 fresh WASM 패널이며 게시 전에 승인받는다.

C430 전체20FAIL 중 header 한 건을 집중 경로에서 복구했지만 전체 nextest를 다시 실행한
결과로19FAIL이라고 보고하지 않는다. 다른19건과 복합 수식·전쪽/실제 제품 소비 검증은 남아 있다.
BCP의 fixed4a07 RenderTree1줄과 현재2줄 차이를 원본으로 재현했다. 당시 Print도1줄이라고
판단한 내용은 **C432의 독립 PDF 재판독으로 정정한다**. 실제 Print는2줄이며 현재와 같다.
source pi83의 run폭 증가는 글리프 폭 변화가 아니라 첫 줄 공백 분배2.77926CSSpx다.
`repair50-bcp-analysis/`의 원 로그와 독립 Print 소유 대조를 보존한다. 현재 PR 준비 완료가 아니고,
PR 생성·원격 push·게시·merge는 준비 결과 보고와 새 사용자 승인 후 진행한다.


## C432 BCP 한 줄 기대값의 독립 Print 재검토와 중요한 줄 경계 보호

C431의 BCP 판단을 정정한다. 정확한 source/PDF SHA를 재확인한 독립 한컴2020 Print3쪽은
`공사 정보처리… Planning) 수`와 다음 줄 `립`으로 구성된다. PDF 첫 기준선677.40pt,
둘째698.16pt·둘째 원점107.52pt다. 현재 출력의 두 줄과 맞으며, fixed4a07의 한 줄은
`is_sample16_2022_bcp_orphan_tail_lineseg`라는 해당 문구 전용 예외에서 왔다. branch0de2가
그 예외를 제거한 뒤 기존 테스트의 한 줄 기대값이 남아 잘못 실패했다. 본문 run312→325.896,
Latin230.827→239.164CSSpx 차이도 글리프 자연 폭 변화가 아니라 첫 줄 공백 분배2.77926px다.
독립 Print의 원점도 이 분배를 확인하므로 다시 한 줄로 합치는 구현을 넣지 않았다.
원 PR8064/기존 stage496의 관계형 테스트 추가 근거를 추적했고, stage496에도 재판독 정정을
연결했다. 원문·PDF·잘못된 기대값의 실제 실패 로그를 보존했다.

검사 변경 전에 현재 runtime d34926(frozen47/fresh49)의 같은 원본 전64쪽을 현재 canonical
Native PNG/fresh WASM PNG로 산출했다.192DPI/compat2022, raw2px 문턱90%를 그대로 썼다.
Native 최저94.14342%, WASM94.60213%,64/64쪽 일치·미달0쪽이다. 해당3쪽은99.48430/
99.81839%이며 Native review와 양쪽 overlay의 `수|립`, 내용·원점·뒤 문단을 직접 판독했다.
원본 SHAfec3671c…/PDF6737b57a…의 전체값은 asset README에 둔다. 각64쪽 manifest의
실제PNG/PDF hash192쌍과 runtime1840 source ledger도 검증했다. 별도 pdftoppm audit는
canonical 수치와 섞지 않았다. source/runtime를 바꾸지 않아 Native47/fresh49를 재빌드하지 않았다.

기존 BCP 함수 안의 원문 전체 구절 선택을 유지하고 독립 Print의 두 줄 소유, 첫 줄 `수`까지·
둘째 `립`, 전체 내용 순서/누락/중복, 두 줄 비겹침과 **마지막 줄 뒤** 후속 원문 문단 보존을
검사한다. 완료3쪽의 양쪽98이상·직접 판독 조건을 확인한 뒤 PDF의 text origin 기준선
903.20/930.88 CSSpx와 둘째 글줄 원점143.36 CSSpx를 ±2px로 추가 보호한다. PDF 잉크 상단과
글줄 상단을 섞지 않는다. 기존7쪽 표의 소유·세 줄·원본 내용·앞 문단/내부 겹침 검사는 유지한다.

현재 코드의 이전 검사 실행은1PASS/1FAIL(실제2 vs 기대1), 교정 후 prototype은2PASS다.
같은 새 검사를 fixed4a07에서 실행하면1PASS/1FAIL(실제1 vs 독립2)로 옛 예외를 검출한다.
정식 NativeSkia suite018은 **2PASS/0FAIL**, filter285, test0.164초·compile6.89초·전체9.550초,
exit0이다. prepare 후 실제 배정을 다시 계산했다. 초기 fmt check는 긴 assertion 줄바꿈만
지적했고 서식 적용·prepare 뒤 정식 실행했다. prototype의 hash는 서식 적용 전이며 정식 로그는
현재 파일을 사용한다. `repair52-sample16-2022-whole/`에 모든 원 로그/관측을 보존한다.

고정base Native도 동일 입력/Print·글꼴 경로·DPI의 전64쪽을 실행했다. 최저59.36766%,
미달50쪽이며3쪽72.37067→현재99.48430%다. 다만 Native63/64쪽은 base99.80954/99.67493%
대비 현재97.65514/98.22800%로 각각 **2.15440/1.44693%p 하락**했다. WASM 현재63/64쪽은
99.80727/99.67064%이고 RenderTree의 위치 차이는 대부분0.1~0.3CSSpx여서 Native paint의
선택/실제 자원 소비도 추적한다. Native의 두 하락을 평균 향상이나 전체90통과로 상쇄하지 않는다.

처음 driver가 계획한 base WASM은 현재 문서의 자동 prepared-program 환경이 non-null임을
확인한 guard에서 중단했다. 이전 base package의 구 API를 같은 실제 글꼴 선택으로 추정하지
않았다. 이 네 번째 비교는 미검증이고 원 failed state/log를 보존한다. 이미 종료·hash 검증된
base Native/current Native/current WASM 세 실행만 qualification-result에 유효 범위로 남겼다.
새 프로세스를 중복 시작하거나 guard를 우회하지 않았다. #8064 Native-only 자동 래칫과
기존 Native/fresh WASM 시각 수용의 범위를 구분하며 base WASM 수용을 주장하지 않는다.

원본 TSV6개(canonical3/audit3), Native64쪽 비교 TSV와3쪽 대표 review/overlay6장은
`mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/sample16-2022-bcp-ownership/`에
보존했다. C431·C432의 집중 통과는 전체 nextest 재실행이 아니다. Native63/64 하락·다른
실패·전쪽/실제 소비 경로 검증이 남아 있으며, PR은 준비 결과 보고 후 새 승인을 받아 진행한다.


## C433 복합 테두리의 분해 출처와 Native 재스냅 회귀 복구

63/64쪽의 Native 하락 원인은 글꼴 선택이나 HFT transform이 아니었다. 실제 Native font trace,
transform0건의 동등 출력 대조, base/현재26개 선 op의 byte동일을 확인했다. 독립 Print의 아래
이중선은192DPI에서 각2행에 coverage가 있지만 현재 Native는 각1행으로 이동했다.
`double_line_parts`가 정한600DPI 부모 기하를 `create_parallel_lines*`에서 Single로 분해한 뒤
`raster_line_endpoints`가 다시 일반 실선으로 판단해 각 획을 픽셀 중심에 옮긴 것이 원인이다.
기존 helper의 복합선 제외 조건은 부모 line type이 소실되어 작동하지 않았다.

일반 이중/삼중선 분해 생산자에 `compound_component` 출처를 남겨 Native 소비에서 다시
스냅하지 않는다. JSON의 compoundComponent→Studio 타입→CanvasKit 실제 stroke 소비에도
같은 결과를 전달한다. 페이지 번호·문구·폭·색상으로 대상을 좁히지 않았으며 기존 일반 실선의
선명도 계약은 유지했다. 측정 좌표/내용 소속을 바꾸거나 글꼴·비교 기준을 바꾸지 않았다.

Native frozen59/fresh60을 source2198파일 ledger로 고정한 뒤 sample16 전64쪽 양쪽,
생물독205쪽 양쪽 before/after를 순차 캡처했다(새 실행948쪽/capture8). canonical와 별도
pdftoppm audit의12개 완료 manifest·PNG1896쌍 SHA/쪽수/누락·중복을 확인했다.
sample16 Native 전쪽 최저94.14342→94.59411%, fresh WASM94.60213% 유지, 미달0이다.
63쪽97.65514→99.80596%,64쪽98.22800→99.66659%이며 fixed4a07 대비 각각
-0.00358/-0.00834%p로0.1%p 안이다. 생물독 전205쪽은 Native98.06390%,WASM98.12078%
그대로이고143쪽 양쪽100%다. 전체538전후쌍에서0.1%p 하락0,PNG동일468쌍이다.
기존 미달 대조군 pic-in-head02 4쪽57.60580%·sample11 Native50쪽89.13433%는 이번
수정 전후 그대로이며, 전쪽 수용이나 upstream 기존 미달로 추정하지 않는다.

대표 review/overlay와 아래 이중선 확대를 직접 판독했다.63/64쪽 양쪽98이상 확인 후 독립
Print 띠 중심±2CSSpx·두 분리 획의 인접 픽셀 coverage를 실제 Native PNG로 보호했다.
같은 쪽의 원본 문단·내용 순서/누락/중복도 함께 검사하며 기존BCP/표 검사를 유지한다.
prototype은2PASS1FAIL→3PASS(수정 전 띠가 한 행)이다. 정식 NativeSkia는 새 sample16세 건,
기존 회색 실선143쪽, 본문/종이 테두리 관계 두 건 **6PASS/0FAIL**,선택720skip,
test2.072초·compile2분19초·전체147.375초,exit0. prepare 후 실제016/017/002 배정을 다시
계산했다. CanvasKit 단위4PASS와 fmt도 통과했다. 실행할 필요 없는 tsx loader를 지정한 최초
unit 명령은 실패 원본을 보존하고, 현재 Node24 정식 node --test로4PASS를 확인했다.

Studio7736의 root pkg/public/초기 HTTP200 JS/WASM byte를 frozen60과 대조했다.
정상 한 쪽 이중선 입력에서 실제 Canvas2D/CanvasKit Print 모두8개 출처를 받았고 software
CanvasKit readiness PASS·fallback/미지원op/숨은 Canvas2D overlay0을 확인했다. 최초 시도의
inspector cache 소진은 버퍼 명시 후 새 실행으로 재검증했으며 이전 로그를 보존했다.
이는 실제 제품 소비 한 조건이며 actual36 전체가 아니다. Mac WASM은 locked wrapper의
web/pkg/no-opt 로컬 대체 빌드다. 모든 원 로그는 ignored repair58~65에 보존했다.
최종 TSV17개·대표 review/overlay12개와 상세 출처는
`mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/compound-stroke-coverage/`다.
전체 nextest 재실행·남은 기존 실패·HFT 수식·전쪽/actual36 검증은 미완료이며 PR은 새 승인 후 진행한다.

C433 추가 정적 검증: NativeSkia lib Clippy `-D warnings`는38.81초 exit0, Studio TypeScript
`--noEmit`도 exit0이다. 필수 전체 all-target/다른 feature gate 완료를 대신하지 않는다.


## C434 명시 기호 공급자의 자연 전진 전달과65쪽 중요 원점 보호

2026-10-19~20 KST. #4962의 기존 양성 집계 중 휴먼명조U+2024만 exact-hit1 대신
char-miss1이었다. 원 해결 #4962와 정합 보정0f2ed852, 명시 계약64c886의 source를
추적했다. 선언 벡터만 만들고 언어별 값을 넣지 않아 glyphFallbacks가 paint에는 보이지만
측정의 명시 조건에 닿지 않았다. 기존 기대값과 집계 분류를 유지하고 각 슬롯의 같은
비내장·명시 환경 결정에서 값을 만들었다. 문서ID·문구·좌표 분기는 추가하지 않았다.

`resolve_single_char_style`→`glyph_fallback_advance_explicit_for_lang`→text_measurement의
TextStyle→확인된cmap 결측·선언 공급자의 자연 전진→실제 후속 원점까지 같은 결과를
전달한다. 기본 공급 체인과 HFT 폭 보호는 유지했다. 같은 기존 집계 검사는 frozen59에서
1FAIL→frozen68에서1PASS다. runtime2198파일을 고정해 원본과 대응 Print의 PII66 선언/
기본 조건을 Native/fresh WASM 전쪽 before/after528쪽으로 대조했다.

| 선언 환경의 완료 쪽 | Native before→after | fresh WASM before→after |
| --- | ---: | ---: |
| 17 | 98.35193→99.95382% | 98.41237→99.95563% |
| 40 | 97.72414→99.95671% | 97.81731→99.95802% |
| 65 | 98.69031→99.97348% | 98.75526→99.97486% |

전후264쌍 중6쌍 개선·258쌍 PNG byte동일·0.1%p 하락0이다. 중간에 같다고 보고한 것은
최저값과 선택쪽의 부분 관측이었으며 전체 비교로 정정했다. canonical PDFium과 별도
pdftoppm audit를 혼합하지 않았다. 동일 핵심 동작의 후보 중 양쪽 raw 최솟값이 가장 높은
65쪽의 review/overlay를 직접 판독하고 정식 회귀를 추가했다. source/PDF SHA와192DPI는
asset README에 연결했다. 원문 문단·한점 리더의 소속, 순서·누락·중복과 실제 전진 끝에서
후속 글자를 이어 그리는 관계를 보호한다. 독립 PDF의 기호240.60pt·후속243.24pt·기준선
69.48pt를 CSS320.80/324.32/92.64±2px로 검사한다. 새 쪽수 검사는 추가하지 않았다.

새 검사를 먼저 ignored prototype으로 보존하고 완료 페이지를 판독한 뒤 활성화했다.
prototype은 기존 대시1PASS+후속 원점1FAIL→2PASS다. 정식 NativeSkia suite013도2PASS,
247선택skip, test0.188초·전체10.214초다. 관련 기존4묶음은53PASS/959선택skip,
test8.820초·전체293.693초였다. libClippy-D warnings66.467초·fmt도 통과했다.
새 검사 수를 반영해 prepare와 실제 배정을 재계산했다. runtime은 같으며 전체 nextest
통과를 대신하지 않는다. 기존 검사는 제거하거나 완화하지 않았다.

PII 전66쪽 최저 Native35.07344/W35.59924%는 이번 수정 전후 그대로다. 선언3/36쪽,
기본3/4/20/21/36쪽 미달은 남아 있다. 이 다른 표의 미달을 완료된 기호 검사로 면제하지
않는다. fixed4a07 Native도 동일 조건 전66쪽×2환경을 산출했다. 최저35.58967/35.55311%,
미달42/47쪽이며, 현재가 고정base보다0.1%p 넘게 하락한 쪽/조건도5개다. 전쪽 비교는
asset TSV에 보존했다. 원본을 이관하거나 검사를 제거하지 않고 별도 하락도 보정한다.
3쪽의 독립 Print77.515→95.735CSSpx 빈 띠와 앞쪽 row9의 물리 소비를 추적 중이며,
선언 행 높이에서 실제 소비를 빼는 것만으로 해당 띠와 일치한다고 단정하지 않는다.

Native68/WASM69 source ledger·root pkg/public·실제 Studio7736 초기 HTTP 응답 hash를
검증했다. 첫 추가 응답 body 읽기 실패는 로그를 보존하고 durable buffer를 쓴 새 실행으로
재검증했다. Mac 빌드는 locked web/pkg/no-opt 로컬 대체다. CK/actual36 수용을 주장하거나
PII guard를 우회하지 않았다. canonical/audit20개 완료 manifest와 PNG1320쌍의 hash·전쪽/
누락·중복을 확인했다. 원본 TSV22개와65쪽 review/overlay6개는 assets의
`rebase-4a07d3ae-20261019/explicit-onepoint-advance/`에 있다.

SVG #6520은 별도로 확인 중이다. 실제 한컴돋움의 보정 전체 TTF는 cmap·요청9문자와
Chrome 로드가 정상이다. 공개 Noto의 기존 PDF용 subsetter 출력31,300bytes는 cmap/OS2가
없고 Chrome이 실제 로드를 거부했다. MIME 기대값만 TTF 허용으로 완화하지 않았다.
SVG 코드·검사·의존성 변경은 아직 없고 원 증거는 ignored repair67에 있다. 전체 미완료
상태를 유지하며 PR은 최종 보고 뒤 새 승인을 받아 진행한다.


## C435 SVG 부분 글꼴의 웹 Unicode 계약과 실제 소비 복구

2026-10-20 KST. #6520 생산 SVG 경로의 원 구현9ee885와 cherry-pick fa21c26,
#6668 검토 기록을 대조했다. 기존 검사는 결정성/MIME만 확인했다. 실제 시스템 한컴돋움은
보정 전체 TTF로 남아 MIME 검사에서 실패했고, public Noto의 PDF용 부분집합은 cmap/OS2가
없어 Chrome에서 거부됐다. MIME 기대값을 완화하지 않고 웹용 Unicode 부분집합으로 고쳤다.

`font_style_requests`의 실제 face/문자→선택 face의 기존 정규화→skera의 Unicode/GSUB
closure·GPOS·이름·힌팅 보존→data-URI→실제 FontFace/SVG raster로 연결했다.
미지원 CFF 단일 face·legacy kern·부분집합 금지 표시에서는 선택한 전체 프로그램을 공급한다.
세 대조군의 전체 byte동일과 실제 Chrome 로드를 최종96에서 확인했다. 실제 Bold와 문서
내장 프로그램은 기존 공급을 유지하며 공유 측정·줄 배치에 문서별 예외를 넣지 않았다.

최종 검토 중 독립 glyph가 없는 VS를 필터링해 cmap14 closure에서 누락하는 경계도
재현·보정했다. 원 요청 Unicode를 모두 전달한다. 합성 A+VS→B의 윤곽/hmtx와 Chrome 폭
26.2799988px/PNG가 원본과 동일해졌고 보정 전에는24.3199921px의 A로 돌아갔다.
glyph ID 재배정 뒤 이름 차이를 오류로 판정했던 최초 진단은 실제 윤곽 hash와 폭으로 정정했다.
이는 글꼴 자원 계약이며 독립 Print의 VS 문서 조판 완료라고 주장하지 않는다.

기존 검사 입력은 para-001의 문단만 바꾸면서 관련 없는 저장 줄 폭을 남겼다. 공개 blank/
insert/font 등록/서식/save/reopen으로 정상 입력을 만들고 같은 저장본을 독립 한컴2020
Print와 Native/fresh WASM으로 먼저 확인한 뒤 보강했다. 첫 수동 모델 대조의70.99664/
63.74932%와 새 입력의 결과는 서로 다른 입력이다. 실제 첫 PDF face는 바탕이었고 새 PDF는
돋움이다. raw 스트림 잔재만으로 그 저장 차이 원인을 확정한 것은 아니며 첫 자료도 보존했다.

새 입력은 전1쪽이다. 최종96/97의 직접 PNG 양쪽100.00000%, 정식 검사와 같은 public-only
SVG 양쪽97.85523%다. 별도 pdftoppm audit는 각각100.00000/98.62920%로 분리했다.
public SVG의98% 수용이나 완전 픽셀 일치를 주장하지 않고 새 절대 px 핀은 넣지 않았다.
검사에는 기존8회 결정성을 유지하고 cmap/OS2·em/수직 메트릭·9자 glyph/전진폭/외곽 상자와
내용 순서·누락·중복을 추가했다. 이전68에서 cmap 누락1FAIL→최종96에서1PASS이며
prepare 후 실제 suite015의25건 PASS/243선택 제외,test1.891초/전체135.495초,exit0이다.
NativeSkia lib Clippy30.787초도 통과했다. 원 para-001을 쓰는 다른 정상 경로 검사는 유지했다.

Native96 build122.614초/exit0, source2199파일을 고정했다. Mac wrapper97은 Native 전용
helper만 달라 byte동일 WASM82 target을 재사용했음을 기록했다. root pkg/public/실제7736
초기 HTTP 응답 hash를 다시 확인했다. Docker 최적화 통과가 아니다. 실제 public Subset은
전체2,817,876→4,128bytes이며 Chrome의 Full/Subset SVG PNG byte동일이다.
최종 전쪽 TSV를 재산출하고8개 완료 manifest·PNG8쌍을 감사했다. review/overlay8장은
직접 판독한90/92 패널과 byte동일도 대조했다. 최종 TSV8개·PNG8개·출처/입력/PDF hash는
assets의 `rebase-4a07d3ae-20261019/svg-unicode-subset/`에 보존했다.

잘못된 compat2020·PDFium 없는 기본 Python·standalone extern 누락·기본 Vite7700 접속은
증거를 보존하고 명시 환경으로 정정했다. 최종 초기 응답은 기존7736을 대상으로 재실행했다.
로그/JSON/중간 글꼴·SVG는 ignored output만 사용했다. 최종 증거는 repair96~105이며
전체 회귀의 다른 실패·PII 물리 꼬리·수식·전체 문서/actual36는 아직 남아 있다.
지침·GitHub·remote 변경은 하지 않았다. PR 준비 완료로 처리하지 않고 사용자 승인을 유지한다.

## C436 완료 내용 컷과 일반 칸의 독립 물리 최소를 구분

2026-10-20 KST. #5846 원 이슈·PR #5904·실제 통합 #5912/b9eb551과 후속
363ef22d2c7의 #7642 계약을 먼저 대조했다. 원 계약의59→60쪽 중첩 내용 소유와
이번2→3쪽 일반 행의 물리 잔여는 다르다. 완료 걸침 셀의 내용 유래 높이를 다시
열지 않는 기존 억제는 유지하고, 원본 일반 칸이 따로 소유한 저장 최소만 전달했다.

PII 원본 section0/para13 row9의3758HU 최소는50.106667CSSpx다. 스캔은 첫 조각에
31.826667px를 수용했지만 paint가26.2px로 줄였고, 다음 조각은 완료 컷을 보고
row9를 건너뛰었다. 실제 다음 쪽의 독립 Print 괘선은77.514667→95.734667이다.
paint26.2px에서 잔여를 역산하지 않고, 스캔이 수용한31.826667px를 한 번 차감한다.

`stored_independent_row_minimum`은 원본의 정상 저장 일반 칸·RowBreak에만 적용하고
dirty/편집/재래핑은 제외한다. RSPAN_BAND가 수용한 행·최소를 refit에도 전달하고,
emit은 같은 최종 행·완료 컷과 최종 수용 상자로 잔여18.28px를 만든다. 그 결과를
앞쪽 `end_row_owns_stored_frame`과 뒤쪽 `next_start_row_height_override`가 함께
소비한다. rowspan 공간 원장과 저장 reset/frame은 별도로 유지한다. 기존 first-frame
일반 칸 경로는 걸침 셀이 있는 행을 배제하므로 이번 경로와 경쟁하지 않는다.

Native108/fresh WASM111을 source2199파일과 artifact SHA로 고정했다. Mac locked
web/pkg/no-opt WASM은 실제 재컴파일210.334초/exit0이며 root pkg/Studio public과
실제7736 초기 HTTP 응답을 대조했다. Docker 최적화 통과가 아니다. 기존 네 글꼴
공급 경로의 hash도 기록했고 원본·PDF·공유 target은 변경하지 않았다.

전66쪽을 선언/기본 환경의 Native/fresh WASM에서 모두 새로 출력해264쪽 TSV를
산출했다. canonical PDFium과 별도 pdftoppm audit를 구분한다.2쪽은 Native
97.57528→99.69840%, WASM97.17350→99.70746%,3쪽은35.07344→99.53257%,
35.59924→99.54130%다. 네 조건 모두 prior C434와 PNG byte가 다른 쪽은2·3뿐이며
나머지64쪽은 동일하다. 새2·3쪽 양쪽 review/overlay와 원 중첩 범위59쪽 Native/
60쪽 WASM을 직접 판독했다. 작은 기존 자간 차이를 새 golden으로 고정하지 않았다.

기존 #5846 관계 검사는 변경 없이 before96 1FAIL→after108 1PASS다. 양쪽 raw99%대와
확대 판독 후 기존 함수에 독립 Print 벡터 선의96dpi CSS 좌표3개를 ±2px로 추가했다.
강화한 최종 함수도 before96에서 실제 행 하단1038.1 vs 독립1042.698667로1FAIL,
after108에서1PASS다. 소유·순서·누락·중복·본문 포함·후속 내용 검사를 유지했다.
관련 기존7case 정식21PASS/1675선택skip(test3.978초/전체339.080초), 강화 뒤
재계산한 suite026에서1PASS/227선택skip(test3.313초/전체56.919초)다.
Native Skia lib Clippy94.678초/exit0, 최종fmt와 고정base manifest 검사도 통과했다.
초기 nextest 중복 동시성 옵션 및 첫 px tuple fmt 오류는 보존·정정 후 실제 재실행했다.

전체66쪽 최저는 선언 양쪽85.44699%(36쪽), 기본 Native81.66959%/WASM81.93226%
(4쪽)다. 선언36쪽, 기본4·20·21·36쪽은 여전히 미달이다. Native 고정4a07 대비
남은0.1%p 이상 하락도 선언36쪽3.92801%p, 기본22쪽0.23111%p·36쪽3.92801%p다.
이번 수용 개선으로 이 미달을 상쇄하지 않는다.36쪽은 본문 없이 자동 번호만 있고,
실제 Native snapshot의 HCR Dotum 선택·독립 PDF의 HCRDotum과 번호 잉크의 약1.5px
세로 차이를 확인했다. 같은 family를 glyph/file 동일성으로 단정하지 않고 계속 추적한다.

정본 ignored 실행은 repair106~123, 최종 제출 증적은
`mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/pii-independent-row-minimum/`이다.
TSV12개·대표 review/overlay12개와 Native 비교 TSV를 보존했다. 생성 SVG만 완료 뒤
hash manifest를 남겨 정리했고 삭제 SVG를 재사용 증거로 쓰지 않는다. 전체 회귀의 다른
실패·최종 전 문서 검증·actual36는 남아 있다. PR 준비 완료가 아니며 PR은 새 승인을 받는다.


## C437 자동 쪽번호의 원본 TTF 수직 프레임과 일반 행의 물리 예약

- 부모 head는 `799daa2d4c84780c62a3a4e140dee988f0a7214e`다. 아래 증거를 생산할 당시 후보는
  미커밋이므로 runtime source/patch와 producer hash를 함께 고정했다. PR 제출 완료가 아니다.
- C436 전후 PII36의 좌표·스타일은 같지만, 고정 base의 잘못된 Malgun Gothic 표시가 HCR Dotum으로
  복구되면서 원래 `font_size/3` 수직 추정의 차이가 드러났다. 두 frozen library의 실제 glyph 선택을
  `repair121`·`repair126`, 같은 좌표·스타일을 `repair125`에서 각각 확인했다.
- 설치한 원본 `HANDotum.ttf`의 HCR Dotum Regular는 UPEM1000, Win1070/230,
  hhea1070/-230, Typo830/-170이다. SHA는
  `1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898`다.
  파일 바이너리는 제출하지 않는다. PDF descriptor의 Typo descent170과 Win 줄 프레임230은 구분한다.
- `font_metrics_vertical.rs`는 기존 exact style/alias 선택의 원본 메트릭에 확인된 Win 프레임만 연결한다.
  공통 `footer_page_number_y`가 최종 TextRun 기준선을 만들고 각 paint가 이를 소비한다.
  HFT source Type3의0.2em과 확인하지 않은 face/style의 기존 경로는 유지한다.
  문서 ID·쪽 번호·용지 치수로 나누지 않는다.
- 독립 PII PDF의1~3쪽은 `cm.y=-.119869`,4~66쪽은 `-.12`이나 원시 번호 Tm6761은 같다.
  원본7336도 같은 원시 Tm6642에 두 matrix가 사용된다. fine matrix에 맞추려고 문서 엔진에
  페이지별 PDF driver 이동을 복제하지 않는다. 독립 source 물리 좌표와 Print matrix를 나눠 기록했다.
- frozen Native130 빌드는114.587초/exit0이다. 수정 전108과 후보130의 PII1/36쪽 실제 PNG 비교:
  1쪽99.98158→99.98159%,36쪽85.44699→100.00000%(raw2px).
  별도 pdftoppm36쪽도100%다. review·overlay를 직접 읽어 위치 개선과 남은 래스터 프린지를 확인했다.
  이100%는 허용2px 실루엣이며 byte 또는 무관용 픽셀 동일을 뜻하지 않는다.
- fresh WASM134는 locked web/pkg/no-opt 로컬 대체 빌드221.120초/exit0, 실제 root 재컴파일이다.
  JS `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`,
  WASM `16f54ae3405a51aab03599e8fe6b2c35a8f894fa68d8e3689661356278bb59e5`.
  root pkg/Studio public 및 새 browser 초기화 응답138을 대조했다. 초기화는 문서 paint/actual36의 증거가 아니다.
- 기존1133 자동 번호7검사는 그대로 실행해2PASS/5FAIL이다(`repair133`). 기존0.6px raw PDF 좌표와
  source 물리 좌표의 약1.2px 차이를 확인했지만, 검사 기대값·허용치를 변경하지 않았다.
  해당 출력의90% 선행 조건과 독립 좌표계 근거 없이 갱신하지 않는다.
- 대조군6문서의 이전Native/fresh WASM48쪽을 `repair129`에 새 산출했다.
  footer15/bottom15의3쪽 본문은 이전96 및 고정4a07에서도42%대이며 현재108 PNG는96과 전쪽 byte동일이다.
  원본1133 HWP/HWPX는 양쪽95%대다. 기존 미달을 현재 회귀나 수용 완료로 바꾸어 쓰지 않는다.
- 위 여백 대조군의 이어받기 문제는 `repair136`에서 실제 경로와 독립 PDF vector를 대조 중이다.
  row4의 source cellSz5222HU=69.626667px인데 현재 앞뒤 실제 상자는27.1+22.4=49.5px다.
  Print의 앞39.477333+뒤30.206667≈69.684px와 달리 물리 최소 공간을 잃고 있다.
  반복된 안내문으로 row3을 잘못 고른 첫 진단은 INVALID로 별도 보존했으며 row4로 정정했다.
  `stored_ordinary_row_end_frame`의 저장 reset 인증과 실제 가용 높이 예약,
  `stored_ordinary_row_remainder`의 같은 수용 상자 차감을 대조한다. paint에서만 상자를 늘리지 않는다.
- frozen130/134의264쪽 PII Native/fresh WASM 선언/기본 전쪽(`repair137`)과
  fresh WASM 정상/여백 대조군24쪽(`repair139`)은 driver exit0으로 완료했다.
  선언 환경은Native 최저94.58132%,WASM94.60117%, 각각미달0/66쪽이다.
  기본 환경은Native81.70425%,WASM81.96642%, 각각미달4/20/21쪽이다.
  36쪽은양쪽100%이며 PNG byte는 서로 다르다. 이전108/111 대비0.1%p 이상 하락과98% 이탈은
  이 PII132쪽/대조군48쪽 비교에서0이다. 고정4a07 Native 선언66쪽도하락0이나,
  기본22쪽은91.56665→91.36857%(0.19808%p하락)가 남는다. fresh WASM의고정base전쪽 비교는 이 결과가 아니다.
  fmt와Native Skia libClippy-Dwarnings는각각exit0(10.313/236.085초)이다.
  결과·TSV·직접 판독·좌표/폰트 감사는 ignored `repair142-page-number-comparison`에 연결했다.
- `repair137`의마지막source guard·exit0·실제종료를확인한뒤에만분할행후보141을적용했다.
  CGMP원구현5d22772bf의저장reset계약은유지하며,실제앞/뒤내용이있는용량컷에서
  독립셀최소도같은물리프레임근거로쓴다. 이미내용을마친첫프레임에가짜빈꼬리를붙이지않는다.
  같은 조회가 scan의 예약·emit의 잔여·배치의 정렬을 소비한다. 첫 후보143/144에서는
  실제 호출이 반복 머리행 경로에 한정되어 여백 대조군이 개선되지 않았다. 아래 최종 보정과 구분한다.
  130/134의 위 결과를 이후 후보의 전쪽 통과로 재인용하지 않는다.

### 실제 수용 예약과 독립 첫 프레임의 우선순위

내용 컷의 보이는 높이로 앞서 수용한 공간을 역산하지 않는다. 일반 행 스캔이 실제 추가한
`partial_row_reservation`을 refit에도 전달하고, 일반 행의 실제 부분 컷에서 이를 독립 원본의
물리 상자로 교체한다. 같은 `consumed`와 `end_row_height_override`를 emit이 소비하고
원본 최소에서 같은 수용 상자를 차감해 다음 조각에 전달한다. 문서·쪽별 분기는 추가하지 않았다.
완료한 원본 행에는 새 빈 꼬리를 만들지 않고 저장 reset·rowspan 경로도 유지한다.

Native147에서 여백 대조군3쪽은42%대→100%로 복구됐으나, 기존 #6860 엄격 검사는
첫 조각835.413333px 대신834.106667px를 수용하는 실제 회귀를 검출했다. 이미 인증된
`saved_first_fragment_source_frame` 안에서 일반 용량 컷의100HU inset을 중복 차감한 원인이다.
최종159에서는 원본 첫 프레임을 우선하고 해당 경로의 기대값을 변경하지 않았다. 원본 프레임이
없는 여백 대조군은 일반 물리 예산을 사용한다. 최종 기존 #6035/#6950/#6860/#5585의
9건과 진단 관계3건이 모두 통과했다(`repair161`). 중간 standalone serde crate 불일치의
컴파일 실패는 결함 재현으로 세지 않고 맞는 dependency로 재실행한158에 분리했다.

최종 frozen Native159는 빌드128.846618초/exit0이며 CLI SHA는
`c32526be054fb17ec16a3f373372a6162564a80723e856dea326c51b9ac03736`,
library SHA는 `354dcf2f4f00aca13cb57cfdfe19161c8024c113de65826275ecf276284f252d`다.
부모799daa2d와 runtime diff SHA
`a4dcb9c0890f634afb22e9d8397ec104ed508a4aa21670ac81701b7d57725029` 및
새 모듈을 포함한2200 source hash를 고정했다. 테스트 추가 뒤에도 runtime 파일은 같은 hash다.
fresh WASM160은 실제 root 재컴파일219.687032초/exit0이며 locked web/pkg/no-opt Mac
대체 빌드다. JS SHA는 앞과 같고 WASM SHA는
`4dd8ae47c19f5377dea85be474d8a3dcfc24a010312bf7c3421562443c632627`다.
root pkg/Studio public과 새7736 browser 초기 HTTP 응답164를 대조했다. WASM 응답은
byte동일, JS는 Vite의 정확한 asset URL rewrite·source map 원문을 대조했다. 실제 초기화만의
증거이며 문서 paint·CanvasKit 준비·actual36 완료로 대신하지 않는다.

### 최종159/160의 회귀 추가 선행 검증과 정식 검사

canonical PDFium/192DPI/compat2022/2px raw 기준으로 네 입력의 전3쪽을 양쪽에서 다시
산출했다(`repair162`, 네 문서24쪽). 기존 원본·PDF를 그대로 사용했고 새 fixture는 만들지 않았다.
두 여백 입력의 생성 속성·원본/PDF SHA는 기존
[`samples/issue1133/MANIFEST.json`](../../samples/issue1133/MANIFEST.json)과 다시 대조했다.
원본 HWP의 기준은 `pdf/issue7382-regression-review/issue_1133-2020.pdf`,
원본 HWPX의 기준은 `pdf/hwpx/issue_1133.pdf`다.

| 입력 | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 독립 PDF/양쪽 쪽수 |
| --- | ---: | ---: | --- |
| page-number-footer15.hwpx | 95.17397% | 95.18760% | 3/3 |
| page-number-bottom15.hwpx | 95.17397% | 95.18760% | 3/3 |
| issue_1133.hwp | 95.17397% | 95.18760% | 3/3 |
| hwpx/issue_1133.hwpx | 95.17148% | 95.18502% | 3/3 |

두 여백 대조군의2쪽 raw는 Native98.54204%/WASM98.55721%, 이어받기3쪽은 양쪽
100.00000%다. 같은 경계 중100%인3쪽을 우선 골랐다. 원본·여백 대조군의 최종 review와
overlay를 직접 판독해 표의 공간·문단 소유·뒤 행과 잉크 프린지를 구분했다. 2px 관용100%를
엄격 RGB 또는 PNG byte동일로 표현하지 않는다.

새 정식 `tests/cases/issue_1133_partial_row_frame.rs`는 실제 공개 render tree에서 다음을 검사한다.
두 열의 같은 물리 상자, 행·앞뒤 문단의 쪽 소유, 표제·본문의 누락·중복, 원본 셀 최소의 보존,
다음 행과의 연속성, 완료한 원본 HWP/HWPX에 빈 꼬리가 생기지 않음이다. 독립 Print 벡터의
3쪽 위 괘선77.356CSSpx와 다음 행 시작107.562666667CSSpx를 ±2px로 보호한다.
이는 구현 결과를 기대값으로 복사한 것이 아니며 일반 내용 관계 검사도 함께 유지한다.
최종 같은 검사 원문으로 frozen143에서1PASS/2FAIL(exit101)→159에서3PASS(exit0)다.
그 뒤 manifest `--prepare`와 실제 재파생 target으로 정식 Native suite014를 실행해
3PASS/259선택 제외, test0.048초/전체136.355548초, exit0을 확인했다(`repair167`).
같은 실제 suite014 NativeSkia Clippy-Dwarnings도90초/exit0이며, 추가 후 fmt와 고정4a07
manifest 검사는 통과했다(1642 sources/7163 static attrs/48 integration targets).
기존 자동 번호7검사의 기대값·허용치와 다른 실패 검사는 변경하지 않았다.

최종 전체 TSV8개와 직접 표시할 대표 review/overlay12개를
[`ordinary-partial-row-frame`](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/ordinary-partial-row-frame/)에 보존한다.
포함한 원본/PDF8개는 모두 기존 tracked 자료다. 입력/PDF/source/font/도구 hash와 전체 범위,
좌표 추출 원문·정식 실행 로그는 ignored `repair136`·`repair159`~`repair168`에 남겼다.
JSON·로그·중간 SVG·글꼴 프로그램은 커밋하지 않는다.

### 현재 미해결과 완료 범위

최종159/160의 집중 검증162는22조건68쪽 실행을 마쳤다. 위 네 문서 외에는7336/biz의
전6쪽과 기존 번호 검사의 영향 페이지 중심이며 전 문서 검증이 아니다. 도로조명 #6860의
1쪽은 Native87.50262%/WASM88.99403%로 미달이다. 현재 Native 두 쪽은 보정 전143과
PNG byte동일이며 고정4a07의74.58351%/69.70042%보다 개선됐어도 수용으로 바꾸지 않는다.
독립 첫 프레임의 기존 엄격 검사는 복구했지만 제목·캡션 등의 실제 차이는 계속 보정한다.
각주1937은 선택25쪽95.80137%/95.72257%여도 양쪽51쪽/독립50쪽으로 불일치한다.
같은 입력/PDF의 출처 hash를 기존 #7382 변환 증거와 확인했고 고정base도51쪽이므로
현재 수직 프레임 수정의 신규 회귀로 단정하지 않는다. 쪽수 기대값은 갱신하지 않고 원인을 고친다.
이50쪽 각주 입력을 별도의242쪽 각주 문서와 혼동하지 않는다.

최종159의 fmt/NativeSkia lib Clippy/WASM lib Clippy는 각exit0,
9.431608/42.933959/70.476223초다(`repair165`). 이는 all-target/workspace/전체 nextest나
전 문서 Native/fresh WASM·기본 PII 미달/고정base 대비22쪽 하락·actual36의 완료가 아니다.
130/134의 이전264쪽 sweep을 최종159/160의 전쪽 통과로 재사용하지 않는다.
미해결 범위와 필수 검증을 계속 처리하며, PR은 결과 보고 뒤 새 사용자 승인을 받아 진행한다.

## C438 시스템 HFT의 원본 전진폭과 기존 경계 보존

부모는 로컬 C437 `e2ebf76b8a43ce0fc8399e9129001bc7196a892d`다. #6860/#6880 원래
행 첫 프레임·내용 소유 계약과 이후 #6904 통합, 현재 같은 입력의 전2쪽 미달을 구분했다.
현재 Native159의 제목 실제 glyph observer는 원본 `시스템`/altType2를 표시 Dotum으로
치환하고, 원본 한글의 폭도 Dotum의1600HU로 측정한 뒤 −0.78185185px의 배분 간격으로
좁힌다. 독립 Print는 Type3의880/1000 자연 폭과 TJ−52.3의 굵게 간격을 쓴다. 따라서
같은 family가 설치됐다는 것만으로 원본 HFT 메트릭을 공급했다고 볼 수 없다.
trace는512/976개 기록으로 제한됐으며 제목 범위의 실제 선택만 주장한다(`repair169`).

대상 문서를 그대로 메트릭 표본으로 사용하지 않고, 원본의 확인된 HFT 시스템 face 참조를
유지한 별도 표본을 구성했다. 기존 본문·저장 LineSeg는 제거하고 일반/굵게12·16pt에 각각
한글11172자·ASCII95자를 넣었다. 일반 스타일은 기존 굵게 태그만 뺀 복제이며 ratio·원본
언어별 자간은 유지했다. 이 생성 진단은 렌더링 회귀 fixture나 원본 문서 수용 자료가 아니다.
입력 SHA는 `52a8b3d98bdb4dbfb4a2ea7c9fc3fc2a689ae815c40c7bf2d9c5e52b191d83ad`다.
별도 한컴2020 Print job `d8b04619-ef08-48d7-b0ec-522f0d61d62f`는34.080초에 성공했고
실제11.0.0.9136/Print0/driver one-up/preprocess none,39쪽/15,001,588bytes다.
PDF SHA는 `55390717754f3c64735c46ff9da8b49c73ff4c81b09f6300a15812e23a5e77dc`다.
표본·PDF·글리프 프로그램은 ignored `repair170`의 로컬 전용 자료로 남긴다.

ToUnicode·실제 text op의 Type3 Widths를 전수 대조한 결과 네 스타일의 한글11172자는
모두880이며 실제 출력된 ASCII94자의 폭도 전부 같다. Type0 폴백·미해석 문자·폭 충돌은0이다.
공백은 실제 glyph로 출력되지 않아 새 표에서는 미확인으로 남기며 기존 공백 계약을 쓴다.
최초 추출기는 bfrange를 처리하지 않아 중단됐고 로그를 보존했다. 실제 bfrange/배열을
해석한 최종 추출로 전수 결과를 확인했으며 실패 추출을 통과로 세지 않았다.

기존 `legacy_hft_metric_face`의 원본 HFT 경로에 `시스템 (HFT)` 메트릭을 연결했다.
측정은 원본 자연 폭, 공통 HFT 굵게 walk는 원점 추가 간격, glyph-fit walk는 간격을 뺀
자연 폭을 각각 소비한다. 명시 TrueType 환경·다른 TTF family는 이 HFT 선택에 들어오지
않으며 문서 ID·쪽 좌표 조건은 추가하지 않았다. 원본 형태·표/줄 구조나 글꼴 공급 경로를 바꾸지 않았다.

Native171 빌드는123.079257초/exit0이다. CLI SHA는
`eee80c5090e47211760f1030cebcec390c537eb4365aed2e217ac13d2dd1820a`, library SHA는
`b432ba549276b7bc4c6c7fcdfdfe56c80764603bedb45602956ff7a418c65139`다.
현재 runtime2201파일과 patch SHA
`c676b35ca10009f53076c30920142ab4081ce090612bb2d958607df2f62927b6`를 고정했다.
새 모듈은 별도 원문/hash도 남겼다. fresh WASM174는 실제 root 재컴파일204.541393초/exit0,
Mac locked web/pkg/no-opt 대체 빌드다. JS는 앞과 같은23f085e3…,
WASM SHA는 `5355e80c7d841cc08eda97bb195da021af3140bea05ffb1694c54c9efa4e9460`이다.
root pkg/Studio public 및 실제7736 browser 초기 응답178을 대조했다. Docker 최적화나
Studio 문서 paint/actual36 완료를 주장하지 않는다.

동일 원본·독립 Print·네 글꼴 경로·192DPI/compat2022/2px/PDFium의 전2쪽 결과는 다음과 같다.

| 쪽 | 수정 전 Native159 | 최종 Native171 | 수정 전 WASM160 | 최종 fresh WASM174 |
| --- | ---: | ---: | ---: | ---: |
| 1 | 87.50262% | 91.20154% | 88.99403% | 92.44329% |
| 2 | 92.31277% | 92.31277% | 92.86183% | 92.86183% |

양쪽 쪽수2/2, 미달0이며2쪽 PNG는 같은 backend의 수정 전과 byte동일이다. 전후1쪽과
최종 양쪽2쪽 review/overlay를 직접 판독했다. 제목·캡션의 원점 전진이 개선됐지만 대체
글리프 윤곽과 표의 일부 영문·기호 차이는 남는다. 원본 HFT glyph를 새 공급했거나100%라고
표현하지 않는다. 본문·괘선·마지막 행 첫 줄의 기존 소유는 유지했다.

고정4a07 Native는 기존 frozen166의 같은 두 쪽을 재사용했다. 고정 base WASM의 첫181
실행은 새 자동 환경 선택 API가 없어 exporter가 거부했다. guard나 도구를 바꾸지 않고,
현재174가 실제로 `noDeclaredSubstitutions/environment:null/programs:[]`인 것을 확인했다.
기존 exporter가 지원하는 명시적 `None`을 **base와 후보 양쪽**에 적용해 전2쪽을 다시
산출했다(`repair182`). 후보의 두 PNG는 기본174 실행과 byte동일이며 실제 None도 확인했다.
정본 비교·실제 run별 글꼴 공급 검사는 유지했다.181 실패를 시각 결과로 세지 않는다.
고정 base→후보는 Native1/2쪽74.58351/69.70042→91.20154/92.31277%, WASM은
85.43866/92.85946→92.44329/92.86183%다. backend별 Decimal 비교에서0.1%p 이상
하락0쪽이다. base 대비2쪽 Native 개선은 앞선 보정의 효과이며 이번 C438만의 개선으로 세지 않는다.

기존 주요52조건·incoming·번호 대조군을 합쳐 중복 제거한73입력의 글꼴 선언을 조사했다.
HFT 시스템에 해당하는 것은 현재 도로조명 재저장본뿐이다. 이는 선언의 보수적 영향 조사이며
모든 글꼴이 실제 사용됐다는 뜻이 아니다. 별도로 네 정상/여백1133 문서의 전3쪽을 양쪽에서
다시 출력했다(`repair179`,24쪽). 모든 PNG가 수정 전159/160과 byte동일하고 전쪽 raw90% 이상이다.
이전 TSV 재인용이 아니라 새 렌더·TSV이며, 같은 hash인 기존 대표 판독과 연결했다.

새 `issue_7445_system_hft_advance`는 원본 제목의0.88em 자연 폭/0.05em 굵게 전진,
캡션의 같은 프로그램 전진과 기존 반각 공백, TrueType 본문의1em 보존을 공개 Core API와
실제 run 상자로 확인한다. 제목의 내용 순서·누락·중복·본문 포함도 함께 검사한다.
캡션의 전진은 독립 Type3 한글880/하이픈406/꺾쇠434와 굵게 간격의 em 합으로 검증하며,
98% 미만인 화면의 절대 px 위치는 고정하지 않았다. 수정 전159에서1PASS/2FAIL,
현재171에서3PASS다. 처음 private helper/잘못된 필드명을 쓴 컴파일과 캡션 source 식별자가
없는 trace를 고른 진단은 INVALID로 보존하고 최종 같은 원문 재현에서 제외했다.
새 API를 노출하지 않고 캡션은 실제 식별된 run의 상대 전진으로 검사했다.

manifest prepare 후 실제 suite019에서 새3PASS/229선택 제외(test0.123초/전체151.171084초),
기존 #6860 엄격2PASS, C437 이어받기3PASS다. root 메트릭 조회 unit9PASS/3862선택 제외,
나머지 library의 해당 필터0실행은 통과 건수에 더하지 않았다. 같은 runtime의 fmt,
NativeSkia lib/WASM lib Clippy-Dwarnings, 고정4a07 manifest도 exit0이다.
별도 suite019 Clippy-Dwarnings도27.51초/exit0이다. 정식 명령·exit·source hash는 ignored
`repair177`에 보존한다. 이 범위를 전체 nextest/all-target/workspace/전 문서 sweep이나
실제 소비 경로 전체 완료로 확대하지 않는다.

최종 전후·base Native/WASM TSV, 정상 대조군 TSV와 대표12개 review/overlay 및 십진수
비교 TSV는 [`system-hft-program-advance`](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/system-hft-program-advance/)에 보존했다.
표본 PDF·글꼴/윤곽·JSON·로그·중간 산출물은 포함하지 않는다. 전체 회귀와 기존 번호 검사,
1937의51/50쪽 불일치·PII의 기본 환경 미달/22쪽 하락·최종 전 문서/actual 검증은 계속 남는다.
PR은 준비 결과 보고와 새 사용자 승인 뒤 진행한다.


## C439 개인정보 원본 글꼴 공급 범위 정정과 기존 종료 프레임 검사 재현

code head `70708c1b18d380fa3c1a6cf560c6c1e0097c0086`의 같은 frozen171/174로 검증했다.
137이 기존 case의 `includeOriginalHftSymbolOverlay:false`를 무시해, TrueType만 선언한
개인정보 입력에 HFT 부분 검토 프로그램을 primary family로 공급한 드라이버 오류를 정정했다.
엔진·원본·PDF·비교기·문턱·설치 글꼴을 바꾸지 않았다. 원본 IR183과 실제 부분 cmap99/ASCII94/한글0을 확인했다.
과거137 기본 환경4/20/21쪽 미달과22쪽 하락은 현재 올바른 공급 조건의 회귀 근거로 재인용하지 않는다.
HFT 선언이 있는7336은 네 경로를 유지하고 전6쪽 Native95.13463%/WASM94.96148%, 미달0·쪽수6/6을 확인했다.

정정 공급186은 실제 전264쪽 exit0이다. 선언/기본 각각 Native66/66 최저94.58132%,
fresh WASM66/66 최저94.60117%, 미달0이며 네 조건의 페이지별 문단·셀 소유 텍스트 순서 차이0이다.
같은 backend의 두 환경 PNG도 각각66쪽 byte동일하다. 현재32쪽에는 첫 문단 기준선·일부 굵은 글자와 얇은 획 프린지가 남는다.
[전쪽 TSV·A/B·producer·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/pii-declared-font-scope/README.md)에 연결했다.
고정4a07의 같은 세 경로 전쪽 비교188은 기록 당시 실행 중으로, devel 무회귀나 전체 PR 수용을 아직 주장하지 않는다.

기존 #5846 정식 suite026은1FAIL/231선택 제외(0.690초, exit100)이며 종료 프레임57쪽 관계가 실패했다.
source cell83771/common42265/바깥283+283/후속42072HU에서, 이전130은 앞42265·뒤41506HU와 기존1PASS,
현재171은 앞42266·뒤41505HU로1HU를 잘못 배분한다. C437의 일반 물리 예약이 기존1×1 저장 첫 프레임보다
먼저 override를 선택하는 경로를 추적했으며 검사·공차는 변경하지 않았다. 다음 보정에서 같은 예약·paint 결과를 복원한다.
기존 쪽번호7검사도 현재2PASS/5FAIL로 재현했고 기대값을 그대로 유지했다.1937의51/50쪽·전체 회귀·actual은 남는다.

무효 실행도 분리했다:186의 첫 Python은 PDFium 미설치로 비교 전 실패했고 기존 PDFium 환경에서 다시 실행했다.
189의 첫 standalone은 roxmltree 의존성 누락으로 컴파일만 실패했으며 정식 경로의 실제1FAIL로 재현했다.
이 단계는 시각 검증 공급 정정과 실제 회귀 검출 증거이며 소스 결함 해결·새 회귀 통과·PR 완료를 대신하지 않는다.


## C440 개인정보 두 프레임 예약 우선순위 보정 후보

현재 기준 HEAD는 `8f47464827c7f0ea9c36e76d7acef2f159568435`이며, 한국어 로컬 작업만 진행한다.
사용자가 PR은 승인받은 뒤 진행하도록 다시 지정했다. 원격 push·PR·issue/comment·merge는
준비 결과를 보고하고 새 승인을 받기 전에는 수행하지 않는다.

C439 당시 진행 중이던 고정 base 비교188은 이제 6조건396쪽을 모두 산출했다. 선언/기본 각각
Native는0.1%p 이상 하락0쪽이나 WASM은9/18/28/43/52쪽이 각각
0.15301/0.18407/0.11613/0.11248/0.10611%p 하락했다. 기본 환경의 명시적 선택으로 다시
출력한 후보66쪽은 자동 선택186의 같은 backend PNG와 전쪽 byte동일하다.
따라서 자동 API가 없는 base의 명시 선택 경로 비교이며 공급 guard를 우회하지 않았다.
전체90% 통과를 devel 무회귀로 표현하지 않고 해당5쪽을 계속 보정한다.

### 두 물리 프레임과 실제 소비 경로

기존 #5846 검사는 변경하지 않았다. 원본 common42265HU와 전체 cell83771HU 및 후속
원점42072HU의 종료 관계가 독립 기준이다. diagnostic196에서 C437 일반 물리 프레임
블록을 켜면1FAIL, 이 블록만 진단용으로 건너뛰면1PASS였다. 이때 emit의 기존
`original_single_cell_opening_frame`이544.826667px 내용 예약을 원본563.533333px 상자로
바꾼다는 실제 trace를 확인했다. 일반 블록이 먼저563.546667px를 수용하면 기존 원본
상자가 그보다1HU 작아 뒤늦은 원본 선택이 막혔다. 진단 bypass와 trace는 최종 후보에서
모두 제거했다. 실패한195의 줄간격 trim만으로 종료를 설명하려던 제안도 폐기했다.

`stored_single_cell_two_frame_opening`은 새 문서 예외가 아니라 emit이 이미 사용하던
원본 두 프레임 인증을 공용 조회로 추출한다. 저장 재시작 컷, 후속 빈 원점과 셀 전체 종료식,
원본/미편집/미재조판 조건을 함께 요구한다. scan의 첫1행·각주 없음·원본 geometry 경로가
일반 가용 공간 조회보다 이 결과를 먼저 받아, 실제 수용 `consumed`와
`partial_row_reservation` 및 `end_row_height_override`에 같은 높이를 전달한다.
emit도 같은 조회를 쓰며 뒤 조각 차감과 paint가 원본 첫 상자를 공유한다. 실제 내용보다
작거나 가용 공간보다 큰 원본 상자는 수용하지 않는다. footnote/geometry 대체/이어받기
일반 경로의 기존 물리 조회는 유지한다.

정식197 실행은 처음 각주 변수 참조 경로를 잘못 적어 E0425로 컴파일 실패했다.
`input.prepared.table_footnotes`로 고친 두 번째 동일 검사 실행은 컴파일123초,
실제1PASS/231선택 제외0.576초, 명령 전체125.580초/exit0이다. 기존 검사 SHA
`7b115496c2d0da696929eb91726d5b6a8520292102fc92ad054c7b7fa23fd05c`와 기대값·공차를 유지했다.
소스 patch/hash·첫 실패·최종 결과는 ignored `repair197-pii-two-frame-priority`에 분리했다.
관련 정상 대조군은 #1133 이어받기3PASS, #6860 원본 프레임2PASS, 수평선 소유1PASS다.
Native/fresh WASM 전쪽 시각 재검증과 실제 소비 검증은
아직 이 후보에서 완료하지 않았다. 집중 검사 통과를 전체 PR 완료로 확대하지 않는다.

### 공유 수평선의 별도 paint 순서 회귀

193의 같은 PNG 원값 조사에서9쪽 base 미일치56픽셀은 footer에 있었으나 후보174개는
본문의 짧은 흰 수평선 영역에 있었다. footer는 기준보다3px 위에서1px 위로 개선됐으므로
쪽번호 위치를 되돌려 이 하락을 숨기지 않는다. 원본 PDF의 숫자 graphics 연산194에는
검은0.48CSSpx 선(op1165), 같은 경계의 짧은 흰선(op1174), 뒤 검은선(op1183)의 순서가
있다. 후보는 논리적 흰선 소유만 남기고 먼저 그린 검은선의 paint를 잃는다. 현재 흰선
소유·셀 배열 역순 대조군을 보존하면서 실제 paint 순서를 공통 출력으로 전달하는 수정이
다음 과제다. 비교기·이진화·2px·90%·0.1%p 기준은 변경하지 않는다.
