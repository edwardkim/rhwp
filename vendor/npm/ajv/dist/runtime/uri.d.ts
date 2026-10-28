import { parse, resolve, serialize } from "fast-uri";
declare const uri: {
    parse: typeof parse;
    resolve: typeof resolve;
    serialize: typeof serialize;
    code: string;
};
export default uri;
