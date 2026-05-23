(function (global) {
    const runtime = global.protopipeRenderRuntime || {};

    function registry() {
        if (global.rfaRegistry === undefined || global.rfaRegistry === null) {
            global.rfaRegistry = {};
        }

        return global.rfaRegistry;
    }

    runtime.initializeRegistry = function () {
        global.rfaRegistry = {};
    };

    runtime.resetRegistry = function () {
        global.rfaRegistry = {};
    };

    runtime.registerRfa = function (rfaId, renderFunction) {
        registry()[rfaId] = renderFunction;
    };

    runtime.getRfa = function (rfaId) {
        return registry()[rfaId];
    };

    global.protopipeRenderRuntime = runtime;
})(globalThis);
