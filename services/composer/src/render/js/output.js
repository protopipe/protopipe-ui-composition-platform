(function (global) {
    const runtime = global.protopipeRenderRuntime || {};

    runtime.stringifyRenderOutput = function (output) {
        return typeof output === "string" ? output : JSON.stringify(output);
    };

    global.protopipeRenderRuntime = runtime;
})(globalThis);
