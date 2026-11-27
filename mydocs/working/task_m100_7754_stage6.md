# 9개 이슈 소스 36: 전쪽·유입 기능·정식 회귀 검증

## 고정한 소스와 완료 근거

기준은 `1a27faacff48ea7329e76ce8eca15aa9ca8aa877`, 제품 생산물은
`436f8629b7f2b9aee520962297b79cdf15c17743`에서 생성했다. 실제 회귀 등록과
기존 Native CI 연결은 `c48c2b895d31ba0fd4969c837bfc7a6d8a2c65ac`에 고정했다.
두 후보 사이의 제품 Rust·crate·빌드 입력은 같고, 테스트·근거·생성 타입의 순서만 바뀌었다.
`task_m100_7754_stage5.md`와 `mydocs/pr/assets/issues7754_7765/source36/`을 잇는다.

평문 10개 원본 583쪽의 후보 Native 측정은 전부 끝났고 모든 쪽이 98% 이상이다.
안내문 24쪽은 Native 최저 99.50%, fresh WASM 최저 99.48%다. 두 경로의 같은
핵심 동작 중 최고 완료 쪽인 2쪽은 99.99%이며 독립 PDF의 옛한글·후속 글자 원점을
±2px로 보호한다. 여백·내용 순서의 실물 검사와 분할 글꼴의 합성 계약은 별도다.
여백 검사는 실제 수정 전 FAIL/후 PASS, 합성 계약 세 건은 후 PASS를 확보했다.
정식 Cargo suite의 실행은 아래 순서로 별도 확인한다.

## 남은 실행 순서

1. 같은 명령·폰트·입력·Print 조건의 새 기준 Native/fresh WASM을 완성한다.
   sparse worktree의 workspace member와 CLI가 include하는 문서 누락은 준비 오류로
   보존하고 보완했다. 제품 실패나 완료로 세지 않는다. 공유 `target/pr-review`는 보존한다.
2. 기준/후보 평문 583쪽씩을 두 경로에서 전쪽 비교하고, 실제 암호 원본도 별도로
   24쪽씩 검사한다. 암호는 stdin으로만 전달하며 명령·기록에 넣지 않는다.
3. #9302의 메모 저장/재열기·본문 붙여넣기·글꼴 재사용 및 공급자 변경·눈금자 전환을
   기준/후보의 실제 고정 WASM으로 확인한다. Studio·E2E 소스는 두 후보 사이에 같다.
   네트워크 응답 SHA는 기존 공용 artifact observer로 검사한다. 눈금자 검사는 실제
   '대체 글꼴로 보기' 선택 후 원래 assertion 그대로 실행한다. 기존 수용의 가로 스크롤
   25% 목표·실물 관성 미완료는 이 작업에서 완료로 바꾸지 않는다.
4. #9304의 원 CTRL_HEADER/구성 요소 CTRL_DATA 소유·바이트와 두 번 편집·저장·재열기를
   기존 실제 입력에서 기준/후보 Native·WASM으로 대조한다. #9292의 저장 원본과 실제
   중첩 표 편집·재열기도 새 생산물에서 보존한다. #9300의 양쪽 26건과 동일 소스 증거는
   이전 단계에 있으며 CI 보고서 검사를 제품 시각 증거로 대신하지 않는다.
5. 같은 별도 review worktree에서 기준의 해당 기존 계약을 실행한 후 후보로 전환해
   자동 suite를 준비한다. base-ref를 고정해 manifest·unit-tier·fmt를 확인한다.
   최소 글꼴 환경에서 Native unfiltered lib, 기존 Native 검사 묶음, 직접 PDF·자리표시자,
   관련 집중 회귀를 실제 실행하고 lint·WASM Clippy·workspace build·전체 nextest를
   순차 완료한다. 각 최종 exit code와 실제 테스트 수를 기록한다.
6. 최종 후보 편람 HWPX 383쪽과 기여자 공통 4개 문서 및 정본 보호 TSV 전체를 완료한다.
   같은 조건의 표시값 하락·원본 글꼴 미확보·환경 불일치는 통과로 바꾸지 않는다.

## 현재 보류 조건

### 중간 커밋에서 확정한 실행

평문 Native 583쪽의 전체 페이지 TSV와 입력·PDF·생산물 SHA를 정식 증거 경로로
고정했다. 측정 요청 범위에 더해 각 `run_manifest.json`의 실제 Native/PDF 전체
쪽수 일치도 확인했다. 시장구조조사 315쪽의 최저값은 98.40%다.

실제 암호 원본도 Native/fresh WASM 각각 24쪽을 완료했다. 최저값은 각각
99.50%/99.48%이며 같은 경로의 평문 PNG와 24/24 SHA가 같다. stdin 전달을 유지했다.
Studio 유입 여정은 기준·후보 각각 7개 통과, 실제 WASM CTRL_DATA는 각각
4개 입력을 두 번 저장·재열기한 8건에서 독립 물리 레코드의 소유와 payload가 같다.
초기 준비·probe 오류와 최종 실행은 구분해 보존했다.

근거는 `mydocs/pr/assets/issues7754_7765/source36/README.md`와 연결한 원본
TSV·manifest·구조화된 결과에 있다. 기준 Native CtrlData의 정식 Cargo 실행은 최소 글꼴 환경에서 5건 통과했다.
전체 Rust·WASM·보호 비교의 완료와 구분한다.

새 기준 Native도 평문 10개 원본 583쪽을 모두 완료했다. 실제 전체 쪽수는 각
독립 PDF와 같으며 후보와 페이지별로 비교한 하락은 0쪽이다. 시장구조조사는
최저 26.80%에서 98.40%로 개선됐다. 양쪽의 글꼴 공급·Native 대체 계획과
PDF 래스터·raw 조건이 원본별로 같음을 확인했다. 기준의 시각 gate 실패는
수정 전 증거로 보존하며 후보 실패나 전체 검증 완료로 오인하지 않는다.
제품·회귀·Studio 소스의 고정 SHA 동일성도 파일 트리 해시와 함께 기록했다.

유입 편집 기능의 저장 원본 Native도 기준·후보 각 33개 조건·50쪽을 완료했다.
후보 최저값은 98.07%이고 페이지별 하락은 0쪽이며 대응 PNG는 전부 바이트가 같다.
같은 글꼴·래스터 조건과 실제 전체 쪽수도 검증했다. 증거는
`source36/incoming/saved-native/`다. 이 결과는 실제 공개 API 편집이나 WASM의
완료를 대신하지 않는다. 실제 Native 편집의 기준 4개 조건은 끝났고 후보는
정식 Native 라이브러리 생산을 기다린다.

소스 33의 보호 검증은 수정 전 증거이며 최종 후보를 대신하지 않는다. 이전 기준의
Arial 4종·함초롬바탕과 outline 환경의 재현 문제가 남아 있다. 공개 legacy Arial도
원본 배포물과 파일 해시를 확인했지만 기준 네 프로그램과 모두 달라 채택하지 않았다.
글꼴을 설치하거나 진행 중인 측정의 공급 조건을 변경하지 않았다.

다른 본인 PR #9293의 실패 보정에 대한 기존 원격 승인 질문도 그대로 보존한다.
현재 작업의 필수 검증을 계속하며, 원격 변경을 중복 요청하거나 미완료를 Draft로 우회하지 않는다.


## 정식 Native 검사와 보호 환경 복원 중간 확정

최소 글꼴 환경의 정식 Native unfiltered lib는 4,033건 통과, 12건 ignored다.
기존 CI의 Native 논리 검사 21개와 안내문 실물 2건·HFT 추적·CTRL_DATA 집중
검사가 통과했다. 기존 `issue_8153_print_glyph_geometry`의 분할 프로그램 3건도
정식 suite에서 통과했다. fmt·manifest·unit-tier·workspace Clippy·WASM Clippy도
통과했다. 실행 명령·소스 SHA·실제 summary와 로그 해시는
`source36/gates/native-checkpoint/results.json`에 있다.

workspace build는 FFI archive와 incremental graph를 쓰다가 디스크 공간 부족으로
중단됐다. 사용 종료한 이번 작업의 중간 산출물을 확인해 정리하고 재실행한다.
전체 nextest는 아직 실행하지 않았으므로 전체 Rust 검증 완료로 기록하지 않는다.
Native 실제 편집 probe의 별도 rustc 연결도 serde 의존성 선택을 바로잡은 후 재실행한다.

보호 검증의 작업 전용 환경 resolver가 `conditions.font_supply.environment_sha256`를
읽지 않아 chemical·decree의 양쪽 경로 4개 조건을 누락했다. 정본 manifest의 TSV SHA와
고정한 조건 보고서 SHA를 각각 확인한 resolver로 46개 조건의 스키마를 모두 판독했다.
정확한 환경 파일은 45개 확보했고 outline Native의 1개는 아직 미확보다.
정확한 정본 환경을 적용한 decree 16쪽은 Native·fresh WASM 모두 raw 100.00%다.
이전 잘못된 stock 환경의 Native 52.97%는 제품 회귀 근거로 사용하지 않는다.
Native가 실제 선택한 HY중고딕은 이미 원본 TTF였다. Fontconfig의 CFF 선택만으로
원인을 추정했던 진단은 배제했다. 폰트 순서와 제품 구현은 변경하지 않았다.

복원 코드·조건별 SHA·두 경로 TSV·manifest·대응 PNG는
`source36/protected/environment-repair/`에 있다. 이는 16쪽의 환경 원인 확인이며
보호 전체 통과를 뜻하지 않는다. 올바른 조건의 전체 보호 비교에서 남은 하락과
미측정 항목을 계속 조사한다.


## 새 기준·후보 fresh WASM 전쪽과 Native 실제 편집 확정

평문 10개 원본의 fresh WASM도 기준·후보 각 583쪽을 완료했다. 실제 독립 PDF의
전체 쪽수와 같고 후보는 모든 쪽 98% 이상, 페이지별 기준 대비 하락은 0쪽이다.
시장구조조사는 315쪽 최저 98.40%, 온새미로 HWP/HWPX는 46쪽씩 최저 99.98%,
수학 HWP 64쪽 최저 98.11%, HWPX 64쪽 최저 98.69%다.
양쪽 원본 chunk TSV와 합친 전체 TSV·manifest는 `source36/issues/wasm/`에 있다.

Native 실제 중첩 표 편집도 기준·후보에서 각각 원본 HWP와 실제 한컴 HWPX에
30줄·70줄을 추가해 live·HWP 저장/재열기·HWPX 저장/재열기를 완료했다.
24개 상태의 독립 PDF 비교 최저는 98.07%이고 대응 12쌍의 전쪽 PNG가 바이트 단위로
같다. caret와 최종 렌더 트리의 실제 줄 내용·순서·누락·중복 및 21줄·58줄 경계의
쪽 소속을 확인했다. 실제 저장 bytes의 SHA도 보존했다.
`source36/incoming/live-native/`에 probe·연결 시도·실행 결과·TSV를 고정했다.
WASM 실제 편집·저장/재열기의 완료를 대신하지 않는다.

사용 종료한 task 전용 probe 실행 파일과 옛 frozen library 20개를 활성 명령·maps·fd
참조가 없는지 확인한 뒤 정확한 경로로 삭제해 2,054,539,230바이트를 확보했다.
공유 target과 현재 생산물·원본·PDF·TSV·PNG는 보존했다.
workspace build는 같은 최소 글꼴 환경에서 재실행해 통과했고 전체 nextest가 실행 중이다.


## 원본 OTF 백업과 보호 51쪽의 원인 분리

`/home/tsjang/Downloads/hft2otf-deploy-20261117/previous-installed-otf.zip`에서
기존 보호 기록의 SHA와 정확히 같은 OTF 247개를 찾았다. 파일은 Git 밖의 비공개
검증 경로에만 준비했고 시스템에 설치하지 않았다. 원본 프로그램 ID(face 0)도 기록했다.
정확한 옛 OTF를 단순히 `--font-path`로 공급한 세 쪽의 값은 변하지 않았다.
따라서 파일 확보만으로 실제 선택이나 전체 보호 통과를 선언하지 않는다.

sample16 HWP 2022의 51쪽은 새 `SPSMJ.otf`에 추가된 U+25CB 매핑이 영향을 주었다.
기존 정본 SPSMJ에는 이 매핑이 없다. 제품·입력·PDF·DPI·compat·기준 JSON을 유지하고,
별도 bwrap 프로세스에서 현재 SPSMJ 경로에 정확한 옛 파일만 보이도록 하자
raw가 99.44%에서 100.00%로 복원됐다. 두 Native PNG의 변경은 제목의 두 원 기호
영역에만 있고, 그 아래 본문·표의 픽셀은 전부 같다. 같은 배율로 독립 PDF와 직접 읽었다.
Native ordinary typeface trace도 확인했지만 이를 모든 projected outline의 선택 증거로
확대하지 않는다. wrapper의 `/dev/null` 준비·mount 순서 오류는 별도로 보존했다.

정식 근거는 `source36/protected/original-font-context/`다. 이 단일 Native 쪽은
fresh WASM·정본 전체·미확보 글꼴과 정확한 outline 환경의 검증을 대신하지 않는다.
시스템 폰트와 제품의 TTF → 검증된 TTC → CFF 순서는 변경하지 않았다.

전체 nextest의 컴파일은 계속 실행한다. swap 포화의 I/O 경합을 줄이기 위해
이번 작업의 시각 검증 supervisor만 일시 정지했고 이미 시작한 자식은 종료시키지 않았다.
Cargo에는 신호를 보내지 않았다. Rust 종료 또는 45분 뒤 자동 재개하며 목표 작업은 계속한다.
종료한 옛 probe library 17개도 활성 참조가 없음을 확인해 unlink했다. hardlink를 포함하므로
3,171,992,250바이트는 논리 크기이며 실제 확보 용량으로 주장하지 않는다. 공유 target은 보존했다.


## 소스 36 전체 회귀 종료와 다음 보정 조건

최소 글꼴 전체 nextest는 exit 100으로 끝났다. 13,970건 중 13,968건 통과,
2건 실패, 45건 skipped다. 실패는 oracle partition 13·15이며 각각 수출입 19쪽과
시장구조조사 315쪽의 모아 찍기 선언 제외 규약이다. 실제 쪽수 격차 실패로 바꾸어
설명하지 않는다. 이번 복원에서 독립 1-up Print 참조를 추가한 범위와 기존 생성기·
검사기의 제외 규약을 함께 추적하고, 해당 Print 근거에 따른 계약을 보정해야 한다.
쪽수·내용 검사를 제거하거나 기준값을 낮추지 않는다. 실제 summary와 실패 본문은
`source36/gates/full-rust-source36/`에 고정했다.

정본 SPSMJ를 동일하게 적용한 기준/후보의 중첩 표 16쪽은 100.00%/99.98%다.
같은 렌더 트리 위치에서 후보의 U+2027 두 개만 사라지고 21개 PNG 픽셀이 달라졌다.
원인은 선택한 SFNT의 신원을 원본 HFT 논리 이름 자체의 물리 family 인증으로 넓혀
추가한 별칭이다. 설치된 다른 논리 이름 프로그램은 이 문자를 cmap에 가지지만 윤곽이
없다. 선택한 실제 family로만 조회하는 진단은 원 프로그램 ID·bytes·메트릭 보존을
유지하면서 기준 PNG를 정확히 복원했다. 안내문 2쪽은 Native 99.99%와 기존 PNG
동일성을 유지하고 stock WASM에서도 99.99%다.

중첩 표의 private font context WASM 진단도 100.00%다. root read-only에 따른
project cache 쓰기, Snap launcher의 namespace 제약은 준비 오류로 보존했고,
기존 host Chrome 155를 명시해 실행했다. 이 경로는 진단이며 stock Snap 158과
같은 환경의 최종 보호 gate를 대신하지 않는다. 근거는
`source36/protected/alias-regression/`에 있다.

앞선 Human Gothic TTF 미확보 추정은 배제한다. 실제 추가 기준은
`HYGothic-Medium`의 H2GTRM.TTF이며 현재 파일의 SHA는
`0b18f81f9d463f6c498853af3bdad577e9c881a5b59dc7ef4c43287588d696d5`로 정본과 같다.
Human Gothic의 원본 CFF와 다른 프로그램이다. 추가 Poppy 프로그램을 제거한 진단은
오히려 악화했으므로 채택하지 않았다. 시스템 글꼴을 삭제·변경하지 않았다.

소스 36은 최종 제출 후보가 아니다. 현재 실제 출력·기준 측정은 보존한다.
미시작 소스 36 전체 보호 WASM·공통 gate의 idle supervisor만 종료해 중복 실행을
막았다. 실행 중인 Native 자식과 Cargo에는 신호를 보내지 않았다. 두 oracle 계약과
별칭 보정을 다음 단계의 코드로 고정하고 새 생산물에서 필수 검증을 다시 완료한다.
