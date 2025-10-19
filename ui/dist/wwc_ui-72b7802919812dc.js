let wasm;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_1.set(idx, obj);
    return idx;
}

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

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
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
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
    if (builtInMatches && builtInMatches.length > 1) {
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

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(
state => {
    wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b);
}
);

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
                wasm.__wbindgen_export_5.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

export function start() {
    wasm.start();
}

function __wbg_adapter_6(arg0, arg1, arg2) {
    wasm.closure308_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_9(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__hcd85fb05d8146f8d(arg0, arg1, arg2);
}

function __wbg_adapter_12(arg0, arg1, arg2) {
    wasm.closure182_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_232(arg0, arg1, arg2, arg3, arg4) {
    wasm.closure336_externref_shim(arg0, arg1, arg2, arg3, arg4);
}

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_activeElement_da57789542a03158 = function(arg0) {
        const ret = arg0.activeElement;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_addEventListener_775911544ac9d643 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_appendChild_87a6cc0aeb132c06 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.appendChild(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_13410aac570ffff7 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_cancelAnimationFrame_a3b3c0d5b5c0056d = function() { return handleError(function (arg0, arg1) {
        arg0.cancelAnimationFrame(arg1);
    }, arguments) };
    imports.wbg.__wbg_childNodes_5c44c2ec67a90732 = function(arg0) {
        const ret = arg0.childNodes;
        return ret;
    };
    imports.wbg.__wbg_createElementNS_ffbb8bb20b2a7e4c = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createElement_4909dfa2011f2abe = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createTextNode_c71a51271fadf515 = function(arg0, arg1, arg2) {
        const ret = arg0.createTextNode(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_decodeURIComponent_98b0ee74f5078237 = function() { return handleError(function (arg0, arg1) {
        const ret = decodeURIComponent(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_document_7d29d139bd619045 = function(arg0) {
        const ret = arg0.document;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_error_99981e16d476aa5c = function(arg0) {
        console.error(arg0);
    };
    imports.wbg.__wbg_fetch_a9bc66c159c18e19 = function(arg0) {
        const ret = fetch(arg0);
        return ret;
    };
    imports.wbg.__wbg_firstChild_28e9f6aeb33680c1 = function(arg0) {
        const ret = arg0.firstChild;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_focus_8541343802c6721b = function() { return handleError(function (arg0) {
        arg0.focus();
    }, arguments) };
    imports.wbg.__wbg_forEach_d77328dbdd4919da = function(arg0, arg1, arg2) {
        try {
            var state0 = {a: arg1, b: arg2};
            var cb0 = (arg0, arg1, arg2) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_232(a, state0.b, arg0, arg1, arg2);
                } finally {
                    state0.a = a;
                }
            };
            arg0.forEach(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_from_88bc52ce20ba6318 = function(arg0) {
        const ret = Array.from(arg0);
        return ret;
    };
    imports.wbg.__wbg_getAttributeNames_4978b6608b12126c = function(arg0) {
        const ret = arg0.getAttributeNames();
        return ret;
    };
    imports.wbg.__wbg_getAttribute_8bfaf67e99ed2ee3 = function(arg0, arg1, arg2, arg3) {
        const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getElementsByClassName_6464261a3dd4b6be = function(arg0, arg1, arg2) {
        const ret = arg0.getElementsByClassName(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_getRandomValues_1c61fac11405ffdc = function() { return handleError(function (arg0, arg1) {
        globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_get_0da715ceaecea5c8 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_get_2a8fe8079145daf7 = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_hash_0ce7fe010ac2cc6b = function(arg0, arg1) {
        const ret = arg1.hash;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_href_65a798194bf5ead5 = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.href;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_insertBefore_30228206e8f1d3fb = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.insertBefore(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_Element_162e4334c7d6f450 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Element;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Error_76149ae9b431750e = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Error;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlButtonElement_6b05c06f6c294949 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLButtonElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlDataElement_bff36c3367883115 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLDataElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlElement_d60c51c41eb8699a = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlInputElement_486d1ca974d99353 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLInputElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlLiElement_1843a96ee3ce6c7c = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLLIElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlMenuItemElement_bc73aae903535ad5 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLMenuItemElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlMeterElement_1a3013dae01ffcc5 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLMeterElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlOptionElement_de363587d8e61d6d = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLOptionElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlOutputElement_98dba00e69db25e4 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLOutputElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlParamElement_2d032e54fd6efef5 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLParamElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlProgressElement_2478016abd2a4c4e = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLProgressElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlSelectElement_432da62d310182dc = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLSelectElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlTextAreaElement_a3d2c45f48c6ca27 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLTextAreaElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Node_17373399b6647f60 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Node;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Response_50fde2cd696850bf = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Response;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_12d20d558ef92592 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_is_8346b6c36feaf71a = function(arg0, arg1) {
        const ret = Object.is(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_item_39d3c4efb0a47b2a = function(arg0, arg1) {
        const ret = arg0.item(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_length_186546c51cd61acd = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_e7f4a6e30ea139e7 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_location_92d89c32ae076cab = function(arg0) {
        const ret = arg0.location;
        return ret;
    };
    imports.wbg.__wbg_log_6c7b5f4f00b8ce3f = function(arg0) {
        console.log(arg0);
    };
    imports.wbg.__wbg_message_125a1b2998b3552a = function(arg0) {
        const ret = arg0.message;
        return ret;
    };
    imports.wbg.__wbg_name_f733db82b3c2804d = function(arg0) {
        const ret = arg0.name;
        return ret;
    };
    imports.wbg.__wbg_namespaceURI_020a81e6d28c2c96 = function(arg0, arg1) {
        const ret = arg1.namespaceURI;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_new_19c25a3f2fa63a02 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_95e31b8bc5de31d6 = function() { return handleError(function (arg0, arg1) {
        const ret = new URL(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_e8b27dfd3785875f = function() { return handleError(function () {
        const ret = new URLSearchParams();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_f6e53210afea8e45 = function() { return handleError(function () {
        const ret = new Headers();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_newnoargs_254190557c45b4ec = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithbase_96f007ba18c568ff = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = new URL(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_newwithstr_1bc70be98f2e7425 = function() { return handleError(function (arg0, arg1) {
        const ret = new Request(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_newwithstrandinit_b5d168a29a3fd85f = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_nodeType_21b604b4479f9971 = function(arg0) {
        const ret = arg0.nodeType;
        return ret;
    };
    imports.wbg.__wbg_now_886b39d7ec380719 = function(arg0) {
        const ret = arg0.now();
        return ret;
    };
    imports.wbg.__wbg_pathname_e7278f48b2a6a5ad = function(arg0, arg1) {
        const ret = arg1.pathname;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_performance_a221af8decc752fb = function(arg0) {
        const ret = arg0.performance;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_querySelector_438f2df379fbb8d2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_25d0739ac89e8c88 = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queueMicrotask_4488407636f5bf24 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_removeAttribute_cf35412842be6ae4 = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.removeAttribute(getStringFromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_removeChild_1c094e96ff042c2d = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.removeChild(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_removeEventListener_6d5be9c2821a511e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_replaceChild_67a45eaef90078bc = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.replaceChild(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_requestAnimationFrame_ddc84a7def436784 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.requestAnimationFrame(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_resolve_4055c623acdd6a1b = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_searchParams_6858f9baf30712c4 = function(arg0) {
        const ret = arg0.searchParams;
        return ret;
    };
    imports.wbg.__wbg_search_3a02a8f2a1a2e604 = function(arg0, arg1) {
        const ret = arg1.search;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_selectionEnd_3c5a5c4a33d7820f = function() { return handleError(function (arg0) {
        const ret = arg0.selectionEnd;
        return isLikeNone(ret) ? 0x100000001 : (ret) >>> 0;
    }, arguments) };
    imports.wbg.__wbg_selectionStart_3fe43bc260421248 = function() { return handleError(function (arg0) {
        const ret = arg0.selectionStart;
        return isLikeNone(ret) ? 0x100000001 : (ret) >>> 0;
    }, arguments) };
    imports.wbg.__wbg_setAttribute_d1baf9023ad5696f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_set_1c17f9738fac2718 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.set(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setbody_c8460bdf44147df8 = function(arg0, arg1) {
        arg0.body = arg1;
    };
    imports.wbg.__wbg_setchecked_0c6fdca893f4a16f = function(arg0, arg1) {
        arg0.checked = arg1 !== 0;
    };
    imports.wbg.__wbg_setchecked_bb42dd9524e206ad = function(arg0, arg1) {
        arg0.checked = arg1 !== 0;
    };
    imports.wbg.__wbg_setheaders_0052283e2f3503d1 = function(arg0, arg1) {
        arg0.headers = arg1;
    };
    imports.wbg.__wbg_setmethod_9b504d5b855b329c = function(arg0, arg1, arg2) {
        arg0.method = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setsearch_fbee2174e0389ccd = function(arg0, arg1, arg2) {
        arg0.search = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setselectionEnd_ed6b88fb7041a824 = function() { return handleError(function (arg0, arg1) {
        arg0.selectionEnd = arg1 === 0x100000001 ? undefined : arg1;
    }, arguments) };
    imports.wbg.__wbg_setselectionStart_ca6530317ad8a142 = function() { return handleError(function (arg0, arg1) {
        arg0.selectionStart = arg1 === 0x100000001 ? undefined : arg1;
    }, arguments) };
    imports.wbg.__wbg_settextContent_b55fe2f5f1399466 = function(arg0, arg1, arg2) {
        arg0.textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_06e05f2cb17fefdd = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_08cd81b36689dbbe = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_137aa525297803e3 = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_28ca8aa7721ceafe = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_2ceac6e0fd0d0a10 = function(arg0, arg1) {
        arg0.value = arg1;
    };
    imports.wbg.__wbg_setvalue_4c91a711c0108335 = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_5f87742808097b0c = function(arg0, arg1) {
        arg0.value = arg1;
    };
    imports.wbg.__wbg_setvalue_61f519eade1982cb = function(arg0, arg1) {
        arg0.value = arg1;
    };
    imports.wbg.__wbg_setvalue_62c6a57a4254987d = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_6d331c95a4eed136 = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_6ec3fdad4c91f1b5 = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_8921f820c2ce3f12 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_f0a4409105898184 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_995b214ae681ff99 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_cde3890479c675ea = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_tagName_0052175ec2444f12 = function(arg0, arg1) {
        const ret = arg1.tagName;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_target_f2c963b447be6283 = function(arg0) {
        const ret = arg0.target;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_textContent_4e2b2a6c46694642 = function(arg0, arg1) {
        const ret = arg1.textContent;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_text_0f69a215637b9b34 = function() { return handleError(function (arg0) {
        const ret = arg0.text();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_then_b33a773d723afa3e = function(arg0, arg1, arg2) {
        const ret = arg0.then(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_then_e22500defe16819f = function(arg0, arg1) {
        const ret = arg0.then(arg1);
        return ret;
    };
    imports.wbg.__wbg_toString_78df35411a4fd40c = function(arg0) {
        const ret = arg0.toString();
        return ret;
    };
    imports.wbg.__wbg_toString_d8f537919ef401d6 = function(arg0) {
        const ret = arg0.toString();
        return ret;
    };
    imports.wbg.__wbg_type_1ecf41b6baddf209 = function(arg0, arg1) {
        const ret = arg1.type;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_url_79bd91c4e84e8270 = function(arg0, arg1) {
        const ret = arg1.url;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_36a41bd0f6ab3503 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_3dfa084001621c68 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_57637cc189f7a639 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_5c9d2d554a121087 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_6631ad9b28dce85d = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_value_89377c01d73b6102 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_a31e90d20449709c = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_value_a89e049ebc0415f8 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_ad69822795eaa6f7 = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_value_d74fa2e5e985b2ea = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_fdf54c7557edc2e8 = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_wbindgencbdrop_eb10308566512b88 = function(arg0) {
        const obj = arg0.original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        return ret;
    };
    imports.wbg.__wbg_wbindgendebugstring_99ef257a3ddda34d = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_wbindgenisfunction_8cee7dce3725ae74 = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbg_wbindgenisundefined_c4b71d073b92f3c5 = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbg_wbindgenstringget_0f16a6ddddef376f = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_wbindgenthrow_451ec1a8469d7eb6 = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_45acad8ea9821b86 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 307, function: Function { arguments: [Externref], shim_idx: 308, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, 307, __wbg_adapter_6);
        return ret;
    };
    imports.wbg.__wbindgen_cast_868295ae7e2b1078 = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 181, function: Function { arguments: [NamedExternref("Event")], shim_idx: 182, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, 181, __wbg_adapter_12);
        return ret;
    };
    imports.wbg.__wbindgen_cast_cf808be294507a7a = function(arg0, arg1) {
        // Cast intrinsic for `Closure(Closure { dtor_idx: 141, function: Function { arguments: [F64], shim_idx: 86, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
        const ret = makeMutClosure(arg0, arg1, 141, __wbg_adapter_9);
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_1;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('wwc_ui_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
