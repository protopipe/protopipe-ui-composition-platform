(function (global) {
    const runtime = global.protopipeRenderRuntime || {};

    runtime.createPartials = function (context, rootRfaId, rfaReplacements) {
        return {
            include: function (partialId, partialContext, optionalIndex) {
                const baseContext = partialContext === undefined ? context : partialContext;
                const partialNamespace = runtime.createPartialNamespace(baseContext, rootRfaId, partialId, optionalIndex);
                const resolvedPartialId = runtime.resolvePartialId(
                    partialId,
                    partialNamespace,
                    rfaReplacements
                );
                const partial = runtime.getRfa(resolvedPartialId);
                runtime.ensurePartialExists(partial, resolvedPartialId);

                const scopedContext = runtime.createScopedContext(baseContext, partialNamespace);
                const output = partial(scopedContext, this);
                return runtime.stringifyRenderOutput(output);
            }
        };
    };

    runtime.ensurePartialExists = function (partial, resolvedPartialId) {
        if (typeof partial !== "function") {
            throw new Error("RFA not found: " + resolvedPartialId);
        }
    };

    global.protopipeRenderRuntime = runtime;
})(globalThis);
