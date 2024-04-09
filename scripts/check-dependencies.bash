#!/bin/bash

set -e

SED=sed
if [ -z "$(sed --version 2>&1 | grep GNU)" ]; then
    SED=gsed
fi

CURL="curl -sL --fail-with-body"
if ! CARGO_OUTPUT=$(cargo tree -p soroban-env-host 2>&1); then
  echo "The project depends on multiple versions of the soroban-env-host Rust library, please unify them."
  echo "Make sure the soroban-sdk dependency indirectly points to the same soroban-env-host dependency imported explicitly."
  echo
  echo "This is soroban-env-host version imported by soroban-sdk:"
  cargo tree --depth 1  -p soroban-sdk | grep env-host
  echo
  echo
  echo
  echo "Full error:"
  echo $CARGO_OUTPUT
  exit 1
fi

# revision of the https://github.com/hcnet/rs-hcnet-xdr library used by the Rust code
RS_HCNET_XDR_REVISION=""

# revision of https://github.com/hcnet/hcnet-xdr/ used by the Rust code
HCNET_XDR_REVISION_FROM_RUST=""

function hcnet_xdr_version_from_rust_dep_tree {
  LINE=$(grep hcnet-xdr | head -n 1)
  # try to obtain a commit
  COMMIT=$(echo $LINE | $SED -n 's/.*rev=\(.*\)#.*/\1/p')
  if [ -n "$COMMIT" ]; then
    echo "$COMMIT"
    return
  fi
  # obtain a crate version
  echo $LINE | $SED -n  's/.*hcnet-xdr \(v\)\{0,1\}\([^ ]*\).*/\2/p'
}

if CARGO_OUTPUT=$(cargo tree --depth 0 -p hcnet-xdr 2>&1); then
  RS_HCNET_XDR_REVISION=$(echo "$CARGO_OUTPUT" | hcnet_xdr_version_from_rust_dep_tree)
  if [ ${#RS_HCNET_XDR_REVISION} -eq 40 ]; then
    # revision is a git hash
    HCNET_XDR_REVISION_FROM_RUST=$($CURL https://raw.githubusercontent.com/hcnet/rs-hcnet-xdr/${RS_HCNET_XDR_REVISION}/xdr/curr-version)
  else
    # revision is a crate version
    CARGO_SRC_BASE_DIR=$(realpath ${CARGO_HOME:-$HOME/.cargo}/registry/src/index*)
    HCNET_XDR_REVISION_FROM_RUST=$(cat "${CARGO_SRC_BASE_DIR}/hcnet-xdr-${RS_HCNET_XDR_REVISION}/xdr/curr-version")
  fi
else
  echo "The project depends on multiple versions of the Rust rs-hcnet-xdr library"
  echo "Make sure a single version of hcnet-xdr is used"
  echo
  echo
  echo
  echo "Full error:"
  echo $CARGO_OUTPUT
fi
