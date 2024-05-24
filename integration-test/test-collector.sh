#!/usr/bin/env bash
set -e

/usr/lib/libdlt-examples/dlt-example-user -n 5 -l 3 'fooo' 
/modality workspace sync-indices
/conform spec eval --file /dlt-example-user.speqtr --dry-run
