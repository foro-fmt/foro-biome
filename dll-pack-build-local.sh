rustup target add ${DLL_PACK_TARGET}

cargo build --profile super-release --target ${DLL_PACK_TARGET}

mkdir ./artifacts/

dll-pack-builder local $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name') \
    $(dll-pack-builder find ${BUILD_OUT_DIR}) \
    ./artifacts/ ${DLL_PACK_TARGET} ${GITHUB_REPOSITORY} ${GITHUB_REF#refs/tags/}
