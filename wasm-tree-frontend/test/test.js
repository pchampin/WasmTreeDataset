'use strict'

var rdf = require('@graphy/core.data.factory')

let wasmexport = require('../index.js')


let TreeDataset = wasmexport.TreeDataset;
let TreeStore   = wasmexport.TreeStore;

require('./WrappedDatasetCore')(rdf, TreeDataset)
require('./WrappedDatasetCore')(rdf, wasmexport.AlwaysForestDataset)
require('./WrappedDataset'    )(rdf, TreeDataset)
require('./Store'             )(rdf, TreeStore)
