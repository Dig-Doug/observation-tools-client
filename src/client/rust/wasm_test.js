"use strict";

const t = require( "./client_bundler_wasm_bindgen.js");

const fs = require('fs');
const path = require('path');
const assert = require('assert');

const API_HOST = "http://localhost:8080";

console.log(t.create_runs(API_HOST, "idToken", "projectId"));