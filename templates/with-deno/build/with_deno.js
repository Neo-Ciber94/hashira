

const cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function getObject(idx) { return heap[idx]; }

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
};

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

let cachedFloat64Memory0 = null;

function getFloat64Memory0() {
    if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
        cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64Memory0;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_38(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h23792575836ed4f9(arg0, arg1, addHeapObject(arg2));
}

let stack_pointer = 128;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}
function __wbg_adapter_41(arg0, arg1, arg2) {
    try {
        wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h912e2e3b2801df7f(arg0, arg1, addBorrowedObject(arg2));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

/**
*/
export function entry() {
    wasm.entry();
}

/**
* Handle the given request and returns a response.
* @param {Request} web_req
* @returns {Promise<Response>}
*/
export function handler(web_req) {
    const ret = wasm.handler(addHeapObject(web_req));
    return takeObject(ret);
}

/**
* Initializes the environment variables of the application.
* @param {object} envs
*/
export function set_envs(envs) {
    wasm.set_envs(addHeapObject(envs));
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayJsValueFromWasm0(ptr, len) {
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(takeObject(slice[i]));
    }
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
function __wbg_adapter_265(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__he4762d4d16f224ab(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

/**
*/
export class IntoUnderlyingByteSource {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr);
    }
    /**
    * @returns {string}
    */
    get type() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.intounderlyingbytesource_type(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * @returns {number}
    */
    get autoAllocateChunkSize() {
        const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {any} controller
    */
    start(controller) {
        wasm.intounderlyingbytesource_start(this.ptr, addHeapObject(controller));
    }
    /**
    * @param {any} controller
    * @returns {Promise<any>}
    */
    pull(controller) {
        const ret = wasm.intounderlyingbytesource_pull(this.ptr, addHeapObject(controller));
        return takeObject(ret);
    }
    /**
    */
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingbytesource_cancel(ptr);
    }
}
/**
*/
export class IntoUnderlyingSink {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr);
    }
    /**
    * @param {any} chunk
    * @returns {Promise<any>}
    */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.ptr, addHeapObject(chunk));
        return takeObject(ret);
    }
    /**
    * @returns {Promise<any>}
    */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return takeObject(ret);
    }
    /**
    * @param {any} reason
    * @returns {Promise<any>}
    */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, addHeapObject(reason));
        return takeObject(ret);
    }
}
/**
*/
export class IntoUnderlyingSource {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr);
    }
    /**
    * @param {any} controller
    * @returns {Promise<any>}
    */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.ptr, addHeapObject(controller));
        return takeObject(ret);
    }
    /**
    */
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
}
/**
* Raw options for [`pipeTo()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo).
*/
export class PipeOptions {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pipeoptions_free(ptr);
    }
    /**
    * @returns {boolean}
    */
    get preventClose() {
        const ret = wasm.pipeoptions_preventClose(this.ptr);
        return ret !== 0;
    }
    /**
    * @returns {boolean}
    */
    get preventCancel() {
        const ret = wasm.pipeoptions_preventCancel(this.ptr);
        return ret !== 0;
    }
    /**
    * @returns {boolean}
    */
    get preventAbort() {
        const ret = wasm.pipeoptions_preventAbort(this.ptr);
        return ret !== 0;
    }
    /**
    * @returns {AbortSignal | undefined}
    */
    get signal() {
        const ret = wasm.pipeoptions_signal(this.ptr);
        return takeObject(ret);
    }
}
/**
*/
export class QueuingStrategy {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_queuingstrategy_free(ptr);
    }
    /**
    * @returns {number}
    */
    get highWaterMark() {
        const ret = wasm.queuingstrategy_highWaterMark(this.ptr);
        return ret;
    }
}
/**
* Raw options for [`getReader()`](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/getReader).
*/
export class ReadableStreamGetReaderOptions {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_readablestreamgetreaderoptions_free(ptr);
    }
    /**
    * @returns {any}
    */
    get mode() {
        const ret = wasm.readablestreamgetreaderoptions_mode(this.ptr);
        return takeObject(ret);
    }
}

const imports = {
    __wbindgen_placeholder__: {
        __wbindgen_error_new: function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbindgen_string_get: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_String_88810dfeb4021902: function(arg0, arg1) {
            const ret = String(getObject(arg1));
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbindgen_number_get: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        },
        __wbindgen_boolean_get: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
            return ret;
        },
        __wbindgen_number_new: function(arg0) {
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_string_new: function(arg0, arg1) {
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_jsval_loose_eq: function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        },
        __wbindgen_is_object: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbg_set_841ac57cff3d672b: function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        },
        __wbindgen_is_undefined: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg_instanceof_ReadableStream_723f1212419028dc: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ReadableStream;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_getReader_8ecba87d8003e950: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getReader();
            return addHeapObject(ret);
        }, arguments) },
        __wbg_close_e9110ca16e2567db: function(arg0) {
            getObject(arg0).close();
        },
        __wbg_enqueue_d71a1a518e21f5c3: function(arg0, arg1) {
            getObject(arg0).enqueue(getObject(arg1));
        },
        __wbg_byobRequest_08c18cee35def1f4: function(arg0) {
            const ret = getObject(arg0).byobRequest;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_close_da7e6fb9d9851e5a: function(arg0) {
            getObject(arg0).close();
        },
        __wbg_view_231340b0dd8a2484: function(arg0) {
            const ret = getObject(arg0).view;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_respond_8fadc5f5c9d95422: function(arg0, arg1) {
            getObject(arg0).respond(arg1 >>> 0);
        },
        __wbg_buffer_4e79326814bdd393: function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        },
        __wbg_byteOffset_b69b0a07afccce19: function(arg0) {
            const ret = getObject(arg0).byteOffset;
            return ret;
        },
        __wbg_byteLength_5299848ed3264181: function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        },
        __wbg_cancel_7f202496da02cd45: function(arg0) {
            const ret = getObject(arg0).cancel();
            return addHeapObject(ret);
        },
        __wbg_releaseLock_9ae075576f54bf0b: function() { return handleError(function (arg0) {
            getObject(arg0).releaseLock();
        }, arguments) },
        __wbg_read_88c96573fc8b3b01: function(arg0) {
            const ret = getObject(arg0).read();
            return addHeapObject(ret);
        },
        __wbg_done_76252d32deca186b: function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        },
        __wbg_value_ff3741eb46856618: function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        },
        __wbg_listenerid_12315eee21527820: function(arg0, arg1) {
            const ret = getObject(arg1).__yew_listener_id;
            getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        },
        __wbg_setlistenerid_3183aae8fa5840fb: function(arg0, arg1) {
            getObject(arg0).__yew_listener_id = arg1 >>> 0;
        },
        __wbg_subtreeid_e348577f7ef777e3: function(arg0, arg1) {
            const ret = getObject(arg1).__yew_subtree_id;
            getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        },
        __wbg_setsubtreeid_d32e6327eef1f7fc: function(arg0, arg1) {
            getObject(arg0).__yew_subtree_id = arg1 >>> 0;
        },
        __wbg_cachekey_b61393159c57fd7b: function(arg0, arg1) {
            const ret = getObject(arg1).__yew_subtree_cache_key;
            getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        },
        __wbg_setcachekey_80183b7cfc421143: function(arg0, arg1) {
            getObject(arg0).__yew_subtree_cache_key = arg1 >>> 0;
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
        __wbg_error_f851667af71bcfc6: function(arg0, arg1) {
            try {
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(arg0, arg1);
            }
        },
        __wbg_new_abda76e883ba8a5f: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_stack_658279fe44541cf6: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbindgen_cb_drop: function(arg0) {
            const obj = takeObject(arg0).original;
            if (obj.cnt-- == 1) {
                obj.a = 0;
                return true;
            }
            const ret = false;
            return ret;
        },
        __wbindgen_is_string: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        },
        __wbg_error_71d6845bf00a930f: function(arg0, arg1) {
            var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
            wasm.__wbindgen_free(arg0, arg1 * 4);
            console.error(...v0);
        },
        __wbg_instanceof_Window_e266f02eee43b570: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_document_950215a728589a2d: function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_createElement_e2a0e21263eb5416: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_createElementNS_0047de728927ea00: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_createTextNode_866e33a51b47f04c: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        },
        __wbg_method_d1ee174c753ca2be: function(arg0, arg1) {
            const ret = getObject(arg1).method;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_url_bd2775644ef804ec: function(arg0, arg1) {
            const ret = getObject(arg1).url;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_headers_ab5251d2727ac41e: function(arg0) {
            const ret = getObject(arg0).headers;
            return addHeapObject(ret);
        },
        __wbg_body_8e4575efee663de3: function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_instanceof_ShadowRoot_7088dfa874f5499c: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ShadowRoot;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_host_33f0224f975dc46a: function(arg0) {
            const ret = getObject(arg0).host;
            return addHeapObject(ret);
        },
        __wbg_value_00fb0fdc46959169: function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_setvalue_f92ff20dd33356a8: function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_bubbles_80a0700df9c59aee: function(arg0) {
            const ret = getObject(arg0).bubbles;
            return ret;
        },
        __wbg_cancelBubble_c9a8182589205d54: function(arg0) {
            const ret = getObject(arg0).cancelBubble;
            return ret;
        },
        __wbg_composedPath_d4428cc409ddd3e6: function(arg0) {
            const ret = getObject(arg0).composedPath();
            return addHeapObject(ret);
        },
        __wbg_setchecked_cbd6f423c4deba69: function(arg0, arg1) {
            getObject(arg0).checked = arg1 !== 0;
        },
        __wbg_value_1f2c9e357d18d3ea: function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_setvalue_a706abe70dab1b65: function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        },
        __wbg_instanceof_HtmlScriptElement_512730abc326cea2: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLScriptElement;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_type_3c8ce1ca417be1e3: function(arg0, arg1) {
            const ret = getObject(arg1).type;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_text_237693ca051a6432: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).text;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        }, arguments) },
        __wbg_instanceof_Element_cb847a3fc7b1b1a4: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_namespaceURI_436d78f0f18e05c1: function(arg0, arg1) {
            const ret = getObject(arg1).namespaceURI;
            var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_tagName_92d4c105959ede9f: function(arg0, arg1) {
            const ret = getObject(arg1).tagName;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_setinnerHTML_76167cda24d9b96b: function(arg0, arg1, arg2) {
            getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
        },
        __wbg_outerHTML_e29ac244117c6543: function(arg0, arg1) {
            const ret = getObject(arg1).outerHTML;
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_children_93bcc921a4904ad4: function(arg0) {
            const ret = getObject(arg0).children;
            return addHeapObject(ret);
        },
        __wbg_removeAttribute_ad7a5bf2eed30373: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments) },
        __wbg_setAttribute_79c9562d32d05e66: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) },
        __wbg_debug_7960d327fd96f71a: function(arg0, arg1, arg2, arg3) {
            console.debug(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_error_fe807da27c4a4ced: function(arg0) {
            console.error(getObject(arg0));
        },
        __wbg_error_fd84ca2a8a977774: function(arg0, arg1, arg2, arg3) {
            console.error(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_info_5566be377f5b52ae: function(arg0, arg1, arg2, arg3) {
            console.info(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_log_7bb108d119bafbc1: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_log_7b690f184ae4519b: function(arg0, arg1, arg2, arg3) {
            console.log(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_warn_48cbddced45e5414: function(arg0, arg1, arg2, arg3) {
            console.warn(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
        },
        __wbg_length_c54fcfc679a5bfbd: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_newwithoptu8array_feefc1fb34eca018: function() { return handleError(function (arg0, arg1) {
            const ret = new Response(arg0 === 0 ? undefined : getArrayU8FromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_addEventListener_cf5b03cd29763277: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
        }, arguments) },
        __wbg_removeEventListener_b25f5db74f767386: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) },
        __wbg_nodeType_659608c921f57978: function(arg0) {
            const ret = getObject(arg0).nodeType;
            return ret;
        },
        __wbg_parentNode_e81e6d5dc2fc35b0: function(arg0) {
            const ret = getObject(arg0).parentNode;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_parentElement_0e8c9afce5cb9d6e: function(arg0) {
            const ret = getObject(arg0).parentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_childNodes_40585508577e98df: function(arg0) {
            const ret = getObject(arg0).childNodes;
            return addHeapObject(ret);
        },
        __wbg_firstChild_70ff6203aed4a148: function(arg0) {
            const ret = getObject(arg0).firstChild;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_nextSibling_653f43ab9380175f: function(arg0) {
            const ret = getObject(arg0).nextSibling;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_setnodeValue_10d5890cd7e3f998: function(arg0, arg1, arg2) {
            getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        },
        __wbg_textContent_dff59ad5e030bb86: function(arg0, arg1) {
            const ret = getObject(arg1).textContent;
            var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbg_appendChild_b8199dc1655c852d: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_cloneNode_4c5e9ec3203eb137: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).cloneNode(arg1 !== 0);
            return addHeapObject(ret);
        }, arguments) },
        __wbg_insertBefore_77a7d032a91abf86: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_removeChild_794db72cbb6f21d3: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).removeChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_instanceof_Text_1a0eedfe37c49263: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Text;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_at_155d8a5ce48004cd: function(arg0, arg1) {
            const ret = getObject(arg0).at(arg1);
            return addHeapObject(ret);
        },
        __wbg_get_27fe3dac1c4d0224: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_from_67ca20fa722467e6: function(arg0) {
            const ret = Array.from(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_isArray_39d28997bf6b96b4: function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        },
        __wbg_length_e498fbc24f9c1d4f: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_instanceof_ArrayBuffer_a69f02ee4c4f5065: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_new_15d3966e9981a196: function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_newnoargs_2b8b6bd7753c76ba: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_call_95d1ea488d03e4e8: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_call_9495de66fdbe016b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_new_f841cc6f2098f4b5: function() {
            const ret = new Map();
            return addHeapObject(ret);
        },
        __wbg_set_388c4c6422704173: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_next_88560ec06a094dea: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments) },
        __wbg_next_b7d530c04fd8b217: function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        },
        __wbg_done_1ebec03bbd919843: function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        },
        __wbg_value_6ac8da5cc5b3efda: function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        },
        __wbg_entries_4e1315b774245952: function(arg0) {
            const ret = Object.entries(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_is_8f1618fe9a4fd388: function(arg0, arg1) {
            const ret = Object.is(getObject(arg0), getObject(arg1));
            return ret;
        },
        __wbg_new_f9876326328f45ed: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_iterator_55f114446221aa5a: function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        },
        __wbg_new_9d3a9ce4282a18a8: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return __wbg_adapter_265(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return addHeapObject(ret);
            } finally {
                state0.a = state0.b = 0;
            }
        },
        __wbg_resolve_fd40f858d9db1a04: function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_catch_44bf25c15946bac0: function(arg0, arg1) {
            const ret = getObject(arg0).catch(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_then_ec5db6d509eb475f: function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_then_f753623316e2873a: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_globalThis_87cbb8506fecf3a9: function() { return handleError(function () {
            const ret = globalThis.globalThis;
            return addHeapObject(ret);
        }, arguments) },
        __wbg_self_e7c1f827057f6584: function() { return handleError(function () {
            const ret = self.self;
            return addHeapObject(ret);
        }, arguments) },
        __wbg_window_a09ec664e14b1b81: function() { return handleError(function () {
            const ret = window.window;
            return addHeapObject(ret);
        }, arguments) },
        __wbg_global_c85a9259e621f3db: function() { return handleError(function () {
            const ret = global.global;
            return addHeapObject(ret);
        }, arguments) },
        __wbg_instanceof_Uint8Array_01cebe79ca606cca: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_new_537b7341ce90bb31: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_newwithbyteoffsetandlength_9fb2f11355ecadf5: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_length_27a2afe8ab42b09f: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_set_17499e8aa4003ebd: function(arg0, arg1, arg2) {
            getObject(arg0).set(getObject(arg1), arg2 >>> 0);
        },
        __wbindgen_is_function: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg_buffer_cf65c07de34b9a08: function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        },
        __wbg_get_baf4855f9a986186: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) },
        __wbg_set_6aa458a4ebdb65cb: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments) },
        __wbindgen_debug_string: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len0;
            getInt32Memory0()[arg0 / 4 + 0] = ptr0;
        },
        __wbindgen_throw: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_rethrow: function(arg0) {
            throw takeObject(arg0);
        },
        __wbindgen_memory: function() {
            const ret = wasm.memory;
            return addHeapObject(ret);
        },
        __wbindgen_closure_wrapper13311: function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 1507, __wbg_adapter_38);
            return addHeapObject(ret);
        },
        __wbindgen_closure_wrapper13507: function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 1523, __wbg_adapter_41);
            return addHeapObject(ret);
        },
    },

};

const wasm_url = new URL('with_deno_bg.wasm', import.meta.url);
let wasmCode = '';
switch (wasm_url.protocol) {
    case 'file:':
    wasmCode = await Deno.readFile(wasm_url);
    break
    case 'https:':
    case 'http:':
    wasmCode = await (await fetch(wasm_url)).arrayBuffer();
    break
    default:
    throw new Error(`Unsupported protocol: ${wasm_url.protocol}`);
}

const wasmInstance = (await WebAssembly.instantiate(wasmCode, imports)).instance;
const wasm = wasmInstance.exports;

wasm.__wbindgen_start();

