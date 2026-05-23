(function (global) {
    const runtime = global.protopipeRenderRuntime || {};

    runtime.resolvePartialId = function (partialId, namespace, rfaReplacements) {
        const replacement = rfaReplacements.find(function (candidate) {
            return candidate.old === partialId && runtime.namespaceMatches(candidate.namespace, namespace);
        });

        return replacement === undefined ? partialId : replacement.new;
    };

    global.protopipeRenderRuntime = runtime;
})(globalThis);
