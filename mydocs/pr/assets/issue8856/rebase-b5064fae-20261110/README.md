# b5064fae 리베이스와 반영 기능 보존

기준 `b5064fae1eefff92d60464c51844a38debec09f9`에 기존 4개 보정 커밋을 충돌 없이
적용한 head는 `10026faaf8ecf34cc7517cfb738ae0cfe421157f`다.
`after.json`의 기존 소스 22개와 `inputs-after.json`의 원본·PDF 18개가 전부 동일하다.
과거 전쪽 시각 점수는 새 head 검증으로 재명명하지 않는다.

새 #8867의 암호/비암호 PDF 디코드·로그 비노출 검사 2건은 최소 글꼴 환경에서 통과했다.
qcms C ABI의 새 예산 실패·호출자 버퍼 보존·정상 LUT와 기존 검사 전체 30건도 통과했다.
`validation.json`에 실제 종료 코드·로그 해시·소스 해시를 기록했다. ICC 대조군 15개는
upstream 검증 manifest의 Git blob·SHA-256·바이트 수와 각각 일치했다.

독립 qcms 검사에는 Mac ARM의 기존 테스트 조건 누락이 있었다. `transform_neon` 모듈은
`neon` feature가 켜졌을 때만 존재하나 gtest의 import·비교는 아키텍처만 확인했다.
이들을 같은 feature 조건으로 맞췄다. 일반 C ABI 검사는 제거하거나 건너뛰지 않는다.
NEON 강제 활성화는 오래된 nightly 전용 속성 때문에 현 stable에서 실패하므로 그 실행을
검증 성공으로 사용하지 않는다. 새 기능 보존은 실제 비활성 NEON의 일반 C ABI 경로로 확인했다.
런타임 제품의 색 변환·렌더링 구현은 이 테스트 조건 수정으로 바뀌지 않는다.

실행 명령은 공유 `target/pr-review`를 사용했다. 30건은 고정 vendor 소스를 복사한
`output/issue8856/rebase-b5064fae-20261110/qcms-standalone/`에 작은 standalone workspace를
두고 실행했다. ICC·스크립트·로그·임시 manifest는 output에만 있다.
새 head의 전쪽 TSV·fresh WASM 및 최종 제출 게이트는 후속 보정 뒤 완료한다.
