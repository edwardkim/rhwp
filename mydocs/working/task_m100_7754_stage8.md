# 9개 이슈: 최신 devel 동기화와 최종 검증 계속

`bd3cd171331e80e79881dc8e9ed37f837ef6be77` 위로 작업 브랜치를 리베이스했다.
동기화 직후 head는 `46f68fc2abec2b93c7ee0f7743a7054c694d8064`다. 이전 상태는
로컬 `local/7754-source37-before-bd3-sync`에 보존했으며 원격 변경은 하지 않았다.

## 생산물과 검증 소스의 연결

실제 최적화 Native·fresh WASM 생산 소스는
`68b4f6bdfda465aaafd0634c587cd6a17252fed6`이다. 리베이스 후 차이는 README·기록·이미지와
`#[cfg(test)] mod tests`에만 속하는 내부 검사 파일의 이관이다. 나머지 생산 Rust 코드는
바이트 동일하다. 실행 중인 시각 검사의 Python·브라우저 소비 코드도 재적용 구간 전체에서
동일하며 차이는 실행하지 않은 font trace e2e 검사 하나뿐이다. 생산 SHA를 새 SHA로
바꿔 적지 않고 이 연결을 `source38-rebase-production-equivalence.json`에 기록했다.
정식 Rust 검사는 이관한 통합 검사까지 포함해 리베이스 후 소스에서 다시 실행한다.

## 확정한 결과

| 범위 | 실제 결과 |
| --- | --- |
| 10개 평문 원본 Native | 583/583쪽, 전쪽 98% 이상, 새 기준 대비 표시값 하락 0쪽 |
| 중첩 표 독립 Print·기준·수정 전·후 | Native/fresh WASM 각 17쪽 완료, 새 후보 PNG 전쪽이 같은 환경의 기준과 바이트 동일 |
| 별칭 회귀 16쪽 | Native/fresh WASM 각각 99.98% → 100.00%, 두 점 복원 |
| 안내문 평문·실제 암호 원본 | 두 경로 각각 24쪽, 최저 99.48%, 암호/평문 PNG 각각 24개 동일 |
| 실제 Native 편집·저장·재열기 | 원본 HWP·실제 한컴 HWPX에 30/70줄 추가, 후보 12상태와 기준 12상태의 PNG·쪽수 동일, 독립 PDF·내용/소유 관계 확인 |
| 저장 원본 Native 유입 보존 | 33조건·50쪽 완료, 최저 98.07% |
| 리베이스 후 최소 글꼴 Native unfiltered lib | 4,033 PASS, 기존 12 ignored, exit 0 |
| Studio unit·TypeScript | 2,834 PASS·0 skip, 타입 검사 PASS |
| Node 정책·font trace 계약 | 36건·14건 각각 PASS |

평문 Native 최저는 복학원서 99.00%, 폴로 99.93%, 배포 문서 99.78%, 수출입 99.18%,
안내문 99.48%, 온새미로 HWP/HWPX 각각 99.98%, 규제분석 HWP 98.08%·HWPX 98.69%,
시장구조조사 98.40%다. 모든 값은 96 DPI·Print·보정 전 2 px raw의 페이지별 최저다.
정식 원본 TSV·manifest·producer 영수증·대표 PNG는
`mydocs/pr/assets/issues7754_7765/source38/`에 고정했다. 전체 fresh WASM 583쪽은 아직
진행 중이며 위 완료 범위와 구분한다.

## 남은 필수 작업

- 리베이스 후 공식 Native CI 사례·focused 통합/쪽수 oracle·전체 nextest·Clippy·workspace build를
  끝낸다. Cargo는 공용 target 한 곳에서 순차 실행하며 메모리 경합을 줄이기 위해 build jobs는
  2개로 제한했다. 정상 incremental 정책은 유지한다.
- 후보 fresh WASM 원본 전쪽, 유입 기능의 실제 WASM 편집·저장·재열기·내용 보존, Studio 실제
  여정·CTRL_DATA 물리 레코드 보존을 끝낸다. 기준 재사용은 같은 입력·같은 실제 환경·같은 고정
  producer의 완료 결과만 사용한다.
- 원래 CFF 프로그램을 프로세스의 읽기 전용 namespace에만 공급하여 전체 정본 보호와 공식
  편람 gate를 수행한다. 시스템 글꼴은 설치·교체하지 않는다. 캐시·설치되지 않은 파일까지
  4,342개를 읽어 SHA를 확인했고 원래 HDOTUM.TTF 하나도 추가 확인했다. 글꼴 바이트는
  저장소에 넣지 않았다. 정본의 개별 파일 목록과 일치한 공급 항목은 조건별 누적 1,271개다.
- outline Native의 환경 JSON `4ade9f383e2554fddcfa517cf8de6d1dadf51c379836a615795afabe9eeaf41d`은
  아직 정확한 바이트를 확보하지 못했다. 큰 metadata 안의 환경 객체·인코딩된 JSON도 추가
  대조했지만 exact SHA가 일치하지 않았다. 같은 문서의 100% 출력만으로 조건 복원을 완료로
  처리하지 않는다. 전체 보호·원본 조건 확보와 직접 확대 판독을 계속한다.
- 본인 열린 #9293은 reviewer가 없고 같은 head의 CI 실패가 남아 있다. 별도 원격 보정의 승인
  질문은 기존 요청을 유지하며 반복하지 않는다. 준비한 국소 보정은 보존하고 승인 범위를 넘는
  push는 하지 않는다. 9개 이슈의 모든 필수 gate·열린 PR 선행 조건과 병합 시뮬레이션을 마친
  뒤 이미 승인된 PR·통합·후속 처리를 진행한다.

평균값·현재 base와 같다는 이유·98% 유지·다른 페이지 개선으로 정본 하락을 면제하지 않는다.
아직 실행 중인 범위를 완료 또는 통과로 표시하지 않는다.
