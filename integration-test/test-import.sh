#!/usr/bin/env bash
set -ex

/modality-reflector import --ingest-protocol-parent-url ${INGEST_PROTOCOL_PARENT_URL} dlt /foo.dlt

/modality workspace sync-indices
/conform spec eval --file /imported.speqtr --dry-run
