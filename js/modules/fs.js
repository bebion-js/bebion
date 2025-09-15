/**
 * File System Module for Bebion
 * 
 * Provides Node.js-compatible file system operations
 */

const fs = {
    // Constants
    constants: {
        F_OK: 0,
        R_OK: 4,
        W_OK: 2,
        X_OK: 1,
        
        O_RDONLY: 0,
        O_WRONLY: 1,
        O_RDWR: 2,
        O_CREAT: 64,
        O_EXCL: 128,
        O_TRUNC: 512,
        O_APPEND: 1024,
        
        S_IFMT: 61440,
        S_IFREG: 32768,
        S_IFDIR: 16384,
        S_IFCHR: 8192,
        S_IFBLK: 24576,
        S_IFIFO: 4096,
        S_IFLNK: 40960,
        S_IFSOCK: 49152,
    },

    // Async functions
    async readFile(path, options = {}) {
        const encoding = typeof options === 'string' ? options : options.encoding;
        const flag = options.flag || 'r';
        
        const result = await __bebion_runtime.nativeCall('fs.readFile', [path, { encoding, flag }]);
        return result;
    },

    async writeFile(path, data, options = {}) {
        const encoding = typeof options === 'string' ? options : (options.encoding || 'utf8');
        const flag = options.flag || 'w';
        const mode = options.mode || 0o666;
        
        await __bebion_runtime.nativeCall('fs.writeFile', [path, data, { encoding, flag, mode }]);
    },

    async appendFile(path, data, options = {}) {
        const encoding = typeof options === 'string' ? options : (options.encoding || 'utf8');
        const flag = options.flag || 'a';
        const mode = options.mode || 0o666;
        
        await __bebion_runtime.nativeCall('fs.appendFile', [path, data, { encoding, flag, mode }]);
    },

    async access(path, mode = fs.constants.F_OK) {
        await __bebion_runtime.nativeCall('fs.access', [path, mode]);
    },

    async copyFile(src, dest, mode = 0) {
        await __bebion_runtime.nativeCall('fs.copyFile', [src, dest, mode]);
    },

    async mkdir(path, options = {}) {
        const recursive = options.recursive || false;
        const mode = options.mode || 0o777;
        
        await __bebion_runtime.nativeCall('fs.mkdir', [path, { recursive, mode }]);
    },

    async rmdir(path, options = {}) {
        const recursive = options.recursive || false;
        const force = options.force || false;
        
        await __bebion_runtime.nativeCall('fs.rmdir', [path, { recursive, force }]);
    },

    async readdir(path, options = {}) {
        const encoding = options.encoding || 'utf8';
        const withFileTypes = options.withFileTypes || false;
        
        const result = await __bebion_runtime.nativeCall('fs.readdir', [path, { encoding, withFileTypes }]);
        
        if (withFileTypes) {
            return result.map(entry => new Dirent(entry));
        }
        
        return result;
    },

    async stat(path, options = {}) {
        const bigint = options.bigint || false;
        const result = await __bebion_runtime.nativeCall('fs.stat', [path, { bigint }]);
        return new Stats(result);
    },

    async lstat(path, options = {}) {
        const bigint = options.bigint || false;
        const result = await __bebion_runtime.nativeCall('fs.lstat', [path, { bigint }]);
        return new Stats(result);
    },

    async unlink(path) {
        await __bebion_runtime.nativeCall('fs.unlink', [path]);
    },

    async rename(oldPath, newPath) {
        await __bebion_runtime.nativeCall('fs.rename', [oldPath, newPath]);
    },

    async chmod(path, mode) {
        await __bebion_runtime.nativeCall('fs.chmod', [path, mode]);
    },

    async chown(path, uid, gid) {
        await __bebion_runtime.nativeCall('fs.chown', [path, uid, gid]);
    },

    async link(existingPath, newPath) {
        await __bebion_runtime.nativeCall('fs.link', [existingPath, newPath]);
    },

    async symlink(target, path, type = 'file') {
        await __bebion_runtime.nativeCall('fs.symlink', [target, path, type]);
    },

    async readlink(path, options = {}) {
        const encoding = options.encoding || 'utf8';
        return await __bebion_runtime.nativeCall('fs.readlink', [path, { encoding }]);
    },

    async realpath(path, options = {}) {
        const encoding = options.encoding || 'utf8';
        return await __bebion_runtime.nativeCall('fs.realpath', [path, { encoding }]);
    },

    async truncate(path, len = 0) {
        await __bebion_runtime.nativeCall('fs.truncate', [path, len]);
    },

    async utimes(path, atime, mtime) {
        await __bebion_runtime.nativeCall('fs.utimes', [path, atime, mtime]);
    },

    // Sync functions
    readFileSync(path, options = {}) {
        const encoding = typeof options === 'string' ? options : options.encoding;
        const flag = options.flag || 'r';
        
        return __bebion_runtime.nativeCall('fs.readFileSync', [path, { encoding, flag }]);
    },

    writeFileSync(path, data, options = {}) {
        const encoding = typeof options === 'string' ? options : (options.encoding || 'utf8');
        const flag = options.flag || 'w';
        const mode = options.mode || 0o666;
        
        __bebion_runtime.nativeCall('fs.writeFileSync', [path, data, { encoding, flag, mode }]);
    },

    appendFileSync(path, data, options = {}) {
        const encoding = typeof options === 'string' ? options : (options.encoding || 'utf8');
        const flag = options.flag || 'a';
        const mode = options.mode || 0o666;
        
        __bebion_runtime.nativeCall('fs.appendFileSync', [path, data, { encoding, flag, mode }]);
    },

    accessSync(path, mode = fs.constants.F_OK) {
        __bebion_runtime.nativeCall('fs.accessSync', [path, mode]);
    },

    copyFileSync(src, dest, mode = 0) {
        __bebion_runtime.nativeCall('fs.copyFileSync', [src, dest, mode]);
    },

    mkdirSync(path, options = {}) {
        const recursive = options.recursive || false;
        const mode = options.mode || 0o777;
        
        return __bebion_runtime.nativeCall('fs.mkdirSync', [path, { recursive, mode }]);
    },

    rmdirSync(path, options = {}) {
        const recursive = options.recursive || false;
        const force = options.force || false;
        
        __bebion_runtime.nativeCall('fs.rmdirSync', [path, { recursive, force }]);
    },

    readdirSync(path, options = {}) {
        const encoding = options.encoding || 'utf8';
        const withFileTypes = options.withFileTypes || false;
        
        const result = __bebion_runtime.nativeCall('fs.readdirSync', [path, { encoding, withFileTypes }]);
        
        if (withFileTypes) {
            return result.map(entry => new Dirent(entry));
        }
        
        return result;
    },

    statSync(path, options = {}) {
        const bigint = options.bigint || false;
        const result = __bebion_runtime.nativeCall('fs.statSync', [path, { bigint }]);
        return new Stats(result);
    },

    lstatSync(path, options = {}) {
        const bigint = options.bigint || false;
        const result = __bebion_runtime.nativeCall('fs.lstatSync', [path, { bigint }]);
        return new Stats(result);
    },

    unlinkSync(path) {
        __bebion_runtime.nativeCall('fs.unlinkSync', [path]);
    },

    renameSync(oldPath, newPath) {
        __bebion_runtime.nativeCall('fs.renameSync', [oldPath, newPath]);
    },

    chmodSync(path, mode) {
        __bebion_runtime.nativeCall('fs.chmodSync', [path, mode]);
    },

    chownSync(path, uid, gid) {
        __bebion_runtime.nativeCall('fs.chownSync', [path, uid, gid]);
    },

    linkSync(existingPath, newPath) {
        __bebion_runtime.nativeCall('fs.linkSync', [existingPath, newPath]);
    },

    symlinkSync(target, path, type = 'file') {
        __bebion_runtime.nativeCall('fs.symlinkSync', [target, path, type]);
    },

    readlinkSync(path, options = {}) {
        const encoding = options.encoding || 'utf8';
        return __bebion_runtime.nativeCall('fs.readlinkSync', [path, { encoding }]);
    },

    realpathSync(path, options = {}) {
        const encoding = options.encoding || 'utf8';
        return __bebion_runtime.nativeCall('fs.realpathSync', [path, { encoding }]);
    },

    truncateSync(path, len = 0) {
        __bebion_runtime.nativeCall('fs.truncateSync', [path, len]);
    },

    utimesSync(path, atime, mtime) {
        __bebion_runtime.nativeCall('fs.utimesSync', [path, atime, mtime]);
    },

    existsSync(path) {
        try {
            this.accessSync(path);
            return true;
        } catch {
            return false;
        }
    },

    // Stream functions
    createReadStream(path, options = {}) {
        return new ReadStream(path, options);
    },

    createWriteStream(path, options = {}) {
        return new WriteStream(path, options);
    },

    // Watch functions
    watch(filename, options = {}, listener) {
        if (typeof options === 'function') {
            listener = options;
            options = {};
        }
        
        const watcher = new FSWatcher();
        watcher._watch(filename, options, listener);
        return watcher;
    },

    watchFile(filename, options = {}, listener) {
        if (typeof options === 'function') {
            listener = options;
            options = {};
        }
        
        const interval = options.interval || 5007;
        const persistent = options.persistent !== false;
        
        __bebion_runtime.nativeCall('fs.watchFile', [filename, { interval, persistent }, listener]);
    },

    unwatchFile(filename, listener) {
        __bebion_runtime.nativeCall('fs.unwatchFile', [filename, listener]);
    }
};

// Stats class
class Stats {
    constructor(data) {
        Object.assign(this, data);
    }

    isFile() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFREG;
    }

    isDirectory() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFDIR;
    }

    isBlockDevice() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFBLK;
    }

    isCharacterDevice() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFCHR;
    }

    isSymbolicLink() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFLNK;
    }

    isFIFO() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFIFO;
    }

    isSocket() {
        return (this.mode & fs.constants.S_IFMT) === fs.constants.S_IFSOCK;
    }
}

// Dirent class
class Dirent {
    constructor(data) {
        this.name = data.name;
        this._type = data.type;
    }

    isFile() {
        return this._type === 'file';
    }

    isDirectory() {
        return this._type === 'directory';
    }

    isBlockDevice() {
        return this._type === 'block-device';
    }

    isCharacterDevice() {
        return this._type === 'character-device';
    }

    isSymbolicLink() {
        return this._type === 'symlink';
    }

    isFIFO() {
        return this._type === 'fifo';
    }

    isSocket() {
        return this._type === 'socket';
    }
}

// ReadStream class
class ReadStream extends EventTarget {
    constructor(path, options = {}) {
        super();
        this.path = path;
        this.options = options;
        this.readable = true;
        this.destroyed = false;
        this._handle = null;
        
        this._init();
    }

    _init() {
        this._handle = __bebion_runtime.nativeCall('fs.createReadStream', [this.path, this.options]);
        
        // Set up event forwarding
        this._handle.on('data', (chunk) => {
            this.dispatchEvent(new CustomEvent('data', { detail: chunk }));
        });
        
        this._handle.on('end', () => {
            this.dispatchEvent(new CustomEvent('end'));
        });
        
        this._handle.on('error', (error) => {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        });
        
        this._handle.on('close', () => {
            this.dispatchEvent(new CustomEvent('close'));
        });
    }

    read(size) {
        return this._handle.read(size);
    }

    pause() {
        this._handle.pause();
        return this;
    }

    resume() {
        this._handle.resume();
        return this;
    }

    destroy(error) {
        if (this.destroyed) return this;
        
        this.destroyed = true;
        this.readable = false;
        
        if (this._handle) {
            this._handle.destroy(error);
        }
        
        if (error) {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        }
        
        this.dispatchEvent(new CustomEvent('close'));
        return this;
    }

    close(callback) {
        if (callback) {
            this.addEventListener('close', callback, { once: true });
        }
        this.destroy();
    }
}

// WriteStream class
class WriteStream extends EventTarget {
    constructor(path, options = {}) {
        super();
        this.path = path;
        this.options = options;
        this.writable = true;
        this.destroyed = false;
        this._handle = null;
        
        this._init();
    }

    _init() {
        this._handle = __bebion_runtime.nativeCall('fs.createWriteStream', [this.path, this.options]);
        
        // Set up event forwarding
        this._handle.on('drain', () => {
            this.dispatchEvent(new CustomEvent('drain'));
        });
        
        this._handle.on('finish', () => {
            this.dispatchEvent(new CustomEvent('finish'));
        });
        
        this._handle.on('error', (error) => {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        });
        
        this._handle.on('close', () => {
            this.dispatchEvent(new CustomEvent('close'));
        });
    }

    write(chunk, encoding, callback) {
        if (typeof encoding === 'function') {
            callback = encoding;
            encoding = 'utf8';
        }
        
        const result = this._handle.write(chunk, encoding, callback);
        return result;
    }

    end(chunk, encoding, callback) {
        if (typeof chunk === 'function') {
            callback = chunk;
            chunk = null;
            encoding = 'utf8';
        } else if (typeof encoding === 'function') {
            callback = encoding;
            encoding = 'utf8';
        }
        
        if (callback) {
            this.addEventListener('finish', callback, { once: true });
        }
        
        this._handle.end(chunk, encoding);
    }

    destroy(error) {
        if (this.destroyed) return this;
        
        this.destroyed = true;
        this.writable = false;
        
        if (this._handle) {
            this._handle.destroy(error);
        }
        
        if (error) {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        }
        
        this.dispatchEvent(new CustomEvent('close'));
        return this;
    }

    close(callback) {
        if (callback) {
            this.addEventListener('close', callback, { once: true });
        }
        this.destroy();
    }
}

// FSWatcher class
class FSWatcher extends EventTarget {
    constructor() {
        super();
        this._handle = null;
    }

    _watch(filename, options, listener) {
        if (listener) {
            this.addEventListener('change', listener);
        }
        
        this._handle = __bebion_runtime.nativeCall('fs.watch', [filename, options]);
        
        this._handle.on('change', (eventType, filename) => {
            this.dispatchEvent(new CustomEvent('change', { 
                detail: { eventType, filename } 
            }));
        });
        
        this._handle.on('error', (error) => {
            this.dispatchEvent(new CustomEvent('error', { detail: error }));
        });
    }

    close() {
        if (this._handle) {
            this._handle.close();
            this._handle = null;
        }
    }
}

// Promises API
fs.promises = {
    access: fs.access,
    appendFile: fs.appendFile,
    chmod: fs.chmod,
    chown: fs.chown,
    copyFile: fs.copyFile,
    link: fs.link,
    lstat: fs.lstat,
    mkdir: fs.mkdir,
    readdir: fs.readdir,
    readFile: fs.readFile,
    readlink: fs.readlink,
    realpath: fs.realpath,
    rename: fs.rename,
    rmdir: fs.rmdir,
    stat: fs.stat,
    symlink: fs.symlink,
    truncate: fs.truncate,
    unlink: fs.unlink,
    utimes: fs.utimes,
    writeFile: fs.writeFile
};

export default fs;
export { Stats, Dirent, ReadStream, WriteStream, FSWatcher };