"""Test Web API globals — queueMicrotask, structuredClone, Event, AbortController, URL, FormData, File, Request, etc."""

import pytest # pyright: ignore[reportUnusedImport]
from kossjs_interface import KossJS


def _true(koss: KossJS, code: str):
    result = koss.eval(code)
    return str(result).strip() == "true"


class TestWebApiGlobals:
    """Global objects defined by the Web platform (available without require)."""

    def test_self_is_global(self, koss: KossJS):
        assert _true(koss, "globalThis.self === globalThis;")

    def test_queueMicrotask_exists(self, koss: KossJS):
        assert koss.eval("typeof queueMicrotask") == "function"

    def test_queueMicrotask_runs(self, koss: KossJS):
        result = koss.eval("""
        var done = false;
        queueMicrotask(function() { done = true; });
        typeof queueMicrotask === 'function';
        """)
        assert str(result).strip() == "true"

    def test_structuredClone_deep(self, koss: KossJS):
        assert _true(koss, """
        var obj = { a: 1, b: { c: [1, 2, 3] } };
        var clone = structuredClone(obj);
        clone.a === 1 && clone.b.c.length === 3 && clone !== obj;
        """)

    def test_structuredClone_cycle(self, koss: KossJS):
        assert _true(koss, """
        var obj = { x: 1 };
        obj.self = obj;
        var clone = structuredClone(obj);
        clone.self === clone;
        """)

    def test_structuredClone_typed_array(self, koss: KossJS):
        assert _true(koss, """
        var buf = new Uint8Array([9, 8, 7]);
        var c = structuredClone(buf);
        c[2] === 7 && c !== buf;
        """)

    def test_event_class(self, koss: KossJS):
        assert _true(koss, """
        var e = new Event('click', { bubbles: true, cancelable: true });
        e.type === 'click' && e.bubbles === true && e.cancelable === true;
        """)

    def test_custom_event(self, koss: KossJS):
        assert _true(koss, """
        var e = new CustomEvent('go', { detail: { id: 42 } });
        e.type === 'go' && e.detail.id === 42;
        """)

    def test_dom_exception(self, koss: KossJS):
        assert _true(koss, """
        var e = new DOMException('boom', 'AbortError');
        e.name === 'AbortError' && e.message === 'boom';
        """)


class TestWebApiAbort:
    """AbortController / AbortSignal Web API."""

    def test_abort_controller_global(self, koss: KossJS):
        assert koss.eval("typeof AbortController") == "function"
        assert koss.eval("typeof AbortSignal") == "function"

    def test_abort_basic(self, koss: KossJS):
        assert _true(koss, """
        var ac = new AbortController();
        var fired = false;
        ac.signal.addEventListener('abort', function() { fired = true; });
        ac.abort();
        ac.signal.aborted === true && fired === true;
        """)

    def test_abort_reason(self, koss: KossJS):
        assert _true(koss, """
        var ac = new AbortController();
        ac.abort('nope');
        ac.signal.reason === 'nope';
        """)

    def test_abort_signal_abort_static(self, koss: KossJS):
        assert _true(koss, """
        var s = AbortSignal.abort('reason');
        s.aborted === true && s.reason === 'reason';
        """)

    def test_abort_signal_timeout(self, koss: KossJS):
        assert _true(koss, """
        var s = AbortSignal.timeout(100);
        s instanceof AbortSignal;
        """)

    def test_throw_if_aborted(self, koss: KossJS):
        assert _true(koss, """
        (function() {
            var ac = new AbortController();
            ac.abort();
            try { ac.signal.throwIfAborted(); return false; }
            catch (e) { return true; }
        })();
        """)


class TestWebApiUrl:
    """URL / URLSearchParams as Web globals."""

    def test_url_global(self, koss: KossJS):
        assert koss.eval("typeof URL") == "function"
        assert koss.eval("typeof URLSearchParams") == "function"

    def test_url_parse(self, koss: KossJS):
        assert _true(koss, """
        var u = new URL('https://example.com/path?q=1#h');
        u.protocol === 'https:' && u.hostname === 'example.com' &&
        u.pathname === '/path' && u.search === '?q=1' && u.hash === '#h';
        """)

    def test_url_base(self, koss: KossJS):
        assert _true(koss, """
        var u = new URL('/foo', 'https://base.com:8080/dir/');
        u.href === 'https://base.com:8080/foo';
        """)

    def test_url_search_params(self, koss: KossJS):
        assert _true(koss, """
        var u = new URL('https://example.com');
        u.searchParams.set('a', '1');
        u.searchParams.append('a', '2');
        u.searchParams.get('a') === '1' && u.searchParams.getAll('a').length === 2;
        """)

    def test_search_params_iteration(self, koss: KossJS):
        assert _true(koss, """
        var sp = new URLSearchParams('x=1&x=2&y=3');
        var count = 0;
        var it = sp.entries();
        var e = it.next();
        while (!e.done) { count++; e = it.next(); }
        count === 3;
        """)

    def test_search_params_for_of(self, koss: KossJS):
        assert _true(koss, """
        var sp = new URLSearchParams('a=1&b=2');
        var keys = [];
        for (var entry of sp) keys.push(entry[0]);
        keys.join(',') === 'a,b';
        """)


class TestWebApiBlobFileFormData:
    """Blob / File / FormData Web APIs."""

    def test_blob_global(self, koss: KossJS):
        assert koss.eval("typeof Blob") == "function"

    def test_blob_text(self, koss: KossJS):
        assert _true(koss, """
        var b = new Blob(['hello world'], { type: 'text/plain' });
        b.type === 'text/plain' && b.size === 11;
        """)

    def test_file_global(self, koss: KossJS):
        assert koss.eval("typeof File") == "function"

    def test_file_basic(self, koss: KossJS):
        assert _true(koss, """
        var f = new File(['data'], 'test.txt', { type: 'text/plain' });
        f.name === 'test.txt' && f instanceof Blob;
        """)

    def test_formdata_global(self, koss: KossJS):
        assert koss.eval("typeof FormData") == "function"

    def test_formdata_append_get(self, koss: KossJS):
        assert _true(koss, """
        var fd = new FormData();
        fd.append('a', '1');
        fd.append('a', '2');
        fd.append('b', '3');
        fd.get('a') === '1' && fd.getAll('a').length === 2 && fd.get('b') === '3';
        """)

    def test_formdata_iteration(self, koss: KossJS):
        assert _true(koss, """
        var fd = new FormData();
        fd.append('k1', 'v1');
        fd.append('k2', 'v2');
        var keys = [];
        var it = fd.keys();
        var e = it.next();
        while (!e.done) { keys.push(e.value); e = it.next(); }
        keys.join(',') === 'k1,k2';
        """)


class TestWebApiRequestFetch:
    """Request class and fetch signal integration."""

    def test_request_global(self, koss: KossJS):
        assert koss.eval("typeof Request") == "function"

    def test_request_basic(self, koss: KossJS):
        assert _true(koss, """
        var req = new Request('https://api.example.com', { method: 'POST', body: 'hello' });
        req.url === 'https://api.example.com' && req.method === 'POST';
        """)

    def test_request_clone(self, koss: KossJS):
        assert _true(koss, """
        var req = new Request('https://example.com');
        var c = req.clone();
        c.url === req.url;
        """)

    def test_request_with_abort_signal(self, koss: KossJS):
        assert _true(koss, """
        var ac = new AbortController();
        var req = new Request('https://example.com', { signal: ac.signal });
        req.signal === ac.signal;
        """)

    def test_fetch_returns_promise(self, koss: KossJS):
        assert _true(koss, """
        var p = fetch('https://example.com');
        p instanceof Promise;
        """)


class TestWebApiPerformance:
    """performance global."""

    def test_performance_global(self, koss: KossJS):
        assert koss.eval("typeof performance") == "object"

    def test_performance_now_monotonic(self, koss: KossJS):
        assert _true(koss, """
        var t0 = performance.now();
        var t1 = performance.now();
        typeof t0 === 'number' && t1 >= t0;
        """)


class TestWebApiTimers:
    """Global timers must actually be callable (regression guard)."""

    def test_settimeout_callable(self, koss: KossJS):
        assert _true(koss, """
        var id = globalThis.setTimeout(function() {}, 10);
        typeof id === 'number';
        """)

    def test_setinterval_callable(self, koss: KossJS):
        assert _true(koss, """
        var id = globalThis.setInterval(function() {}, 10);
        typeof id === 'number';
        """)

    def test_setimmediate_callable(self, koss: KossJS):
        assert _true(koss, """
        var id = globalThis.setImmediate(function() {});
        typeof id === 'number';
        """)

    def test_cleartimeout_callable(self, koss: KossJS):
        assert _true(koss, """
        var id = globalThis.setTimeout(function() {}, 1000);
        globalThis.clearTimeout(id);
        true;
        """)


class TestWebApiEventTarget:
    """EventTarget base class and event subclasses."""

    def test_event_target_global(self, koss: KossJS):
        assert koss.eval("typeof EventTarget") == "function"

    def test_event_target_dispatch(self, koss: KossJS):
        assert _true(koss, """
        var et = new EventTarget();
        var fired = 0;
        et.addEventListener('go', function(e) { fired++; });
        et.dispatchEvent(new Event('go'));
        fired === 1;
        """)

    def test_event_target_once(self, koss: KossJS):
        assert _true(koss, """
        var et = new EventTarget();
        var fired = 0;
        et.addEventListener('x', function() { fired++; }, { once: true });
        et.dispatchEvent(new Event('x'));
        et.dispatchEvent(new Event('x'));
        fired === 1;
        """)

    def test_message_event(self, koss: KossJS):
        assert _true(koss, """
        var e = new MessageEvent('message', { data: { x: 1 }, origin: 'o' });
        e.type === 'message' && e.data.x === 1 && e.origin === 'o';
        """)

    def test_error_event(self, koss: KossJS):
        assert _true(koss, """
        var e = new ErrorEvent('error', { message: 'boom', lineno: 5 });
        e.message === 'boom' && e.lineno === 5;
        """)

    def test_close_event(self, koss: KossJS):
        assert _true(koss, """
        var e = new CloseEvent('close', { wasClean: true, code: 1000 });
        e.wasClean === true && e.code === 1000;
        """)


class TestWebApiAbortAny:
    """AbortSignal.any."""

    def test_abort_signal_any(self, koss: KossJS):
        assert _true(koss, """
        var a = new AbortController();
        var b = new AbortController();
        var s = AbortSignal.any([a.signal, b.signal]);
        var aborted = false;
        s.addEventListener('abort', function() { aborted = true; });
        b.abort('from-b');
        aborted === true && s.aborted === true;
        """)


class TestWebApiCrypto:
    """Web Crypto global."""

    def test_crypto_global(self, koss: KossJS):
        assert koss.eval("typeof crypto") == "object"

    def test_get_random_values(self, koss: KossJS):
        assert _true(koss, """
        var arr = new Uint8Array(16);
        crypto.getRandomValues(arr);
        arr.length === 16;
        """)

    def test_random_uuid(self, koss: KossJS):
        assert _true(koss, """
        var u = crypto.randomUUID();
        typeof u === 'string' && u.length === 36;
        """)

    def test_subtle_digest_promise(self, koss: KossJS):
        assert _true(koss, """
        crypto.subtle.digest('SHA-256', new Uint8Array([1,2,3])) instanceof Promise;
        """)


class TestWebApiNavigator:
    """navigator global."""

    def test_navigator_global(self, koss: KossJS):
        assert koss.eval("typeof navigator") == "object"

    def test_navigator_user_agent(self, koss: KossJS):
        assert _true(koss, """
        typeof navigator.userAgent === 'string' && navigator.userAgent.indexOf('KossJS') !== -1;
        """)

    def test_navigator_hardware_concurrency(self, koss: KossJS):
        assert _true(koss, """
        typeof navigator.hardwareConcurrency === 'number' && navigator.hardwareConcurrency >= 1;
        """)


class TestWebApiMessaging:
    """MessageChannel / BroadcastChannel."""

    def test_message_channel(self, koss: KossJS):
        assert _true(koss, """
        var ch = new MessageChannel();
        var got = null;
        ch.port1.onmessage = function(e) { got = e.data; };
        ch.port2.postMessage('hi');
        typeof MessageChannel === 'function' && got === null;
        """)

    def test_broadcast_channel(self, koss: KossJS):
        assert _true(koss, """
        typeof BroadcastChannel === 'function';
        """)


class TestWebApiStorage:
    """localStorage / sessionStorage."""

    def test_local_storage(self, koss: KossJS):
        assert _true(koss, """
        localStorage.setItem('k', 'v');
        var ok = localStorage.getItem('k') === 'v';
        localStorage.removeItem('k');
        ok && localStorage.getItem('k') === null;
        """)

    def test_session_storage(self, koss: KossJS):
        assert _true(koss, """
        sessionStorage.setItem('a', '1');
        sessionStorage.setItem('b', '2');
        var ok = sessionStorage.length === 2;
        sessionStorage.clear();
        ok && sessionStorage.length === 0;
        """)


class TestWebApiUrlPattern:
    """URLPattern."""

    def test_url_pattern_global(self, koss: KossJS):
        assert koss.eval("typeof URLPattern") == "function"

    def test_url_pattern_test(self, koss: KossJS):
        assert _true(koss, """
        var p = new URLPattern('/users/:id');
        p.test('https://example.com/users/123') && !p.test('https://example.com/other');
        """)

    def test_url_pattern_groups(self, koss: KossJS):
        assert _true(koss, """
        var p = new URLPattern('/posts/:slug');
        var m = p.exec('https://example.com/posts/hello');
        m !== null && m.groups.slug === 'hello';
        """)


class TestWebApiStreams:
    """Web Streams (ReadableStream / WritableStream / TransformStream)."""

    def test_readable_stream_global(self, koss: KossJS):
        assert koss.eval("typeof ReadableStream") == "function"

    def test_writable_stream_global(self, koss: KossJS):
        assert koss.eval("typeof WritableStream") == "function"

    def test_transform_stream_global(self, koss: KossJS):
        assert koss.eval("typeof TransformStream") == "function"

    def test_queuing_strategy(self, koss: KossJS):
        assert _true(koss, """
        var c = new CountQueuingStrategy({ highWaterMark: 3 });
        var b = new ByteLengthQueuingStrategy({ highWaterMark: 1024 });
        c.highWaterMark === 3 && c.size() === 1 && b.highWaterMark === 1024;
        """)

    def test_text_encoder_stream(self, koss: KossJS):
        assert _true(koss, """
        var tes = new TextEncoderStream();
        tes.readable instanceof ReadableStream && tes.writable instanceof WritableStream;
        """)

    def test_compression_stream(self, koss: KossJS):
        assert _true(koss, """
        var cs = new CompressionStream('gzip');
        cs.readable instanceof ReadableStream && cs.writable instanceof WritableStream;
        """)
