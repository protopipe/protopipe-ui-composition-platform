(function (global) {
    const runtime = global.protopipeRenderRuntime || {};

    runtime.ensureRootNamespace = function (context, rootRfaId) {
        if (context.namespace === undefined || context.namespace === null || context.namespace === "") {
            context.namespace = rootRfaId;
        }
    };

    runtime.currentNamespace = function (baseContext, rootRfaId) {
        return typeof baseContext.namespace === "string" && baseContext.namespace.length > 0
            ? baseContext.namespace
            : rootRfaId;
    };

    runtime.createPartialNamespace = function (baseContext, rootRfaId, partialId, partialIndex) {
        const baseNamespace = runtime.currentNamespace(baseContext, rootRfaId);
        if (partialIndex === undefined || partialIndex === null) {
            return baseNamespace + "." + partialId;
        }
        return baseNamespace + "." + partialIndex + "." + partialId;
    };

    runtime.namespaceMatches = function (pattern, namespace) {
        if (pattern === undefined || pattern === null || pattern === "") {
            return true;
        }
        if (pattern === namespace) {
            return true;
        }
        if (pattern.endsWith(".*")) {
            const prefix = pattern.slice(0, -2);
            return namespace.startsWith(prefix + ".");
        }
        return false;
    };

    runtime.createScopedContext = function (baseContext, partialNamespace) {
        return Object.assign({}, baseContext, { namespace: partialNamespace });
    };

    global.protopipeRenderRuntime = runtime;
})(globalThis);
