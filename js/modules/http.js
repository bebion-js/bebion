/**
 * HTTP Module for Bebion
 * 
 * Provides Node.js-compatible HTTP client and server functionality
 */

import { EventTarget } from '../runtime/events.js';
import { Readable, Writable } from '../runtime/streams.js';

// HTTP status codes
const STATUS_CODES = {
    100: 'Continue',
    101: 'Switching Protocols',
    102: 'Processing',
    103: 'Early Hints',
    200: 'OK',
    201: 'Created',
    202: 'Accepted',
    203: 'Non-Authoritative Information',
    204: 'No Content',
    205: 'Reset Content',
    206: 'Partial Content',
    207: 'Multi-Status',
    208: 'Already Reported',
    226: 'IM Used',
    300: 'Multiple Choices',
    301: 'Moved Permanently',
    302: 'Found',
    303: 'See Other',
    304: 'Not Modified',
    305: 'Use Proxy',
    307: 'Temporary Redirect',
    308: 'Permanent Redirect',
    400: 'Bad Request',
    401: 'Unauthorized',
    402: 'Payment Required',
    403: 'Forbidden',
    404: 'Not Found',
    405: 'Method Not Allowed',
    406: 'Not Acceptable',
    407: 'Proxy Authentication Required',
    408: 'Request Timeout',
    409: 'Conflict',
    410: 'Gone',
    411: 'Length Required',
    412: 'Precondition Failed',
    413: 'Payload Too Large',
    414: 'URI Too Long',
    415: 'Unsupported Media Type',
    416: 'Range Not Satisfiable',
    417: 'Expectation Failed',
    418: "I'm a Teapot",
    421: 'Misdirected Request',
    422: 'Unprocessable Entity',
    423: 'Locked',
    424: 'Failed Dependency',
    425: 'Too Early',
    426: 'Upgrade Required',
    428: 'Precondition Required',
    429: 'Too Many Requests',
    431: 'Request Header Fields Too Large',
    451: 'Unavailable For Legal Reasons',
    500: 'Internal Server Error',
    501: 'Not Implemented',
    502: 'Bad Gateway',
    503: 'Service Unavailable',
    504: 'Gateway Timeout',
    505: 'HTTP Version Not Supported',
    506: 'Variant Also Negotiates',
    507: 'Insufficient Storage',
    508: 'Loop Detected',
    509: 'Bandwidth Limit Exceeded',
    510: 'Not Extended',
    511: 'Network Authentication Required'
};

// HTTP methods
const METHODS = [
    'ACL', 'BIND', 'CHECKOUT', 'CONNECT', 'COPY', 'DELETE',
    'GET', 'HEAD', 'LINK', 'LOCK', 'M-SEARCH', 'MERGE',
    'MKACTIVITY', 'MKCALENDAR', 'MKCOL', 'MOVE', 'NOTIFY',
    'OPTIONS', 'PATCH', 'POST', 'PROPFIND', 'PROPPATCH',
    'PURGE', 'PUT', 'REBIND', 'REPORT', 'SEARCH', 'SOURCE',
    'SUBSCRIBE', 'TRACE', 'UNBIND', 'UNLINK', 'UNLOCK',
    'UNSUBSCRIBE'
];

// IncomingMessage class
class IncomingMessage extends Readable {
    constructor(socket) {
        super();
        this.socket = socket;
        this.httpVersion = '1.1';
        this.httpVersionMajor = 1;
        this.httpVersionMinor = 1;
        this.headers = {};
        this.rawHeaders = [];
        this.trailers = {};
        this.rawTrailers = [];
        this.method = null;
        this.url = null;
        this.statusCode = null;
        this.statusMessage = null;
        this.complete = false;
        this.aborted = false;
        this.upgrade = false;
    }

    setTimeout(msecs, callback) {
        if (callback) {
            this.addEventListener('timeout', callback, { once: true });
        }
        
        if (this.socket) {
            this.socket.setTimeout(msecs, () => {
                this.dispatchEvent(new CustomEvent('timeout'));
            });
        }
        
        return this;
    }

    destroy(error) {
        if (this.destroyed) return this;
        
        this.destroyed = true;
        this.readable = false;
        
        if (this.socket) {
            this.socket.destroy(error);
        }
        
        if (error) {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        }
        
        this.dispatchEvent(new CustomEvent('close'));
        return this;
    }
}

// ServerResponse class
class ServerResponse extends Writable {
    constructor(req) {
        super();
        this.req = req;
        this.socket = req.socket;
        this.statusCode = 200;
        this.statusMessage = STATUS_CODES[200];
        this.headersSent = false;
        this.finished = false;
        this.sendDate = true;
        this._headers = {};
        this._headerNames = {};
        this._removedHeaders = {};
    }

    writeHead(statusCode, statusMessage, headers) {
        if (typeof statusMessage === 'object') {
            headers = statusMessage;
            statusMessage = STATUS_CODES[statusCode];
        }
        
        this.statusCode = statusCode;
        this.statusMessage = statusMessage || STATUS_CODES[statusCode];
        
        if (headers) {
            for (const [key, value] of Object.entries(headers)) {
                this.setHeader(key, value);
            }
        }
        
        this._writeHead();
        return this;
    }

    _writeHead() {
        if (this.headersSent) return;
        
        this.headersSent = true;
        
        // Add default headers
        if (this.sendDate && !this.getHeader('date')) {
            this.setHeader('date', new Date().toUTCString());
        }
        
        if (!this.getHeader('connection')) {
            this.setHeader('connection', 'close');
        }
        
        // Write status line and headers
        const statusLine = `HTTP/1.1 ${this.statusCode} ${this.statusMessage}\r\n`;
        let headerLines = '';
        
        for (const [name, value] of Object.entries(this._headers)) {
            if (Array.isArray(value)) {
                for (const v of value) {
                    headerLines += `${name}: ${v}\r\n`;
                }
            } else {
                headerLines += `${name}: ${value}\r\n`;
            }
        }
        
        const response = statusLine + headerLines + '\r\n';
        this.socket.write(response);
    }

    setHeader(name, value) {
        if (this.headersSent) {
            throw new Error('Cannot set headers after they are sent to the client');
        }
        
        const key = name.toLowerCase();
        this._headerNames[key] = name;
        this._headers[name] = value;
        delete this._removedHeaders[key];
    }

    getHeader(name) {
        const key = name.toLowerCase();
        const headerName = this._headerNames[key];
        return headerName ? this._headers[headerName] : undefined;
    }

    getHeaders() {
        return { ...this._headers };
    }

    getHeaderNames() {
        return Object.keys(this._headers);
    }

    hasHeader(name) {
        return this.getHeader(name) !== undefined;
    }

    removeHeader(name) {
        if (this.headersSent) {
            throw new Error('Cannot remove headers after they are sent to the client');
        }
        
        const key = name.toLowerCase();
        const headerName = this._headerNames[key];
        
        if (headerName) {
            delete this._headers[headerName];
            delete this._headerNames[key];
            this._removedHeaders[key] = true;
        }
    }

    write(chunk, encoding, callback) {
        if (!this.headersSent) {
            this._writeHead();
        }
        
        return super.write(chunk, encoding, callback);
    }

    end(chunk, encoding, callback) {
        if (!this.headersSent) {
            this._writeHead();
        }
        
        if (chunk !== null && chunk !== undefined) {
            this.write(chunk, encoding);
        }
        
        this.finished = true;
        
        if (callback) {
            this.addEventListener('finish', callback, { once: true });
        }
        
        this.socket.end();
        this.dispatchEvent(new CustomEvent('finish'));
        
        return this;
    }

    setTimeout(msecs, callback) {
        if (callback) {
            this.addEventListener('timeout', callback, { once: true });
        }
        
        if (this.socket) {
            this.socket.setTimeout(msecs, () => {
                this.dispatchEvent(new CustomEvent('timeout'));
            });
        }
        
        return this;
    }

    addTrailers(headers) {
        0.24
    }

    cork() {
        if (this.socket && this.socket.cork) {
            this.socket.cork();
        }
    }

    uncork() {
        if (this.socket && this.socket.uncork) {
            this.socket.uncork();
        }
    }
}

// ClientRequest class
class ClientRequest extends Writable {
    constructor(options, callback) {
        super();
        
        this.method = options.method || 'GET';
        this.path = options.path || '/';
        this.host = options.host || options.hostname || 'localhost';
        this.port = options.port || (options.protocol === 'https:' ? 443 : 80);
        this.headers = options.headers || {};
        this.timeout = options.timeout || 0;
        this.agent = options.agent;
        this.auth = options.auth;
        this.protocol = options.protocol || 'http:';
        
        this.aborted = false;
        this.finished = false;
        this.socket = null;
        this.response = null;
        
        if (callback) {
            this.addEventListener('response', callback, { once: true });
        }
        
        this._connect();
    }

    async _connect() {
        try {
            this.socket = await __bebion_runtime.nativeCall('http.connect', [{
                host: this.host,
                port: this.port,
                protocol: this.protocol
            }]);
            
            this.socket.addEventListener('connect', () => {
                this.dispatchEvent(new CustomEvent('connect'));
                this._sendRequest();
            });
            
            this.socket.addEventListener('data', (event) => {
                this._handleResponse(event.detail);
            });
            
            this.socket.addEventListener('error', (event) => {
                this.dispatchEvent(new CustomEvent('error', { detail: event.detail }));
            });
            
            this.socket.addEventListener('close', () => {
                this.dispatchEvent(new CustomEvent('close'));
            });
            
        } catch (error) {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        }
    }

    _sendRequest() {
        // Build request line
        const requestLine = `${this.method} ${this.path} HTTP/1.1\r\n`;
        
        // Build headers
        const headers = { ...this.headers };
        if (!headers.host) {
            headers.host = this.port === 80 || this.port === 443 ? 
                this.host : `${this.host}:${this.port}`;
        }
        
        let headerLines = '';
        for (const [name, value] of Object.entries(headers)) {
            if (Array.isArray(value)) {
                for (const v of value) {
                    headerLines += `${name}: ${v}\r\n`;
                }
            } else {
                headerLines += `${name}: ${value}\r\n`;
            }
        }
        
        const request = requestLine + headerLines + '\r\n';
        this.socket.write(request);
    }

    _handleResponse(data) {
        if (!this.response) {
            // Parse response headers
            const responseText = data.toString();
            const [headerSection, ...bodyParts] = responseText.split('\r\n\r\n');
            const lines = headerSection.split('\r\n');
            const statusLine = lines[0];
            const [, statusCode, statusMessage] = statusLine.match(/HTTP\/1\.1 (\d+) (.+)/) || [];
            
            this.response = new IncomingMessage(this.socket);
            this.response.statusCode = parseInt(statusCode);
            this.response.statusMessage = statusMessage;
            
            // Parse headers
            for (let i = 1; i < lines.length; i++) {
                const [name, ...valueParts] = lines[i].split(': ');
                const value = valueParts.join(': ');
                const key = name.toLowerCase();
                
                if (this.response.headers[key]) {
                    if (Array.isArray(this.response.headers[key])) {
                        this.response.headers[key].push(value);
                    } else {
                        this.response.headers[key] = [this.response.headers[key], value];
                    }
                } else {
                    this.response.headers[key] = value;
                }
                
                this.response.rawHeaders.push(name, value);
            }
            
            this.dispatchEvent(new CustomEvent('response', { detail: this.response }));
            
            // Handle body if present
            if (bodyParts.length > 0) {
                const body = bodyParts.join('\r\n\r\n');
                if (body) {
                    this.response._push(body);
                }
            }
        } else {
            // Additional body data
            this.response._push(data);
        }
    }

    write(chunk, encoding, callback) {
        if (this.finished) {
            throw new Error('Cannot write after end');
        }
        
        if (this.socket) {
            return this.socket.write(chunk, encoding, callback);
        }
        
        return false;
    }

    end(chunk, encoding, callback) {
        if (this.finished) return this;
        
        this.finished = true;
        
        if (chunk !== null && chunk !== undefined) {
            this.write(chunk, encoding);
        }
        
        if (callback) {
            this.addEventListener('finish', callback, { once: true });
        }
        
        this.dispatchEvent(new CustomEvent('finish'));
        return this;
    }

    abort() {
        if (this.aborted) return;
        
        this.aborted = true;
        
        if (this.socket) {
            this.socket.destroy();
        }
        
        this.dispatchEvent(new CustomEvent('abort'));
    }

    setTimeout(timeout, callback) {
        if (callback) {
            this.addEventListener('timeout', callback, { once: true });
        }
        
        if (this.socket) {
            this.socket.setTimeout(timeout, () => {
                this.dispatchEvent(new CustomEvent('timeout'));
            });
        }
        
        return this;
    }

    setNoDelay(noDelay = true) {
        if (this.socket && this.socket.setNoDelay) {
            this.socket.setNoDelay(noDelay);
        }
        return this;
    }

    setSocketKeepAlive(enable = false, initialDelay = 0) {
        if (this.socket && this.socket.setKeepAlive) {
            this.socket.setKeepAlive(enable, initialDelay);
        }
        return this;
    }
}

// Server class
class Server extends EventTarget {
    constructor(options, requestListener) {
        super();
        
        if (typeof options === 'function') {
            requestListener = options;
            options = {};
        }
        
        this.options = options || {};
        this.listening = false;
        this.maxHeadersCount = null;
        this.headersTimeout = 60000;
        this.requestTimeout = 0;
        this.timeout = 120000;
        this.keepAliveTimeout = 5000;
        this._handle = null;
        this._connections = new Set();
        
        if (requestListener) {
            this.addEventListener('request', requestListener);
        }
    }

    listen(port, hostname, backlog, callback) {
        if (typeof port === 'function') {
            callback = port;
            port = 0;
            hostname = undefined;
            backlog = undefined;
        } else if (typeof hostname === 'function') {
            callback = hostname;
            hostname = undefined;
            backlog = undefined;
        } else if (typeof backlog === 'function') {
            callback = backlog;
            backlog = undefined;
        }
        
        if (callback) {
            this.addEventListener('listening', callback, { once: true });
        }
        
        this._listen(port, hostname, backlog);
        return this;
    }

    async _listen(port = 0, hostname = '0.0.0.0', backlog = 511) {
        try {
            this._handle = await __bebion_runtime.nativeCall('http.createServer', [{
                port,
                hostname,
                backlog,
                options: this.options
            }]);
            
            this.listening = true;
            
            this._handle.addEventListener('connection', (event) => {
                this._handleConnection(event.detail);
            });
            
            this._handle.addEventListener('error', (event) => {
                this.dispatchEvent(new CustomEvent('error', { detail: event.detail }));
            });
            
            this.dispatchEvent(new CustomEvent('listening'));
            
        } catch (error) {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        }
    }

    _handleConnection(socket) {
        this._connections.add(socket);
        
        socket.addEventListener('close', () => {
            this._connections.delete(socket);
        });
        
        socket.addEventListener('data', (event) => {
            this._handleRequest(socket, event.detail);
        });
        
        this.dispatchEvent(new CustomEvent('connection', { detail: socket }));
    }

    _handleRequest(socket, data) {
        try {
            // Parse HTTP request
            const requestText = data.toString();
            const [headerSection, ...bodyParts] = requestText.split('\r\n\r\n');
            const lines = headerSection.split('\r\n');
            const requestLine = lines[0];
            const [method, url, version] = requestLine.split(' ');
            
            const req = new IncomingMessage(socket);
            req.method = method;
            req.url = url;
            req.httpVersion = version.replace('HTTP/', '');
            
            // Parse headers
            for (let i = 1; i < lines.length; i++) {
                const [name, ...valueParts] = lines[i].split(': ');
                const value = valueParts.join(': ');
                const key = name.toLowerCase();
                
                if (req.headers[key]) {
                    if (Array.isArray(req.headers[key])) {
                        req.headers[key].push(value);
                    } else {
                        req.headers[key] = [req.headers[key], value];
                    }
                } else {
                    req.headers[key] = value;
                }
                
                req.rawHeaders.push(name, value);
            }
            
            // Handle body if present
            if (bodyParts.length > 0) {
                const body = bodyParts.join('\r\n\r\n');
                if (body) {
                    req._push(body);
                }
            }
            
            const res = new ServerResponse(req);
            
            this.dispatchEvent(new CustomEvent('request', { 
                detail: { request: req, response: res } 
            }));
            
        } catch (error) {
            this.dispatchEvent(new CustomEvent('clientError', { 
                detail: { error, socket } 
            }));
        }
    }

    close(callback) {
        if (callback) {
            this.addEventListener('close', callback, { once: true });
        }
        
        this.listening = false;
        
        // Close all connections
        for (const connection of this._connections) {
            connection.destroy();
        }
        this._connections.clear();
        
        if (this._handle) {
            this._handle.close();
            this._handle = null;
        }
        
        this.dispatchEvent(new CustomEvent('close'));
        return this;
    }

    address() {
        if (!this.listening || !this._handle) {
            return null;
        }
        
        return this._handle.address();
    }

    getConnections(callback) {
        const count = this._connections.size;
        if (callback) {
            process.nextTick(() => callback(null, count));
        }
        return count;
    }

    setTimeout(msecs, callback) {
        this.timeout = msecs;
        
        if (callback) {
            this.addEventListener('timeout', callback);
        }
        
        return this;
    }
}

// Agent class
class Agent {
    constructor(options = {}) {
        this.options = options;
        this.requests = {};
        this.sockets = {};
        this.freeSockets = {};
        this.keepAliveMsecs = options.keepAliveMsecs || 1000;
        this.keepAlive = options.keepAlive || false;
        this.maxSockets = options.maxSockets || Infinity;
        this.maxFreeSockets = options.maxFreeSockets || 256;
        this.scheduling = options.scheduling || 'lifo';
        this.timeout = options.timeout || 0;
    }

    createConnection(options, callback) {
        return __bebion_runtime.nativeCall('http.createConnection', [options, callback]);
    }

    keepSocketAlive(socket) {
        socket.setKeepAlive(this.keepAlive, this.keepAliveMsecs);
        socket.unref();
        return true;
    }

    reuseSocket(socket, request) {
        socket.ref();
    }

    destroy() {
        for (const sockets of Object.values(this.sockets)) {
            for (const socket of sockets) {
                socket.destroy();
            }
        }
        
        for (const sockets of Object.values(this.freeSockets)) {
            for (const socket of sockets) {
                socket.destroy();
            }
        }
        
        this.sockets = {};
        this.freeSockets = {};
    }
}

// Global agent
const globalAgent = new Agent();

// HTTP module functions
const http = {
    STATUS_CODES,
    METHODS,
    
    IncomingMessage,
    ServerResponse,
    ClientRequest,
    Server,
    Agent,
    
    globalAgent,
    
    createServer(options, requestListener) {
        return new Server(options, requestListener);
    },

    request(options, callback) {
        if (typeof options === 'string') {
            options = new URL(options);
        }
        
        return new ClientRequest(options, callback);
    },

    get(options, callback) {
        const req = http.request(options, callback);
        req.end();
        return req;
    }
};

// Add convenience methods for each HTTP method
for (const method of METHODS) {
    const methodName = method.toLowerCase();
    if (methodName !== 'get') {
        http[methodName] = function(options, callback) {
            if (typeof options === 'string') {
                options = { ...new URL(options), method };
            } else {
                options = { ...options, method };
            }
            return http.request(options, callback);
        };
    }
}

export default http;
export {
    STATUS_CODES,
    METHODS,
    IncomingMessage,
    ServerResponse,
    ClientRequest,
    Server,
    Agent,
    globalAgent
};
