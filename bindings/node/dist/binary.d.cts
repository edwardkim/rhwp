/**
 * rhwp 실행 파일 탐색.
 *
 * 탐색 순서는 `mydocs/tech/bindings_foundation.md` §3 이 고정한 그대로다:
 *
 * 1. 환경변수 `RHWP_BIN`
 * 2. 패키지 동봉 (`dist/_bin/`)
 * 3. `PATH`
 *
 * 순서 자체가 계약이다 — 개발자가 로컬 빌드를 가리키고 싶을 때(1) 패키지 동봉본(2)이
 * 가로채면 "왜 내 수정이 반영 안 되지"라는 진단 불가 상황이 생긴다.
 *
 * @packageDocumentation
 */
/** 바이너리 경로 환경변수 이름 — 문서 §3 고정. */
export declare const ENV_VAR = "RHWP_BIN";
/** 플랫폼별 실행 파일 이름. */
export declare function binaryName(): string;
/**
 * 탐색 캐시를 비운다.
 *
 * 테스트에서 환경변수를 바꿔 가며 검사할 때 필요하다.
 */
export declare function clearBinaryCache(): void;
/** 패키지 동봉 바이너리가 놓이는 디렉터리. */
export declare function bundledDir(): string;
/** {@link findBinary} 옵션. */
export interface FindBinaryOptions {
    /** 참이면 캐시를 무시하고 다시 탐색한다. */
    readonly refresh?: boolean;
}
/**
 * rhwp 실행 파일 경로를 돌려준다.
 *
 * @throws {BinaryNotFoundError} 세 경로 모두에서 찾지 못했을 때. 메시지에 시도한
 *   위치를 전부 담는다 — "없다"만 알려주면 사용자가 어디에 둬야 할지 모른다.
 */
export declare function findBinary(options?: FindBinaryOptions): string;
