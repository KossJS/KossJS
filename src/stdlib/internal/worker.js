'use strict';

const EventEmitter = require('events');

const isMainThread = true;
const isInternalThread = false;
const threadId = 0;
const SHARE_ENV = Symbol('nodejs.worker_threads.SHARE_ENV');

const workers = new Map();
let nextWorkerId = 1;

class Worker extends EventEmitter {
    constructor(filename, options = {}) {
        super();

        const workerId = nextWorkerId++;
        this._workerId = workerId;
        this._filename = filename;
        this._options = options;
        this._running = false;
        this._exited = false;

        const poolSize = Math.max(1, __koss_create_worker_pool ? 1 : 0);
        if (typeof __koss_create_worker_pool !== 'function') {
            throw new Error('Worker threads not supported in this build');
        }

        const result = __koss_create_worker_pool(poolSize);
        workers.set(workerId, this);

        if (typeof filename === 'string') {
            const code = `const { parentPort } = require('worker_threads');\n` +
                `require(${JSON.stringify(filename)});`;
            const execResult = __koss_worker_execute(0, code);
            if (execResult) {
                this._running = true;
            }
        }

        process.nextTick(() => {
            this.emit('online');
        });
    }

    postMessage(data) {
        if (!this._running) return;
        const json = JSON.stringify(data);
        __koss_worker_post_message(0, json);
    }

    terminate() {
        if (this._exited) return Promise.resolve();
        this._running = false;
        this._exited = true;
        __koss_worker_terminate(0);
        __koss_worker_shutdown();
        workers.delete(this._workerId);
        this.emit('exit', 0);
        return Promise.resolve(0);
    }

    get threadId() {
        return this._workerId;
    }

    ref() {}
    unref() {}
}

function receiveMessageOnPort() {
    return undefined;
}

function getEnvironmentData(key) {
    return undefined;
}

function setEnvironmentData(key, value) {
}

module.exports = {
    isMainThread,
    isInternalThread,
    SHARE_ENV,
    Worker,
    threadId,
    threadName: 'main',
    receiveMessageOnPort,
    getEnvironmentData,
    setEnvironmentData,
    resourceLimits: {},
    ownsProcessState: true,
    markAsUntransferable: () => {},
    isMarkedAsUntransferable: () => false,
    moveMessagePortToContext: () => {},
    BroadcastChannel: class {},
    MessagePort: class extends EventEmitter {
        postMessage(data) {}
        start() {}
        close() {}
        ref() {}
        unref() {}
    },
    MessageChannel: class {
        constructor() {
            const { port1, port2 } = createMessageChannel();
            this.port1 = port1;
            this.port2 = port2;
        }
    },
    markAsUncloneable: () => {},
};

function createMessageChannel() {
    const port1 = new (module.exports.MessagePort)();
    const port2 = new (module.exports.MessagePort)();
    const messages1 = [];
    const messages2 = [];

    port1.postMessage = function(data) {
        messages2.push(data);
        port2.emit('message', data);
    };
    port2.postMessage = function(data) {
        messages1.push(data);
        port1.emit('message', data);
    };

    return { port1, port2 };
}
