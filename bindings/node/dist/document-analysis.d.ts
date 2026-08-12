/**
 * 표 교환·구조 데이터·읽기 전용 보안 검사의 1층 래퍼.
 *
 * `commands.ts`는 오래된 공개 import 경로를 유지하기 위해 이 모듈을 재내보낸다.
 * 문서 분석 명령의 옵션과 인자 조립을 여기 한 곳에 모아 주 명령 모듈이 1000줄을
 * 넘지 않게 한다.
 */
import { Envelope } from './envelope.js';
import type { CommandOptions, EditOptions, OutputOptions, PathLike } from './commands.js';
/**
 * 명령별 문서 파생 필드의 출처 지도.
 *
 * 문서를 입력으로 받지 않는다. `untrustedContent`·`untrustedFields` 표지가 어느
 * 명령의 어느 필드에 붙는지, 그리고 그 값이 어떤 엔진 경로에서 유래하는지를
 * 기계가 읽을 수 있는 JSON으로 돌려준다.
 */
export declare function exportProvenanceMap(options?: CommandOptions): Promise<Envelope>;
/** {@link tableToCsv} 옵션. */
export interface TableToCsvOptions extends OutputOptions {
    /** 내보낼 최상위 표 번호. 생략하면 모든 표. */
    readonly table?: number | undefined;
    /** UTF-8 BOM을 붙인다. Excel 호환이 필요할 때만 켠다. */
    readonly bom?: boolean | undefined;
}
/**
 * 본문 최상위 표를 RFC 4180 CSV로 내보낸다.
 *
 * 병합된 칸은 빈 칸으로 유지한다. 그 결과를 {@link csvToTable} 에 다시 넣으면
 * 병합 구조를 바꾸지 않은 채 셀 내용을 왕복할 수 있다.
 */
export declare function tableToCsv(path: PathLike, options?: TableToCsvOptions): Promise<Envelope>;
/** {@link csvToTable} 옵션. */
export interface CsvToTableOptions extends EditOptions {
    /** 적용할 RFC 4180 CSV 파일 경로. */
    readonly csv: PathLike;
    /** 덮어쓸 최상위 표 번호. */
    readonly table: number;
}
/**
 * CSV 내용으로 기존 표 하나의 셀 값을 덮어쓴다.
 *
 * 표의 행·열 수와 CSV가 다르면 문서를 쓰지 않고 CLI가 usage 판정을 낸다. 표
 * 구조를 늘리거나 줄이는 명령이 아니므로, 먼저 {@link tableToCsv} 로 현재 격자를
 * 확인한 뒤 같은 모양의 CSV를 넘겨야 한다.
 */
export declare function csvToTable(path: PathLike, options: CsvToTableOptions): Promise<Envelope>;
/** `extract-data`가 식별하는 값 종류. */
export type ExtractDataKind = 'date' | 'amount' | 'number' | 'all';
/** {@link extractData} 옵션. */
export interface ExtractDataOptions extends CommandOptions {
    /** 추출할 값 종류. 생략하면 CLI 기본 종류를 사용한다. */
    readonly kind?: ExtractDataKind | undefined;
    /** 반환할 항목 최대 수. 전체 건수는 봉투의 totalItemCount로 확인한다. */
    readonly limit?: number | undefined;
}
/** 날짜·금액·수량을 문서 주소(구역·문단·쪽·문자 오프셋)와 함께 추출한다. */
export declare function extractData(path: PathLike, options?: ExtractDataOptions): Promise<Envelope>;
/** `inspect` 하위 검사 종류. */
export type InspectTarget = 'hidden-text' | 'injection' | 'unicode';
/** `inspect unicode`가 필터링할 유니코드 기만 축. */
export type UnicodeInspectionKind = 'zero-width' | 'bidi' | 'tag' | 'confusable' | 'all';
/** `inspect hidden-text` 옵션. */
export interface InspectHiddenTextOptions extends CommandOptions {
    /** 이 pt보다 작은 글자를 은닉 텍스트 후보로 본다. */
    readonly thresholdPt?: number | undefined;
    /** 쪽 경계 밖에 완전히 놓인 문단도 포함한다. */
    readonly includeOffpage?: boolean | undefined;
}
/** `inspect injection` 옵션. */
export interface InspectInjectionOptions extends CommandOptions {
    /** 이 등급보다 낮은 프롬프트 주입 신호를 제외한다. */
    readonly minConfidence?: 'low' | 'medium' | 'high' | undefined;
    /** 누름틀·숨은 설명까지 검사 범위를 확장한다. */
    readonly includeFields?: boolean | undefined;
}
/** `inspect unicode` 옵션. */
export interface InspectUnicodeOptions extends CommandOptions {
    /** 제로폭·방향 제어·태그·동형자 중 검사할 축. */
    readonly kind?: UnicodeInspectionKind | undefined;
}
/** `inspect` 하위 명령별 옵션의 합집합. */
export type InspectOptions = InspectHiddenTextOptions | InspectInjectionOptions | InspectUnicodeOptions;
/** 사람 눈에 안 보이지만 추출기에는 보이는 본문 텍스트를 검사한다. */
export declare function inspect(target: 'hidden-text', path: PathLike, options?: InspectHiddenTextOptions): Promise<Envelope>;
/** 문서 본문에 섞인 프롬프트 주입 신호를 읽기 전용으로 검사한다. */
export declare function inspect(target: 'injection', path: PathLike, options?: InspectInjectionOptions): Promise<Envelope>;
/** 화면 표시와 실제 순서가 다른 유니코드 기만 문자를 검사한다. */
export declare function inspect(target: 'unicode', path: PathLike, options?: InspectUnicodeOptions): Promise<Envelope>;
