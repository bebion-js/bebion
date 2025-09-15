/**
 * Bebion JavaScript Runtime Frontend
 * 
 * This module provides the JavaScript layer that interfaces with the native
 * Rust/C engine for developer tooling, REPL functionality, and runtime APIs.
 */

class BebionRuntime {
    constructor() {
        this.version = '0.1.0';
        this.engine = null;
        this.modules = new Map();
        this.globals = new Map();
        
        this.initializeGlobals();
        this.setupErrorHandling();
    }

    /**
     * Initialize global objects and functions
     */
    initializeGlobals() {
        // Console object
        global.console = {
            log: (...args) => this.nativeCall('console.log', args),
            error: (...args) => this.nativeCall('console.error', args),
            warn: (...args) => this.nativeCall('console.warn', args),
            info: (...args) => this.nativeCall('console.info', args),
            debug: (...args) => this.nativeCall('console.debug', args),
            trace: (...args) => this.nativeCall('console.trace', args),
            clear: () => this.nativeCall('console.clear', []),
            time: (label) => this.nativeCall('console.time', [label]),
            timeEnd: (label) => this.nativeCall('console.timeEnd', [label]),
            assert: (condition, ...args) => {
                if (!condition) {
                    this.nativeCall('console.error', ['Assertion failed:', ...args]);
                }
            },
            count: (label = 'default') => this.nativeCall('console.count', [label]),
            countReset: (label = 'default') => this.nativeCall('console.countReset', [label]),
            group: (...args) => this.nativeCall('console.group', args),
            groupCollapsed: (...args) => this.nativeCall('console.groupCollapsed', args),
            groupEnd: () => this.nativeCall('console.groupEnd', []),
            table: (data) => this.nativeCall('console.table', [data]),
            dir: (obj) => this.nativeCall('console.dir', [obj]),
            dirxml: (obj) => this.nativeCall('console.dirxml', [obj])
        };

        // Process object
        global.process = {
            version: this.version,
            versions: {
                bebion: this.version,
                v8: 'N/A',
                node: 'N/A'
            },
            platform: this.nativeCall('process.platform', []),
            arch: this.nativeCall('process.arch', []),
            pid: this.nativeCall('process.pid', []),
            argv: this.nativeCall('process.argv', []),
            env: new Proxy({}, {
                get: (target, prop) => this.nativeCall('process.getEnv', [prop]),
                set: (target, prop, value) => {
                    this.nativeCall('process.setEnv', [prop, value]);
                    return true;
                },
                has: (target, prop) => this.nativeCall('process.hasEnv', [prop]),
                ownKeys: () => this.nativeCall('process.envKeys', []),
                getOwnPropertyDescriptor: (target, prop) => ({
                    enumerable: true,
                    configurable: true,
                    value: this.nativeCall('process.getEnv', [prop])
                })
            }),
            cwd: () => this.nativeCall('process.cwd', []),
            chdir: (dir) => this.nativeCall('process.chdir', [dir]),
            exit: (code = 0) => this.nativeCall('process.exit', [code]),
            nextTick: (callback, ...args) => {
                this.nativeCall('process.nextTick', [callback, ...args]);
            },
            hrtime: (time) => this.nativeCall('process.hrtime', [time]),
            uptime: () => this.nativeCall('process.uptime', []),
            memoryUsage: () => this.nativeCall('process.memoryUsage', [])
        };

        // Global functions
        global.setTimeout = (callback, delay, ...args) => {
            return this.nativeCall('timers.setTimeout', [callback, delay, ...args]);
        };

        global.clearTimeout = (id) => {
            return this.nativeCall('timers.clearTimeout', [id]);
        };

        global.setInterval = (callback, delay, ...args) => {
            return this.nativeCall('timers.setInterval', [callback, delay, ...args]);
        };

        global.clearInterval = (id) => {
            return this.nativeCall('timers.clearInterval', [id]);
        };

        global.setImmediate = (callback, ...args) => {
            return this.nativeCall('timers.setImmediate', [callback, ...args]);
        };

        global.clearImmediate = (id) => {
            return this.nativeCall('timers.clearImmediate', [id]);
        };

        // URL and URLSearchParams
        global.URL = class URL {
            constructor(url, base) {
                const parsed = this.nativeCall('url.parse', [url, base]);
                Object.assign(this, parsed);
            }

            toString() {
                return this.href;
            }

            toJSON() {
                return this.href;
            }
        };

        global.URLSearchParams = class URLSearchParams {
            constructor(init) {
                this._params = new Map();
                if (typeof init === 'string') {
                    this._parseString(init);
                } else if (init instanceof URLSearchParams) {
                    this._params = new Map(init._params);
                } else if (Array.isArray(init)) {
                    for (const [key, value] of init) {
                        this.append(key, value);
                    }
                } else if (init && typeof init === 'object') {
                    for (const [key, value] of Object.entries(init)) {
                        this.append(key, value);
                    }
                }
            }

            append(name, value) {
                const key = String(name);
                const val = String(value);
                if (this._params.has(key)) {
                    this._params.get(key).push(val);
                } else {
                    this._params.set(key, [val]);
                }
            }

            delete(name) {
                this._params.delete(String(name));
            }

            get(name) {
                const values = this._params.get(String(name));
                return values ? values[0] : null;
            }

            getAll(name) {
                return this._params.get(String(name)) || [];
            }

            has(name) {
                return this._params.has(String(name));
            }

            set(name, value) {
                this._params.set(String(name), [String(value)]);
            }

            sort() {
                const sorted = new Map([...this._params.entries()].sort());
                this._params = sorted;
            }

            toString() {
                const parts = [];
                for (const [key, values] of this._params) {
                    for (const value of values) {
                        parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(value)}`);
                    }
                }
                return parts.join('&');
            }

            *entries() {
                for (const [key, values] of this._params) {
                    for (const value of values) {
                        yield [key, value];
                    }
                }
            }

            *keys() {
                for (const [key] of this.entries()) {
                    yield key;
                }
            }

            *values() {
                for (const [, value] of this.entries()) {
                    yield value;
                }
            }

            [Symbol.iterator]() {
                return this.entries();
            }

            _parseString(str) {
                if (str.startsWith('?')) {
                    str = str.slice(1);
                }
                for (const pair of str.split('&')) {
                    const [key, value = ''] = pair.split('=');
                    this.append(
                        decodeURIComponent(key),
                        decodeURIComponent(value)
                    );
                }
            }
        };

        // TextEncoder and TextDecoder
        global.TextEncoder = class TextEncoder {
            constructor() {
                this.encoding = 'utf-8';
            }

            encode(input = '') {
                return this.nativeCall('text.encode', [input]);
            }
        };

        global.TextDecoder = class TextDecoder {
            constructor(encoding = 'utf-8', options = {}) {
                this.encoding = encoding;
                this.fatal = options.fatal || false;
                this.ignoreBOM = options.ignoreBOM || false;
            }

            decode(input, options = {}) {
                return this.nativeCall('text.decode', [input, this.encoding, options]);
            }
        };

        // AbortController and AbortSignal
        global.AbortController = class AbortController {
            constructor() {
                this.signal = new AbortSignal();
            }

            abort(reason) {
                this.signal._abort(reason);
            }
        };

        global.AbortSignal = class AbortSignal extends EventTarget {
            constructor() {
                super();
                this.aborted = false;
                this.reason = undefined;
            }

            _abort(reason) {
                if (this.aborted) return;
                this.aborted = true;
                this.reason = reason;
                this.dispatchEvent(new Event('abort'));
            }

            throwIfAborted() {
                if (this.aborted) {
                    throw this.reason || new DOMException('The operation was aborted', 'AbortError');
                }
            }

            static abort(reason) {
                const signal = new AbortSignal();
                signal._abort(reason);
                return signal;
            }

            static timeout(delay) {
                const controller = new AbortController();
                setTimeout(() => {
                    controller.abort(new DOMException('The operation timed out', 'TimeoutError'));
                }, delay);
                return controller.signal;
            }
        };
    }

    /**
     * Setup global error handling
     */
    setupErrorHandling() {
        global.addEventListener = global.addEventListener || function(type, listener) {
            if (type === 'error') {
                process.on('uncaughtException', listener);
            } else if (type === 'unhandledrejection') {
                process.on('unhandledRejection', listener);
            }
        };

        global.removeEventListener = global.removeEventListener || function(type, listener) {
            if (type === 'error') {
                process.removeListener('uncaughtException', listener);
            } else if (type === 'unhandledrejection') {
                process.removeListener('unhandledRejection', listener);
            }
        };

        // Handle uncaught exceptions
        process.on('uncaughtException', (error) => {
            console.error('Uncaught Exception:', error);
            process.exit(1);
        });

        // Handle unhandled promise rejections
        process.on('unhandledRejection', (reason, promise) => {
            console.error('Unhandled Promise Rejection:', reason);
            console.error('Promise:', promise);
        });
    }

    /**
     * Call native function through FFI
     */
    nativeCall(functionName, args) {
        return `[Native call: ${functionName}(${args.join(', ')})]`;
    }

    /**
     * Load and initialize a module
     */
    async loadModule(specifier, referrer) {
        if (this.modules.has(specifier)) {
            return this.modules.get(specifier);
        }

        const moduleInfo = await this.nativeCall('module.load', [specifier, referrer]);
        const module = new BebionModule(moduleInfo);
        this.modules.set(specifier, module);
        
        return module;
    }

    /**
     * Create a new execution context
     */
    createContext(options = {}) {
        return new BebionContext(this, options);
    }

    /**
     * Evaluate code in the current context
     */
    evaluate(code, options = {}) {
        return this.nativeCall('runtime.evaluate', [code, options]);
    }

    /**
     * Get runtime statistics
     */
    getStats() {
        return this.nativeCall('runtime.getStats', []);
    }

    /**
     * Force garbage collection
     */
    gc() {
        return this.nativeCall('runtime.gc', []);
    }

    /**
     * Get memory usage information
     */
    memoryUsage() {
        return this.nativeCall('runtime.memoryUsage', []);
    }
}

/**
 * Module wrapper class
 */
class BebionModule {
    constructor(moduleInfo) {
        this.url = moduleInfo.url;
        this.exports = moduleInfo.exports;
        this.loaded = moduleInfo.loaded;
        this.error = moduleInfo.error;
    }

    async import() {
        if (!this.loaded) {
            await this.nativeCall('module.import', [this.url]);
            this.loaded = true;
        }
        return this.exports;
    }

    nativeCall(functionName, args) {
        // Delegate to runtime
        return global.__bebion_runtime.nativeCall(functionName, args);
    }
}

/**
 * Execution context class
 */
class BebionContext {
    constructor(runtime, options) {
        this.runtime = runtime;
        this.options = options;
        this.globals = new Map();
    }

    evaluate(code, options = {}) {
        return this.runtime.nativeCall('context.evaluate', [code, { ...this.options, ...options }]);
    }

    setGlobal(name, value) {
        this.globals.set(name, value);
        return this.runtime.nativeCall('context.setGlobal', [name, value]);
    }

    getGlobal(name) {
        return this.runtime.nativeCall('context.getGlobal', [name]);
    }
}

/**
 * REPL utilities
 */
class BebionREPL {
    constructor(runtime) {
        this.runtime = runtime;
        this.history = [];
        this.context = runtime.createContext();
    }

    async evaluate(input) {
        try {
            this.history.push(input);
            const result = await this.context.evaluate(input);
            return { success: true, result };
        } catch (error) {
            return { success: false, error };
        }
    }

    getHistory() {
        return [...this.history];
    }

    clearHistory() {
        this.history = [];
    }

    getCompletions(line, cursor) {
        return this.runtime.nativeCall('repl.getCompletions', [line, cursor]);
    }
}

/**
 * Developer tools
 */
class BebionDevTools {
    constructor(runtime) {
        this.runtime = runtime;
    }

    inspect(object, options = {}) {
        return this.runtime.nativeCall('devtools.inspect', [object, options]);
    }

    profile(name) {
        return this.runtime.nativeCall('devtools.profile', [name]);
    }

    profileEnd(name) {
        return this.runtime.nativeCall('devtools.profileEnd', [name]);
    }

    heapSnapshot() {
        return this.runtime.nativeCall('devtools.heapSnapshot', []);
    }

    debugger() {
        return this.runtime.nativeCall('devtools.debugger', []);
    }
}

/**
 * Performance measurement utilities
 */
class BebionPerformance {
    constructor() {
        this.marks = new Map();
        this.measures = new Map();
    }

    now() {
        return this.nativeCall('performance.now', []);
    }

    mark(name) {
        const time = this.now();
        this.marks.set(name, time);
        return time;
    }

    measure(name, startMark, endMark) {
        const startTime = this.marks.get(startMark) || 0;
        const endTime = endMark ? this.marks.get(endMark) : this.now();
        const duration = endTime - startTime;
        
        this.measures.set(name, {
            name,
            startTime,
            duration,
            entryType: 'measure'
        });
        
        return duration;
    }

    clearMarks(name) {
        if (name) {
            this.marks.delete(name);
        } else {
            this.marks.clear();
        }
    }

    clearMeasures(name) {
        if (name) {
            this.measures.delete(name);
        } else {
            this.measures.clear();
        }
    }

    getEntries() {
        return [
            ...Array.from(this.marks.entries()).map(([name, time]) => ({
                name,
                startTime: time,
                entryType: 'mark'
            })),
            ...Array.from(this.measures.values())
        ];
    }

    getEntriesByName(name) {
        return this.getEntries().filter(entry => entry.name === name);
    }

    getEntriesByType(type) {
        return this.getEntries().filter(entry => entry.entryType === type);
    }

    nativeCall(functionName, args) {
        return global.__bebion_runtime.nativeCall(functionName, args);
    }
}

// Initialize the runtime
const runtime = new BebionRuntime();
const repl = new BebionREPL(runtime);
const devtools = new BebionDevTools(runtime);
const performance = new BebionPerformance();

// Export globals
global.__bebion_runtime = runtime;
global.__bebion_repl = repl;
global.__bebion_devtools = devtools;
global.performance = performance;

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        BebionRuntime,
        BebionModule,
        BebionContext,
        BebionREPL,
        BebionDevTools,
        BebionPerformance,
        runtime,
        repl,
        devtools,
        performance
    };
}

// AMD support
if (typeof define === 'function' && define.amd) {
    define([], () => ({
        BebionRuntime,
        BebionModule,
        BebionContext,
        BebionREPL,
        BebionDevTools,
        BebionPerformance,
        runtime,
        repl,
        devtools,
        performance
    }));
}

console.log('Bebion JavaScript Runtime initialized');
