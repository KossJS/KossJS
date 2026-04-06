/**
 * This file is from Node.js official source code.
 * Source: https://github.com/nodejs/node
 * 
 * Modified for KossJS (Boa engine) compatibility:
 * - Removed internalBinding() calls that require Node.js C++ bindings
 * - Adapted to work with KossJS runtime
 */

'use strict';

const {
  isInternalThread,
  isMainThread,
  SHARE_ENV,
  resourceLimits,
  setEnvironmentData,
  getEnvironmentData,
  threadId,
  threadName,
  Worker,
} = require('internal/worker');

const {
  MessagePort,
  MessageChannel,
  markAsUncloneable,
  moveMessagePortToContext,
  receiveMessageOnPort,
  BroadcastChannel,
} = require('internal/worker/io');

const {
  postMessageToThread,
} = require('internal/worker/messaging');

const {
  markAsUntransferable,
  isMarkedAsUntransferable,
} = require('internal/buffer');

const { locks } = require('internal/locks');

module.exports = {
  isInternalThread,
  isMainThread,
  MessagePort,
  MessageChannel,
  markAsUncloneable,
  markAsUntransferable,
  isMarkedAsUntransferable,
  moveMessagePortToContext,
  receiveMessageOnPort,
  resourceLimits,
  postMessageToThread,
  threadId,
  threadName,
  SHARE_ENV,
  Worker,
  parentPort: null,
  workerData: null,
  BroadcastChannel,
  setEnvironmentData,
  getEnvironmentData,
  locks,
};

