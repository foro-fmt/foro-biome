set -euo pipefail

gh_tag="${DLL_PACK_GH_TAG:-${GITHUB_REF_NAME:-${GITHUB_REF:-}}}"
gh_tag="${gh_tag#refs/tags/}"
if [ -z "${gh_tag}" ]; then
    gh_tag="${GITHUB_SHA:-local}"
fi

rustup target add "${DLL_PACK_TARGET}"

cargo build --profile super-release --target "${DLL_PACK_TARGET}"

mkdir -p ./artifacts/

dll-pack-builder local $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name') \
    $(dll-pack-builder find "${BUILD_OUT_DIR}") \
    ./artifacts/ "${DLL_PACK_TARGET}" "${GITHUB_REPOSITORY}" "${gh_tag}"
