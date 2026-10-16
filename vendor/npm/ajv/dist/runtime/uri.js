"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const fastUri = require("fast-uri");
const uri = {
    parse: fastUri.parse,
    resolve: fastUri.resolve,
    serialize: fastUri.serialize,
    code: 'require("ajv/dist/runtime/uri").default',
};
exports.default = uri;
