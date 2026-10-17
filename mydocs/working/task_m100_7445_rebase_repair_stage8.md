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
