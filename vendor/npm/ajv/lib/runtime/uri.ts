import {parse, resolve, serialize} from "fast-uri"

// Keep Ajv's codegen metadata separate from the shared fast-uri 4 module.
const uri = {parse, resolve, serialize, code: 'require("ajv/dist/runtime/uri").default'}

export default uri
