# #6901 Stage 1: PR merge-tree 증거 발행의 오래된 event base 보정

## 분석

- 기준 devel: `045a04e4c9009f9e3b7a917e8b64f012fc13fbc5`.
- #7037 Full CI `34609842928`은 B/C/D duration을 모두 발행했지만 merge-tree 발행 job `103297274820`은 부모 불일치로 거부했다. 최종 trailing CI `34612013620`도 동일했다.
- Full caller SHA `e96a26f80be90239c2f4350e83b9bd1e552785c9`의 API 부모는 `b5549292d8f6854fbc86ce825580dbe9496a567a`, `05eeaeef8773d206bdb3afc32a909ad6a087b16b`다. trailing caller `aaeb68771a581ff3cc7c573ee2fbad2cc4b28191`도 첫 부모가 `b5549292d`다.
- 같은 Full run preflight는 event base 계열 값인 `ddc7bdf229db7f61fdeefee106fd1787c8964995`를 checkout했다. API compare에서 `ddc7bdf22 -> b5549292d`는 ahead이고 merge-base가 ddc7bdf22이며 변경은 오늘할일·#7039 리뷰 두 Markdown뿐이다. 캡처의 event base 동일성 요구가 실제 테스트 merge와 어긋나는 재현이다.
- devel CI `34612441307` 및 CodeQL `34612441059`는 `fork-pr-merge-tree-evidence-unavailable`로 Full 실행했다. 증거 없는 재사용 거부는 유지해야 한다.

## 수정 범위와 신뢰 계약

1. caller의 정확한 merge SHA·tree·두 부모 및 event head 일치를 유지한다. 최신 PR merge ref를 대신 사용하지 않는다.
2. event base와 실제 첫 부모가 다르면 API compare로 event base가 실제 첫 부모의 조상임을 증명한다. 실제 첫 부모가 원본 저장소 현재 devel의 조상이라는 별도 증명도 요구한다. API 오류·역방향·분기·잘못된 merge-base·다른 branch 신원은 거부한다.
3. 발행 JSON에는 실제 parents와 event base를 함께 남긴다. post-merge의 run/attempt/PR/fork/정확한 tree 대조, duration ZIP/JSON 검증과 CodeQL worker 성공 요건은 완화하지 않는다.
4. 기존 inline capture 코드를 직접 실행하는 JS 계약 테스트로 정상·오래된 base·변조/실패를 검증하고 기존 post-merge/duration JS 및 Python 계약을 함께 실행한다.

## 결과

- reusable workflow의 capture에 두 upstream 계보 증명과 event base 진단 필드를 추가했다. 기존 post-merge 소비자의 정확한 merge-tree·run/attempt 대조와 duration/CodeQL 검증은 변경하지 않았다.
- 실제 inline capture를 실행하는 신규 JS 테스트 28개를 포함해 post-merge·duration 관련 JavaScript 계약 **492개 통과**, 실패·skip 0개.
- Python CodeQL·post-merge·nextest archive workflow 계약 **50개 통과**. 처음 `*duration*.py` 검색 실행은 일치하는 테스트 파일이 없어 0개였으며 성공 검증으로 계산하지 않았다. 실제 관련 세 모듈을 명시해 다시 실행했다.
- `git diff --check` 통과. Rust/Studio 코드를 변경하지 않아 제품 빌드나 전체 Rust 회귀는 중복 실행하지 않았다.
- GitHub API는 계약 테스트에서 모의 응답이다. 실물 #7037의 merge commit 부모 및 base ancestry는 별도 live API로 확인했지만, 새 capture의 원격 배포·후속 fork Full/trailing/merge 재사용 실증은 아직 수행하지 않았다.
- 배포 자체의 enforcement 변경 Full CI, 후속 fork의 실제 merge-tree/B/C/D 발행과 post-merge heavy skip·duration refresh·CodeQL 재사용이 남아 있다. #6901은 OPEN 유지하며 완료로 주장하지 않는다.
