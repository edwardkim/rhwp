---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-8403/validation/README.md
last_verified: 2026-10-31
---

# 최신 devel 기준 최종 검증

기준은 `99a176b7cb34fde54454c03433baa1382b2de9fa`다. Native는 `21a4902f14`,
fresh WASM은 `b13334f327`에서 만들었다. 두 커밋 사이에는 `cfg(test)` helper 위치만
달라 제품 코드는 동일하다. 후속 문서·하네스·공개 입력 첨부는 제품 코드 변경이 아니다.

- `candidate/`, `base/`: 같은 기준 PDF·글꼴 조건의 전쪽 raw TSV와 실제 HTTP 자산 해시.
- `base-candidate.tsv`: 각 페이지의 이전/후보 값과 차이. `comparison-summary.json`은 집계.
- `hft-preservation/`: #8397의 763쪽 한글 3.0 문서에 대한 양쪽 Native/WASM 전쪽 결과.
- `gates.json`, `visual-run.json`, `base-run.json`, `finish-local.json`: 명령·종료 상태·시간.
- `provenance.json`: 생산 코드와 로그 해시. 글꼴 바이트나 비공개 원본 내용은 포함하지 않는다.
- `attached-wasm/`: [첨부한 6개 HWP/HWPX](../inputs/README.md)의 동일 바이트 재열기 38쪽.
- `studio-screen/`: 브라우저가 실제 합성한 화면과 backend 진단. 공개 대조군만 PNG로 보존한다.
- `*-review.png`, `*-overlay.png`: Native와 WASM의 같은 공개 페이지를 독립 PDF와 비교한 패널.

## 검증 하네스 보정 이력

제품 기능의 미해결 실패를 성공으로 바꾸어 기록하지 않는다. `initial-failures.json`과
`base-run-initial.json`은 최초 실패·재실행 경위를 남긴다.

1. 최신 글꼴 선택창을 하네스가 처리하지 않아 열기 완료를 기다리던 경우에는 실제 UI에서
   대체 글꼴 선택을 실행했다. 이 선택은 host가 공급한 명시적 글꼴의 사용을 막지 않는다.
2. 원시 WASM Canvas 출력이 새 Studio host 글꼴 세션을 우회해 원본 1쪽이 87.15%였다.
   공식 host font provider와 Studio bridge를 사용한 최종 원본 8쪽은 모두 90% 이상이다.
3. 큰 로컬 글꼴을 하나의 CDP 메시지로 전달할 때 브라우저가 종료되었다. 글꼴 파일별로
   전송하고 프로그램 바이트를 Uint8Array로 보관한 뒤 전체 열기·저장·편집 검사를 통과했다.
4. base의 원 결함으로 90% 미만인 TSV는 기준선 관측이다. 후보에는 계속 90% 게이트를
   적용하며 base의 낮은 점수를 통과한 후보 결과로 표시하지 않는다.
5. base Native 실행 전 generated suite 준비가 빠졌던 실행은 준비 후 같은 base에서 다시
   실행했다. candidate WASM은 최초 host 관찰 프로세스 종료 뒤에도 같은 Docker 빌드를
   계속 관찰해 exit 0을 확인했다. 컴파일을 다시 시작한 결과로 바꾸어 기록하지 않았다.

6. base 비교 뒤 공용 target이 기준 브랜치의 PDF backend 메타데이터를 재사용해 후보의
   새 타입을 찾지 못했다. 변경 Rust 파일의 timestamp를 갱신해 후보를 재컴파일했다.
   소스 내용·기대값은 변경하지 않았다. `candidate-restored-source-units.json`에 대상을 기록했다.
7. base에는 없는 문자 paint를 확인하는 SVG 진단은 base에서 실행하지 않는다. base의 편집
   검증은 실제 Canvas 출력·14곳 치환·저장 후 내용 보존으로 수행한다. 후보의 paint SVG
   assertion은 유지한다.

8. 신규 sample 래칫 준비가 `--tests`로 관계없는 실행 파일까지 빌드하던 호출은 로컬에서
   중단(exit 130)했다. manifest로 확인한 여섯 target을 명시해 같은 여섯 종류의 래칫을
   다시 실행했다. 기존에 통과한 보안 검사·helper 검사는 반복하지 않았다.

폰트 두 개에서 OTS가 거절한 빈 VTT 소스 테이블은 기존 `font_catalog` 정규화로 제거했다.
윤곽·메트릭은 유지하고 원본/처리 후 SHA와 repairs를 `candidate/original-font-evidence.json`에
기록했다. Native/fresh WASM 비교에 사용한 로컬 글꼴 프로그램 자체는 게시하지 않는다.

## 실제 화면 판독

공개 패턴 3쪽의 오른쪽 반쪽 타일은 빨강/파랑 열만 보이며 Native·Canvas2D·CanvasKit에서
동일하다. 문자 2쪽의 파란 획 전용 글자는 내부가 비고 아래의 일반 검정 채움 글자와
구분된다. 직접 화면에서도 글자 획·한글 내부 공간·기준선과 뒤 내용 위치를 확인했다.

비공개 원본의 실제 CanvasKit 요청은 Arial·Droid Sans Fallback 리소스 미준비로
Canvas2D에 fallback되었다. 해당 실제 backend를 result.json에 보존했다. 공개 패턴·문자
12쪽의 CanvasKit 성공으로 원본 CanvasKit 성공까지 주장하지 않는다.

HFT 대조 문서 610쪽의 Native 최저 구간은 제목의 한글 글꼴 모양 차이가 남고 WASM은
독립 출력의 모양을 유지한다. 본문 순서·열·하단 쪽 번호와 제목의 점유 위치는 보존된다.
이를 제목까지 바이트 동일한 결과로 표현하지 않으며, 같은 명시적 글꼴 조건의 base와
candidate를 비교해 이번 패턴·문자 paint 변경의 후퇴 여부를 판정한다.

## 최종 비교 집계

기본 비교와 그림 간격 대조를 합한 241개 페이지가 모두 90% 이상이다. base 대비 후보의
최대 감소는 0.02%p이며 2%p 초과 감소는 없다. HFT 별도 대조 763쪽은 Native 최저 96.61%,
WASM 최저 99.17%다. 각 backend에서 base/candidate PNG 763개가 전부 바이트 동일하다.
해당 결과는 `hft-preservation/*-png-equality.json`과 전쪽 TSV로 함께 보존한다.

신규 HWP/HWPX 6개를 명시한 보안 검사 20건과 샘플 포함 6종 레이아웃 래칫 81건이
모두 통과했다. 기대값·baseline·검사 목록에서 입력을 제외하는 보정은 하지 않았다.
base 브라우저 재현은 후보 `browser-verify.mjs`를 base 작업공간에 복사한 뒤
`base-browser-verify.patch`를 적용한다. 차이는 base에 없는 개선 assertion과 오류 사례이며,
원본·폰트 공급·Canvas 출력·저장·치환 경로는 동일하다.
