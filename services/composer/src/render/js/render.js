(function (global) {
    const runtime = global.protopipeRenderRuntime || {};

    runtime.renderRfa = function (request) {
        const render = runtime.getRfa(request.rfaId);
        const rootRfaId = request.rfaId;
        const context = JSON.parse(request.contextJson);
        runtime.ensureRootNamespace(context, rootRfaId);

        const rfaReplacements = JSON.parse(request.rfaReplacementsJson);
        const partials = runtime.createPartials(context, rootRfaId, rfaReplacements);
        const output = render(context, partials);
        return runtime.stringifyRenderOutput(output);
    };

    global.protopipeRenderRuntime = runtime;
})(globalThis);
