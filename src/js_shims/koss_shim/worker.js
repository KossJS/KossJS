// Copyright (C) 2026 TT23XR Studio
// 
// This file is licensed under GNU Affero General Public License v3.0
// with the TT23XR Studio Additional Permission:
// "非本软件模块的源代码公开义务例外"

// koss:worker — Koss 原生工作线程模块
// 基于 __koss_worker_* 全局绑定

var __koss_create_worker_pool = globalThis.__koss_create_worker_pool;
var __koss_worker_post_message = globalThis.__koss_worker_post_message;
var __koss_worker_execute = globalThis.__koss_worker_execute;
var __koss_worker_try_recv = globalThis.__koss_worker_try_recv;
var __koss_worker_terminate = globalThis.__koss_worker_terminate;
var __koss_worker_shutdown = globalThis.__koss_worker_shutdown;

function createPool(size) {
  if (typeof __koss_create_worker_pool !== 'function') {
    throw new Error('Worker pool not available');
  }
  var result = __koss_create_worker_pool(Number(size) || 4);
  var poolInfo;
  try { poolInfo = typeof result === 'string' ? JSON.parse(result) : result; }
  catch (e) { poolInfo = {}; }

  return {
    execute: function(code) {
      if (typeof __koss_worker_execute !== 'function') {
        throw new Error('Worker execute not available');
      }
      var cmdResult = __koss_worker_execute(0, String(code));
      return Promise.resolve(cmdResult);
    },
    post: function(data) {
      if (typeof __koss_worker_post_message !== 'function') {
        throw new Error('Worker post not available');
      }
      __koss_worker_post_message(0, typeof data === 'string' ? data : JSON.stringify(data));
    },
    receive: function() {
      if (typeof __koss_worker_try_recv !== 'function') return null;
      var result = __koss_worker_try_recv();
      if (!result) return null;
      try { return JSON.parse(result); }
      catch (e) { return result; }
    },
    terminate: function() {
      if (typeof __koss_worker_terminate === 'function') {
        __koss_worker_terminate(0);
      }
    },
    shutdown: function() {
      if (typeof __koss_worker_shutdown === 'function') {
        __koss_worker_shutdown();
      }
    },
  };
}

function post(data) {
  if (typeof __koss_worker_post_message !== 'function') {
    throw new Error('Worker post not available');
  }
  __koss_worker_post_message(0, typeof data === 'string' ? data : JSON.stringify(data));
}

function receive() {
  if (typeof __koss_worker_try_recv !== 'function') return null;
  var result = __koss_worker_try_recv();
  if (!result) return null;
  try { return JSON.parse(result); }
  catch (e) { return result; }
}

function terminate() {
  if (typeof __koss_worker_terminate === 'function') {
    __koss_worker_terminate(0);
  }
}

module.exports = {
  createPool: createPool, post: post, receive: receive, terminate: terminate,
};
