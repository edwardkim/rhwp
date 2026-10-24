# Stage70: 첫 저장 띠의 최소와 Center 잔여를 구분하는 검사 교정

Issue: #7445 / #7470. 부모 HEAD `e30ef94440ee792519ac8ceffa1055330609c058`. 기존 inventory55/56을 유지했으며 새 HWP 보정 대상을 추가하지 않았다.

## 선행 근거와 교정 이유

원 해결 PR #7567 통합 `1d6bc70767`은 18쪽 원본의 쪽·셀·중첩 꼬리·후속 본문 소유를 보호한다. 이후 `0fa99ae38d`가 첫 저장 띠와 문단 앞 간격을 정확히 2000HU로 고정했다. Stage46도 그 정확한 등식을 완료 근거로 썼지만, 현재 독립 자료와 실제 소유 프레임을 대조해 그 가정을 정정한다. 과거 보고서를 현재 기대값의 독립 근거로 대신하지 않았다.

현재 셀은 원본 Center이고 실제 closing/frame 소유로 배치된다. 앞 띠와 앞 간격은 각 1000HU의 최소 공간이며 잔여 Center 공간도 합법적으로 남는다. 실제 첫 띠 관계는 2094HU다. 독립 PDF 제목 기준선 354.720001pt와 Native 논리 기준선 472.933333CSSpx를 대조하면 원본 가운데 정렬이 대응한다. PDF glyph 잉크 외곽과 줄 기준선을 혼동하지 않았다.

## 유지·보완한 검사

기존 쪽수 18, 절 제목/첫 제목의 소속·누락/중복, source 패딩·앞 간격, 중첩 끝 줄·뒤 본문·빈 띠·float sibling 관계를 유지했다. 잘못된 정확한 등식만 원본 최소 보존과 실제 소유 내용 앞뒤의 Center 여유 관계로 바꿨다. 직렬화 허용은 기존 7.5HU와 같은 0.1CSSpx다. 실제 마지막 TextLine 상자를 사용해 빈 줄도 공간으로 보존한다.

한컴 2024 기준과 CLI의 compat2024를 명시했다. 전 18쪽 Native/fresh WASM90% 이상 및 해당 13쪽 98.77313/98.77812%·직접 판독 조건을 확인하고 독립 제목 기준선±2CSSpx를 같은 기존 함수에 추가했다. 새 함수·fixture·기준 PDF·skip을 추가하지 않았다. 렌더링 구현·글꼴·임계값은 변경하지 않았다.

## 실제 실행과 재사용

- 기존 검사 실제 1 FAIL:2094HU 대 2000HU, 테스트 4.102초다.
- 최종 prepare 뒤 실제 배정된 suite의 모듈 전체 6 PASS다. 최종 명령의 빌드 포함 시간은 9.886초다.
- 명시적 기준선 핀을 포함한 최종 요약: `Summary [   2.010s] 6 tests run: 6 passed, 250 skipped`.
- 제품 소스 14개 SHA는 Stage69와 같고, 같은 producer의 HWP/HWPX 각 18쪽 Native/fresh WASM TSV·PNG를 재사용했다. 재빌드나 새 캡처 통과로 표현하지 않는다.
- 이 교정은 검사 기대값의 오류를 고친 것이며 새 렌더링 결함을 FAIL→PASS로 보정한 주장과 구분한다. 전체 nextest 결과도 아니다.

[독립 근거·전쪽 TSV·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-7bb143879-20261023/center-first-band/README.md). source/test hash·명령·원 로그는 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage70-center-source-assertion/`에 보존한다.

직전 집중 목록의 남은 3개와 C354/C356 대조, 최종 전체 회귀·gate·devel 비교·실제 소비 검증은 계속 필요하다. 원격 push·PR·merge는 실행하지 않았다.
