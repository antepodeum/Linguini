export function inputFromRequest(request: Request, input: Record<string, unknown> = {}): Record<string, unknown> {

    const requestInput: Record<string, unknown> = { ...input };
    requestInput.url ??= request.url;
    requestInput.currentUrl ??= requestInput.url;
    requestInput.headers ??= request.headers;
    requestInput.cookie ??= readHeader(requestInput.headers, "cookie") ?? readHeader(request.headers, "cookie") ?? undefined;
    return requestInput;
}

function readHeader(headers: unknown, name: string): string | undefined {
    if (!headers)
        return undefined;
    const getter = ( headers as {
        get?: (header: string) => string | null | undefined;
    }).get;
    if (typeof getter !== "function")
        return undefined;
    try {
        return getter.call(headers, name) ?? undefined;
    }
    catch {
        return undefined;
    }
}
