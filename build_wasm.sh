#!/usr/bin/env bash

set -e

HELP_STRING=$(cat <<- END
	usage: build_wasm.sh PROJECT_NAME [--release]

	Build script for combining a Macroquad project with wasm-bindgen,
	allowing integration with the greater wasm-ecosystem.

	example: build_wasm.sh flappy-bird

	This'll go through the following steps:

	    1. Build as target 'wasm32-unknown-unknown'
	    2. Create the directory 'web/wbindgen' if it doesn't already exist
	    3. Run wasm-bindgen with output into the wbindgen directory
	    4. Apply patches to the output js file (detailed here: https://github.com/not-fl3/macroquad/issues/212#issuecomment-835276147)

	Required arguments:

	    PROJECT_NAME            The name of the artifact/target/project

	Arguments:

	    --release               Build in release mode


	Author: Tom Solberg <me@sbg.dev>
	Version: 0.1
END
)


die () {
    echo >&2 "usage: build_wasm.sh PROJECT_NAME [--release]"
    echo >&2 "Error: $@"
    echo >&2
    exit 1
}


# Storage
RELEASE=no
POSITIONAL=()

# Parse primary commands
while [[ $# -gt 0 ]]
do
    key="$1"
    case $key in
        --release)
            RELEASE=yes
            shift
            ;;

        -h|--help)
            echo "$HELP_STRING"
            exit 0
            ;;

        *)
            POSITIONAL+=("$1")
            shift
            ;;
    esac
done


# Restore positionals
set -- "${POSITIONAL[@]}"
[ $# -ne 1 ] && die "too many arguments provided"

PROJECT_NAME=$1

EXTRA_ARGS=""
if [ "$RELEASE" == "yes" ]; then
    EXTRA_ARGS=" --release"
fi

# Build
cargo build --target wasm32-unknown-unknown $EXTRA_ARGS

# Generate bindgen outputs
mkdir -p web/wbindgen
wasm-bindgen --target web --out-dir web/wbindgen/ target/wasm32-unknown-unknown/release/$PROJECT_NAME.wasm

# Optimize for size
wasm-opt -Os -o web/wbindgen/main_bg.wasm web/wbindgen/main_bg.wasm

# Shims to tie it all together
if type "gsed" > /dev/null; then
    gsed -i "s/import \* as __wbg_star0 from 'env';//" ./web/wbindgen/$PROJECT_NAME.js
    gsed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" ./web/wbindgen/$PROJECT_NAME.js
    gsed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" ./web/wbindgen/$PROJECT_NAME.js
else
    sed -i "s/import \* as __wbg_star0 from 'env';//" ./web/wbindgen/$PROJECT_NAME.js
    sed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" ./web/wbindgen/$PROJECT_NAME.js
    sed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" ./web/wbindgen/$PROJECT_NAME.js
fi
