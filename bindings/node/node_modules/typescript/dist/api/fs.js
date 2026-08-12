import { getPathComponents } from "./path.js";
/** The callback names supported by the Go server for virtual FS delegation. */
export const fsCallbackNames = ["readFile", "fileExists", "directoryExists", "getAccessibleEntries", "realpath"];
export function createVirtualFileSystem(files) {
    const root = {
        type: "directory",
        children: {},
    };
    const content = {};
    for (const filePath of Object.keys(files)) {
        content[filePath] = files[filePath];
        addToTree(filePath);
    }
    return {
        directoryExists,
        fileExists,
        getAccessibleEntries,
        readFile,
        realpath: path => path,
        writeFile,
        removeFile,
    };
    function getNodeFromPath(path) {
        if (!path || path === "/") {
            return root;
        }
        const segments = getPathComponents(path).slice(1);
        let current = root;
        for (const segment of segments) {
            if (current.type !== "directory") {
                return undefined;
            }
            const child = current.children[segment];
            if (!child) {
                return undefined;
            }
            current = child;
        }
        return current;
    }
    function ensureDirectory(segments) {
        let current = root;
        for (const segment of segments) {
            if (!current.children[segment]) {
                current.children[segment] = { type: "directory", children: {} };
            }
            else if (current.children[segment].type !== "directory") {
                throw new Error(`Cannot create directory: a file already exists at "/${segments.join("/")}"`);
            }
            current = current.children[segment];
        }
        return current;
    }
    function addToTree(path) {
        const segments = getPathComponents(path).slice(1);
        if (segments.length === 0) {
            throw new Error(`Invalid file path: "${path}"`);
        }
        const filename = segments.pop();
        const dirNode = ensureDirectory(segments);
        dirNode.children[filename] = { type: "file" };
    }
    function writeFile(path, data) {
        content[path] = data;
        addToTree(path);
    }
    function removeFile(path) {
        delete content[path];
        const segments = getPathComponents(path).slice(1);
        if (segments.length === 0)
            return;
        const filename = segments.pop();
        const dirNode = getNodeFromPath("/" + segments.join("/"));
        if (dirNode && dirNode.type === "directory") {
            delete dirNode.children[filename];
        }
    }
    function directoryExists(directoryName) {
        const node = getNodeFromPath(directoryName);
        return !!node && node.type === "directory";
    }
    function fileExists(fileName) {
        return fileName in content;
    }
    function getAccessibleEntries(directoryName) {
        const node = getNodeFromPath(directoryName);
        if (!node || node.type !== "directory") {
            return undefined;
        }
        const fileEntries = [];
        const directories = [];
        for (const [name, child] of Object.entries(node.children)) {
            if (child.type === "file") {
                fileEntries.push(name);
            }
            else {
                directories.push(name);
            }
        }
        return { files: fileEntries, directories };
    }
    function readFile(fileName) {
        if (fileName in content) {
            return content[fileName];
        }
        return undefined;
    }
}
//# sourceMappingURL=fs.js.map